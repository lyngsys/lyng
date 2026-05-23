#[test]
fn llint_handlers_do_not_use_legacy_bridge_terminology() {
    let cold_handlers = include_str!("../dsl/handlers/cold.rs");
    let hot_handlers = include_str!("../dsl/handlers/hot.rs");
    let lowerer = include_str!("../../../vm-dsl/src/lower.rs");
    let forbidden_bridge = concat!("call_", "fast!");
    let forbidden_helper_suffix = concat!("_fast", "_rs");

    assert!(
        !cold_handlers.contains(forbidden_bridge),
        "Rust helper probes from LLInt handlers must be called call_rust_probe!, not the legacy bridge macro"
    );
    assert!(
        !hot_handlers.contains(forbidden_bridge),
        "hot LLInt handlers must not use the legacy Rust bridge macro"
    );
    assert!(
        !lowerer.contains(forbidden_bridge),
        "vm-dsl bridge collection must reserve hit-path terminology for inline LLInt code"
    );
    assert!(
        !cold_handlers.contains(forbidden_helper_suffix),
        "LLInt Rust bridge helpers must be named as probes or slow paths"
    );
}

#[test]
fn llint_rust_probes_are_explicitly_enumerated() {
    let cold_handlers = include_str!("../dsl/handlers/cold.rs");
    let probe_call_count = cold_handlers.matches("call_rust_probe!(").count();

    assert!(
        cold_handlers.contains("call_rust_probe!(op_load_global_rust_probe_rs"),
        "LoadGlobal is an explicit temporary Rust probe until its LLInt IC layout exists"
    );
    assert!(
        cold_handlers.contains("call_rust_probe!(op_assign_named_property_rust_probe_rs"),
        "AssignNamedProperty is an explicit temporary Rust probe until its LLInt store IC layout exists"
    );
    assert_eq!(
        probe_call_count, 2,
        "Rust probe bridges are not LLInt fast paths; only LoadGlobal and AssignNamedProperty are currently allowed"
    );
}

#[test]
fn rust_vm_hot_paths_do_not_use_llint_fast_path_terminology() {
    let files = [
        ("vm/feedback.rs", include_str!("../vm/feedback.rs")),
        (
            "vm/dispatch/property.rs",
            include_str!("../vm/dispatch/property.rs"),
        ),
        ("vm/names.rs", include_str!("../vm/names.rs")),
        ("vm/call.rs", include_str!("../vm/call.rs")),
        (
            "vm/bytecode_calls.rs",
            include_str!("../vm/bytecode_calls.rs"),
        ),
        (
            "vm/builtin_dispatch/dispatch_context/public.rs",
            include_str!("../vm/builtin_dispatch/dispatch_context/public.rs"),
        ),
        (
            "vm/builtin_dispatch/dispatch_context/support.rs",
            include_str!("../vm/builtin_dispatch/dispatch_context/support.rs"),
        ),
        (
            "vm/builtin_dispatch/function_helpers.rs",
            include_str!("../vm/builtin_dispatch/function_helpers.rs"),
        ),
    ];

    for (path, contents) in files {
        assert!(
            !contents.contains("fast path")
                && !contents.contains("fast-path")
                && !contents.contains("Fast path")
                && !contents.contains("Fast-path"),
            "{path} must reserve fast-path wording for inline LLInt DSL handlers"
        );
        assert!(
            !contents.contains("fast_") && !contents.contains("_fast"),
            "{path} must name Rust hit paths as direct/cache/specialized paths, not fast paths"
        );
    }
}
