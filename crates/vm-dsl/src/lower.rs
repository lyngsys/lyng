//! AST → `naked_asm!` body lowerer.
//!
//! The lowerer is where the design's "DSL surface ≈ asm shape" decision
//! pays out: each body statement is one DSL-op invocation, and the
//! lowerer composes them all into a single `naked_asm!` template. Each
//! per-arch DSL-op `macro_rules!` macro (under
//! `crates/vm/src/dsl/backend/aarch64/`, added in Batch 4 /
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
//!         "/* len={length} regs={state_pc}{state_pb}{state_regs}{state_fv}{state_object_records}{state_object_slots}{state_prefix}{vm_poll}{feedback_entry_stride}{entry_observed}{feedback_scalar_execution_count} */\n",
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
//! `AArch64` register operands like `w9, x10`. If the proc-macro spliced
//! the raw idents into `naked_asm!`, the assembler would see `w<a>` —
//! invalid asm. The lowerer therefore walks the body `TokenStream` and
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
//! for bridge invocations such as `call_slow!(shim_name, ...)` and
//! `call_rust_probe!(shim_name, ...)`.
//!
//! An "unused" comment fragment at the top of the asm template
//! unconditionally references every named binding, keeping rustc
//! quiet about unused named args regardless of which backend macros
//! the body actually uses.

use proc_macro2::Spacing;
use proc_macro2::{Delimiter, Group, Ident, Literal, Punct, Span, TokenStream, TokenTree};
use quote::quote;
use std::collections::BTreeSet;
use syn::{LitInt, Result};

use crate::layouts::Layout;
use crate::parse::{BodyStmt, DecodeMode, HandlerAst};
use crate::scratch::ScratchAllocator;

const fn expected_operand_arity(layout: Layout, decode_mode: DecodeMode) -> usize {
    match decode_mode {
        DecodeMode::Asm => layout.operand_arity(),
        DecodeMode::Rust => 0,
    }
}

fn validate_operand_arity(ast: &HandlerAst, layout: Layout, operand_count: usize) -> Result<()> {
    let expected_arity = expected_operand_arity(layout, ast.decode_mode);
    if operand_count == expected_arity {
        return Ok(());
    }

    Err(syn::Error::new(
        ast.layout.span(),
        format!(
            "layout {} with {} decoding has arity {}, got {} operand bindings",
            ast.layout,
            ast.decode_mode.name(),
            expected_arity,
            operand_count,
        ),
    ))
}

fn handler_decode_prologue(
    layout: Layout,
    decode_mode: DecodeMode,
    operands: &[Ident],
) -> TokenStream {
    match decode_mode {
        DecodeMode::Asm => layout.decode_prologue_tokens(operands),
        DecodeMode::Rust => quote! { "" },
    }
}

pub fn lower_handler(ast: &HandlerAst) -> Result<TokenStream> {
    let layout = Layout::from_ident(&ast.layout)?;
    let operands: Vec<_> = ast.operand_idents.iter().cloned().collect();
    validate_operand_arity(ast, layout, operands.len())?;

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
        ::lyng_vm::inc_dispatch_counter!(#opcode_byte)
    };
    let prologue_raw = handler_decode_prologue(layout, ast.decode_mode, &operands);
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
    // Track whether we've crossed the first `.label:` declaration. The
    // counter-injection discipline differs for labeled inline handlers
    // vs label-free cold stubs:
    //
    // - `call_slow!` BEFORE any label in a handler that later declares
    //   a label: these are hit-side tail invocations. They run on every
    //   successful inline dispatch, NOT just on slow-path entry.
    //   Injecting `opcode_byte = N` here would emit
    //   `inc_slow_semantic_counter!` on every dispatch and falsely
    //   report ~100% slow-path-share for that opcode.
    // - `call_slow!` AFTER the first label: these are inside a label
    //   scope (typically `.slow:`), executed only when the inline hit path
    //   bails. Counter-injection here is semantically correct.
    // - `call_slow!` in a label-free handler: this is a pure cold stub,
    //   so every dispatch is a semantic slow-path entry and must be
    //   counted.
    //
    // `poll_safepoint!` is unaffected — its asm shape uses a runtime
    // `cbz`/`cbnz` so the counter-bump fires only on the pending-poll
    // branch regardless of label position; injection is safe everywhere.
    let handler_has_label = ast
        .body
        .iter()
        .any(|stmt| matches!(stmt, BodyStmt::Label(_)));
    let mut seen_label = false;
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
                //
                // `gate_call_slow` suppresses injection into
                // `call_slow!` invocations on the hit-side tail
                // (before any `.label:` in a handler that has labels)
                // to avoid double-counting hit-side record-smi shim
                // calls as slow-path entries. Label-free cold stubs are
                // always semantic slow-path entries, so they are not
                // gated.
                let gate_call_slow = handler_has_label && !seen_label;
                let injected = inject_opcode_byte(tokens.clone(), opcode_byte, gate_call_slow);
                let rewritten = substitute_idents(injected, &mut scratch, &label_prefix)?;
                collect_shim_names(&rewritten, &mut shim_names);
                body_tokens.push(rewritten);
            }
            BodyStmt::Label(name) => {
                let asm = format!("{label_prefix}{name}:\n");
                body_tokens.push(quote! { #asm });
                seen_label = true;
            }
        }
    }

    // Emission order: counter-inc → operand-decode prologue → body
    // statements. The counter increment must precede the prologue so
    // it bumps the dispatch counter regardless of which inline
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
                "/* len={length} pc={state_pc} pb={state_pb} regs={state_regs} fv={state_fv} mt={state_mt} objects={state_object_records} object_slots={state_object_slots} prefix={state_prefix} poll={vm_poll} fb_stride_shift={entry_stride_shift} fb_stride={feedback_entry_stride} fb_mode={feedback_mode} fb_named_handler={feedback_named_handler_bits} fb_named_aux_bits={feedback_named_aux_bits} fb_observed={entry_observed} fb_count={feedback_scalar_execution_count} arith_obs={arith_metadata_observed_bits_offset} arith_cnt={arith_metadata_exec_count_offset} obj_shape={object_shape} obj_prototype={object_prototype} obj_named_slots={object_named_slots} obj_inline_slots={object_inline_slots} ctr={vm_counter_base} const_base={vm_const_base} this_value={state_this_value} uninit_lex={value_uninit_lex_bits} exit={exit} */\n",
                #(#template_entries)*
                length = const #length as u32,
                state_pc = const ::lyng_vm::dsl::reg_convention::LLINT_STATE_FRAME_PC_OFFSET,
                state_pb = const ::lyng_vm::dsl::reg_convention::LLINT_STATE_FRAME_PB_BASE,
                state_regs = const ::lyng_vm::dsl::reg_convention::LLINT_STATE_FRAME_REGS_BASE,
                state_fv = const ::lyng_vm::dsl::reg_convention::LLINT_STATE_FRAME_FV_BASE,
                // Phase C.4: byte offset of `LlIntState::frame_metadata_table_base`.
                // x21 (MT pin) holds this pointer. `load_feedback_site!` and `record_*!`
                // macros both resolve through x21 = MetadataTable buffer base.
                state_mt = const ::lyng_vm::dsl::reg_convention::LLINT_STATE_FRAME_METADATA_TABLE_BASE,
                state_object_records = const ::lyng_vm::dsl::reg_convention::LLINT_STATE_OBJECT_RECORDS_BASE,
                state_object_slots = const ::lyng_vm::dsl::reg_convention::LLINT_STATE_OBJECT_SLOTS_BASE,
                state_prefix = const ::lyng_vm::dsl::reg_convention::LLINT_STATE_PREFIX,
                vm_poll = const ::lyng_vm::dsl::reg_convention::VM_POLL_PENDING_OFFSET,
                entry_stride_shift = const ::lyng_vm::dsl::feedback_flat::FEEDBACK_ENTRY_STRIDE_SHIFT,
                feedback_entry_stride = const ::lyng_vm::dsl::feedback_flat::FEEDBACK_ENTRY_STRIDE,
                feedback_mode = const ::lyng_vm::dsl::feedback_flat::FEEDBACK_ENTRY_MODE_OFFSET,
                feedback_named_handler_bits = const ::lyng_vm::dsl::feedback_flat::FEEDBACK_ENTRY_NAMED_HANDLER_BITS_OFFSET,
                feedback_named_aux_bits = const ::lyng_vm::dsl::feedback_flat::FEEDBACK_ENTRY_NAMED_AUX_BITS_OFFSET,
                entry_observed = const ::lyng_vm::dsl::feedback_flat::FEEDBACK_ENTRY_SCALAR_OBSERVED_BITS_OFFSET,
                feedback_scalar_execution_count = const ::lyng_vm::dsl::feedback_flat::FEEDBACK_ENTRY_SCALAR_EXECUTION_COUNT_OFFSET,
                // Phase C precomputed-offset optimization: `load_feedback_site!` and
                // `record_*!` macros now resolve slots via the slot_to_entry_offset
                // table at buffer[0..N*4]. Only the field-level offsets are needed.
                arith_metadata_observed_bits_offset = const ::lyng_vm::dsl::reg_convention::ARITH_METADATA_OBSERVED_BITS_OFFSET,
                arith_metadata_exec_count_offset = const ::lyng_vm::dsl::reg_convention::ARITH_METADATA_EXEC_COUNT_OFFSET,
                object_shape = const ::lyng_vm::dsl::reg_convention::RUNTIME_OBJECT_SHAPE_OFFSET,
                object_prototype = const ::lyng_vm::dsl::reg_convention::RUNTIME_OBJECT_PROTOTYPE_OFFSET,
                object_named_slots = const ::lyng_vm::dsl::reg_convention::RUNTIME_OBJECT_NAMED_SLOTS_OFFSET,
                object_inline_slots = const ::lyng_vm::dsl::reg_convention::RUNTIME_OBJECT_INLINE_NAMED_SLOTS_OFFSET,
                // `vm_counter_base` is the byte offset of `Vm::dispatch_counters`
                // (a `Box<DispatchCounters>` whose raw pointer reads through the
                // `*mut DispatchCounters` repr-equivalence). When the
                // `opcode-counters` feature is off the binding falls back to `0`
                // via the sentinel const in `reg_convention.rs`; the counter
                // macros themselves emit empty strings in that config, so the
                // binding is referenced only by the leading comment.
                vm_counter_base = const ::lyng_vm::dsl::reg_convention::VM_DISPATCH_COUNTERS_PTR_OFFSET,
                // Phase 1.B.1: byte offset of `LlIntState::frame_const_base`,
                // the pre-resolved constants-array pointer. Read by
                // `load_constant!` (backend/aarch64/constants.rs). Universally
                // bound even when no handler in this translation unit uses the
                // macro — the leading reference comment keeps rustc quiet about
                // unused named args.
                vm_const_base = const ::lyng_vm::dsl::reg_convention::LLINT_STATE_FRAME_CONST_BASE,
                // Phase 1.B.1: byte offset of `LlIntState::frame_this_value`,
                // the asm-side `this` mirror (either the real `this` Value or
                // `Value::uninitialized_lexical()` sentinel — see
                // `resolve_initial_this_value`). Targeted by `load_state_value!`
                // (backend/aarch64/frame.rs) via
                // `load_state_value!(dst, vm_state_offset = state_this_value)`.
                // Universally bound; unused-binding warning is suppressed by
                // the reference comment above.
                state_this_value = const ::lyng_vm::dsl::reg_convention::LLINT_STATE_FRAME_THIS_VALUE,
                // Phase 1.B.2: 64-bit bit pattern of
                // `Value::uninitialized_lexical()`, used by
                // `load_uninit_lex_sentinel!` (backend/aarch64/values.rs)
                // to materialize the sentinel in a scratch register for
                // the `op_load_this` sentinel-bail comparison. Mirrors
                // the `state_this_value` pattern; universally bound and
                // referenced by the leading comment so unused-binding
                // warnings stay silent in translation units that don't
                // expand the macro.
                value_uninit_lex_bits = const ::lyng_vm::dsl::backend::aarch64::prelude::VALUE_UNINIT_LEX_BITS,
                exit = sym ::lyng_vm::dsl::entry::_interpreter_exit,
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
                    out.push(TokenTree::Punct(proc_macro2::Punct::new(
                        '.',
                        Spacing::Alone,
                    )));
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
/// `gate_call_slow` controls whether `call_slow!` injection is
/// suppressed. The caller (`lower_handler`) sets this to `true` for
/// statements emitted BEFORE the first `.label:` declaration — those
/// `call_slow!`s are hit-side tail invocations (e.g. record-smi shim
/// calls for feedback recording) and must not be counted as slow-path
/// semantic entries. When `gate_call_slow` is `true`, only
/// `poll_safepoint!` invocations receive injection (their asm shape
/// branches on a runtime flag, so counter-bumping is naturally
/// hit-side safe — see `safepoint.rs`). When `false`, both macros are
/// rewritten (the original DSL-1 Phase 1.B.0 Task 5 behavior).
///
/// Each statement passed in is a single macro call of the shape
/// `<path>!(<args>)`, but we still recurse into groups so any nested
/// invocations are also rewritten.
fn inject_opcode_byte(
    tokens: TokenStream,
    opcode_byte: &LitInt,
    gate_call_slow: bool,
) -> TokenStream {
    let trees: Vec<TokenTree> = tokens.into_iter().collect();
    let mut out: Vec<TokenTree> = Vec::with_capacity(trees.len());
    let mut i = 0;
    while i < trees.len() {
        // Detect `<name> ! (...)` where <name> is one of the slow-path
        // bridge macros that we want to enrich.
        let name = if let TokenTree::Ident(id) = &trees[i] {
            id.to_string()
        } else {
            // Recurse into groups (e.g. macro args that themselves
            // contain nested invocations — rare but defensive).
            out.push(rewrite_group(trees[i].clone(), opcode_byte, gate_call_slow));
            i += 1;
            continue;
        };
        // `call_slow!` is suppressed when `gate_call_slow` (hit-side
        // tail). Poll shims are counted by the paired `poll_safepoint!`
        // branch, so they must not also increment the semantic bank.
        // `poll_safepoint!` itself is always a candidate.
        let is_target = match name.as_str() {
            "call_slow" => !gate_call_slow && !is_poll_shim_call(&trees, i),
            "poll_safepoint" => true,
            _ => false,
        };
        if !is_target {
            out.push(rewrite_group(trees[i].clone(), opcode_byte, gate_call_slow));
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

fn is_poll_shim_call(trees: &[TokenTree], ident_index: usize) -> bool {
    let Some(TokenTree::Punct(bang)) = trees.get(ident_index + 1) else {
        return false;
    };
    if bang.as_char() != '!' {
        return false;
    }
    let Some(TokenTree::Group(group)) = trees.get(ident_index + 2) else {
        return false;
    };
    if group.delimiter() != Delimiter::Parenthesis {
        return false;
    }
    group.stream().into_iter().next().is_some_and(
        |tree| matches!(tree, TokenTree::Ident(name) if name.to_string().ends_with("_poll_rs")),
    )
}

/// Recurse into a `TokenTree::Group`, applying `inject_opcode_byte` to
/// the inner stream. Non-group trees pass through unchanged. The
/// `gate_call_slow` flag is propagated so the label-scope discipline is
/// preserved inside nested groups (a `call_slow!` nested inside another
/// macro's args still respects the hit-side-tail suppression).
fn rewrite_group(tt: TokenTree, opcode_byte: &LitInt, gate_call_slow: bool) -> TokenTree {
    match tt {
        TokenTree::Group(g) => {
            let inner = inject_opcode_byte(g.stream(), opcode_byte, gate_call_slow);
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

/// Scan `tokens` for bridge invocations of the shape
/// `call_slow!(<shim_name>, args = [...])` or
/// `call_rust_probe!(<shim_name>, args = [...])` and collect the shim name.
/// The shim name is a bare ident; we treat it as a linker symbol that
/// must be supplied as `<name> = sym <name>` to `naked_asm!`.
fn collect_shim_names(tokens: &TokenStream, out: &mut BTreeSet<String>) {
    // Heuristic: walk the stream looking for
    // `(call_slow|call_rust_probe) ! ( IDENT , ...)`.
    let trees: Vec<TokenTree> = tokens.clone().into_iter().collect();
    for i in 0..trees.len() {
        let TokenTree::Ident(id) = &trees[i] else {
            continue;
        };
        if id != "call_slow" && id != "call_rust_probe" {
            continue;
        }
        // Followed by `!`?
        let Some(TokenTree::Punct(p)) = trees.get(i + 1) else {
            continue;
        };
        if p.as_char() != '!' {
            continue;
        }
        // Followed by a delimited group `(...)`.
        let Some(TokenTree::Group(g)) = trees.get(i + 2) else {
            continue;
        };
        if g.delimiter() != Delimiter::Parenthesis {
            continue;
        }
        // Read the first ident inside the group — that's the shim name.
        let inner: Vec<TokenTree> = g.stream().into_iter().collect();
        if let Some(TokenTree::Ident(name)) = inner.first() {
            out.insert(name.to_string());
        }
    }
    // Recurse into groups so bridge calls inside nested macro args are
    // found too (rare but defensive).
    for tt in tokens.clone() {
        if let TokenTree::Group(g) = tt {
            collect_shim_names(&g.stream(), out);
        }
    }
}

#[cfg(test)]
mod tests {
    //! Token-level unit tests for `inject_opcode_byte`'s label-scope
    //! discipline (DSL-1 Phase 1.C followup #1).
    //!
    //! The lowerer can't be exercised end-to-end here — proc-macro
    //! crates can only emit tokens for downstream crates to compile —
    //! but the `inject_opcode_byte` helper operates on
    //! `proc_macro2::TokenStream` and is fully testable in-crate.
    //!
    //! Strategy: parse a representative macro-call statement, run it
    //! through `inject_opcode_byte` with both `gate_call_slow` settings,
    //! and assert the presence/absence of the `opcode_byte = N` named
    //! arg in the output stream. This mirrors what the lowerer does
    //! before each `BodyStmt::MacroCall` is spliced into `naked_asm!`.
    use super::*;
    use proc_macro2::TokenStream;
    use syn::LitInt;

    /// Returns `true` if the output stream contains a top-level
    /// `opcode_byte` ident (the marker for "injection happened").
    /// Mirrors `arg_group_has_opcode_byte` but walks the OUTER stream —
    /// the helper emits `call_slow ! ( ... , opcode_byte = N )` so the
    /// marker sits inside the parenthesized arg group.
    fn output_has_opcode_byte_for(name: &str, tokens: &TokenStream) -> bool {
        let trees: Vec<TokenTree> = tokens.clone().into_iter().collect();
        for i in 0..trees.len() {
            let TokenTree::Ident(id) = &trees[i] else {
                continue;
            };
            if id != name {
                continue;
            }
            let Some(TokenTree::Punct(p)) = trees.get(i + 1) else {
                continue;
            };
            if p.as_char() != '!' {
                continue;
            }
            let Some(TokenTree::Group(g)) = trees.get(i + 2) else {
                continue;
            };
            if g.delimiter() != Delimiter::Parenthesis {
                continue;
            }
            if arg_group_has_opcode_byte(g.stream()) {
                return true;
            }
        }
        false
    }

    fn output_has_opcode_byte_for_recursive(name: &str, tokens: &TokenStream) -> bool {
        if output_has_opcode_byte_for(name, tokens) {
            return true;
        }
        tokens.clone().into_iter().any(|tt| match tt {
            TokenTree::Group(group) => output_has_opcode_byte_for_recursive(name, &group.stream()),
            _ => false,
        })
    }

    fn lower_handler_tokens(source: &str) -> TokenStream {
        let input: TokenStream = syn::parse_str(source).expect("parse handler tokens");
        let ast = crate::parse::parse_handler(input).expect("parse handler ast");
        lower_handler(&ast).expect("lower handler")
    }

    fn lit31() -> LitInt {
        syn::parse_str("31").expect("parse u8 literal")
    }

    #[test]
    fn no_decode_abx_handler_omits_asm_decode_prologue() {
        let lowered = lower_handler_tokens(
            "op_load_smi_dsl, opcode_byte = 9, layout = Abx, length = 4, decode = Rust, || {
                call_slow!(op_load_smi_slow_rs, args = []);
                dispatch_after_slow!();
            }",
        );
        let output = lowered.to_string();

        assert!(
            !output.contains("decode_abx"),
            "decode = Rust must not emit the Abx asm decode prologue. Got: {output}",
        );
        assert!(
            output_has_opcode_byte_for_recursive("call_slow", &lowered),
            "no-decode cold stubs must still receive slow-path counter injection. Got: {output}",
        );
    }

    #[test]
    fn no_decode_abc_handler_omits_asm_decode_prologue() {
        let lowered = lower_handler_tokens(
            "op_define_named_property_dsl, opcode_byte = 73, layout = Abc, length = 4, decode = Rust, || {
                call_slow!(op_define_named_property_slow_rs, args = []);
                dispatch_after_slow!();
            }",
        );
        let output = lowered.to_string();

        assert!(
            !output.contains("decode_abc"),
            "decode = Rust must not emit the Abc asm decode prologue. Got: {output}",
        );
        assert!(
            output_has_opcode_byte_for_recursive("call_slow", &lowered),
            "no-decode cold stubs must still receive slow-path counter injection. Got: {output}",
        );
    }

    #[test]
    fn no_decode_abc_slot_handler_omits_asm_decode_prologue() {
        let lowered = lower_handler_tokens(
            "op_set_named_property_dsl, opcode_byte = 78, layout = AbcSlot, length = 6, decode = Rust, || {
                call_slow!(op_set_named_property_slow_rs, args = []);
                dispatch_after_slow!();
            }",
        );
        let output = lowered.to_string();

        assert!(
            !output.contains("decode_abc_slot"),
            "decode = Rust must not emit the AbcSlot asm decode prologue. Got: {output}",
        );
        assert!(
            output_has_opcode_byte_for_recursive("call_slow", &lowered),
            "no-decode cold stubs must still receive slow-path counter injection. Got: {output}",
        );
    }

    #[test]
    fn default_fast_handler_keeps_direct_asm_decode() {
        let lowered = lower_handler_tokens(
            "op_add_dsl, opcode_byte = 31, layout = AbcSlot, length = 6, |a, b, c, slot| {
                load_reg!(b => t0);
                dispatch!();
            }",
        );
        let output = lowered.to_string();

        assert!(
            output.contains("decode_abc_slot"),
            "normal handlers must keep the direct asm decode prologue. Got: {output}",
        );
    }

    #[test]
    fn jump_i24_handler_uses_exact_i24_decode_prologue() {
        let lowered = lower_handler_tokens(
            "op_jump, opcode_byte = 63, layout = AxI24, length = 4, |offset| {
                dispatch!();
            }",
        );
        let output = lowered.to_string();

        assert!(
            output.contains("decode_ax_i24"),
            "signed i24 control-flow handlers must use the exact i24 decode prologue. Got: {output}",
        );
        assert!(
            !output.contains("decode_ax !"),
            "signed i24 control-flow handlers must not use the 32-bit Ax overread prologue. Got: {output}",
        );
    }

    #[test]
    fn rust_decode_mode_rejects_operand_bindings() {
        let input: TokenStream = syn::parse_str(
            "op_load_smi_dsl, opcode_byte = 9, layout = Abx, length = 4, decode = Rust, |a, bx| {
                call_slow!(op_load_smi_slow_rs, args = []);
                dispatch_after_slow!();
            }",
        )
        .expect("parse handler tokens");
        let ast = crate::parse::parse_handler(input).expect("parse handler ast");
        let error = lower_handler(&ast).expect_err("decode = Rust should reject operand bindings");

        assert!(
            error
                .to_string()
                .contains("layout Abx with Rust decoding has arity 0, got 2 operand bindings"),
            "unexpected error for decode = Rust operands: {error}",
        );
    }

    #[test]
    fn pre_label_call_slow_skipped_when_gated() {
        // A pre-label call_slow is a hit-side tail bridge, not a semantic
        // slow-path entry. The lowerer must not inject slow-path counters
        // there.
        let tokens: TokenStream = syn::parse_str("call_slow!(op_tail_bridge_rs, args = [slot])")
            .expect("parse hit-side call_slow!");
        let rewritten = inject_opcode_byte(tokens, &lit31(), /*gate_call_slow=*/ true);
        assert!(
            !output_has_opcode_byte_for("call_slow", &rewritten),
            "hit-side `call_slow!` must NOT receive `opcode_byte = N` \
             when gate_call_slow is true (would over-count slow-path \
             semantic entries). Got: {rewritten}",
        );
    }

    #[test]
    fn slow_path_call_slow_injected_when_not_gated() {
        // The slow-path call_slow inside a `.slow:` label scope. The
        // lowerer flips `gate_call_slow` to `false` once it has seen
        // any `BodyStmt::Label`.
        let tokens: TokenStream =
            syn::parse_str("call_slow!(op_add_slow_rs, args = [a, b, c, slot])")
                .expect("parse slow-path call_slow!");
        let rewritten = inject_opcode_byte(tokens, &lit31(), /*gate_call_slow=*/ false);
        assert!(
            output_has_opcode_byte_for("call_slow", &rewritten),
            "slow-path `call_slow!` MUST receive `opcode_byte = N` \
             when gate_call_slow is false (correctly counts a slow \
             entry). Got: {rewritten}",
        );
    }

    #[test]
    fn poll_shim_call_slow_is_not_counted_as_semantic_slow_path() {
        let tokens: TokenStream = syn::parse_str("call_slow!(op_jump_poll_rs, args = [offset])")
            .expect("parse poll call_slow!");
        let rewritten = inject_opcode_byte(tokens, &lit31(), /*gate_call_slow=*/ false);
        assert!(
            !output_has_opcode_byte_for("call_slow", &rewritten),
            "poll shims are counted by `poll_safepoint!`, not the semantic slow-path bank. Got: {rewritten}",
        );
    }

    #[test]
    fn label_free_cold_stub_call_slow_is_injected() {
        let lowered = lower_handler_tokens(
            "op_get_named_property_dsl, opcode_byte = 77, layout = AbcSlot, length = 6, |a, b, c, slot| {
                call_slow!(op_get_named_property_slow_rs, args = [a, b, c, slot]);
                dispatch_after_slow!();
            }",
        );

        assert!(
            output_has_opcode_byte_for_recursive("call_slow", &lowered),
            "label-free cold stubs must receive opcode_byte injection so \
             slow-path counters count semantic entries. Got: {lowered}",
        );
    }

    #[test]
    fn rust_probe_shim_name_is_collected() {
        let tokens: TokenStream =
            syn::parse_str("call_rust_probe!(op_load_global_rust_probe_rs, args = [a, bx])")
                .expect("parse call_rust_probe!");
        let mut names = BTreeSet::new();
        collect_shim_names(&tokens, &mut names);
        assert!(
            names.contains("op_load_global_rust_probe_rs"),
            "call_rust_probe! bridge shims must be emitted as naked_asm named symbols. Got: {names:?}",
        );
    }

    #[test]
    fn poll_safepoint_always_injected() {
        // `poll_safepoint!` is structurally hit-side safe: the
        // counter-bump emission sits behind a runtime `cbz`/`cbnz`
        // branch on `vm.poll_pending`. Injection is correct in both
        // pre- and post-label contexts.
        let tokens: TokenStream =
            syn::parse_str("poll_safepoint!(.poll_pending)").expect("parse poll_safepoint!");
        let gated = inject_opcode_byte(tokens.clone(), &lit31(), true);
        let ungated = inject_opcode_byte(tokens, &lit31(), false);
        assert!(
            output_has_opcode_byte_for("poll_safepoint", &gated),
            "`poll_safepoint!` should be rewritten even when \
             gate_call_slow is true. Got: {gated}",
        );
        assert!(
            output_has_opcode_byte_for("poll_safepoint", &ungated),
            "`poll_safepoint!` should be rewritten when \
             gate_call_slow is false. Got: {ungated}",
        );
    }

    #[test]
    fn idempotent_when_opcode_byte_already_present() {
        // Hand-written call sites can opt out by spelling
        // `opcode_byte = N` explicitly. The injection must be a no-op
        // in both gating modes.
        let tokens: TokenStream =
            syn::parse_str("call_slow!(op_add_slow_rs, args = [a, b, c, slot], opcode_byte = 99)")
                .expect("parse call_slow! with explicit opcode_byte");
        let rewritten = inject_opcode_byte(tokens, &lit31(), false);
        // The output stream should still contain exactly one
        // `opcode_byte = 99` (no duplicate `opcode_byte = 31` appended).
        let s = rewritten.to_string();
        let count_99 = s.matches("opcode_byte = 99").count();
        let count_31 = s.matches("opcode_byte = 31").count();
        assert_eq!(
            count_99, 1,
            "explicit opcode_byte = 99 should pass through unchanged. \
             Got: {s}",
        );
        assert_eq!(
            count_31, 0,
            "lowerer must not inject a second opcode_byte when the \
             user already spelled one. Got: {s}",
        );
    }

    #[test]
    fn non_target_macros_pass_through_unchanged() {
        // `dispatch!()`, `check_smi!(...)`, etc. should not be
        // rewritten by either gating mode.
        let tokens: TokenStream =
            syn::parse_str("check_smi!(t0, .slow)").expect("parse check_smi!");
        let gated = inject_opcode_byte(tokens.clone(), &lit31(), true);
        let ungated = inject_opcode_byte(tokens, &lit31(), false);
        assert!(
            !output_has_opcode_byte_for("check_smi", &gated),
            "`check_smi!` should never receive opcode_byte injection. \
             Got: {gated}",
        );
        assert!(
            !output_has_opcode_byte_for("check_smi", &ungated),
            "`check_smi!` should never receive opcode_byte injection. \
             Got: {ungated}",
        );
    }
}
