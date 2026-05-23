# `op_jump8` + conditional-jump variants DSL port (B44)

Five warm-category jump handlers landed together: `op_jump8`,
`op_jump_if_true`, `op_jump_if_true8`, `op_jump_if_false`,
`op_jump_if_false8`. All five are slow-path-only delegates in DSL-0b
— their bodies are `call_slow! + dispatch_after_slow!`, routing into
the existing semantic bodies in `vm/semantics/control_flow.rs` which
already handle backward-edge polling and PC arithmetic.

## DSL source

`crates/vm/src/dsl/handlers/warm.rs`:

```rust
llint_handler! { op_jump8, layout = A, length = 2, |offset| { ... } }
llint_handler! { op_jump_if_true,  layout = Abx, length = 4, |condition, offset| { ... } }
llint_handler! { op_jump_if_true8, layout = Ab,  length = 3, |condition, offset| { ... } }
llint_handler! { op_jump_if_false,  layout = Abx, length = 4, |condition, offset| { ... } }
llint_handler! { op_jump_if_false8, layout = Ab,  length = 3, |condition, offset| { ... } }
```

## Slow-path shims

Each handler ships with a hand-written shim that:
1. Reconstructs an `LlIntDispatchState` from `*mut LlIntState`
2. Sign-extends the raw operand bytes into the semantic body's i32 delta
3. Builds the appropriate `OpJumpArgs` / `OpJumpIfArgs`
4. Calls into `op_jump8_semantic` / `op_jump_if_true_semantic` / etc.
5. Translates the outcome via `dispatch.translate_outcome(...)`

The semantic bodies already poll the incremental-mark safepoint on
negative `delta`, so the "backward-edge poll" requirement from the
validation cases is honored without inline asm.

## Current asm

See per-opcode files in `reports/lyng/dsl-asm-baseline-aarch64/`:
- `op_jump8.asm`
- `op_jump_if_true.asm` / `op_jump_if_true8.asm`
- `op_jump_if_false.asm` / `op_jump_if_false8.asm`

Each emits ~14 instructions (1-2 operand decode + 5 call setup + 4
dispatch_after_slow continue path + ~7 unusual / exit fallback). The
shape is uniform; only the shim symbol and the operand decode
prologue differ.

## LLInt reference

- `reports/lyng/llint-reference/op_jmp.md` (for op_jump8 shape)
- `reports/lyng/llint-reference/op_jtrue.md`
- `reports/lyng/llint-reference/op_jfalse.md`

JSC inlines the condition test + branch into the asm — Lyng's
DSL-0b form pays a slow-path call per dispatch. DSL-1 can inline
the truthy/falsy test + the forward-jump fast path for hot trace
sites; the slow path stays for `delta < 0` (backward edges that
need the safepoint poll).

## Validation cases

- Case 5: backward unconditional jump (`tests/dsl_validation_safepoint_backward_jump.rs`)
- Case 6: backward conditional jump (`tests/dsl_validation_safepoint_backward_cond_jump.rs`)

Both continue to pass.
