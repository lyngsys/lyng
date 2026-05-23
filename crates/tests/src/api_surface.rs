//! Public API boundary checks for Lyng JS runtime crates.

use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("test crate should live under crates/tests")
        .to_path_buf()
}

#[test]
fn obsolete_marker_apis_are_not_part_of_runtime_public_surface() {
    let root = workspace_root();
    let forbidden = [
        (
            "crates/types/src/lib.rs",
            "pub use marker::TypeOwnershipMarker",
        ),
        (
            "crates/types/src/marker.rs",
            "pub struct TypeOwnershipMarker",
        ),
        ("crates/gc/src/lib.rs", "pub struct PrimitiveHeapMarker"),
        (
            "crates/ops/src/lib.rs",
            "pub use marker::PrimitiveOpsMarker",
        ),
        ("crates/ops/src/marker.rs", "pub struct PrimitiveOpsMarker"),
        ("crates/host/src/lib.rs", "pub struct HostMarker"),
        (
            "crates/objects/src/lib.rs",
            "pub use self::marker::ObjectSubstrateMarker",
        ),
        (
            "crates/objects/src/marker.rs",
            "pub struct ObjectSubstrateMarker",
        ),
        ("crates/env/src/lib.rs", "RuntimeSubstrateMarker"),
        (
            "crates/env/src/runtime.rs",
            "pub struct RuntimeSubstrateMarker",
        ),
        ("crates/builtins/src/lib.rs", "pub struct BuiltinsMarker"),
        (
            "crates/bytecode/src/function.rs",
            "pub use marker::BytecodeMarker",
        ),
        (
            "crates/bytecode/src/function/marker.rs",
            "pub struct BytecodeMarker",
        ),
        ("crates/bytecode/src/lib.rs", "BytecodeMarker"),
        ("crates/compiler/src/lib.rs", "pub struct CompilerMarker"),
        (
            "crates/compiler/src/lib.rs",
            "pub const fn installable_script_unit",
        ),
        (
            "crates/compiler/src/lib.rs",
            "pub const fn installable_module_unit",
        ),
        (
            "crates/compiler/src/lib.rs",
            "pub const fn installable_function_unit",
        ),
        ("crates/vm/src/lib.rs", "pub use marker::VmMarker"),
        ("crates/vm/src/marker.rs", "pub struct VmMarker"),
        (
            "tools/lyng-bench/src/runtime.rs",
            concat!("module-heavy.", "place", "holder", "-compile"),
        ),
        (
            "tools/lyng-bench/src/runtime.rs",
            concat!("place", "holder compile_module"),
        ),
    ];

    for (relative_path, snippet) in forbidden {
        let path = root.join(relative_path);
        if !path.exists() {
            continue;
        }

        let source = std::fs::read_to_string(&path).expect("source file should be readable");
        assert!(
            !source.contains(snippet),
            "{relative_path} still exposes obsolete marker API snippet `{snippet}`"
        );
    }
}
