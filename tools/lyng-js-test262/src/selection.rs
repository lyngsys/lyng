use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::helpers::HelperCatalog;
use crate::metadata::{is_module_test, TestMetadata};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProposalStage {
    Stage4,
    Stage3,
    Stage2_7,
}

impl ProposalStage {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Stage4 => "Stage 4",
            Self::Stage3 => "Stage 3+",
            Self::Stage2_7 => "Stage 2.7+",
        }
    }

    const fn rank(self) -> u8 {
        match self {
            Self::Stage4 => 40,
            Self::Stage3 => 30,
            Self::Stage2_7 => 27,
        }
    }

    const fn includes(self, feature_stage: Self) -> bool {
        feature_stage.rank() >= self.rank()
    }
}

const UNSUPPORTED_FEATURES: &[&str] = &[
    "decorators",
    "import-assertions",
    "regexp-v-flag",
    "regexp-duplicate-named-groups",
    "regexp-modifiers",
    "resizable-arraybuffer",
    "arraybuffer-transfer",
    "immutable-arraybuffer",
    "ShadowRealm",
];

const PROPOSAL_FEATURE_STAGES: &[(&str, ProposalStage)] = &[
    ("Temporal", ProposalStage::Stage4),
    ("regexp-duplicate-named-groups", ProposalStage::Stage4),
    ("legacy-regexp", ProposalStage::Stage3),
    ("decorators", ProposalStage::Stage3),
    ("explicit-resource-management", ProposalStage::Stage3),
    ("source-phase-imports", ProposalStage::Stage3),
    ("source-phase-imports-module-source", ProposalStage::Stage3),
    ("Atomics.pause", ProposalStage::Stage3),
    ("import-defer", ProposalStage::Stage3),
    ("import-text", ProposalStage::Stage3),
    ("nonextensible-applies-to-private", ProposalStage::Stage3),
    ("joint-iteration", ProposalStage::Stage3),
    ("ShadowRealm", ProposalStage::Stage2_7),
    ("immutable-arraybuffer", ProposalStage::Stage2_7),
    ("import-bytes", ProposalStage::Stage2_7),
    ("await-dictionary", ProposalStage::Stage2_7),
];

fn proposal_feature_stage(feature: &str) -> Option<ProposalStage> {
    PROPOSAL_FEATURE_STAGES
        .iter()
        .find_map(|(candidate, stage)| (*candidate == feature).then_some(*stage))
}

const FEATURE_REASON_ALIASES: &[(&str, &str)] = &[
    (
        "iterator-sequencing",
        "unsupported feature: iterator-helpers",
    ),
    ("joint-iteration", "unsupported feature: iterator-helpers"),
];

const UNSUPPORTED_HOST_FEATURES: &[(&str, &str)] =
    &[("IsHTMLDDA", "unsupported host feature: IsHTMLDDA")];

const EXPLICIT_TEST_SKIPS: &[(&str, &str)] = &[
    (
        "built-ins/Iterator/concat/proto.js",
        "unsupported feature: iterator-helpers",
    ),
    (
        "built-ins/Iterator/concat/is-function.js",
        "unsupported feature: iterator-helpers",
    ),
    (
        "built-ins/Iterator/concat/non-constructible.js",
        "unsupported feature: iterator-helpers",
    ),
    (
        "built-ins/Iterator/concat/throws-typeerror-when-iterable-not-an-object.js",
        "unsupported feature: iterator-helpers",
    ),
    (
        "built-ins/Iterator/zip/is-function.js",
        "unsupported feature: iterator-helpers",
    ),
    (
        "built-ins/Iterator/zip/iterables-primitive.js",
        "unsupported feature: iterator-helpers",
    ),
    (
        "built-ins/Iterator/zip/non-constructible.js",
        "unsupported feature: iterator-helpers",
    ),
    (
        "built-ins/Iterator/zip/iterator-non-iterable.js",
        "unsupported feature: iterator-helpers",
    ),
    (
        "built-ins/Iterator/zip/options-padding.js",
        "unsupported feature: iterator-helpers",
    ),
    (
        "built-ins/Iterator/zip/options.js",
        "unsupported feature: iterator-helpers",
    ),
    (
        "built-ins/Iterator/zip/proto.js",
        "unsupported feature: iterator-helpers",
    ),
    (
        "built-ins/Iterator/zipKeyed/is-function.js",
        "unsupported feature: iterator-helpers",
    ),
    (
        "built-ins/Iterator/zipKeyed/iterables-iteration-deleted.js",
        "unsupported feature: iterator-helpers",
    ),
    (
        "built-ins/Iterator/zipKeyed/iterables-primitive.js",
        "unsupported feature: iterator-helpers",
    ),
    (
        "built-ins/Iterator/zipKeyed/non-constructible.js",
        "unsupported feature: iterator-helpers",
    ),
    (
        "built-ins/Iterator/zipKeyed/options-padding.js",
        "unsupported feature: iterator-helpers",
    ),
    (
        "built-ins/Iterator/zipKeyed/options.js",
        "unsupported feature: iterator-helpers",
    ),
    (
        "built-ins/Iterator/zipKeyed/proto.js",
        "unsupported feature: iterator-helpers",
    ),
    (
        "built-ins/Iterator/zipKeyed/results-object-has-no-undefined-iterables-properties.js",
        "unsupported feature: iterator-helpers",
    ),
    (
        "built-ins/Object/getPrototypeOf/15.2.3.2-1.js",
        "needs builtin bootstrap: Number",
    ),
    (
        "built-ins/Object/getPrototypeOf/15.2.3.2-1-4.js",
        "needs builtin bootstrap: String",
    ),
    (
        "built-ins/Object/getPrototypeOf/15.2.3.2-2-5.js",
        "needs builtin bootstrap: Array",
    ),
    (
        "built-ins/Object/getPrototypeOf/15.2.3.2-2-6.js",
        "needs builtin bootstrap: String",
    ),
    (
        "built-ins/Object/getPrototypeOf/15.2.3.2-2-7.js",
        "needs builtin bootstrap: Number",
    ),
    (
        "built-ins/Object/getPrototypeOf/15.2.3.2-2-8.js",
        "needs builtin bootstrap: Math",
    ),
    (
        "built-ins/Object/getPrototypeOf/15.2.3.2-2-18.js",
        "needs builtin bootstrap: JSON",
    ),
    (
        "built-ins/Object/getPrototypeOf/15.2.3.2-2-22.js",
        "needs builtin bootstrap: String",
    ),
    (
        "built-ins/Object/getPrototypeOf/15.2.3.2-2-24.js",
        "needs builtin bootstrap: Number",
    ),
    (
        "built-ins/Object/prototype/toString/Object.prototype.toString.call-number.js",
        "needs builtin bootstrap: Number",
    ),
    (
        "built-ins/Object/prototype/toString/Object.prototype.toString.call-string.js",
        "needs builtin bootstrap: String",
    ),
    (
        "built-ins/Object/prototype/toString/symbol-tag-non-str-builtin.js",
        "needs builtin bootstrap: Math/JSON",
    ),
    (
        "built-ins/Object/prototype/toString/symbol-tag-non-str.js",
        "needs builtin bootstrap: String",
    ),
    (
        "built-ins/Object/prototype/toString/symbol-tag-override-primitives.js",
        "needs builtin bootstrap: Number/String",
    ),
    (
        "built-ins/Object/getOwnPropertyDescriptor/15.2.3.3-4-4.js",
        "needs builtin bootstrap: eval",
    ),
    (
        "built-ins/Object/getOwnPropertyDescriptor/15.2.3.3-4-16.js",
        "needs builtin bootstrap: Object.getOwnPropertyNames",
    ),
    (
        "built-ins/Object/getOwnPropertyDescriptor/15.2.3.3-4-19.js",
        "needs builtin bootstrap: Object.defineProperties",
    ),
    (
        "built-ins/Object/getOwnPropertyDescriptor/15.2.3.3-4-26.js",
        "needs builtin bootstrap: Object.keys",
    ),
    (
        "built-ins/Object/getOwnPropertyDescriptor/15.2.3.3-4-33.js",
        "needs builtin bootstrap: Object.prototype.toLocaleString",
    ),
    (
        "built-ins/Object/create/15.2.3.5-4-6.js",
        "needs builtin bootstrap: Array",
    ),
    (
        "built-ins/Object/create/15.2.3.5-4-10.js",
        "needs builtin bootstrap: Math",
    ),
    (
        "built-ins/Object/create/15.2.3.5-4-13.js",
        "needs builtin bootstrap: JSON",
    ),
    (
        "built-ins/Object/create/15.2.3.5-4-14.js",
        "needs builtin bootstrap: Object.getOwnPropertyNames",
    ),
    (
        "built-ins/Object/create/15.2.3.5-4-37.js",
        "needs builtin bootstrap: Object.getOwnPropertyNames",
    ),
    (
        "built-ins/Object/create/properties-arg-to-object.js",
        "needs builtin bootstrap: Object.getOwnPropertyNames/Object.getOwnPropertySymbols",
    ),
    (
        "built-ins/Function/prototype/call/S15.3.4.4_A6_T1.js",
        "needs builtin bootstrap: Array",
    ),
    (
        "built-ins/Function/prototype/call/15.3.4.4-1-s.js",
        "needs builtin bootstrap: String",
    ),
    (
        "built-ins/Function/prototype/call/15.3.4.4-2-s.js",
        "needs builtin bootstrap: Number",
    ),
    (
        "built-ins/Function/prototype/bind/15.3.4.5-2-6.js",
        "needs builtin bootstrap: Number",
    ),
    (
        "built-ins/ArrayBuffer/allocation-limit.js",
        "runtime abort: oversize ArrayBuffer allocation guard missing",
    ),
    (
        "built-ins/ArrayBuffer/length-is-too-large-throws.js",
        "runtime abort: oversize ArrayBuffer allocation guard missing",
    ),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ExclusionManifest {
    pub(crate) path: String,
    pub(crate) rules: Vec<ExclusionRule>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ExclusionRule {
    pub(crate) kind: ExclusionKind,
    pub(crate) pattern: String,
    pub(crate) reason: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExclusionKind {
    SuitePrefix,
    Path,
}

impl ExclusionKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::SuitePrefix => "suite",
            Self::Path => "path",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SkipDecision {
    ExcludedFromSelection(String),
    Skip(String),
}

pub(crate) fn disabled_manifest(configured_path: &str) -> ExclusionManifest {
    ExclusionManifest {
        path: format!("disabled via --no-skip (configured path: {configured_path})"),
        rules: Vec::new(),
    }
}

pub(crate) fn select_test_paths(
    filter: Option<&str>,
    test_dir: &Path,
) -> Result<Vec<PathBuf>, String> {
    if let Some(filter) = filter {
        let relative_candidate = test_dir.join(filter);
        if relative_candidate.exists() {
            return Ok(if relative_candidate.is_file() {
                vec![relative_candidate]
            } else {
                collect_tests(&relative_candidate)
            });
        }

        let absolute_candidate = PathBuf::from(filter);
        if absolute_candidate.exists() {
            return Ok(if absolute_candidate.is_file() {
                vec![absolute_candidate]
            } else {
                collect_tests(&absolute_candidate)
            });
        }

        let mut filtered = collect_tests(test_dir);
        filtered.retain(|path| {
            path.strip_prefix(test_dir)
                .ok()
                .and_then(|relative| relative.to_str())
                .is_some_and(|relative| relative.contains(filter))
        });
        if filtered.is_empty() {
            return Err(format!("filter path or fragment not found: {filter}"));
        }
        return Ok(filtered);
    }

    Ok(collect_tests(test_dir))
}

pub(crate) fn category_for_test(path: &Path, test_dir: &Path) -> String {
    let relative = path.strip_prefix(test_dir).unwrap_or(path);
    let mut parts = Vec::new();
    for component in relative.components().take(1) {
        if let Component::Normal(part) = component {
            parts.push(part.to_string_lossy().to_string());
        }
    }
    if parts.is_empty() {
        relative.display().to_string()
    } else {
        parts.join("/")
    }
}

pub(crate) fn relative_test_path(path: &Path, test_dir: &Path) -> String {
    path.strip_prefix(test_dir)
        .unwrap_or(path)
        .display()
        .to_string()
}

pub(crate) fn skip_decision(
    path: &Path,
    test_dir: &Path,
    manifest: &ExclusionManifest,
    metadata: &TestMetadata,
    helpers: &HelperCatalog,
    no_skip: bool,
    proposal_stage: ProposalStage,
) -> Option<SkipDecision> {
    if no_skip {
        return None;
    }
    if let Some(reason) = selection_exclusion_reason(metadata, proposal_stage) {
        return Some(SkipDecision::ExcludedFromSelection(reason));
    }

    skip_reason(path, test_dir, manifest, metadata, helpers).map(SkipDecision::Skip)
}

pub(crate) fn load_manifest(
    workspace_root: &Path,
    manifest_path: &str,
) -> Result<ExclusionManifest, String> {
    let absolute_path = if Path::new(manifest_path).is_absolute() {
        PathBuf::from(manifest_path)
    } else {
        workspace_root.join(manifest_path)
    };
    let source = fs::read_to_string(&absolute_path).map_err(|error| {
        format!(
            "failed to read manifest {}: {error}",
            absolute_path.display()
        )
    })?;
    let mut rules = Vec::new();
    for (index, line) in source.lines().enumerate() {
        match parse_manifest_line(line) {
            Ok(Some(rule)) => rules.push(rule),
            Ok(None) => {}
            Err(error) => {
                return Err(format!(
                    "manifest parse error at {}:{}: {error}",
                    manifest_path,
                    index + 1
                ));
            }
        }
    }
    Ok(ExclusionManifest {
        path: manifest_path.to_string(),
        rules,
    })
}

fn collect_tests(dir: &Path) -> Vec<PathBuf> {
    let mut tests = Vec::new();
    if !dir.exists() {
        return tests;
    }
    if let Ok(entries) = fs::read_dir(dir) {
        let mut entries: Vec<_> = entries.flatten().collect();
        entries.sort_by_key(std::fs::DirEntry::path);
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                tests.extend(collect_tests(&path));
            } else if path.extension().is_some_and(|ext| ext == "js") {
                if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
                    if !name.contains("_FIXTURE") {
                        tests.push(path);
                    }
                }
            }
        }
    }
    tests
}

fn skip_reason(
    path: &Path,
    test_dir: &Path,
    manifest: &ExclusionManifest,
    metadata: &TestMetadata,
    helpers: &HelperCatalog,
) -> Option<String> {
    should_skip_manifest(path, test_dir, manifest)
        .or_else(|| should_skip_path(path))
        .or_else(|| should_skip_metadata(metadata, helpers))
}

fn should_skip_metadata(metadata: &TestMetadata, helpers: &HelperCatalog) -> Option<String> {
    if metadata.flags.iter().any(|flag| flag == "CanBlockIsFalse") {
        return Some("host runs with [[CanBlock]] true".to_string());
    }
    let is_temporal_test = metadata
        .features
        .iter()
        .any(|feature| feature == "Temporal");
    for feature in &metadata.features {
        if let Some((_, reason)) = UNSUPPORTED_HOST_FEATURES
            .iter()
            .find(|(unsupported, _)| feature == unsupported)
        {
            return Some((*reason).to_string());
        }
        if let Some((_, reason)) = FEATURE_REASON_ALIASES
            .iter()
            .find(|(alias, _)| feature == alias)
        {
            return Some((*reason).to_string());
        }
        if UNSUPPORTED_FEATURES
            .iter()
            .any(|unsupported| feature == unsupported)
        {
            if is_temporal_test && feature == "BigInt" {
                continue;
            }
            return Some(format!("unsupported feature: {feature}"));
        }
    }
    for include in &metadata.includes {
        if !helpers.supports_include(include) {
            if include == "atomicsHelper.js" {
                return Some("requires $262.agent multi-agent harness".to_string());
            }
            return Some(format!("unsupported harness include: {include}"));
        }
    }
    if metadata
        .negative
        .as_ref()
        .is_some_and(|negative| negative.phase == "resolution" && !is_module_test(metadata))
    {
        return Some("resolution-phase tests are deferred".to_string());
    }
    None
}

fn selection_exclusion_reason(
    metadata: &TestMetadata,
    proposal_stage: ProposalStage,
) -> Option<String> {
    metadata
        .features
        .iter()
        .filter_map(|feature| proposal_feature_stage(feature).map(|stage| (feature, stage)))
        .find(|(_, feature_stage)| !proposal_stage.includes(*feature_stage))
        .map(|(feature, _)| format!("proposal stage below {}: {feature}", proposal_stage.label()))
}

fn should_skip_path(path: &Path) -> Option<String> {
    let display = path.to_string_lossy();
    for (suffix, reason) in EXPLICIT_TEST_SKIPS {
        if display.ends_with(suffix) {
            return Some((*reason).to_string());
        }
    }
    None
}

fn parse_manifest_line(line: &str) -> Result<Option<ExclusionRule>, String> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return Ok(None);
    }

    let mut fields = trimmed.splitn(3, '|');
    let kind = fields
        .next()
        .ok_or_else(|| format!("missing kind in manifest line: {trimmed}"))?
        .trim();
    let pattern = fields
        .next()
        .ok_or_else(|| format!("missing pattern in manifest line: {trimmed}"))?
        .trim();
    let reason = fields
        .next()
        .ok_or_else(|| format!("missing reason in manifest line: {trimmed}"))?
        .trim();

    if pattern.is_empty() || reason.is_empty() {
        return Err(format!("invalid manifest line: {trimmed}"));
    }

    let kind = match kind {
        "suite" => ExclusionKind::SuitePrefix,
        "path" => ExclusionKind::Path,
        other => {
            return Err(format!(
                "unknown manifest kind `{other}` in line: {trimmed}"
            ))
        }
    };

    Ok(Some(ExclusionRule {
        kind,
        pattern: pattern.to_string(),
        reason: reason.to_string(),
    }))
}

fn manifest_matches_pattern(rule: &ExclusionRule, relative_path: &str) -> bool {
    match rule.kind {
        ExclusionKind::Path => relative_path == rule.pattern,
        ExclusionKind::SuitePrefix => {
            let prefix = rule.pattern.trim_end_matches('*').trim_end_matches('/');
            if prefix.is_empty() {
                return false;
            }
            relative_path == prefix || relative_path.starts_with(&format!("{prefix}/"))
        }
    }
}

fn should_skip_manifest(
    path: &Path,
    test_dir: &Path,
    manifest: &ExclusionManifest,
) -> Option<String> {
    let relative_path = path.strip_prefix(test_dir).ok()?.to_string_lossy();
    for rule in &manifest.rules {
        if manifest_matches_pattern(rule, &relative_path) {
            return Some(format!(
                "manifest exclusion ({}): {}",
                rule.kind.as_str(),
                rule.reason
            ));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::helpers::HelperCatalog;
    use crate::metadata::parse_metadata;

    use super::{
        category_for_test, disabled_manifest, manifest_matches_pattern, parse_manifest_line,
        skip_decision, ExclusionKind, ExclusionRule, ProposalStage, SkipDecision,
    };

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("workspace root should exist")
    }

    #[test]
    fn category_groups_first_relative_component_only() {
        let test_dir = Path::new("/tmp/test262/test");
        let path = Path::new("/tmp/test262/test/built-ins/Array/from.js");
        assert_eq!(category_for_test(path, test_dir), "built-ins");
    }

    #[test]
    fn parse_manifest_line_supports_suite_rules() {
        let rule = parse_manifest_line("suite|intl402/*|out of scope")
            .expect("manifest line should parse")
            .expect("rule should be present");
        assert_eq!(
            rule,
            ExclusionRule {
                kind: ExclusionKind::SuitePrefix,
                pattern: "intl402/*".to_string(),
                reason: "out of scope".to_string(),
            }
        );
    }

    #[test]
    fn manifest_matching_treats_suite_prefixes_as_directory_prefixes() {
        let rule = ExclusionRule {
            kind: ExclusionKind::SuitePrefix,
            pattern: "intl402/*".to_string(),
            reason: "out of scope".to_string(),
        };

        assert!(manifest_matches_pattern(
            &rule,
            "intl402/DateTimeFormat/basic.js"
        ));
        assert!(!manifest_matches_pattern(&rule, "intl4022/not-a-match.js"));
    }

    #[test]
    fn skip_decision_does_not_exclude_stage_4_error_is_error_tests() {
        let helpers = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        let metadata = parse_metadata(
            r"
            /*---
            features: [Error.isError]
            ---*/
            ",
        );

        let decision = skip_decision(
            Path::new("/tmp/test.js"),
            Path::new("/tmp"),
            &disabled_manifest("reports/js/lyng-js/test262-exclusions.txt"),
            &metadata,
            &helpers,
            false,
            ProposalStage::Stage3,
        );

        assert_eq!(decision, None);
    }

    #[test]
    fn skip_decision_reports_atomics_helper_gap_narrowly() {
        let helpers = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        let metadata = parse_metadata(
            r"
            /*---
            includes: [atomicsHelper.js]
            ---*/
            ",
        );

        let decision = skip_decision(
            Path::new("/tmp/test.js"),
            Path::new("/tmp"),
            &disabled_manifest("reports/js/lyng-js/test262-exclusions.txt"),
            &metadata,
            &helpers,
            false,
            ProposalStage::Stage3,
        );

        assert_eq!(
            decision,
            Some(SkipDecision::Skip(
                "requires $262.agent multi-agent harness".to_string()
            ))
        );
    }

    #[test]
    fn skip_decision_runs_raw_tests() {
        let helpers = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        let metadata = parse_metadata(
            r"
            /*---
            flags: [raw]
            ---*/
            ",
        );

        let decision = skip_decision(
            Path::new("/tmp/test.js"),
            Path::new("/tmp"),
            &disabled_manifest("reports/js/lyng-js/test262-exclusions.txt"),
            &metadata,
            &helpers,
            false,
            ProposalStage::Stage3,
        );

        assert_eq!(decision, None);
    }

    #[test]
    fn skip_decision_reports_is_htmldda_host_gap() {
        let helpers = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        let metadata = parse_metadata(
            r"
            /*---
            features: [IsHTMLDDA]
            ---*/
            ",
        );

        let decision = skip_decision(
            Path::new("/tmp/test.js"),
            Path::new("/tmp"),
            &disabled_manifest("reports/js/lyng-js/test262-exclusions.txt"),
            &metadata,
            &helpers,
            false,
            ProposalStage::Stage3,
        );

        assert_eq!(
            decision,
            Some(SkipDecision::Skip(
                "unsupported host feature: IsHTMLDDA".to_string()
            ))
        );
    }

    #[test]
    fn skip_decision_runs_module_proposal_feature_buckets() {
        let helpers = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        for feature in [
            "source-phase-imports",
            "source-phase-imports-module-source",
            "import-defer",
            "json-modules",
            "import-text",
        ] {
            let metadata = parse_metadata(&format!(
                r"
                /*---
                features: [{feature}]
                flags: [module]
                ---*/
                "
            ));

            let decision = skip_decision(
                Path::new("/tmp/test.js"),
                Path::new("/tmp"),
                &disabled_manifest("reports/js/lyng-js/test262-exclusions.txt"),
                &metadata,
                &helpers,
                false,
                ProposalStage::Stage3,
            );

            assert_eq!(
                decision, None,
                "module proposal feature `{feature}` should remain runnable"
            );
        }
    }

    #[test]
    fn skip_decision_runs_top_level_await_modules() {
        let helpers = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        let metadata = parse_metadata(
            r"
            /*---
            features: [top-level-await]
            flags: [module]
            ---*/
            await Promise.resolve(1);
            ",
        );

        let decision = skip_decision(
            Path::new("/tmp/test.js"),
            Path::new("/tmp"),
            &disabled_manifest("reports/js/lyng-js/test262-exclusions.txt"),
            &metadata,
            &helpers,
            false,
            ProposalStage::Stage3,
        );

        assert_eq!(decision, None);
    }

    #[test]
    fn skip_decision_excludes_stage_2_7_proposals_by_default() {
        let helpers = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        let metadata = parse_metadata(
            r"
            /*---
            features: [ShadowRealm]
            ---*/
            ",
        );

        let decision = skip_decision(
            Path::new("/tmp/test.js"),
            Path::new("/tmp"),
            &disabled_manifest("reports/js/lyng-js/test262-exclusions.txt"),
            &metadata,
            &helpers,
            false,
            ProposalStage::Stage3,
        );

        assert_eq!(
            decision,
            Some(SkipDecision::ExcludedFromSelection(
                "proposal stage below Stage 3+: ShadowRealm".to_string()
            ))
        );
    }

    #[test]
    fn skip_decision_includes_stage_2_7_proposals_when_opted_in() {
        let helpers = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        let metadata = parse_metadata(
            r"
            /*---
            features: [ShadowRealm]
            ---*/
            ",
        );

        let decision = skip_decision(
            Path::new("/tmp/test.js"),
            Path::new("/tmp"),
            &disabled_manifest("reports/js/lyng-js/test262-exclusions.txt"),
            &metadata,
            &helpers,
            false,
            ProposalStage::Stage2_7,
        );

        assert_eq!(
            decision,
            Some(SkipDecision::Skip(
                "unsupported feature: ShadowRealm".to_string()
            ))
        );
    }

    #[test]
    fn skip_decision_stage_4_policy_excludes_stage_3_and_stage_2_7_proposals() {
        let helpers = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        for feature in ["source-phase-imports", "ShadowRealm"] {
            let metadata = parse_metadata(&format!(
                r"
                /*---
                features: [{feature}]
                ---*/
                "
            ));

            let decision = skip_decision(
                Path::new("/tmp/test.js"),
                Path::new("/tmp"),
                &disabled_manifest("reports/js/lyng-js/test262-exclusions.txt"),
                &metadata,
                &helpers,
                false,
                ProposalStage::Stage4,
            );

            assert_eq!(
                decision,
                Some(SkipDecision::ExcludedFromSelection(format!(
                    "proposal stage below Stage 4: {feature}"
                ))),
                "feature `{feature}` should be excluded by strict Stage 4 policy"
            );
        }
    }

    #[test]
    fn skip_decision_no_skip_bypasses_proposal_stage_exclusions() {
        let helpers = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        let metadata = parse_metadata(
            r"
            /*---
            features: [ShadowRealm]
            ---*/
            ",
        );

        let decision = skip_decision(
            Path::new("/tmp/test.js"),
            Path::new("/tmp"),
            &disabled_manifest("reports/js/lyng-js/test262-exclusions.txt"),
            &metadata,
            &helpers,
            true,
            ProposalStage::Stage4,
        );

        assert_eq!(decision, None);
    }
}
