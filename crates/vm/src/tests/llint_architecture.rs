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
fn llint_rust_probe_hits_use_no_refresh_dispatch() {
    let cold_handlers = include_str!("../dsl/handlers/cold.rs");

    assert_eq!(
        cold_handlers.matches("dispatch_probe_hit_no_refresh!();").count(),
        2,
        "LoadGlobal and AssignNamedProperty probe hits must use the documented no-refresh dispatch form"
    );
    assert!(
        !cold_handlers.contains("dispatch_from_payload!();"),
        "probe-hit dispatch should use a name that carries the no-refresh contract"
    );
}

#[test]
fn llint_rust_probe_no_refresh_dispatch_does_not_reload_frame_pins() {
    let backend = include_str!("../dsl/backend/aarch64/control.rs");
    let start = backend
        .find("macro_rules! dispatch_probe_hit_no_refresh")
        .expect("backend must define the no-refresh Rust probe hit dispatch macro");
    let rest = &backend[start..];
    let end = rest
        .find("// ===========================================================================\n// Branches.")
        .expect("dispatch macro section should end before branch helpers");
    let dispatch_macro = &rest[..end];

    assert!(
        dispatch_macro.contains("no frame switch")
            && dispatch_macro.contains("no register-stack relocation")
            && dispatch_macro.contains("no feedback-vector relocation"),
        "no-refresh probe-hit dispatch must document the contract that makes skipping REGS/FV reloads valid"
    );
    assert!(
        !dispatch_macro.contains("{state_regs}") && !dispatch_macro.contains("{state_fv}"),
        "probe-hit no-refresh dispatch must not reload REGS/FV from LlIntState"
    );
    assert!(
        !dispatch_macro.contains("x20") && !dispatch_macro.contains("x21"),
        "probe-hit no-refresh dispatch must leave pinned REGS/FV untouched"
    );
}

#[test]
fn llint_handlers_do_not_use_hit_side_feedback_bridges() {
    let cold_handlers = include_str!("../dsl/handlers/cold.rs");
    let hot_handlers = include_str!("../dsl/handlers/hot.rs");
    let forbidden = "_record_smi_rs";

    assert!(
        !cold_handlers.contains(forbidden),
        "SMI arithmetic LLInt hits must record feedback through asm-visible flat feedback, not Rust feedback shims"
    );
    assert!(
        !hot_handlers.contains(forbidden),
        "hot LLInt handlers must not call hit-side Rust feedback shims"
    );
}

#[test]
fn llint_feedback_addressing_uses_precomputed_entry_offset() {
    let feedback_backend = include_str!("../dsl/backend/aarch64/feedback.rs");

    // Phase C precomputed-offset optimization: all four feedback macros
    // (load_feedback_site!, record_smi!, record_object!, record_double!)
    // resolve a slot to its entry via a 3-instruction sequence:
    //   sub  x17, x{slot}, #1
    //   ldr  w16, [x21, x17, lsl #2]   ← reads slot_to_entry_offset table
    //   add  x{dst}, x21, x16          ← buffer_base + precomputed_offset
    // No in-buffer header/kind-offsets dispatch; no stride shifts in asm.
    assert!(
        feedback_backend.contains("ldr    w16, [x21, x17, lsl #2]"),
        "LLInt feedback slot addressing should load the precomputed entry offset via [x21, x17, lsl #2]"
    );
    assert!(
        !feedback_backend.contains("#{mt_slot_index_table_offset}"),
        "LLInt feedback macros must not reference the old slot-index table offset binding"
    );
    assert!(
        !feedback_backend.contains("#{mt_kind_offsets_offset}"),
        "LLInt feedback macros must not reference the old kind-offsets table offset binding"
    );
    assert!(
        !feedback_backend.contains("#{arith_metadata_stride_shift}"),
        "LLInt record_* macros must not shift by stride after precomputed-offset optimization"
    );
    assert!(
        !feedback_backend.contains("#{property_metadata_stride_shift}"),
        "LLInt load_feedback_site! must not shift by stride after precomputed-offset optimization"
    );
    assert!(
        !feedback_backend.contains("feedback_entry_stride} & 0xffff"),
        "LLInt feedback hot paths must not materialize the FeedbackEntry stride with movz/movk"
    );
    assert!(
        !feedback_backend.contains("madd   x16, x17, x16, x21"),
        "LLInt feedback hot paths should not multiply by a materialized stride"
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
