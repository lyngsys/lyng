use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use crate::cli::{DEFAULT_MANIFEST_PATH, DEFAULT_TIMEOUT_MS};
use crate::execution::{self, PreparedTest};
use crate::helpers::HelperCatalog;
use crate::metadata::{
    has_async_flag, is_module_test, parse_metadata, variants_for_metadata, TestMetadata,
};
use crate::selection::{
    category_for_test, disabled_manifest, load_manifest, relative_test_path, select_test_paths,
    skip_decision, ExclusionManifest, ProposalStage,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Test262DiagnosticProposalStage {
    Stage4,
    Stage3,
    Stage2_7,
}

impl Test262DiagnosticProposalStage {
    const fn into_selection(self) -> ProposalStage {
        match self {
            Self::Stage4 => ProposalStage::Stage4,
            Self::Stage3 => ProposalStage::Stage3,
            Self::Stage2_7 => ProposalStage::Stage2_7,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Test262DiagnosticConfig {
    pub filter: Option<String>,
    pub manifest_path: String,
    pub no_skip: bool,
    pub timeout_ms: u64,
    pub proposal_stage: Test262DiagnosticProposalStage,
}

impl Default for Test262DiagnosticConfig {
    fn default() -> Self {
        Self {
            filter: None,
            manifest_path: DEFAULT_MANIFEST_PATH.to_string(),
            no_skip: false,
            timeout_ms: DEFAULT_TIMEOUT_MS,
            proposal_stage: Test262DiagnosticProposalStage::Stage3,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Test262DiagnosticTest {
    pub index: usize,
    pub file: String,
    pub variant: Option<String>,
    pub category: String,
    pub flags: Vec<String>,
    pub features: Vec<String>,
    pub includes: Vec<String>,
    pub negative_phase: Option<String>,
    pub async_test: bool,
    pub module_goal: bool,
    pub timeout_ms: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Test262DiagnosticTimings {
    pub read_source: Duration,
    pub runtime_assembly: Duration,
    pub frontend_check: Duration,
    pub parse: Duration,
    pub sema: Duration,
    pub lowering: Duration,
    pub script_install: Duration,
    pub realm_bootstrap: Duration,
    pub extension_install: Duration,
    pub global_instantiation: Duration,
    pub bytecode_execution: Duration,
    pub job_checkpoint: Duration,
    pub install_or_load: Duration,
    pub evaluation: Duration,
    pub total: Duration,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Test262RuntimeDiagnostics {
    pub function_count: usize,
    pub instruction_words: usize,
    pub wide_prefixes: usize,
    pub metadata_records: usize,
    pub constants: usize,
    pub source_map_entries: usize,
    pub safepoints: usize,
    pub deopt_snapshots: usize,
    pub feedback_slots: usize,
    pub live_feedback_sites: usize,
    pub megamorphic_sites: usize,
    pub tier_hotness: u32,
    pub tier_feedback_events: u32,
    pub tier_backedge_events: u32,
    pub runtime_live_bytes_before: usize,
    pub runtime_live_bytes_after: usize,
    pub runtime_live_bytes_delta: isize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Test262DiagnosticOutcome {
    pub identity: Test262DiagnosticTest,
    pub outcome: String,
    pub timings: Test262DiagnosticTimings,
    pub diagnostics: Option<Test262RuntimeDiagnostics>,
}

pub struct Test262DiagnosticSuite {
    candidate_total: usize,
    tests: Vec<Test262DiagnosticTest>,
    prepared: Vec<PreparedTest>,
    helpers: Arc<HelperCatalog>,
}

impl Test262DiagnosticSuite {
    #[must_use]
    pub const fn candidate_total(&self) -> usize {
        self.candidate_total
    }

    #[must_use]
    pub fn tests(&self) -> &[Test262DiagnosticTest] {
        &self.tests
    }

    /// Execute one prepared variant with diagnostic timings.
    ///
    /// # Errors
    ///
    /// Returns an error when the requested prepared-test index is out of range.
    pub fn run_diagnostic(&self, index: usize) -> Result<Test262DiagnosticOutcome, String> {
        let prepared = self
            .prepared
            .get(index)
            .ok_or_else(|| format!("diagnostic test index out of range: {index}"))?;
        let identity = self
            .tests
            .get(index)
            .ok_or_else(|| format!("diagnostic identity index out of range: {index}"))?
            .clone();
        let run = execution::run_test_with_diagnostics(prepared, &self.helpers);
        Ok(Test262DiagnosticOutcome {
            identity,
            outcome: run.outcome_label,
            timings: run.timings,
            diagnostics: run.diagnostics,
        })
    }
}

/// Prepare a reusable diagnostic suite using Test262 selection semantics.
///
/// # Errors
///
/// Returns an error when the workspace cannot be resolved, helpers or
/// manifests cannot be loaded, or selected tests cannot be prepared.
pub fn prepare_diagnostic_suite(
    config: &Test262DiagnosticConfig,
) -> Result<Test262DiagnosticSuite, String> {
    let workspace_root = workspace_root()?;
    let helpers = Arc::new(HelperCatalog::load(&workspace_root)?);
    let test_dir = helpers.test_dir();
    if !test_dir.exists() {
        return Err(format!(
            "test262 test directory not found: {}",
            test_dir.display()
        ));
    }
    let manifest = if config.no_skip {
        disabled_manifest(&config.manifest_path)
    } else {
        load_manifest(&workspace_root, &config.manifest_path)?
    };
    prepare_with_helpers(config, &test_dir, &manifest, helpers)
}

fn prepare_with_helpers(
    config: &Test262DiagnosticConfig,
    test_dir: &Path,
    manifest: &ExclusionManifest,
    helpers: Arc<HelperCatalog>,
) -> Result<Test262DiagnosticSuite, String> {
    let mut paths = select_test_paths(config.filter.as_deref(), test_dir)?;
    paths.sort();
    paths.dedup();

    let candidate_total = paths.len();
    let mut prepared = Vec::new();
    let mut tests = Vec::new();

    for path in &paths {
        let source = fs::read_to_string(path)
            .map_err(|error| format!("{}: read error: {error}", path.display()))?;
        let metadata = parse_metadata(&source);
        let category = category_for_test(path, test_dir);
        if skip_decision(
            path,
            test_dir,
            manifest,
            &metadata,
            &helpers,
            config.no_skip,
            config.proposal_stage.into_selection(),
        )
        .is_some()
        {
            continue;
        }

        for variant in variants_for_metadata(&metadata) {
            let index = prepared.len();
            let prepared_test = PreparedTest {
                path: path.clone(),
                category: category.clone(),
                metadata: metadata.clone(),
                variant,
            };
            tests.push(identity_for_test(
                index,
                &prepared_test,
                test_dir,
                &metadata,
                config.timeout_ms,
            ));
            prepared.push(prepared_test);
        }
    }

    Ok(Test262DiagnosticSuite {
        candidate_total,
        tests,
        prepared,
        helpers,
    })
}

fn identity_for_test(
    index: usize,
    test: &PreparedTest,
    test_dir: &Path,
    metadata: &TestMetadata,
    timeout_ms: u64,
) -> Test262DiagnosticTest {
    Test262DiagnosticTest {
        index,
        file: relative_test_path(&test.path, test_dir),
        variant: test.variant.report_label().map(ToString::to_string),
        category: test.category.clone(),
        flags: metadata.flags.clone(),
        features: metadata.features.clone(),
        includes: metadata.includes.clone(),
        negative_phase: metadata.negative.as_ref().map(|negative| {
            negative.error_type.as_ref().map_or_else(
                || negative.phase.clone(),
                |kind| format!("{}:{kind}", negative.phase),
            )
        }),
        async_test: has_async_flag(metadata),
        module_goal: is_module_test(metadata),
        timeout_ms,
    }
}

fn workspace_root() -> Result<PathBuf, String> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(|error| format!("failed to resolve workspace root: {error}"))
}
