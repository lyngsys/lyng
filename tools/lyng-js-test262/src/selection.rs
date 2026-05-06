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

const UNSUPPORTED_FEATURES: &[&str] =
    &["import-assertions", "immutable-arraybuffer", "ShadowRealm"];

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

const FEATURE_REASON_ALIASES: &[(&str, &str)] = &[];

const EXPLICIT_SELECTION_EXCLUSIONS: &[(&str, &str)] = &[];

const EXPLICIT_TEST_SKIPS: &[(&str, &str)] = &[];

const SUPPORTED_FEATURE_TESTS: &[(&str, &[&str])] = &[(
    "regexp-v-flag",
    &[
        "built-ins/RegExp/prototype/exec/regexp-builtin-exec-v-u-flag.js",
        "built-ins/RegExp/prototype/flags/this-val-regexp.js",
        "built-ins/RegExp/prototype/unicodeSets/cross-realm.js",
        "built-ins/RegExp/prototype/unicodeSets/length.js",
        "built-ins/RegExp/prototype/unicodeSets/name.js",
        "built-ins/RegExp/prototype/unicodeSets/prop-desc.js",
        "built-ins/RegExp/prototype/unicodeSets/this-val-invalid-obj.js",
        "built-ins/RegExp/prototype/unicodeSets/this-val-non-obj.js",
        "built-ins/RegExp/prototype/unicodeSets/this-val-regexp-prototype.js",
        "built-ins/RegExp/prototype/unicodeSets/this-val-regexp.js",
        "built-ins/RegExp/prototype/unicodeSets/uv-flags-constructor.js",
        "built-ins/RegExp/prototype/unicodeSets/uv-flags.js",
    ],
)];

const SUPPORTED_FEATURE_PREFIXES: &[(&str, &[&str])] = &[
    (
        "regexp-v-flag",
        &[
            "built-ins/RegExp/property-escapes/generated/strings",
            "built-ins/RegExp/prototype/unicodeSets/breaking-change-from-u-to-v",
            "built-ins/RegExp/unicodeSets/generated",
        ],
    ),
    (
        "regexp-modifiers",
        &[
            "built-ins/RegExp/early-err-modifiers",
            "built-ins/RegExp/regexp-modifiers",
            "built-ins/RegExp/syntax-err-arithmetic-modifiers",
        ],
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
    if let Some(reason) = explicit_selection_exclusion_reason(path) {
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
        .or_else(|| should_skip_metadata(path, metadata, helpers))
}

fn should_skip_metadata(
    path: &Path,
    metadata: &TestMetadata,
    helpers: &HelperCatalog,
) -> Option<String> {
    let is_temporal_test = metadata
        .features
        .iter()
        .any(|feature| feature == "Temporal");
    for feature in &metadata.features {
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
            if is_supported_feature_test(path, feature) {
                continue;
            }
            return Some(format!("unsupported feature: {feature}"));
        }
    }
    for include in &metadata.includes {
        if !helpers.supports_include(include) {
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

fn is_supported_feature_test(path: &Path, feature: &str) -> bool {
    let display = path.to_string_lossy();
    if SUPPORTED_FEATURE_TESTS
        .iter()
        .find(|(candidate, _)| *candidate == feature)
        .is_some_and(|(_, suffixes)| suffixes.iter().any(|suffix| display.ends_with(suffix)))
    {
        return true;
    }
    SUPPORTED_FEATURE_PREFIXES
        .iter()
        .find(|(candidate, _)| *candidate == feature)
        .is_some_and(|(_, prefixes)| prefixes.iter().any(|prefix| display.contains(prefix)))
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

fn explicit_selection_exclusion_reason(path: &Path) -> Option<String> {
    let display = path.to_string_lossy();
    for (suffix, reason) in EXPLICIT_SELECTION_EXCLUSIONS {
        if display.ends_with(suffix) {
            return Some((*reason).to_string());
        }
    }
    None
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
        category_for_test, disabled_manifest, load_manifest, manifest_matches_pattern,
        parse_manifest_line, skip_decision, ExclusionKind, ExclusionRule, ProposalStage,
        SkipDecision,
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
    fn skip_decision_runs_atomics_helper_tests() {
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

        assert_eq!(decision, None);
    }

    #[test]
    fn skip_decision_runs_can_block_false_tests() {
        let helpers = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        let metadata = parse_metadata(
            r"
            /*---
            flags: [CanBlockIsFalse]
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
    fn skip_decision_runs_duplicate_named_group_tests_without_backreferences() {
        let helpers = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        let metadata = parse_metadata(
            r"
            /*---
            features: [regexp-duplicate-named-groups]
            ---*/
            ",
        );

        let decision = skip_decision(
            Path::new("/tmp/test/built-ins/RegExp/duplicate-named-capturing-groups-syntax.js"),
            Path::new("/tmp/test"),
            &disabled_manifest("reports/js/lyng-js/test262-exclusions.txt"),
            &metadata,
            &helpers,
            false,
            ProposalStage::Stage3,
        );

        assert_eq!(decision, None);
    }

    #[test]
    fn skip_decision_runs_duplicate_named_backreference_tests() {
        let helpers = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        let metadata = parse_metadata(
            r"
            /*---
            features: [regexp-duplicate-named-groups]
            ---*/
            ",
        );

        let decision = skip_decision(
            Path::new("/tmp/test/built-ins/RegExp/named-groups/duplicate-names-exec.js"),
            Path::new("/tmp/test"),
            &disabled_manifest("reports/js/lyng-js/test262-exclusions.txt"),
            &metadata,
            &helpers,
            false,
            ProposalStage::Stage3,
        );

        assert_eq!(decision, None);
    }

    #[test]
    fn skip_decision_runs_supported_unicode_sets_accessor_tests() {
        let helpers = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        let metadata = parse_metadata(
            r"
            /*---
            features: [regexp-v-flag]
            ---*/
            ",
        );

        let decision = skip_decision(
            Path::new("/tmp/test/built-ins/RegExp/prototype/unicodeSets/this-val-regexp.js"),
            Path::new("/tmp/test"),
            &disabled_manifest("reports/js/lyng-js/test262-exclusions.txt"),
            &metadata,
            &helpers,
            false,
            ProposalStage::Stage3,
        );

        assert_eq!(decision, None);
    }

    #[test]
    fn skip_decision_runs_supported_unicode_sets_flags_test() {
        let helpers = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        let metadata = parse_metadata(
            r"
            /*---
            features: [regexp-dotall, regexp-match-indices, regexp-v-flag]
            ---*/
            ",
        );

        let decision = skip_decision(
            Path::new("/tmp/test/built-ins/RegExp/prototype/flags/this-val-regexp.js"),
            Path::new("/tmp/test"),
            &disabled_manifest("reports/js/lyng-js/test262-exclusions.txt"),
            &metadata,
            &helpers,
            false,
            ProposalStage::Stage3,
        );

        assert_eq!(decision, None);
    }

    #[test]
    fn skip_decision_runs_supported_unicode_sets_exec_test() {
        let helpers = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        let metadata = parse_metadata(
            r"
            /*---
            features: [regexp-v-flag, regexp-unicode-property-escapes]
            includes: [compareArray.js]
            ---*/
            ",
        );

        let decision = skip_decision(
            Path::new("/tmp/test/built-ins/RegExp/prototype/exec/regexp-builtin-exec-v-u-flag.js"),
            Path::new("/tmp/test"),
            &disabled_manifest("reports/js/lyng-js/test262-exclusions.txt"),
            &metadata,
            &helpers,
            false,
            ProposalStage::Stage3,
        );

        assert_eq!(decision, None);
    }

    #[test]
    fn skip_decision_runs_supported_unicode_sets_cross_realm_test() {
        let helpers = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        let metadata = parse_metadata(
            r"
            /*---
            features: [regexp-v-flag, cross-realm]
            ---*/
            ",
        );

        let decision = skip_decision(
            Path::new("/tmp/test/built-ins/RegExp/prototype/unicodeSets/cross-realm.js"),
            Path::new("/tmp/test"),
            &disabled_manifest("reports/js/lyng-js/test262-exclusions.txt"),
            &metadata,
            &helpers,
            false,
            ProposalStage::Stage3,
        );

        assert_eq!(decision, None);
    }

    #[test]
    fn skip_decision_runs_supported_unicode_sets_breaking_change_tests() {
        let helpers = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        let metadata = parse_metadata(
            r"
            /*---
            features: [regexp-v-flag]
            ---*/
            ",
        );

        let decision = skip_decision(
            Path::new(
                "/tmp/test/built-ins/RegExp/prototype/unicodeSets/breaking-change-from-u-to-v-01.js",
            ),
            Path::new("/tmp/test"),
            &disabled_manifest("reports/js/lyng-js/test262-exclusions.txt"),
            &metadata,
            &helpers,
            false,
            ProposalStage::Stage3,
        );

        assert_eq!(decision, None);
    }

    #[test]
    fn skip_decision_runs_supported_unicode_sets_generated_union_tests() {
        let helpers = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        let metadata = parse_metadata(
            r"
            /*---
            features: [regexp-v-flag]
            ---*/
            ",
        );

        let decision = skip_decision(
            Path::new(
                "/tmp/test/built-ins/RegExp/unicodeSets/generated/character-union-character.js",
            ),
            Path::new("/tmp/test"),
            &disabled_manifest("reports/js/lyng-js/test262-exclusions.txt"),
            &metadata,
            &helpers,
            false,
            ProposalStage::Stage3,
        );

        assert_eq!(decision, None);
    }

    #[test]
    fn skip_decision_runs_supported_unicode_sets_string_property_tests() {
        let helpers = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        let metadata = parse_metadata(
            r"
            /*---
            features: [regexp-v-flag]
            ---*/
            ",
        );

        let decision = skip_decision(
            Path::new(
                "/tmp/test/built-ins/RegExp/property-escapes/generated/strings/Basic_Emoji.js",
            ),
            Path::new("/tmp/test"),
            &disabled_manifest("reports/js/lyng-js/test262-exclusions.txt"),
            &metadata,
            &helpers,
            false,
            ProposalStage::Stage3,
        );

        assert_eq!(decision, None);
    }

    #[test]
    fn skip_decision_runs_supported_unicode_sets_generated_set_expression_tests() {
        let helpers = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        let metadata = parse_metadata(
            r"
            /*---
            features: [regexp-v-flag]
            ---*/
            ",
        );

        let decision = skip_decision(
            Path::new(
                "/tmp/test/built-ins/RegExp/unicodeSets/generated/string-literal-union-string-literal.js",
            ),
            Path::new("/tmp/test"),
            &disabled_manifest("reports/js/lyng-js/test262-exclusions.txt"),
            &metadata,
            &helpers,
            false,
            ProposalStage::Stage3,
        );

        assert_eq!(decision, None);
    }

    #[test]
    fn skip_decision_runs_supported_regexp_modifier_syntax_errors() {
        let helpers = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        let metadata = parse_metadata(
            r"
            /*---
            features: [regexp-modifiers]
            ---*/
            ",
        );

        let decision = skip_decision(
            Path::new("/tmp/test/built-ins/RegExp/early-err-modifiers-other-code-point-g.js"),
            Path::new("/tmp/test"),
            &disabled_manifest("reports/js/lyng-js/test262-exclusions.txt"),
            &metadata,
            &helpers,
            false,
            ProposalStage::Stage3,
        );

        assert_eq!(decision, None);
    }

    #[test]
    fn skip_decision_runs_supported_regexp_modifier_semantics() {
        let helpers = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        let metadata = parse_metadata(
            r"
            /*---
            features: [regexp-modifiers]
            ---*/
            ",
        );

        let decision = skip_decision(
            Path::new("/tmp/test/built-ins/RegExp/regexp-modifiers/add-ignoreCase.js"),
            Path::new("/tmp/test"),
            &disabled_manifest("reports/js/lyng-js/test262-exclusions.txt"),
            &metadata,
            &helpers,
            false,
            ProposalStage::Stage3,
        );

        assert_eq!(decision, None);
    }

    #[test]
    fn skip_decision_runs_regexp_modifier_backend_gap_tests() {
        let helpers = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        let metadata = parse_metadata(
            r"
            /*---
            features: [regexp-modifiers]
            ---*/
            ",
        );

        let decision = skip_decision(
            Path::new(
                "/tmp/test/built-ins/RegExp/regexp-modifiers/add-ignoreCase-affects-backreferences.js",
            ),
            Path::new("/tmp/test"),
            &disabled_manifest("reports/js/lyng-js/test262-exclusions.txt"),
            &metadata,
            &helpers,
            false,
            ProposalStage::Stage3,
        );

        assert_eq!(decision, None);
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
    fn skip_decision_keeps_staging_typedarray_sort_negative_nan_runnable() {
        let root = workspace_root();
        let test_dir = root.join("testdata/test262/test");
        let path = test_dir.join("staging/sm/TypedArray/sort-negative-nan.js");
        let source = std::fs::read_to_string(&path).expect("test fixture should be readable");
        let metadata = parse_metadata(&source);
        let helpers = HelperCatalog::load(&root).expect("helper catalog");

        let decision = skip_decision(
            &path,
            &test_dir,
            &disabled_manifest("reports/js/lyng-js/test262-exclusions.txt"),
            &metadata,
            &helpers,
            false,
            ProposalStage::Stage3,
        );

        assert_eq!(decision, None);
    }

    #[test]
    fn skip_decision_keeps_staging_typedarray_counting_sort_stress_runnable() {
        let root = workspace_root();
        let test_dir = root.join("testdata/test262/test");
        let path = test_dir.join("staging/sm/TypedArray/sort_large_countingsort.js");
        let source = std::fs::read_to_string(&path).expect("test fixture should be readable");
        let metadata = parse_metadata(&source);
        let helpers = HelperCatalog::load(&root).expect("helper catalog");

        let decision = skip_decision(
            &path,
            &test_dir,
            &disabled_manifest("reports/js/lyng-js/test262-exclusions.txt"),
            &metadata,
            &helpers,
            false,
            ProposalStage::Stage3,
        );

        assert_eq!(decision, None);
    }

    #[test]
    fn skip_decision_keeps_staging_typedarray_to_string_runnable() {
        let root = workspace_root();
        let test_dir = root.join("testdata/test262/test");
        let path = test_dir.join("staging/sm/TypedArray/toString.js");
        let source = std::fs::read_to_string(&path).expect("test fixture should be readable");
        let metadata = parse_metadata(&source);
        let helpers = HelperCatalog::load(&root).expect("helper catalog");

        let decision = skip_decision(
            &path,
            &test_dir,
            &disabled_manifest("reports/js/lyng-js/test262-exclusions.txt"),
            &metadata,
            &helpers,
            false,
            ProposalStage::Stage3,
        );

        assert_eq!(decision, None);
    }

    #[test]
    fn skip_decision_keeps_staging_extension_recursion_runnable() {
        let root = workspace_root();
        let test_dir = root.join("testdata/test262/test");
        let path = test_dir.join("staging/sm/extensions/recursion.js");
        let source = std::fs::read_to_string(&path).expect("test fixture should be readable");
        let metadata = parse_metadata(&source);
        let helpers = HelperCatalog::load(&root).expect("helper catalog");

        let decision = skip_decision(
            &path,
            &test_dir,
            &disabled_manifest("reports/js/lyng-js/test262-exclusions.txt"),
            &metadata,
            &helpers,
            false,
            ProposalStage::Stage3,
        );

        assert_eq!(decision, None);
    }

    #[test]
    fn skip_decision_keeps_staging_deep_empty_block_eval_stress_runnable() {
        let root = workspace_root();
        let test_dir = root.join("testdata/test262/test");
        let path = test_dir.join("staging/sm/regress/regress-610026.js");
        let source = std::fs::read_to_string(&path).expect("test fixture should be readable");
        let metadata = parse_metadata(&source);
        let helpers = HelperCatalog::load(&root).expect("helper catalog");

        let decision = skip_decision(
            &path,
            &test_dir,
            &disabled_manifest("reports/js/lyng-js/test262-exclusions.txt"),
            &metadata,
            &helpers,
            false,
            ProposalStage::Stage3,
        );

        assert_eq!(decision, None);
    }

    #[test]
    fn skip_decision_runs_unmanifested_is_htmldda_tests() {
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

        assert_eq!(decision, None);
    }

    #[test]
    fn checked_in_manifest_runs_is_htmldda_tests() {
        let root = workspace_root();
        let test_dir = root.join("testdata/test262/test");
        let path = test_dir.join("annexB/language/expressions/typeof/emulates-undefined.js");
        let source = std::fs::read_to_string(&path).expect("test fixture should be readable");
        let metadata = parse_metadata(&source);
        let helpers = HelperCatalog::load(&root).expect("helper catalog");
        let manifest = load_manifest(&root, "reports/js/lyng-js/test262-exclusions.txt")
            .expect("checked-in manifest should load");

        let decision = skip_decision(
            &path,
            &test_dir,
            &manifest,
            &metadata,
            &helpers,
            false,
            ProposalStage::Stage3,
        );

        assert_eq!(decision, None);
    }

    #[test]
    fn selected_builtin_bootstrap_regressions_are_runnable() {
        let root = workspace_root();
        let test_dir = root.join("testdata/test262/test");
        let path = test_dir.join("built-ins/Object/getPrototypeOf/15.2.3.2-1.js");
        let source = std::fs::read_to_string(&path).expect("test fixture should be readable");
        let metadata = parse_metadata(&source);
        let helpers = HelperCatalog::load(&root).expect("helper catalog");

        let decision = skip_decision(
            &path,
            &test_dir,
            &disabled_manifest("reports/js/lyng-js/test262-exclusions.txt"),
            &metadata,
            &helpers,
            false,
            ProposalStage::Stage3,
        );

        assert_eq!(decision, None);
    }

    #[test]
    fn selected_arraybuffer_feature_tests_are_runnable() {
        let helpers = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        for feature in ["resizable-arraybuffer", "arraybuffer-transfer"] {
            let metadata = parse_metadata(&format!(
                r"
                /*---
                features: [{feature}]
                ---*/
                ",
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

            assert_eq!(decision, None, "feature {feature} should be runnable");
        }
    }

    #[test]
    fn selected_arraybuffer_safety_regressions_are_runnable() {
        let root = workspace_root();
        let test_dir = root.join("testdata/test262/test");
        let helpers = HelperCatalog::load(&root).expect("helper catalog");
        for relative_path in [
            "built-ins/ArrayBuffer/allocation-limit.js",
            "built-ins/ArrayBuffer/length-is-too-large-throws.js",
        ] {
            let path = test_dir.join(relative_path);
            let source = std::fs::read_to_string(&path).expect("test fixture should be readable");
            let metadata = parse_metadata(&source);

            let decision = skip_decision(
                &path,
                &test_dir,
                &disabled_manifest("reports/js/lyng-js/test262-exclusions.txt"),
                &metadata,
                &helpers,
                false,
                ProposalStage::Stage3,
            );

            assert_eq!(decision, None, "{relative_path} should be runnable");
        }
    }

    #[test]
    fn selected_iterator_helper_feature_tests_are_runnable() {
        let helpers = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        for feature in ["iterator-sequencing", "joint-iteration"] {
            let metadata = parse_metadata(&format!(
                r"
                /*---
                features: [{feature}]
                ---*/
                ",
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

            assert_eq!(decision, None, "feature {feature} should be runnable");
        }
    }

    #[test]
    fn selected_iterator_helper_path_regressions_are_runnable() {
        let root = workspace_root();
        let test_dir = root.join("testdata/test262/test");
        let path = test_dir.join("built-ins/Iterator/concat/proto.js");
        let source = std::fs::read_to_string(&path).expect("test fixture should be readable");
        let metadata = parse_metadata(&source);
        let helpers = HelperCatalog::load(&root).expect("helper catalog");

        let decision = skip_decision(
            &path,
            &test_dir,
            &disabled_manifest("reports/js/lyng-js/test262-exclusions.txt"),
            &metadata,
            &helpers,
            false,
            ProposalStage::Stage3,
        );

        assert_eq!(decision, None);
    }

    #[test]
    fn selected_regexp_feature_tests_are_runnable() {
        let helpers = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        for feature in ["regexp-v-flag", "regexp-modifiers"] {
            let metadata = parse_metadata(&format!(
                r"
                /*---
                features: [{feature}]
                ---*/
                ",
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

            assert_eq!(decision, None, "feature {feature} should be runnable");
        }
    }

    #[test]
    fn selected_staging_duplicate_named_group_tests_are_runnable() {
        let root = workspace_root();
        let test_dir = root.join("testdata/test262/test");
        let path = test_dir.join("staging/built-ins/RegExp/named-groups/duplicate-named-groups.js");
        let source = std::fs::read_to_string(&path).expect("test fixture should be readable");
        let metadata = parse_metadata(&source);
        let helpers = HelperCatalog::load(&root).expect("helper catalog");

        let decision = skip_decision(
            &path,
            &test_dir,
            &disabled_manifest("reports/js/lyng-js/test262-exclusions.txt"),
            &metadata,
            &helpers,
            false,
            ProposalStage::Stage3,
        );

        assert_eq!(decision, None);
    }

    #[test]
    fn selected_decorator_feature_tests_are_runnable() {
        let helpers = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        let metadata = parse_metadata(
            r"
            /*---
            features: [decorators]
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
