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

    assert_eq!(
        probe_call_count, 3,
        "Rust probe bridges are not LLInt fast paths; update this audit when one is ported or added"
    );
}
