//! AST → `naked_asm!` body lowerer.
//!
//! The lowerer is where the design's "DSL surface ≈ asm shape" decision
//! pays out: each body statement is one DSL-op invocation, and the
//! lowerer composes them all into a single `naked_asm!` template. Each
//! per-arch DSL-op `macro_rules!` macro (under
//! `crates/lyng-js/vm/src/dsl/backend/aarch64/`, added in Batch 4 /
//! tasks B20–B28) expands at the *consumer crate's* call site into a
//! `concat!`-produced `&'static str` fragment.
//!
//! ## Emission shape
//!
//! For a handler `op_xxx, layout = L, length = N, |args| { m1!(...); m2!(...); }`:
//!
//! ```ignore
//! #[unsafe(naked)]
//! pub extern "C" fn op_xxx() -> ! {
//!     ::core::arch::naked_asm!(
//!         "/* len={length} regs={state_pc}{state_pb}{state_regs}{state_fv}{state_prefix}{vm_poll}{entry_stride_shift}{entry_observed} */\n",
//!         decode_<layout>!(<operand idents as scratch regs>),
//!         m1!(...),                  // macro call returning &'static str (via concat!)
//!         m2!(...),                  // ditto
//!         length = const N as u32,
//!         state_pc = const ...,
//!         state_pb = const ...,
//!         /* etc. */
//!         <shim_name> = sym <shim_path>,  // per call_slow! reference
//!     )
//! }
//! ```
//!
//! ## Substitution
//!
//! The DSL body references operand idents (`a`, `b`, `c`, `slot`,
//! `src`, `dst`, ...) and internal scratch idents (`t0..t6`). The
//! backend macros uniformly `stringify!` their arguments to build
//! AArch64 register operands like `w9, x10`. If the proc-macro spliced
//! the raw idents into `naked_asm!`, the assembler would see `w<a>` —
//! invalid asm. The lowerer therefore walks the body TokenStream and
//! rewrites every recognized scratch-name ident into its allocated
//! register number literal *before* splicing into `naked_asm!`.
//!
//! Recognized scratch names:
//! - Operand bindings declared in the handler signature.
//! - Internal scratch slots `t0..t6` (allocated lazily on first use).
//!
//! Other idents (label names like `.slow`, macro names like
//! `dispatch`, Rust paths like `op_add_slow_rs`) pass through verbatim.
//!
//! ## Standard named bindings
//!
//! Backend macros reference a fixed set of `{name}` placeholders for
//! `LlIntState` field offsets, the VM polling flag, and the shim
//! symbol. The lowerer always supplies the layout-stable bindings; the
//! per-call-site `{shim}` binding is collected by scanning the body
//! for `call_slow!(shim_name, ...)` invocations.
//!
//! An "unused" comment fragment at the top of the asm template
//! unconditionally references every named binding, keeping rustc
//! quiet about unused named args regardless of which backend macros
//! the body actually uses.

use proc_macro2::{Delimiter, Group, Ident, Literal, Punct, Span, TokenStream, TokenTree};
use proc_macro2::Spacing;
use quote::quote;
use std::collections::BTreeSet;
use syn::{LitInt, Result};

use crate::layouts::Layout;
use crate::parse::{BodyStmt, HandlerAst};
use crate::scratch::ScratchAllocator;

pub(crate) fn lower_handler(ast: HandlerAst) -> Result<TokenStream> {
    let layout = Layout::from_ident(&ast.layout)?;
    let operands: Vec<_> = ast.operand_idents.iter().cloned().collect();
    if operands.len() != layout.operand_arity() {
        return Err(syn::Error::new(
            ast.layout.span(),
            format!(
                "layout {} has arity {}, got {} operand bindings",
                ast.layout,
                layout.operand_arity(),
                operands.len(),
            ),
        ));
    }

    // Pre-assign operand identifiers to scratch registers. Internal
    // scratch names `t0..t6` are allocated lazily inside
    // `substitute_idents` as they're encountered in the body.
    let mut scratch = ScratchAllocator::new();
    for name in &operands {
        scratch.assign(name)?;
    }

    let name = &ast.name;
    let length = &ast.length;
    let opcode_byte = &ast.opcode_byte;
    // Counter-increment fragment, emitted as the FIRST body fragment
    // BEFORE the operand-decode prologue. When the `opcode-counters`
    // feature is on the macro emits 4 instructions bumping the
    // dispatch-bank slot for this opcode; when off it expands to the
    // empty string and is invisible. Either way the `vm_counter_base`
    // binding below is supplied (with a fallback `= 0` sentinel const
    // when the feature is off — see `reg_convention.rs`) so rustc
    // never complains about an unused named arg.
    let counter_increment = quote! {
        crate::inc_dispatch_counter!(#opcode_byte)
    };
    let prologue_raw = layout.decode_prologue_tokens(&operands);
    // The prologue invokes `decode_xxx!(<operand idents>)`. The backend
    // macros stringify their args verbatim — feeding them `dst, src`
    // yields invalid asm like `wdst, wsrc`. Substitute the operand idents
    // here so the prologue sees `9, 10, ...` register-number literals
    // just like the body does.
    let label_prefix_for_prologue = format!("L{name}__");
    let prologue = substitute_idents(prologue_raw, &mut scratch, &label_prefix_for_prologue)?;

    // Substitute operand/t-scratch idents in the body, and harvest
    // call_slow! shim names for sym bindings.
    //
    // Label prefix: `L<handler_name>__` — namespaces every body label
    // (e.g. `.slow` → `Lop_add__slow`) so multiple `naked_asm!` blocks
    // in the same translation unit can't collide. `L*` keeps the
    // label assembler-local; the handler-name infix makes them
    // unique across handlers.
    let label_prefix = format!("L{name}__");
    let mut shim_names: BTreeSet<String> = BTreeSet::new();
    let mut body_tokens: Vec<TokenStream> = Vec::with_capacity(ast.body.len());
    for stmt in &ast.body {
        match stmt {
            BodyStmt::MacroCall(tokens) => {
                // Inject `opcode_byte = N` into call_slow!() and
                // poll_safepoint!() invocations (DSL-1 Phase 1.B.0
                // Task 5). The handler's opcode discriminant from the
                // `llint_handler!` signature is threaded through so the
                // backend macros emit `inc_slow_semantic_counter!(N)` /
                // `inc_slow_safepoint_counter!(N)` for slow-path-share
                // accounting. Idempotent — if the user already wrote
                // `opcode_byte = N` explicitly, the injection is skipped.
                let injected = inject_opcode_byte(tokens.clone(), opcode_byte);
                let rewritten = substitute_idents(injected, &mut scratch, &label_prefix)?;
                collect_shim_names(&rewritten, &mut shim_names);
                body_tokens.push(rewritten);
            }
            BodyStmt::Label(name) => {
                let asm = format!("{label_prefix}{name}:\n");
                body_tokens.push(quote! { #asm });
            }
        }
    }

    // Emission order: counter-inc → operand-decode prologue → body
    // statements. The counter increment must precede the prologue so
    // it bumps the dispatch counter regardless of which fast-path
    // branch the body's later `dispatch!`/`dispatch_after_slow!` takes
    // (slow-path round-trips re-enter the dispatch table; their target
    // handler's own counter increment fires on entry there).
    let template_entries = std::iter::once(counter_increment)
        .chain(std::iter::once(prologue))
        .chain(body_tokens)
        .map(|tokens| quote! { #tokens, });

    // Per-shim `<name> = sym <path>` bindings. The body author writes
    // `call_slow!(op_add_slow_rs, args = [a, b, c, slot])`; the lowerer
    // turns that into `bl {op_add_slow_rs}` in asm, then supplies the
    // binding `op_add_slow_rs = sym op_add_slow_rs` so the asm references
    // the linker symbol.
    let shim_bindings = shim_names.iter().map(|name| {
        let ident = Ident::new(name, Span::call_site());
        quote! { #ident = sym #ident, }
    });

    // Emit a sibling `pub const <NAME>_LENGTH: u32 = N;` so a runtime
    // consistency test can cross-check the declared length against the
    // canonical `Opcode::encoded_len()`. The const name is the uppercase
    // of the handler ident (`op_move` → `OP_MOVE_LENGTH`).
    //
    // This is "Option C-light" from DSL-0c's commit-3 plan: the
    // proc-macro emits the const, the hand-written test imports it and
    // compares against `Opcode::<Variant>.encoded_len()`. The const lives
    // in the same module as its handler — no extra symbol-management
    // ceremony.
    let length_const_name = Ident::new(
        &format!("{}_LENGTH", name.to_string().to_uppercase()),
        name.span(),
    );

    Ok(quote! {
        /// Declared instruction length (narrow form) for the
        /// sibling handler. Kept in sync with the `length = N`
        /// attribute by construction (emitted by `llint_handler!`).
        pub const #length_const_name: u32 = #length as u32;

        #[unsafe(naked)]
        pub extern "C" fn #name() -> ! {
            // `naked_asm!` implies `noreturn`; explicit `options(noreturn)`
            // is rejected. The leading "/* len={length} ... */" comment
            // fragment references every named binding so rustc never
            // complains about an unused named arg, regardless of which
            // backend macros the body uses. Asm comments are stripped
            // by the assembler — this is free at runtime.
            ::core::arch::naked_asm!(
                "/* len={length} pc={state_pc} pb={state_pb} regs={state_regs} fv={state_fv} prefix={state_prefix} poll={vm_poll} fb_stride={entry_stride_shift} fb_observed={entry_observed} ctr={vm_counter_base} exit={exit} */\n",
                #(#template_entries)*
                length = const #length as u32,
                state_pc = const ::lyng_js_vm::dsl::reg_convention::LLINT_STATE_FRAME_PC_OFFSET,
                state_pb = const ::lyng_js_vm::dsl::reg_convention::LLINT_STATE_FRAME_PB_BASE,
                state_regs = const ::lyng_js_vm::dsl::reg_convention::LLINT_STATE_FRAME_REGS_BASE,
                state_fv = const ::lyng_js_vm::dsl::reg_convention::LLINT_STATE_FRAME_FV_BASE,
                state_prefix = const ::lyng_js_vm::dsl::reg_convention::LLINT_STATE_PREFIX,
                vm_poll = const ::lyng_js_vm::dsl::reg_convention::VM_POLL_PENDING_OFFSET,
                entry_stride_shift = const 6_u32,
                entry_observed = const 0_u32,
                // `vm_counter_base` is the byte offset of `Vm::dispatch_counters`
                // (a `Box<DispatchCounters>` whose raw pointer reads through the
                // `*mut DispatchCounters` repr-equivalence). When the
                // `opcode-counters` feature is off the binding falls back to `0`
                // via the sentinel const in `reg_convention.rs`; the counter
                // macros themselves emit empty strings in that config, so the
                // binding is referenced only by the leading comment.
                vm_counter_base = const ::lyng_js_vm::dsl::reg_convention::VM_DISPATCH_COUNTERS_PTR_OFFSET,
                exit = sym ::lyng_js_vm::dsl::entry::_interpreter_exit,
                #(#shim_bindings)*
            )
        }
    })
}

/// Walk `tokens` and:
///
/// 1. Rewrite `.label` references (the DSL's label-reference syntax)
///    into `<label_prefix><label>` — an assembler-local label
///    identifier scoped to the current handler. The `label_prefix`
///    is built from the handler name so labels never collide across
///    `naked_asm!` blocks in the same translation unit.
/// 2. Replace recognized scratch idents (operands + `t0..t6`) with
///    their assigned scratch register numbers.
///
/// Other tokens pass through unchanged. Recurses into groups so macro
/// arguments inside `(..)` / `[..]` / `{..}` are rewritten too.
fn substitute_idents(
    tokens: TokenStream,
    scratch: &mut ScratchAllocator,
    label_prefix: &str,
) -> Result<TokenStream> {
    let mut out = Vec::new();
    let mut iter = tokens.into_iter().peekable();
    while let Some(tt) = iter.next() {
        match tt {
            TokenTree::Punct(ref p) if p.as_char() == '.' => {
                if let Some(TokenTree::Ident(id)) = iter.peek().cloned() {
                    iter.next();
                    let prefixed = Ident::new(&format!("{label_prefix}{id}"), id.span());
                    out.push(TokenTree::Ident(prefixed));
                } else {
                    out.push(TokenTree::Punct(proc_macro2::Punct::new('.', Spacing::Alone)));
                }
            }
            TokenTree::Ident(id) => {
                if let Some(reg) = scratch.substitute(&id)? {
                    out.push(TokenTree::Literal(Literal::u8_unsuffixed(reg)));
                } else {
                    out.push(TokenTree::Ident(id));
                }
            }
            TokenTree::Group(g) => {
                let inner = substitute_idents(g.stream(), scratch, label_prefix)?;
                out.push(TokenTree::Group(Group::new(g.delimiter(), inner)));
            }
            other => out.push(other),
        }
    }
    Ok(out.into_iter().collect())
}

/// Inject `, opcode_byte = <N>` into the argument group of every
/// `call_slow!(...)` / `poll_safepoint!(...)` macro call found in
/// `tokens`. This threads the handler's opcode discriminant (from the
/// `llint_handler!` signature) into the slow-path bridge macros so they
/// can emit `inc_slow_semantic_counter!(N)` / `inc_slow_safepoint_counter!(N)`
/// at the correct site (DSL-1 Phase 1.B.0 Task 5).
///
/// Skipped if the user already wrote `opcode_byte = N` explicitly in
/// the invocation. This keeps the rewrite idempotent and allows hand
/// override during testing.
///
/// Each statement passed in is a single macro call of the shape
/// `<path>!(<args>)`, but we still recurse into groups so any nested
/// invocations are also rewritten.
fn inject_opcode_byte(tokens: TokenStream, opcode_byte: &LitInt) -> TokenStream {
    let trees: Vec<TokenTree> = tokens.into_iter().collect();
    let mut out: Vec<TokenTree> = Vec::with_capacity(trees.len());
    let mut i = 0;
    while i < trees.len() {
        // Detect `<name> ! (...)` where <name> is one of the slow-path
        // bridge macros that we want to enrich.
        let name = match &trees[i] {
            TokenTree::Ident(id) => id.to_string(),
            _ => {
                // Recurse into groups (e.g. macro args that themselves
                // contain nested invocations — rare but defensive).
                out.push(rewrite_group(trees[i].clone(), opcode_byte));
                i += 1;
                continue;
            }
        };
        let is_target = name == "call_slow" || name == "poll_safepoint";
        if !is_target {
            out.push(rewrite_group(trees[i].clone(), opcode_byte));
            i += 1;
            continue;
        }
        // Followed by `!` Punct?
        let bang = match trees.get(i + 1) {
            Some(TokenTree::Punct(p)) if p.as_char() == '!' => p,
            _ => {
                out.push(trees[i].clone());
                i += 1;
                continue;
            }
        };
        // Followed by a parenthesized arg group?
        let arg_group = match trees.get(i + 2) {
            Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Parenthesis => g,
            _ => {
                out.push(trees[i].clone());
                i += 1;
                continue;
            }
        };
        // Already has `opcode_byte = ...`? Skip injection.
        if arg_group_has_opcode_byte(arg_group.stream()) {
            out.push(trees[i].clone());
            out.push(TokenTree::Punct(bang.clone()));
            out.push(TokenTree::Group(arg_group.clone()));
            i += 3;
            continue;
        }
        // Append `, opcode_byte = <N>` to the arg-group's contents.
        let mut new_inner: Vec<TokenTree> = arg_group.stream().into_iter().collect();
        let span = bang.span();
        new_inner.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
        new_inner.push(TokenTree::Ident(Ident::new("opcode_byte", span)));
        new_inner.push(TokenTree::Punct(Punct::new('=', Spacing::Alone)));
        new_inner.push(TokenTree::Literal(opcode_byte.token()));
        let new_group = Group::new(arg_group.delimiter(), new_inner.into_iter().collect());
        out.push(trees[i].clone());
        out.push(TokenTree::Punct(bang.clone()));
        out.push(TokenTree::Group(new_group));
        i += 3;
    }
    out.into_iter().collect()
}

/// Recurse into a TokenTree::Group, applying `inject_opcode_byte` to
/// the inner stream. Non-group trees pass through unchanged.
fn rewrite_group(tt: TokenTree, opcode_byte: &LitInt) -> TokenTree {
    match tt {
        TokenTree::Group(g) => {
            let inner = inject_opcode_byte(g.stream(), opcode_byte);
            TokenTree::Group(Group::new(g.delimiter(), inner))
        }
        other => other,
    }
}

/// Returns `true` if the token stream of a `call_slow!` / `poll_safepoint!`
/// argument group already contains a top-level `opcode_byte = ...` named
/// arg. Used to keep the lowerer's injection idempotent so hand-written
/// callsites can opt out.
fn arg_group_has_opcode_byte(stream: TokenStream) -> bool {
    for tt in stream {
        if let TokenTree::Ident(id) = tt {
            if id == "opcode_byte" {
                return true;
            }
        }
    }
    false
}

/// Scan `tokens` for `call_slow!(<shim_name>, args = [...])` invocations
/// and collect the shim name. The shim name is a bare ident; we treat it
/// as a linker symbol that must be supplied as `<name> = sym <name>` to
/// `naked_asm!`.
fn collect_shim_names(tokens: &TokenStream, out: &mut BTreeSet<String>) {
    // Heuristic: walk the stream looking for `call_slow ! ( IDENT , ...)`.
    let trees: Vec<TokenTree> = tokens.clone().into_iter().collect();
    for i in 0..trees.len() {
        let TokenTree::Ident(id) = &trees[i] else { continue };
        if id != "call_slow" {
            continue;
        }
        // Followed by `!`?
        let Some(TokenTree::Punct(p)) = trees.get(i + 1) else { continue };
        if p.as_char() != '!' {
            continue;
        }
        // Followed by a delimited group `(...)`.
        let Some(TokenTree::Group(g)) = trees.get(i + 2) else { continue };
        if g.delimiter() != Delimiter::Parenthesis {
            continue;
        }
        // Read the first ident inside the group — that's the shim name.
        let inner: Vec<TokenTree> = g.stream().into_iter().collect();
        if let Some(TokenTree::Ident(name)) = inner.first() {
            out.insert(name.to_string());
        }
    }
    // Recurse into groups so call_slow! inside nested macro args is found
    // too (rare but defensive).
    for tt in tokens.clone() {
        if let TokenTree::Group(g) = tt {
            collect_shim_names(&g.stream(), out);
        }
    }
}
