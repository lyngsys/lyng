//! Per-opcode JS source snippets used to drive the microbench inner loop.
//!
//! Each entry is a JS function that exercises the named opcode in a hot
//! `for` loop. The harness compiles the function, calls it with the
//! iteration count, and measures wall time. ns/dispatch = wall_time_ns /
//! (iters * opcodes_per_iter).

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Snippet {
    /// Pascal-case opcode name from `lyng_js_bytecode::Opcode`.
    pub opcode: &'static str,
    /// JS source — a function named `bench` that takes `iters` and runs the loop.
    pub source: &'static str,
    /// Number of times the opcode dispatches per loop iteration. Used to
    /// convert wall time to ns/dispatch.
    pub opcodes_per_iter: u32,
}

/// Hand-maintained snippet table. Add entries as new opcodes need coverage.
/// Snippets that need accurate per-iter counts can be verified by running
/// the snippet under `lyng-js-bench runtime --count-opcodes`.
#[must_use]
pub fn all_snippets() -> HashMap<&'static str, Snippet> {
    let mut map = HashMap::new();

    // Move: a single register-to-register copy per loop body line.
    // The compiler is permitted to fuse Move with other ops; the
    // opcodes_per_iter is verified empirically.
    map.insert("Move", Snippet {
        opcode: "Move",
        source: r"
            function bench(iters) {
                let x = 1;
                for (let i = 0; i < iters; i++) {
                    let a = x;
                    let b = a;
                    let c = b;
                    let d = c;
                    x = d;
                }
                return x;
            }
        ",
        opcodes_per_iter: 4, // 4 Move ops in the loop body (calibrate with --count-opcodes)
    });

    // Add: SMI fast-path arithmetic.
    map.insert("Add", Snippet {
        opcode: "Add",
        source: r"
            function bench(iters) {
                let x = 0;
                for (let i = 0; i < iters; i++) {
                    x = x + 1;
                }
                return x;
            }
        ",
        opcodes_per_iter: 1,
    });

    // Sub: SMI fast-path arithmetic (DSL-1 Phase 1.C.1).
    // Two locals + `x - y` keeps the rhs as a register (Sub) rather
    // than collapsing to `SubSmi` for a literal RHS.
    map.insert("Sub", Snippet {
        opcode: "Sub",
        source: r"
            function bench(iters) {
                let x = 0;
                let y = 1;
                for (let i = 0; i < iters; i++) {
                    x = x - y;
                }
                return x;
            }
        ",
        opcodes_per_iter: 1,
    });

    // Mul: SMI fast-path arithmetic (DSL-1 Phase 1.C.1 Task 3).
    // Two locals + `x * y` keeps the rhs as a register (Mul) rather
    // than collapsing to `MulSmi` for a literal RHS. The reduction
    // (`x = (x * y) | 0`) keeps `x` bounded as a 32-bit signed int
    // so the SMI fast path can stay on every iteration; the trailing
    // `| 0` emits a `BitOr` per iter but it executes the inline shape
    // and is excluded from the per-opcode timing (we measure Mul).
    map.insert("Mul", Snippet {
        opcode: "Mul",
        source: r"
            function bench(iters) {
                let x = 1;
                let y = 3;
                for (let i = 0; i < iters; i++) {
                    x = (x * y) | 0;
                }
                return x;
            }
        ",
        opcodes_per_iter: 1,
    });

    // BitAnd: SMI fast-path bitwise AND (DSL-1 Phase 1.C.2 Task 5).
    // Two locals + `x & y` keeps the rhs as a register (BitAnd) rather
    // than collapsing to `BitAndSmi` for a literal RHS. `bit_and_smi!`
    // has no overflow branch so the fast path is shorter than op_sub's
    // by one instruction. `x` is reset each iteration to a positive SMI
    // so the SMI fast path stays armed indefinitely.
    map.insert("BitAnd", Snippet {
        opcode: "BitAnd",
        source: r"
            function bench(iters) {
                let x = 0;
                let y = 31;
                for (let i = 0; i < iters; i++) {
                    x = i & y;
                }
                return x;
            }
        ",
        opcodes_per_iter: 1,
    });

    // GetNamedProperty: monomorphic property read.
    map.insert("GetNamedProperty", Snippet {
        opcode: "GetNamedProperty",
        source: r"
            function bench(iters) {
                let o = { x: 1, y: 2, z: 3 };
                let s = 0;
                for (let i = 0; i < iters; i++) {
                    s = o.x + o.y + o.z;
                }
                return s;
            }
        ",
        opcodes_per_iter: 3,
    });

    // Jump: pure-jump tight loop.
    map.insert("Jump", Snippet {
        opcode: "Jump",
        source: r"
            function bench(iters) {
                for (let i = 0; i < iters; i++) {}
                return iters;
            }
        ",
        opcodes_per_iter: 1,
    });

    // =====================================================================
    // Phase-1.A opcodes (DSL-1 Phase 1.B.0 Task 7).
    //
    // Constant-loader opcodes. Each loop body writes the constant into
    // four fresh `let` bindings per iteration. The `let` lifetime keeps
    // the destination registers above the slot-0 accumulator window, so
    // the bytecode-builder peephole keeps the non-`Lda*` form (e.g.
    // `LoadNull r8` rather than `LdaNull` for slot-0 writes). The actual
    // dispatch counts are verified empirically against the declared
    // `opcodes_per_iter` (see `verify_opcodes_per_iter` test).
    // =====================================================================

    // `undefined` in JS is a global binding (lookup → LoadGlobal), so a
    // literal `undefined` snippet wouldn't actually drive `LoadUndefined`.
    // The `void X` operator unconditionally emits `LoadUndefined dest`
    // after evaluating its argument for side effects, giving us a clean
    // 4-per-iter driver.
    map.insert("LoadUndefined", Snippet {
        opcode: "LoadUndefined",
        source: r"
            function bench(iters) {
                for (let i = 0; i < iters; i++) {
                    let a = void 0;
                    let b = void 0;
                    let c = void 0;
                    let d = void 0;
                }
                return iters;
            }
        ",
        opcodes_per_iter: 4,
    });

    map.insert("LoadNull", Snippet {
        opcode: "LoadNull",
        source: r"
            function bench(iters) {
                for (let i = 0; i < iters; i++) {
                    let a = null;
                    let b = null;
                    let c = null;
                    let d = null;
                }
                return iters;
            }
        ",
        opcodes_per_iter: 4,
    });

    map.insert("LoadTrue", Snippet {
        opcode: "LoadTrue",
        source: r"
            function bench(iters) {
                for (let i = 0; i < iters; i++) {
                    let a = true;
                    let b = true;
                    let c = true;
                    let d = true;
                }
                return iters;
            }
        ",
        opcodes_per_iter: 4,
    });

    map.insert("LoadFalse", Snippet {
        opcode: "LoadFalse",
        source: r"
            function bench(iters) {
                for (let i = 0; i < iters; i++) {
                    let a = false;
                    let b = false;
                    let c = false;
                    let d = false;
                }
                return iters;
            }
        ",
        opcodes_per_iter: 4,
    });

    map.insert("LoadZero", Snippet {
        opcode: "LoadZero",
        source: r"
            function bench(iters) {
                for (let i = 0; i < iters; i++) {
                    let a = 0;
                    let b = 0;
                    let c = 0;
                    let d = 0;
                }
                return iters;
            }
        ",
        opcodes_per_iter: 4,
    });

    map.insert("LoadOne", Snippet {
        opcode: "LoadOne",
        source: r"
            function bench(iters) {
                for (let i = 0; i < iters; i++) {
                    let a = 1;
                    let b = 1;
                    let c = 1;
                    let d = 1;
                }
                return iters;
            }
        ",
        opcodes_per_iter: 4,
    });

    map.insert("LoadSmi8", Snippet {
        opcode: "LoadSmi8",
        source: r"
            function bench(iters) {
                for (let i = 0; i < iters; i++) {
                    let a = 42;
                    let b = -7;
                    let c = 100;
                    let d = -42;
                }
                return iters;
            }
        ",
        opcodes_per_iter: 4,
    });

    // LoadConst8: 4 distinct float literals per iter. Floats are stored in
    // the per-function constant pool and dispatched via `LoadConst8 dst, idx`
    // (the i8-immediate form of `LoadConst`). i8/i16-range integers would
    // peephole to `LoadSmi8`/`LoadSmi`, so we use `3.14`-style float literals
    // that don't fit any immediate form. The peephole that rewrites
    // `LoadConst dst, idx` → `LoadConst8 dst, idx` fires when `idx <= 255`,
    // which holds trivially for any short snippet (only a handful of
    // constants in the pool).
    //
    // Backfilled in cleanup batch 1 (DSL-1 Phase 1.B cleanup) — the original
    // Phase 1.B.0 Tasks 7+8 commit (`ad240f50`) framing implied this snippet
    // was added but it was not.
    map.insert("LoadConst8", Snippet {
        opcode: "LoadConst8",
        source: r"
            function bench(iters) {
                for (let i = 0; i < iters; i++) {
                    let a = 3.14;
                    let b = 1.5;
                    let c = 2.5;
                    let d = 0.5;
                }
                return iters;
            }
        ",
        opcodes_per_iter: 4,
    });

    // LoadThis: 4 reads of `this` per iter. In a non-arrow function called
    // bare (the harness invokes `bench(iters)` at script level), `this`
    // binds to the global object in sloppy mode (lyng-js scripts are sloppy
    // by default) — but the compiler still emits `LoadThis dst` regardless
    // of the runtime arm. The slot-0 accumulator slot is avoided by
    // assigning to fresh `let` bindings.
    //
    // The harness calls `bench(iters)` from script level, so the
    // `ThisState` at trampoline entry is `Value(globalThis)` — fast path,
    // no sentinel bail. This exercises the inline fast path of
    // `op_load_this_dsl`.
    //
    // Backfilled in cleanup batch 1 (DSL-1 Phase 1.B cleanup) — the original
    // Phase 1.B.0 Tasks 7+8 commit (`ad240f50`) framing implied this snippet
    // was added but it was not.
    map.insert("LoadThis", Snippet {
        opcode: "LoadThis",
        source: r"
            function bench(iters) {
                for (let i = 0; i < iters; i++) {
                    let a = this;
                    let b = this;
                    let c = this;
                    let d = this;
                }
                return iters;
            }
        ",
        opcodes_per_iter: 4,
    });

    // =====================================================================
    // Phase-1.B anchor opcodes (DSL-1 Phase 1.B.0 Task 8).
    //
    // The slot-specialised local loaders (`LoadLocalN`) and the captured-
    // environment loader (`LoadEnvSlot`) are exercised by reading the
    // target slot four times per iteration.
    //
    // Slot placement: function parameters are the only reliable way to
    // land a binding in register slots 1..3. `let` bindings in the lyng-js
    // frame layout begin at slot 4 (slots 0-3 are reserved for the calling
    // convention), and lexical TDZ checks defeat the peephole's Move →
    // LoadLocalN rewrite. Using extra `bench(iters, p1, ...)` parameters
    // we get `p1` at slot 1, `p2` at slot 2, etc., and each `pN`-read
    // peepholes to `LoadLocalN` cleanly.
    //
    // Verified empirically via the dispatch counter (see
    // `verify_opcodes_per_iter` test). Counts within ±5% of declared.
    // =====================================================================

    // LoadLocal0: bench(iters)'s `iters` parameter sits at register 0 in
    // the lyng-js calling convention. Reading `iters` four times per iter
    // emits `LoadLocal0 dst, r0` because the peephole prefers the slot-0
    // specialized form over a generic `Move dst, r0`. Plus the loop's
    // `i < iters` test loads `iters` once more per iteration, yielding
    // 5 LoadLocal0 dispatches per iter total (verified empirically:
    // 5001 dispatches for ITERS=1000 in the verify_opcodes_per_iter test).
    map.insert("LoadLocal0", Snippet {
        opcode: "LoadLocal0",
        source: r"
            function bench(iters) {
                let s = 0;
                for (let i = 0; i < iters; i++) {
                    s = iters + iters + iters + iters;
                }
                return s;
            }
        ",
        opcodes_per_iter: 5,
    });

    // LoadLocal1: read parameter `p1` (slot 1) four times per iter.
    //
    // Function parameters reliably land at register slots 0..N-1, while
    // `let` bindings are allocated at slots >= 4 in the lyng-js frame
    // layout (slots 0-3 are reserved for the calling convention). The
    // only way to drive `LoadLocalN` for N in 1..3 is via parameters.
    map.insert("LoadLocal1", Snippet {
        opcode: "LoadLocal1",
        source: r"
            function bench(iters, p1) {
                let s = 0;
                for (let i = 0; i < iters; i++) {
                    s = p1 + p1 + p1 + p1;
                }
                return s;
            }
        ",
        opcodes_per_iter: 4,
    });

    // LoadLocal2: read parameter `p2` (slot 2) four times per iter.
    map.insert("LoadLocal2", Snippet {
        opcode: "LoadLocal2",
        source: r"
            function bench(iters, p1, p2) {
                let s = 0;
                for (let i = 0; i < iters; i++) {
                    s = p2 + p2 + p2 + p2;
                }
                return s + p1;
            }
        ",
        opcodes_per_iter: 4,
    });

    // LoadLocal3: read parameter `p3` (slot 3) four times per iter.
    map.insert("LoadLocal3", Snippet {
        opcode: "LoadLocal3",
        source: r"
            function bench(iters, p1, p2, p3) {
                let s = 0;
                for (let i = 0; i < iters; i++) {
                    s = p3 + p3 + p3 + p3;
                }
                return s + p1 + p2;
            }
        ",
        opcodes_per_iter: 4,
    });

    // StoreLocal3: four stores to a slot-3 location per iter. Same trick
    // as LoadLocalN — parameters live in slots 0..N-1, and the peephole
    // rewrites `Move dst=3, src=...` to `StoreLocal3`. We use a write to
    // a parameter `p3` (which JS permits — parameters are mutable bindings).
    map.insert("StoreLocal3", Snippet {
        opcode: "StoreLocal3",
        source: r"
            function bench(iters, p1, p2, p3) {
                for (let i = 0; i < iters; i++) {
                    p3 = i;
                    p3 = i;
                    p3 = i;
                    p3 = i;
                }
                return p1 + p2 + p3;
            }
        ",
        opcodes_per_iter: 4,
    });

    // StoreLocal1/2: symmetric pairs of StoreLocal3 (DSL-1 Phase 1.B.3
    // Task 4). Each writes to the corresponding parameter slot four
    // times per iter; the bytecode-builder peephole
    // (`compact_move_instruction` in `crates/lyng-js/bytecode/src/
    // builder.rs:150-166`) rewrites `Move dst=N, src=...` to
    // `StoreLocalN` for N in 1..3 just as it does for N=3.
    //
    // **StoreLocal0 is intentionally omitted from this list** — the
    // peephole's `dst==0` branch fires BEFORE `store_local_opcode`,
    // rewriting `Move dst=0, src=B` to `Ldar B` (load accumulator
    // from register B). Slot 0 is the accumulator by the calling
    // convention; emitting an explicit `StoreLocal0` would be
    // redundant with `Ldar`. The handler exists (and is inline-ported
    // in DSL-1 Phase 1.B.3 Task 3 for symmetry with the
    // `store_local_fixed!` macro), but is unreachable via the standard
    // emit pipeline — see the per-handler report at
    // `reports/js/lyng-js/dsl-handlers/op_store_local_0.md` for the
    // detailed finding.
    map.insert("StoreLocal1", Snippet {
        opcode: "StoreLocal1",
        source: r"
            function bench(iters, p1, p2, p3) {
                for (let i = 0; i < iters; i++) {
                    p1 = i;
                    p1 = i;
                    p1 = i;
                    p1 = i;
                }
                return p1 + p2 + p3;
            }
        ",
        opcodes_per_iter: 4,
    });

    map.insert("StoreLocal2", Snippet {
        opcode: "StoreLocal2",
        source: r"
            function bench(iters, p1, p2, p3) {
                for (let i = 0; i < iters; i++) {
                    p2 = i;
                    p2 = i;
                    p2 = i;
                    p2 = i;
                }
                return p1 + p2 + p3;
            }
        ",
        opcodes_per_iter: 4,
    });

    // LoadEnvSlot: inner closure reads a captured variable four times per
    // iter. The captured var lives in the enclosing environment, so each
    // read dispatches LoadEnvSlot rather than LoadLocalN. The outer loop
    // also performs two LoadEnvSlot dispatches per iteration: the loop's
    // induction `i` lives in the iteration env (one load for `i < iters`,
    // one more for the `s = inner()` callee lookup), yielding 6 LoadEnvSlot
    // dispatches per iter total (verified empirically: 6001 dispatches for
    // ITERS=1000 in the verify_opcodes_per_iter test).
    map.insert("LoadEnvSlot", Snippet {
        opcode: "LoadEnvSlot",
        source: r"
            function bench(iters) {
                let captured = 7;
                function inner() {
                    return captured + captured + captured + captured;
                }
                let s = 0;
                for (let i = 0; i < iters; i++) {
                    s = inner();
                }
                return s;
            }
        ",
        opcodes_per_iter: 6,
    });

    // Ldar: the accumulator load. Emitted by the bytecode-builder peephole
    // when an emitted `Move` has destination register 0 — i.e. writes back
    // into the first parameter (the accumulator slot). Mutating the first
    // parameter via `bench_param = X` produces `Move dst=0, src=tmp` which
    // the peephole rewrites to `Ldar tmp`.
    //
    // The driver uses a second parameter `iters_bound` (slot 1) for the
    // loop bound and reads slot 1 with `LoadLocal1` so the loop's exit
    // condition stays decoupled from the slot-0 mutation. The harness
    // only passes one argument (`iters`), so `iters_bound` is undefined;
    // the snippet rebinds it to `iters` on entry.
    map.insert("Ldar", Snippet {
        opcode: "Ldar",
        source: r"
            function bench(p0) {
                let n = p0;
                let v = 0;
                for (let i = 0; i < n; i++) {
                    p0 = v;
                    p0 = v;
                    p0 = v;
                    p0 = v;
                }
                return p0;
            }
        ",
        opcodes_per_iter: 4,
    });

    // Add additional snippets as needed for the hot-30 set.
    // For opcodes not present here, the microbench skips with a warning
    // (and the report records "no snippet" for that opcode).

    map
}

/// Look up a snippet by opcode name.
#[must_use]
pub fn for_opcode(name: &str) -> Option<Snippet> {
    all_snippets().get(name).cloned()
}

#[cfg(test)]
mod verify_counts {
    //! Verify per-snippet opcode counts against declared `opcodes_per_iter`.
    //!
    //! Run with: `cargo test --release -p lyng-js-bench --features
    //! lyng-js-vm/opcode-counters verify_opcodes_per_iter -- --nocapture`.
    //!
    //! The test compiles each snippet, runs one bench(iters) call with a
    //! small iters value, and snapshots the dispatch counts. It then
    //! prints the top opcodes for that snippet and asserts that the
    //! declared opcode's actual dispatch count matches `iters *
    //! opcodes_per_iter`, modulo per-call setup overhead.

    use super::*;
    use lyng_js_builtins::BootstrapMode;
    use lyng_js_bytecode::disassemble;
    use lyng_js_common::{AtomTable, SourceId};
    use lyng_js_compiler::compile_script;
    use lyng_js_env::Runtime;
    use lyng_js_host::NoopHostHooks;
    use lyng_js_parser::parse_script;
    use lyng_js_sema::analyze_script;
    use lyng_js_vm::Vm;

    fn run_one(snippet: &Snippet, iters: u64) -> Vec<(String, u64)> {
        let src = format!("{}\nbench({});\n", snippet.source, iters);

        let mut atoms = AtomTable::new();
        let source_id = SourceId::new(1);
        let parsed = parse_script(&mut atoms, source_id, &src);
        assert!(
            !parsed.diagnostics.has_errors(),
            "parse errors for {}: {:?}",
            snippet.opcode,
            parsed.diagnostics.as_slice()
        );
        let sema = analyze_script(&parsed, &atoms);
        assert!(
            !sema.diagnostics.has_errors(),
            "sema errors for {}: {:?}",
            snippet.opcode,
            sema.diagnostics.as_slice()
        );
        let unit = compile_script(&parsed, &sema, &mut atoms)
            .unwrap_or_else(|err| panic!("lowering failed for {}: {err:?}", snippet.opcode));

        // Set DUMP_SNIPPETS=1 to print the disassembled functions; useful
        // when calibrating a new snippet's opcodes_per_iter.
        if std::env::var("DUMP_SNIPPETS").is_ok() {
            for func in unit.functions() {
                eprintln!("=== {} ===", snippet.opcode);
                eprintln!("{}", disassemble(func));
            }
        }

        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let realm = agent
            .default_realm()
            .expect("default realm should exist for snippet verification");
        let realm_id = realm.id();
        let mut vm = Vm::new();
        vm.bootstrap_realm(agent, realm_id, BootstrapMode::SpecOnly)
            .unwrap_or_else(|err| panic!("spec bootstrap failed: {err:?}"));
        let installed = vm
            .install_script(agent, realm_id, &unit)
            .unwrap_or_else(|err| {
                panic!("install_script failed for {}: {err:?}", snippet.opcode)
            });
        Vm::instantiate_global_script(agent, &realm, unit.instantiation_plan()).unwrap_or_else(
            |err| panic!("instantiate_global_script failed for {}: {err:?}", snippet.opcode),
        );

        // Warmup once, then reset and measure a single call.
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap_or_else(|err| panic!("warmup eval failed for {}: {err:?}", snippet.opcode));

        vm.reset_opcode_dispatch_counts();
        vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .unwrap_or_else(|err| panic!("measured eval failed for {}: {err:?}", snippet.opcode));
        let counts = vm
            .opcode_dispatch_counts()
            .expect("opcode-counters feature should provide counts");
        counts
            .top(8)
            .iter()
            .map(|entry| (entry.opcode().name().to_string(), entry.count()))
            .collect()
    }

    fn count_of(top: &[(String, u64)], name: &str) -> u64 {
        top.iter()
            .find(|(opname, _)| opname == name)
            .map_or(0, |(_, count)| *count)
    }

    #[test]
    fn verify_opcodes_per_iter() {
        // Use a small inner-iter count so the measured snippet runs fast.
        const ITERS: u64 = 1_000;

        let snippets = all_snippets();
        // Verify the newly-added Phase-1.A and Phase-1.B anchor opcodes.
        // `LoadConst8` and `LoadThis` were backfilled in DSL-1 Phase 1.B
        // cleanup batch 1 — the Phase 1.B.0 framing implied they were
        // present but they were not.
        let names = [
            "LoadUndefined",
            "LoadNull",
            "LoadTrue",
            "LoadFalse",
            "LoadZero",
            "LoadOne",
            "LoadSmi8",
            "LoadConst8",
            "LoadThis",
            "LoadLocal0",
            "LoadLocal1",
            "LoadLocal2",
            "LoadLocal3",
            // Phase 1.B.3 Task 4: StoreLocal1/2 backfilled to close the
            // snippets-coverage gap (StoreLocal3 was the only one
            // present in Phase 1.B.0). **StoreLocal0 is intentionally
            // omitted** — the bytecode-builder peephole rewrites
            // `Move dst=0, src=B` to `Ldar B` before the
            // `store_local_opcode` branch fires, so StoreLocal0 cannot
            // be emitted via the standard pipeline. See the per-handler
            // report at `reports/js/lyng-js/dsl-handlers/op_store_local_0.md`.
            "StoreLocal1",
            "StoreLocal2",
            "StoreLocal3",
            "LoadEnvSlot",
            "Ldar",
        ];

        let mut report = String::new();
        let mut bad: Vec<String> = Vec::new();
        for name in names {
            let snippet = snippets.get(name).unwrap_or_else(|| {
                panic!("snippet for {name} missing");
            });
            let top = run_one(snippet, ITERS);
            let declared_per_iter = u64::from(snippet.opcodes_per_iter);
            let expected = ITERS * declared_per_iter;
            let actual = count_of(&top, name);
            let ratio = if expected == 0 {
                0.0
            } else {
                actual as f64 / expected as f64
            };
            report.push_str(&format!(
                "[{name:>14}] declared per-iter={declared_per_iter} expected={expected} \
                 actual_for_op={actual} ratio={ratio:.3}\n  top: {top:?}\n",
            ));
            // Require actual count to be within ±5% of declared expected,
            // and at least one dispatch of the named opcode.
            if actual == 0 {
                bad.push(format!(
                    "{name}: expected at least {expected} dispatches of `{name}`, got 0 \
                     (top: {top:?})"
                ));
            } else if ratio < 0.95 || ratio > 1.05 {
                bad.push(format!(
                    "{name}: expected {expected} dispatches of `{name}` (within 5%), got {actual} \
                     (ratio={ratio:.3}, top: {top:?})"
                ));
            }
        }
        println!("{report}");
        assert!(bad.is_empty(), "snippet verification failures:\n{}", bad.join("\n"));
    }
}
