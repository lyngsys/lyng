//! Public API boundary checks for Lyng JS runtime crates.

use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("test crate should live under crates/lyng-js/tests")
        .to_path_buf()
}

#[test]
fn obsolete_marker_apis_are_not_part_of_runtime_public_surface() {
    let root = workspace_root();
    let forbidden = [
        (
            "crates/lyng-js/types/src/lib.rs",
            "pub use marker::TypeOwnershipMarker",
        ),
        (
            "crates/lyng-js/types/src/marker.rs",
            "pub struct TypeOwnershipMarker",
        ),
        (
            "crates/lyng-js/gc/src/lib.rs",
            "pub struct PrimitiveHeapMarker",
        ),
        (
            "crates/lyng-js/ops/src/lib.rs",
            "pub use marker::PrimitiveOpsMarker",
        ),
        (
            "crates/lyng-js/ops/src/marker.rs",
            "pub struct PrimitiveOpsMarker",
        ),
        ("crates/lyng-js/host/src/lib.rs", "pub struct HostMarker"),
        (
            "crates/lyng-js/objects/src/lib.rs",
            "pub use self::marker::ObjectSubstrateMarker",
        ),
        (
            "crates/lyng-js/objects/src/marker.rs",
            "pub struct ObjectSubstrateMarker",
        ),
        ("crates/lyng-js/env/src/lib.rs", "RuntimeSubstrateMarker"),
        (
            "crates/lyng-js/env/src/runtime.rs",
            "pub struct RuntimeSubstrateMarker",
        ),
        (
            "crates/lyng-js/builtins/src/lib.rs",
            "pub struct BuiltinsMarker",
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
