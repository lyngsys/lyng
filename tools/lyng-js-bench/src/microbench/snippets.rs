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
