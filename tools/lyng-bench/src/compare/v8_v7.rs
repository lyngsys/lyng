use super::{CompareResult, MetricKind, Workload};

const BASE: &str = include_str!("../../../../testdata/js-benchmarks/v8-v7/base.js");
const RICHARDS: &str = include_str!("../../../../testdata/js-benchmarks/v8-v7/richards.js");
const DELTABLUE: &str = include_str!("../../../../testdata/js-benchmarks/v8-v7/deltablue.js");
const CRYPTO: &str = include_str!("../../../../testdata/js-benchmarks/v8-v7/crypto.js");
const RAYTRACE: &str = include_str!("../../../../testdata/js-benchmarks/v8-v7/raytrace.js");
const EARLEY_BOYER: &str = include_str!("../../../../testdata/js-benchmarks/v8-v7/earley-boyer.js");
const REGEXP: &str = include_str!("../../../../testdata/js-benchmarks/v8-v7/regexp.js");
const SPLAY: &str = include_str!("../../../../testdata/js-benchmarks/v8-v7/splay.js");
const NAVIER_STOKES: &str =
    include_str!("../../../../testdata/js-benchmarks/v8-v7/navier-stokes.js");

struct BenchmarkSpec {
    name: &'static str,
    slug: &'static str,
    source: &'static str,
}

const BENCHMARKS: &[BenchmarkSpec] = &[
    BenchmarkSpec {
        name: "Richards",
        slug: "richards",
        source: RICHARDS,
    },
    BenchmarkSpec {
        name: "DeltaBlue",
        slug: "deltablue",
        source: DELTABLUE,
    },
    BenchmarkSpec {
        name: "Crypto",
        slug: "crypto",
        source: CRYPTO,
    },
    BenchmarkSpec {
        name: "RayTrace",
        slug: "raytrace",
        source: RAYTRACE,
    },
    BenchmarkSpec {
        name: "EarleyBoyer",
        slug: "earley-boyer",
        source: EARLEY_BOYER,
    },
    BenchmarkSpec {
        name: "RegExp",
        slug: "regexp",
        source: REGEXP,
    },
    BenchmarkSpec {
        name: "Splay",
        slug: "splay",
        source: SPLAY,
    },
    BenchmarkSpec {
        name: "NavierStokes",
        slug: "navier-stokes",
        source: NAVIER_STOKES,
    },
];

pub(super) fn build_workloads(
    filter: Option<&str>,
    full_suite: bool,
) -> CompareResult<Vec<Workload>> {
    if full_suite {
        return Ok(vec![Workload {
            name: "Score",
            category: "v8-v7",
            file_name: "v8-v7-full-suite.js",
            source: full_suite_bundle(),
            metric_kind: MetricKind::Score,
            requires_lyng_shell: true,
        }]);
    }

    let workloads = BENCHMARKS
        .iter()
        .filter(|spec| filter.is_none_or(|filter| benchmark_matches(spec, filter)))
        .map(|spec| Workload {
            name: spec.name,
            category: "v8-v7",
            file_name: file_name(spec),
            source: benchmark_bundle(spec.name, spec.source),
            metric_kind: MetricKind::Score,
            requires_lyng_shell: true,
        })
        .collect::<Vec<_>>();

    if workloads.is_empty() {
        return Err(format!(
            "no v8-v7 benchmarks matched filter `{}`",
            filter.unwrap_or("")
        ));
    }

    Ok(workloads)
}

fn benchmark_matches(spec: &BenchmarkSpec, filter: &str) -> bool {
    let filter = filter.to_ascii_lowercase();
    spec.name.to_ascii_lowercase().contains(&filter) || spec.slug.contains(&filter)
}

fn file_name(spec: &BenchmarkSpec) -> &'static str {
    match spec.slug {
        "richards" => "v8-v7-richards.js",
        "deltablue" => "v8-v7-deltablue.js",
        "crypto" => "v8-v7-crypto.js",
        "raytrace" => "v8-v7-raytrace.js",
        "earley-boyer" => "v8-v7-earley-boyer.js",
        "regexp" => "v8-v7-regexp.js",
        "splay" => "v8-v7-splay.js",
        "navier-stokes" => "v8-v7-navier-stokes.js",
        _ => "v8-v7-unknown.js",
    }
}

fn benchmark_bundle(name: &str, source: &str) -> String {
    format!("{BASE}\n{source}\n{}", runner(name))
}

fn full_suite_bundle() -> String {
    let mut source = String::new();
    source.push_str(BASE);
    for spec in BENCHMARKS {
        source.push('\n');
        source.push_str(spec.source);
    }
    source.push('\n');
    source.push_str(&runner("Score"));
    source
}

fn runner(result_name: &str) -> String {
    format!(
        r#"
var LyngV8V7Success = true;

function LyngV8V7Print(line) {{
  if (typeof print === 'function') {{
    print(line);
  }} else if (typeof console !== 'undefined' && typeof console.log === 'function') {{
    console.log(line);
  }}
}}

function LyngV8V7PrintResult(name, result) {{
  LyngV8V7Print(name + ': ' + result);
}}

function LyngV8V7PrintError(name, error) {{
  LyngV8V7PrintResult(name, error);
  LyngV8V7Success = false;
}}

function LyngV8V7PrintScore(score) {{
  if (LyngV8V7Success && "{result_name}" === "Score") {{
    LyngV8V7Print('Score: ' + score);
  }}
}}

BenchmarkSuite.RunSuites({{
  NotifyResult: LyngV8V7PrintResult,
  NotifyError: LyngV8V7PrintError,
  NotifyScore: LyngV8V7PrintScore
}});
"#
    )
}
