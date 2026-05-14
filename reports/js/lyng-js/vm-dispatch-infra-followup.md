# VM Dispatch Infrastructure Follow-Up

Issue: `lyng-1o9z`

Date: 2026-05-14

## Result

This pass implements the low-risk dispatch infrastructure work without unsafe Rust,
computed goto, nightly tail calls, public API changes, or bytecode re-encoding.

- Operand-form dispatch is folded into opcode arms in
  `crates/lyng-js/vm/src/vm/dispatch.rs`.
- The release hot loop no longer calls `dispatch_operand_form`.
- The per-iteration frame validity check is replaced by an epoch-gated boundary
  check. Debug builds still assert the invariant every opcode.
- The first hot helper set is marked `#[inline]`:
  `execute_get_named_property_opcode`, `execute_get_keyed_property_opcode`, and
  `call_value_small`.
- Prefix collapse is intentionally left as a measured follow-up. The post-change
  disassembly still has two indirect branches, so the remaining prefix/main split
  is visible and can be measured separately.

## Baseline Evidence

Captured from `codex/vm-dispatch-infra-followup` before source changes.

Commands:

```sh
cargo build --release -p lyng-js-cli
sample 31827 15 1 -file /tmp/lyng-richards-baseline.sample.txt
nm target/release/lyng-js | rg 'run_dispatch_loop'
objdump --disassemble-symbols=__ZN10lyng_js_vm2vm8dispatch36_\$LT\$impl\$u20\$lyng_js_vm..vm..Vm\$GT\$17run_dispatch_loop17h3ae27891d3804b79E target/release/lyng-js
```

Artifacts:

- `reports/js/lyng-js/vm-dispatch-infra-followup-baseline.sample.txt`
- local disassembly scratch: `/tmp/rdl-baseline-disasm.txt`

Richards sample highlights:

| Offset | Samples | Meaning |
| ---: | ---: | --- |
| `+740,+420,...` | `4783` | dispatch-loop back-edge/frame validation and nearby jump-table setup |
| `+9740` | `2915` | post-call return handling after named-property load helper |
| `+2256` | `1002` | post-call return handling after `call_value_small` |
| `+2904` | `918` | another post-call return-handling site |

Baseline disassembly for the sampled `run_dispatch_loop` monomorph:

| Offset | Instruction | Meaning |
| ---: | --- | --- |
| `+628` / `0x274` | `br x0` | prefix/semantic opcode dispatch table |
| `+764` / `0x2fc` | `br x0` | main opcode dispatch table |
| `+2092` / `0x830` | `br x13` | operand-form dispatch table from `dispatch_operand_form` |

## After Evidence

Commands:

```sh
cargo build --release -p lyng-js-cli
sample 54488 15 1 -file /tmp/lyng-richards-after.sample.txt
nm target/release/lyng-js | rg 'run_dispatch_loop'
objdump --disassemble-symbols=__ZN10lyng_js_vm2vm8dispatch36_\$LT\$impl\$u20\$lyng_js_vm..vm..Vm\$GT\$17run_dispatch_loop17h2434c437e8d71cc3E target/release/lyng-js
```

Artifacts:

- `reports/js/lyng-js/vm-dispatch-infra-followup-after.sample.txt`
- local disassembly scratch: `/tmp/rdl-after-disasm.txt`

Post-change disassembly for the sampled `run_dispatch_loop` monomorph:

| Offset | Instruction | Meaning |
| ---: | --- | --- |
| `+584` / `0x248` | `br x15` | prefix/semantic dispatch remains |
| `+800` / `0x320` | `br x24` | main opcode dispatch remains |

The operand-form indirect branch is gone. The old third branch at
`+2092` / `0x830` does not appear in the post-change monomorph.

The frame check now starts with an epoch compare:

| Offset | Instruction | Meaning |
| ---: | --- | --- |
| `+344` / `0x158` | `ldr w9, [sp, #0x49c]` | load dispatch-state frame-check epoch |
| `+348` / `0x15c` | `ldr w8, [x25, #0x678]` | load VM frame-check epoch |
| `+352` / `0x160` | `cmp w9, w8` | compare epochs |
| `+356` / `0x164` | `b.eq ... +0x1a8` | skip full `active_in` validation on the common path |

If the epochs differ, the old frame depth/code checks still run before dispatch
continues. Frame-changing boundaries increment the epoch after calls, returns,
exception transfer, generator suspension/resume, async await suspension, and
initial dispatch entry.

Richards sample highlights after the change:

| Offset | Samples | Meaning |
| ---: | ---: | --- |
| `+784,+460,...` | `4496` | back-edge and dispatch setup with epoch-gated validation |
| `+29900` | `2875` | named-property helper return path still visible |
| `+3056` | `1047` | `call_value_small` return path still visible |
| `+23784` | `865` | helper/dispatch site in the post-folded layout |
| `+13008` | `789` | helper/dispatch site in the post-folded layout |

The helper functions remain visible as separate sample symbols despite `#[inline]`:

| Symbol | Baseline samples | After samples | Interpretation |
| --- | ---: | ---: | --- |
| `execute_get_named_property_opcode` | `680` | `659` | marked inline, not fully inlined by LLVM in this build |
| `call_value_small` | `115` | `88` | marked inline, not fully inlined by LLVM in this build |

This means the change removed the operand-form dispatch work, but it should not be
described as having removed the hot helper-return work. Further helper splitting
should be based on another assembly pass.

## Verification

Structural tests added in `dispatch.rs`:

- `dispatch_loop_folds_operand_decode_into_opcode_arms`
- `dispatch_loop_avoids_unconditional_frame_active_check`

Commands run:

```sh
cargo fmt --all
cargo test -p lyng-js-vm dispatch_loop_
cargo test -p lyng-js-tests execution_semantics::script_core::regexp_and_annex_b::script_core_recursive_generator_resume_uses_vm_depth_guard -- --exact
cargo test -p lyng-js-vm -p lyng-js-bytecode -p lyng-js-objects -p lyng-js-tests
cargo clippy -p lyng-js-vm -p lyng-js-objects --all-targets -- -W clippy::pedantic -W clippy::nursery
cargo build --release -p lyng-js-cli
```

All commands above passed.

Full Test262:

```sh
cargo run --release -p lyng-js-test262 -- --report reports/js/lyng-js/vm-dispatch-infra-followup-test262.md -j 4
```

Result: `49722 / 49729` runnable files passed, `7` failed, `0` panicked,
`3324` skipped.

Note: the plan text called out a `49724 / 49729` baseline, but the checked-in
`reports/js/lyng-js/test262.md` in this checkout records `49722 / 49729` with
the same seven known failure files. Two `-j 12` runs of this branch produced one
extra `staging/sm/RegExp/unicode-class-braced.js` timeout at the 1s threshold;
the isolated file and the `staging/sm/RegExp` slice both pass. The stored report
uses the lower-parallelism full-corpus run that matches the checked-in baseline.

## Performance Artifacts

Runtime profile-target benchmark:

- `reports/js/lyng-js/vm-dispatch-infra-followup-runtime-after.md`
- `reports/js/lyng-js/vm-dispatch-infra-followup-runtime-after.json`

V8 v7 sweep, 3 samples per workload, Lyng JS rows only. QuickJS/Boa were
intentionally pointed at `/usr/bin/false` for these filtered runs because the
first all-corpus comparison attempt stalled in an unrelated external-engine row.

| Workload | Lyng JS score median | Wall-time median | Report |
| --- | ---: | ---: | --- |
| `Richards` | `233.000` | `2.015s` | `vm-dispatch-infra-followup-v8-v7-richards-after.md` |
| `DeltaBlue` | `280.000` | `2.037s` | `vm-dispatch-infra-followup-v8-v7-deltablue-after.md` |
| `Crypto` | `262.000` | `17.836s` | `vm-dispatch-infra-followup-v8-v7-crypto-after.md` |
| `RayTrace` | `394.000` | `8.214s` | `vm-dispatch-infra-followup-v8-v7-raytrace-after.md` |
| `NavierStokes` | `445.000` | `12.114s` | `vm-dispatch-infra-followup-v8-v7-navierstokes-after.md` |
| `Splay` | `1220.000` | `2.699s` | `vm-dispatch-infra-followup-v8-v7-splay-after.md` |

## Follow-Up

- Measure prefix collapse from the two-branch post-change state before spending
  opcode space on duplicate prefixed opcodes.
- Re-sample helper return paths after any property/call helper splitting. The
  current `#[inline]` hints alone did not remove those helper symbols.
- If Test262 full-corpus verification continues to run at `-j 12`, consider
  raising the harness timeout or isolating known near-threshold RegExp generated
  tests; the semantics slice passes, but the 1s global timeout is load-sensitive.
