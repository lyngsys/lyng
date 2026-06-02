//! AST → `naked_asm!` body lowerer.
//!
//! Each body statement is one DSL-op invocation; the lowerer composes them
//! into a single `naked_asm!` template. Each per-arch `macro_rules!` macro
//! (under `crates/vm/src/dsl/backend/aarch64/`) expands at the consumer
//! crate's call site into a `concat!`-produced `&'static str` fragment.
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
    // Counter-increment fragment: emitted BEFORE the operand-decode prologue.
    // When `diagnostic-counters` is off it expands to the empty string;
    // `vm_counter_base` is always bound (fallback `= 0`) to keep rustc quiet.
    let counter_increment = quote! {
        ::lyng_vm::inc_dispatch_counter!(#opcode_byte)
    };
    let prologue_raw = handler_decode_prologue(layout, ast.decode_mode, &operands);
    // Substitute operand idents so the prologue sees register-number literals.
    let label_prefix_for_prologue = format!("L{name}__");
    let prologue = substitute_idents(prologue_raw, &mut scratch, &label_prefix_for_prologue)?;

    // Substitute operand/t-scratch idents and harvest call_slow! shim names.
    // Label prefix `L<name>__` namespaces body labels (e.g. `.slow` →
    // `Lop_add__slow`) so labels can't collide across handlers.
    let label_prefix = format!("L{name}__");
    let mut shim_names: BTreeSet<String> = BTreeSet::new();
    let mut body_tokens: Vec<TokenStream> = Vec::with_capacity(ast.body.len());
    // Gate call_slow! counter injection on label scope:
    // - Before any label in a labeled handler: hit-side tail, must NOT count.
    // - After first label (typically `.slow:`): slow entry, MUST count.
    // - Label-free handler: every dispatch is a slow entry, always count.
    // poll_safepoint! is always injected (its cbz/cbnz fires only on pending).
    let handler_has_label = ast
        .body
        .iter()
        .any(|stmt| matches!(stmt, BodyStmt::Label(_)));
    let mut seen_label = false;
    for stmt in &ast.body {
        match stmt {
            BodyStmt::MacroCall(tokens) => {
                // Inject `opcode_byte = N` into call_slow! / poll_safepoint!
                // for slow-path-share accounting. Idempotent if already present.
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

    // Emission order: counter-inc → decode prologue → body statements.
    let template_entries = std::iter::once(counter_increment)
        .chain(std::iter::once(prologue))
        .chain(body_tokens)
        .map(|tokens| quote! { #tokens, });

    // Per-shim `<name> = sym <name>` bindings for linker symbols.
    let shim_bindings = shim_names.iter().map(|name| {
        let ident = Ident::new(name, Span::call_site());
        quote! { #ident = sym #ident, }
    });

    // Emit `pub const <NAME>_LENGTH: u32 = N` so tests can cross-check
    // the declared length against `Opcode::encoded_len()`.
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
            // The leading comment references every named binding (rustc
            // unused-named-arg suppression). Stripped by the assembler.
            ::core::arch::naked_asm!(
                "/* len={length} pc={state_pc} pb={state_pb} regs={state_regs} mt={state_mt} objects={state_object_records} object_slots={state_object_slots} prefix={state_prefix} poll={vm_poll} fb_mode={feedback_mode} fb_named_handler={feedback_named_handler_bits} fb_named_aux_bits={feedback_named_aux_bits} fb_gen={feedback_generation} vm_gic={vm_global_ic_gen} state_value_cells={state_value_cells} cell_stored={cell_stored_value} arith_obs={arith_metadata_observed_bits_offset} arith_cnt={arith_metadata_exec_count_offset} obj_shape={object_shape} obj_prototype={object_prototype} obj_named_slots={object_named_slots} obj_inline_slots={object_inline_slots} ctr={vm_counter_base} const_base={vm_const_base} this_value={state_this_value} uninit_lex={value_uninit_lex_bits} exit={exit} */\n",
                #(#template_entries)*
                length = const #length as u32,
                state_pc = const ::lyng_vm::dsl::reg_convention::LLINT_STATE_FRAME_PC_OFFSET,
                state_pb = const ::lyng_vm::dsl::reg_convention::LLINT_STATE_FRAME_PB_BASE,
                state_regs = const ::lyng_vm::dsl::reg_convention::LLINT_STATE_FRAME_REGS_BASE,
                // x21 (MT pin) holds the MetadataTable buffer base.
                state_mt = const ::lyng_vm::dsl::reg_convention::LLINT_STATE_FRAME_METADATA_TABLE_BASE,
                state_object_records = const ::lyng_vm::dsl::reg_convention::LLINT_STATE_OBJECT_RECORDS_BASE,
                state_object_slots = const ::lyng_vm::dsl::reg_convention::LLINT_STATE_OBJECT_SLOTS_BASE,
                state_prefix = const ::lyng_vm::dsl::reg_convention::LLINT_STATE_PREFIX,
                vm_poll = const ::lyng_vm::dsl::reg_convention::VM_POLL_PENDING_OFFSET,
                feedback_mode = const ::lyng_vm::dsl::reg_convention::PROPERTY_METADATA_MODE_OFFSET,
                feedback_named_handler_bits = const ::lyng_vm::dsl::reg_convention::PROPERTY_METADATA_HANDLER_BITS_OFFSET,
                feedback_named_aux_bits = const ::lyng_vm::dsl::reg_convention::PROPERTY_METADATA_AUX_BITS_OFFSET,
                // `PropertyMetadata::generation` — compared by `branch_global_cell_generation_mismatch!`.
                feedback_generation = const ::lyng_vm::dsl::reg_convention::PROPERTY_METADATA_GENERATION_OFFSET,
                // `Vm::dsl_global_ic_generation` — live realm IC generation mirror.
                vm_global_ic_gen = const ::lyng_vm::dsl::reg_convention::VM_GLOBAL_IC_GENERATION_OFFSET,
                // `LlIntState` value-cell table base — resolves 1-based cell refs.
                state_value_cells = const ::lyng_vm::dsl::reg_convention::LLINT_STATE_VALUE_CELLS_BASE,
                // Stored-value offset within `PrimitiveValueCellRecord` (= 0).
                cell_stored_value = const ::lyng_gc::PRIMITIVE_VALUE_CELL_RECORD_STORED_VALUE_OFFSET,
                arith_metadata_observed_bits_offset = const ::lyng_vm::dsl::reg_convention::ARITH_METADATA_OBSERVED_BITS_OFFSET,
                arith_metadata_exec_count_offset = const ::lyng_vm::dsl::reg_convention::ARITH_METADATA_EXEC_COUNT_OFFSET,
                object_shape = const ::lyng_vm::dsl::reg_convention::RUNTIME_OBJECT_SHAPE_OFFSET,
                object_prototype = const ::lyng_vm::dsl::reg_convention::RUNTIME_OBJECT_PROTOTYPE_OFFSET,
                object_named_slots = const ::lyng_vm::dsl::reg_convention::RUNTIME_OBJECT_NAMED_SLOTS_OFFSET,
                object_inline_slots = const ::lyng_vm::dsl::reg_convention::RUNTIME_OBJECT_INLINE_NAMED_SLOTS_OFFSET,
                // `Vm::dispatch_counters` pointer offset; falls back to 0 when
                // `diagnostic-counters` is off.
                vm_counter_base = const ::lyng_vm::dsl::reg_convention::VM_DISPATCH_COUNTERS_PTR_OFFSET,
                // Pre-resolved constants-array pointer in `LlIntState`.
                vm_const_base = const ::lyng_vm::dsl::reg_convention::LLINT_STATE_FRAME_CONST_BASE,
                // Asm-side `this` mirror (real `this` or `uninitialized_lexical` sentinel).
                state_this_value = const ::lyng_vm::dsl::reg_convention::LLINT_STATE_FRAME_THIS_VALUE,
                // Bit pattern of `Value::uninitialized_lexical()` for sentinel-bail comparison.
                value_uninit_lex_bits = const ::lyng_vm::dsl::backend::aarch64::prelude::VALUE_UNINIT_LEX_BITS,
                exit = sym ::lyng_vm::dsl::entry::_interpreter_exit,
                #(#shim_bindings)*
            )
        }
    })
}

/// Walk `tokens` and:
/// 1. Rewrite `.label` references into `<label_prefix><label>`.
/// 2. Replace recognized scratch idents (operands + `t0..t6`) with register numbers.
///
/// Other tokens pass through unchanged. Recurses into groups.
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

/// Inject `, opcode_byte = <N>` into every `call_slow!(...)` /
/// `poll_safepoint!(...)` in `tokens`. Idempotent if already present.
///
/// `gate_call_slow = true` suppresses injection into `call_slow!` (hit-side
/// tail invocations must not be counted as slow-path entries); only
/// `poll_safepoint!` is injected in that mode.
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

/// Recurse into a `TokenTree::Group` and apply `inject_opcode_byte`.
/// Non-group trees pass through unchanged.
fn rewrite_group(tt: TokenTree, opcode_byte: &LitInt, gate_call_slow: bool) -> TokenTree {
    match tt {
        TokenTree::Group(g) => {
            let inner = inject_opcode_byte(g.stream(), opcode_byte, gate_call_slow);
            TokenTree::Group(Group::new(g.delimiter(), inner))
        }
        other => other,
    }
}

/// Returns `true` if the arg-group stream already contains `opcode_byte`
/// (keeps injection idempotent).
fn arg_group_has_opcode_byte(stream: TokenStream) -> bool {
    for tt in stream {
        if let TokenTree::Ident(id) = tt
            && id == "opcode_byte"
        {
            return true;
        }
    }
    false
}

/// Collect shim names from `call_slow!` / `call_rust_probe!` invocations.
/// Each shim name must be supplied as `<name> = sym <name>` to `naked_asm!`.
fn collect_shim_names(tokens: &TokenStream, out: &mut BTreeSet<String>) {
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
        let inner: Vec<TokenTree> = g.stream().into_iter().collect();
        if let Some(TokenTree::Ident(name)) = inner.first() {
            out.insert(name.to_string());
        }
    }
    for tt in tokens.clone() {
        if let TokenTree::Group(g) = tt {
            collect_shim_names(&g.stream(), out);
        }
    }
}

#[cfg(test)]
mod tests {
    //! Token-level unit tests for `inject_opcode_byte` label-scope discipline.
    use super::*;
    use proc_macro2::TokenStream;
    use syn::LitInt;

    /// Returns `true` if the output stream for `name!()` contains `opcode_byte`
    /// inside its arg group.
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
        // Hit-side tail: must NOT receive counter injection.
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
        // Inside `.slow:` scope: MUST receive injection.
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
        let tokens: TokenStream = syn::parse_str(
            "call_rust_probe!(op_assign_named_property_rust_probe_rs, args = [a, b, c, slot])",
        )
        .expect("parse call_rust_probe!");
        let mut names = BTreeSet::new();
        collect_shim_names(&tokens, &mut names);
        assert!(
            names.contains("op_assign_named_property_rust_probe_rs"),
            "call_rust_probe! bridge shims must be emitted as naked_asm named symbols. Got: {names:?}",
        );
    }

    #[test]
    fn poll_safepoint_always_injected() {
        // cbz/cbnz guards the counter-bump, so injection is safe in all contexts.
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
        // Explicit `opcode_byte = N` must suppress injection (no duplicate).
        let tokens: TokenStream =
            syn::parse_str("call_slow!(op_add_slow_rs, args = [a, b, c, slot], opcode_byte = 99)")
                .expect("parse call_slow! with explicit opcode_byte");
        let rewritten = inject_opcode_byte(tokens, &lit31(), false);
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
