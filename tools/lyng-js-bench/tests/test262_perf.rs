use std::time::Duration;

use lyng_js_bench::test262::{
    aggregate_sampled_variants, cause_hints_for_aggregate, parse_options_for_test,
    render_json_report, render_markdown_report, Test262Aggregate, Test262Mode, Test262Options,
    Test262PhaseTimings, Test262Sample, Test262VariantDiagnostics, Test262VariantIdentity,
};
use serde_json::json;

fn sample(elapsed_ms: u64, eval_ms: u64) -> Test262Sample {
    Test262Sample {
        outcome: "pass".to_string(),
        timings: Test262PhaseTimings {
            total: Duration::from_millis(elapsed_ms),
            evaluation: Duration::from_millis(eval_ms),
            parse: Duration::from_millis(3),
            sema: Duration::from_millis(2),
            lowering: Duration::from_millis(1),
            ..Test262PhaseTimings::default()
        },
        diagnostics: Some(Test262VariantDiagnostics {
            function_count: 4,
            instruction_words: 120,
            wide_operands: 8,
            metadata_records: 17,
            feedback_slots: 6,
            live_feedback_sites: 5,
            megamorphic_sites: 2,
            tier_hotness: 14,
            runtime_live_bytes_delta: 4096,
            ..Test262VariantDiagnostics::default()
        }),
    }
}

#[test]
fn aggregates_samples_for_agent_triage() {
    let identity = Test262VariantIdentity {
        file: "built-ins/Date/dst-offset-caching-2-of-8.js".to_string(),
        variant: Some("strict".to_string()),
        category: "built-ins".to_string(),
        flags: vec!["generated".to_string()],
        features: vec!["Date".to_string()],
        includes: Vec::new(),
        negative_phase: None,
        async_test: false,
        module_goal: false,
        timeout_ms: 1_000,
    };
    let aggregate = aggregate_sampled_variants(vec![(
        identity,
        vec![sample(900, 810), sample(800, 700), sample(1_000, 930)],
    )]);

    assert_eq!(aggregate.len(), 1);
    assert_eq!(aggregate[0].median_total, Duration::from_millis(900));
    assert_eq!(aggregate[0].min_total, Duration::from_millis(800));
    assert_eq!(aggregate[0].max_total, Duration::from_millis(1_000));
    assert_eq!(aggregate[0].dominant_phase, "evaluation");
    assert!(aggregate[0]
        .cause_hints
        .contains(&"evaluation dominated".to_string()));
    assert!(aggregate[0]
        .cause_hints
        .contains(&"Date/timezone candidate".to_string()));
    assert!(aggregate[0]
        .cause_hints
        .contains(&"megamorphic inline-cache activity".to_string()));
}

#[test]
fn renders_markdown_and_json_for_agents() {
    let options = Test262Options::default_for_test();
    let aggregate = Test262Aggregate {
        identity: Test262VariantIdentity {
            file: "staging/sm/Date/dst-offset-caching-3-of-8.js".to_string(),
            variant: Some("non-strict".to_string()),
            category: "staging".to_string(),
            flags: Vec::new(),
            features: Vec::new(),
            includes: vec!["sta.js".to_string()],
            negative_phase: None,
            async_test: false,
            module_goal: false,
            timeout_ms: 60_000,
        },
        samples: vec![sample(950, 900), sample(850, 805)],
        median_total: Duration::from_millis(950),
        min_total: Duration::from_millis(850),
        max_total: Duration::from_millis(950),
        median_evaluation: Duration::from_millis(900),
        dominant_phase: "evaluation".to_string(),
        cause_hints: cause_hints_for_aggregate(
            "staging/sm/Date/dst-offset-caching-3-of-8.js",
            "evaluation",
            Some(&sample(950, 900).diagnostics.unwrap()),
            Duration::from_millis(950),
            Duration::from_millis(850),
            Duration::from_millis(950),
            60_000,
        ),
    };

    let markdown = render_markdown_report(&options, std::slice::from_ref(&aggregate), None);
    assert!(markdown.contains("# Lyng JS Test262 Performance Diagnostics"));
    assert!(markdown.contains("| `staging/sm/Date/dst-offset-caching-3-of-8.js [non-strict]` |"));
    assert!(markdown.contains("Date/timezone candidate"));
    assert!(markdown.contains("Dominant phase"));

    let json = render_json_report(&options, &[aggregate], None);
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["aggregates"][0]["dominant_phase"], "evaluation");
    assert_eq!(
        json["aggregates"][0]["samples"][0]["diagnostics"]["feedback_slots"],
        6
    );
    assert!(json["aggregates"][0]["cause_hints"]
        .as_array()
        .unwrap()
        .iter()
        .any(|hint| hint == "Date/timezone candidate"));
}

#[test]
fn reports_deltas_from_previous_agent_json() {
    let options = Test262Options::default_for_test();
    let aggregate = Test262Aggregate {
        identity: Test262VariantIdentity {
            file: "built-ins/Date/example.js".to_string(),
            variant: Some("strict".to_string()),
            category: "built-ins".to_string(),
            flags: Vec::new(),
            features: Vec::new(),
            includes: Vec::new(),
            negative_phase: None,
            async_test: false,
            module_goal: false,
            timeout_ms: 1_000,
        },
        samples: vec![sample(120, 100)],
        median_total: Duration::from_millis(120),
        min_total: Duration::from_millis(120),
        max_total: Duration::from_millis(120),
        median_evaluation: Duration::from_millis(100),
        dominant_phase: "evaluation".to_string(),
        cause_hints: vec!["evaluation dominated".to_string()],
    };
    let previous = json!({
        "aggregates": [{
            "identity": {
                "file": "built-ins/Date/example.js",
                "variant": "strict"
            },
            "median_total_ms": 90.0
        }]
    });

    let markdown =
        render_markdown_report(&options, std::slice::from_ref(&aggregate), Some(&previous));
    assert!(markdown.contains("Median total delta"));
    assert!(markdown.contains("+30.000ms"));

    let json = render_json_report(&options, &[aggregate], Some(&previous));
    assert_eq!(json["aggregates"][0]["delta"]["median_total_ms"], 30.0);
}

#[test]
fn refined_runtime_phases_drive_dominant_phase_and_reports() {
    let options = Test262Options::default_for_test();
    let identity = Test262VariantIdentity {
        file: "staging/sm/Date/dst-offset-caching-2-of-8.js".to_string(),
        variant: None,
        category: "staging".to_string(),
        flags: Vec::new(),
        features: Vec::new(),
        includes: Vec::new(),
        negative_phase: None,
        async_test: false,
        module_goal: false,
        timeout_ms: 3_000,
    };
    let aggregate = aggregate_sampled_variants(vec![(
        identity,
        vec![Test262Sample {
            outcome: "pass".to_string(),
            timings: Test262PhaseTimings {
                script_install: Duration::from_millis(4),
                realm_bootstrap: Duration::from_millis(6),
                extension_install: Duration::from_millis(8),
                global_instantiation: Duration::from_millis(10),
                bytecode_execution: Duration::from_millis(120),
                job_checkpoint: Duration::from_millis(12),
                evaluation: Duration::from_millis(160),
                total: Duration::from_millis(190),
                ..Test262PhaseTimings::default()
            },
            diagnostics: None,
        }],
    )]);

    assert_eq!(aggregate[0].dominant_phase, "bytecode_execution");
    assert!(aggregate[0]
        .cause_hints
        .contains(&"bytecode execution dominated".to_string()));

    let markdown = render_markdown_report(&options, &aggregate, None);
    assert!(markdown.contains("bytecode_execution"));

    let json = render_json_report(&options, &aggregate, None);
    assert_eq!(
        json["aggregates"][0]["samples"][0]["timings"]["bytecode_execution_ms"],
        120.0
    );
    assert_eq!(
        json["aggregates"][0]["samples"][0]["timings"]["job_checkpoint_ms"],
        12.0
    );
    assert_eq!(
        json["aggregates"][0]["samples"][0]["timings"]["evaluation_ms"],
        160.0
    );
}

#[test]
fn test262_options_support_named_smoke_preset() {
    let options = parse_options_for_test(&[
        "--preset".to_string(),
        "smoke".to_string(),
        "--filter".to_string(),
        "staging/sm/Date/dst-offset-caching-2-of-8".to_string(),
    ])
    .expect("test262 smoke preset should parse");

    assert_eq!(options.mode, Test262Mode::Hybrid);
    assert_eq!(options.samples, 1);
    assert_eq!(options.warmup_samples, 0);
    assert_eq!(options.sample_files, 2);
    assert_eq!(options.timeout_ms, 3_000);
    assert_eq!(
        options.filter.as_deref(),
        Some("staging/sm/Date/dst-offset-caching-2-of-8")
    );
}
