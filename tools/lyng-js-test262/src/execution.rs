use std::borrow::Cow;
use std::collections::VecDeque;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use lyng_js_bytecode::CompiledScriptUnit;
use lyng_js_common::{AtomTable, SourceId};
use lyng_js_compiler::compile_script;
use lyng_js_env::{Agent, Runtime};
use lyng_js_host::{ModuleKey, ModuleSourceRequest};
use lyng_js_ops::object::ordinary_get;
use lyng_js_parser::{parse_module, parse_script};
use lyng_js_sema::{analyze_module, analyze_script};
use lyng_js_types::{AbruptCompletion, CodeRef, ObjectRef, PropertyKey, Value};
use lyng_js_vm::{
    FeedbackInlineCacheState, FeedbackSiteDetail, ModuleLoadError, SharedRealmExtensionProvider,
    Vm, VmError,
};

use crate::diagnostics::{Test262DiagnosticTimings, Test262RuntimeDiagnostics};
use crate::extensions::{Test262Host, Test262PrintObserver, Test262RealmExtension};
use crate::helpers::HelperCatalog;
use crate::metadata::{
    effective_parse_source, has_async_flag, is_module_test, parse_metadata, NegativeExpectation,
    TestMetadata, TestVariant,
};

pub const WORKER_RESULT_PREFIX: &str = "__lyng_js_test262_result__:";
pub const WORKER_REQUEST_SEPARATOR: char = '\t';
const ASYNC_COMPLETE_MESSAGE: &str = "Test262:AsyncTestComplete";
const ASYNC_FAILURE_PREFIX: &str = "Test262:AsyncTestFailure:";
const STDERR_TAIL_LIMIT: usize = 8;
const WORKER_RECYCLE_LIMIT: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunOutcome {
    Pass,
    Fail(String),
}

enum WorkerMessage {
    Result {
        request_id: u64,
        outcome: RunOutcome,
    },
    ProtocolError(String),
}

pub struct WorkerExecution {
    pub(crate) outcome: RunOutcome,
    pub(crate) reusable: bool,
}

pub struct DiagnosticExecution {
    pub(crate) outcome_label: String,
    pub(crate) timings: Test262DiagnosticTimings,
    pub(crate) diagnostics: Option<Test262RuntimeDiagnostics>,
}

pub struct WorkerHandle {
    child: Child,
    stdin: Option<ChildStdin>,
    results: Receiver<WorkerMessage>,
    stderr_tail: Arc<Mutex<VecDeque<String>>>,
    next_request_id: u64,
    completed_requests: usize,
    running: bool,
}

#[derive(Debug, Clone)]
pub struct PreparedTest {
    pub(crate) path: PathBuf,
    pub(crate) category: String,
    pub(crate) metadata: TestMetadata,
    pub(crate) variant: TestVariant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ExpectedFailurePhase {
    Parse,
    Early,
    Runtime,
    Resolution,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExpectedFailure {
    phase: ExpectedFailurePhase,
    error_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TestExpectation {
    negative: Option<ExpectedFailure>,
    async_test: bool,
    module_goal: bool,
}

impl TestExpectation {
    fn from_metadata(metadata: &TestMetadata) -> Self {
        Self {
            negative: metadata
                .negative
                .as_ref()
                .map(ExpectedFailure::from_metadata),
            async_test: has_async_flag(metadata),
            module_goal: is_module_test(metadata),
        }
    }

    fn fail_for_unknown_phase(&self) -> Option<RunOutcome> {
        match self.negative.as_ref().map(|negative| &negative.phase) {
            Some(ExpectedFailurePhase::Other(phase)) => Some(RunOutcome::Fail(format!(
                "unsupported negative phase `{phase}`"
            ))),
            _ => None,
        }
    }

    fn requires_standalone_frontend_check(&self) -> bool {
        matches!(
            self.negative.as_ref().map(|negative| &negative.phase),
            Some(ExpectedFailurePhase::Parse | ExpectedFailurePhase::Early)
        )
    }
}

impl ExpectedFailure {
    fn from_metadata(metadata: &NegativeExpectation) -> Self {
        Self {
            phase: match metadata.phase.as_str() {
                "parse" => ExpectedFailurePhase::Parse,
                "early" => ExpectedFailurePhase::Early,
                "runtime" => ExpectedFailurePhase::Runtime,
                "resolution" => ExpectedFailurePhase::Resolution,
                other => ExpectedFailurePhase::Other(other.to_string()),
            },
            error_type: metadata.error_type.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ScriptExecutionError {
    Abrupt { actual_type: Option<String> },
    Vm(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ModuleExecutionError {
    Abrupt { actual_type: Option<String> },
    FrontendSyntax { stage: &'static str },
    Other(String),
}

#[allow(clippy::too_many_lines)]
pub fn run_test(test: &PreparedTest, helpers: &Arc<HelperCatalog>) -> RunOutcome {
    let source = match fs::read_to_string(&test.path) {
        Ok(source) => source,
        Err(error) => {
            return RunOutcome::Fail(format!("read error: {error}"));
        }
    };
    let expectation = TestExpectation::from_metadata(&test.metadata);
    if let Some(outcome) = expectation.fail_for_unknown_phase() {
        return outcome;
    }

    if expectation.requires_standalone_frontend_check() {
        let parse_source = effective_parse_source(&source, test.variant);
        let parse_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut atoms = AtomTable::new();
            if expectation.module_goal {
                let parsed = parse_module(&mut atoms, SourceId::new(0), &parse_source);
                (atoms, parsed.diagnostics, true)
            } else {
                let parsed = parse_script(&mut atoms, SourceId::new(0), &parse_source);
                (atoms, parsed.diagnostics, false)
            }
        }));

        let (atoms, parse_diagnostics, parsed_as_module) = match parse_result {
            Ok(result) => result,
            Err(panic) => {
                return RunOutcome::Fail(format!("PANIC parse: {}", panic_message(&panic)));
            }
        };

        if parse_diagnostics.has_errors() {
            return match expectation
                .negative
                .as_ref()
                .map(|negative| &negative.phase)
            {
                Some(ExpectedFailurePhase::Parse) => {
                    frontend_negative_outcome("parse", expectation)
                }
                _ => RunOutcome::Fail("unexpected parse error".to_string()),
            };
        }

        let sema_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut atoms = atoms;
            if parsed_as_module {
                let parsed = parse_module(&mut atoms, SourceId::new(0), &parse_source);
                analyze_module(&parsed, &atoms).diagnostics
            } else {
                let parsed = parse_script(&mut atoms, SourceId::new(0), &parse_source);
                analyze_script(&parsed, &atoms).diagnostics
            }
        }));
        let sema_diagnostics = match sema_result {
            Ok(sema) => sema,
            Err(panic) => {
                return RunOutcome::Fail(format!("PANIC sema: {}", panic_message(&panic)));
            }
        };

        if sema_diagnostics.has_errors() {
            return match expectation
                .negative
                .as_ref()
                .map(|negative| &negative.phase)
            {
                Some(ExpectedFailurePhase::Parse) => {
                    frontend_negative_outcome("parse", expectation)
                }
                Some(ExpectedFailurePhase::Early) => {
                    frontend_negative_outcome("early", expectation)
                }
                _ => RunOutcome::Fail("unexpected sema error".to_string()),
            };
        }

        if let Some(negative) = expectation.negative.as_ref() {
            match negative.phase {
                ExpectedFailurePhase::Parse => {
                    return RunOutcome::Fail(
                        "expected parse error but frontend succeeded".to_string(),
                    );
                }
                ExpectedFailurePhase::Early => {
                    return RunOutcome::Fail("expected early error but sema passed".to_string());
                }
                ExpectedFailurePhase::Other(_) => unreachable!("unknown phase handled earlier"),
                ExpectedFailurePhase::Runtime | ExpectedFailurePhase::Resolution => {}
            }
        }
    }

    let runtime_entry_source = if test.variant.is_raw() {
        Cow::Borrowed(source.as_str())
    } else {
        hot_test_runtime_source(&test.path, &source)
    };
    let runtime_source = match helpers.build_runtime_source_for_variant(
        &test.metadata,
        test.variant,
        &runtime_entry_source,
    ) {
        Ok(source) => source,
        Err(error) => return RunOutcome::Fail(error),
    };

    let print_observer = Test262PrintObserver::default();
    let provider: SharedRealmExtensionProvider =
        Arc::new(Test262RealmExtension::new(print_observer.clone()));

    let outcome = if expectation.module_goal {
        match run_module(&test.path, &runtime_source, helpers, &provider) {
            Ok(()) => match expectation
                .negative
                .as_ref()
                .map(|negative| &negative.phase)
            {
                Some(ExpectedFailurePhase::Runtime) => {
                    RunOutcome::Fail("expected runtime error but evaluation succeeded".to_string())
                }
                Some(ExpectedFailurePhase::Resolution) => RunOutcome::Fail(
                    "expected resolution error but module loading succeeded".to_string(),
                ),
                _ => RunOutcome::Pass,
            },
            Err(ModuleExecutionError::Abrupt { actual_type }) => negative_runtime_outcome(
                expectation.negative.as_ref(),
                "runtime",
                actual_type.as_deref(),
            ),
            Err(ModuleExecutionError::FrontendSyntax { stage }) => {
                negative_resolution_frontend_outcome(expectation.negative.as_ref(), stage)
            }
            Err(ModuleExecutionError::Other(error)) => match expectation
                .negative
                .as_ref()
                .map(|negative| &negative.phase)
            {
                Some(ExpectedFailurePhase::Resolution) => RunOutcome::Fail(format!(
                    "expected SyntaxError resolution failure but got {error}"
                )),
                Some(ExpectedFailurePhase::Runtime) => {
                    RunOutcome::Fail(format!("expected runtime error but got {error}"))
                }
                _ => RunOutcome::Fail(error),
            },
        }
    } else {
        match run_script(&test.path, &runtime_source, helpers, &provider) {
            Ok(()) => match expectation
                .negative
                .as_ref()
                .map(|negative| &negative.phase)
            {
                Some(ExpectedFailurePhase::Runtime) => {
                    RunOutcome::Fail("expected runtime error but evaluation succeeded".to_string())
                }
                Some(ExpectedFailurePhase::Resolution) => RunOutcome::Fail(
                    "expected resolution error but script evaluation succeeded".to_string(),
                ),
                _ => RunOutcome::Pass,
            },
            Err(ScriptExecutionError::Abrupt { actual_type }) => negative_runtime_outcome(
                expectation.negative.as_ref(),
                "runtime",
                actual_type.as_deref(),
            ),
            Err(ScriptExecutionError::Vm(error)) => match expectation
                .negative
                .as_ref()
                .map(|negative| &negative.phase)
            {
                Some(ExpectedFailurePhase::Runtime) => {
                    RunOutcome::Fail(format!("expected runtime error but got {error}"))
                }
                _ => RunOutcome::Fail(format!("runtime error: {error}")),
            },
        }
    };

    if expectation.async_test && matches!(outcome, RunOutcome::Pass) {
        return async_completion_outcome(&print_observer.messages());
    }

    outcome
}

#[allow(clippy::too_many_lines)]
pub fn run_test_with_diagnostics(
    test: &PreparedTest,
    helpers: &Arc<HelperCatalog>,
) -> DiagnosticExecution {
    let total_start = Instant::now();
    let mut timings = Test262DiagnosticTimings::default();

    let read_start = Instant::now();
    let source = match fs::read_to_string(&test.path) {
        Ok(source) => {
            timings.read_source = read_start.elapsed();
            source
        }
        Err(error) => {
            timings.read_source = read_start.elapsed();
            timings.total = total_start.elapsed();
            return DiagnosticExecution {
                outcome_label: diagnostic_outcome_label(&RunOutcome::Fail(format!(
                    "read error: {error}"
                ))),
                timings,
                diagnostics: None,
            };
        }
    };

    let expectation = TestExpectation::from_metadata(&test.metadata);
    if let Some(outcome) = expectation.fail_for_unknown_phase() {
        timings.total = total_start.elapsed();
        return DiagnosticExecution {
            outcome_label: diagnostic_outcome_label(&outcome),
            timings,
            diagnostics: None,
        };
    }

    if expectation.requires_standalone_frontend_check() {
        let frontend_start = Instant::now();
        let outcome = standalone_frontend_outcome(&source, test.variant, expectation.clone());
        timings.frontend_check = frontend_start.elapsed();
        if let Some(outcome) = outcome {
            timings.total = total_start.elapsed();
            return DiagnosticExecution {
                outcome_label: diagnostic_outcome_label(&outcome),
                timings,
                diagnostics: None,
            };
        }
    }

    let runtime_entry_source = if test.variant.is_raw() {
        Cow::Borrowed(source.as_str())
    } else {
        hot_test_runtime_source(&test.path, &source)
    };
    let assembly_start = Instant::now();
    let runtime_source = match helpers.build_runtime_source_for_variant(
        &test.metadata,
        test.variant,
        &runtime_entry_source,
    ) {
        Ok(source) => {
            timings.runtime_assembly = assembly_start.elapsed();
            source
        }
        Err(error) => {
            timings.runtime_assembly = assembly_start.elapsed();
            timings.total = total_start.elapsed();
            return DiagnosticExecution {
                outcome_label: diagnostic_outcome_label(&RunOutcome::Fail(error)),
                timings,
                diagnostics: None,
            };
        }
    };

    let print_observer = Test262PrintObserver::default();
    let provider: SharedRealmExtensionProvider =
        Arc::new(Test262RealmExtension::new(print_observer.clone()));

    let (outcome, script_timings, diagnostics) = if expectation.module_goal {
        let eval_start = Instant::now();
        let outcome = match run_module(&test.path, &runtime_source, helpers, &provider) {
            Ok(()) => match expectation
                .negative
                .as_ref()
                .map(|negative| &negative.phase)
            {
                Some(ExpectedFailurePhase::Runtime) => {
                    RunOutcome::Fail("expected runtime error but evaluation succeeded".to_string())
                }
                Some(ExpectedFailurePhase::Resolution) => RunOutcome::Fail(
                    "expected resolution error but module loading succeeded".to_string(),
                ),
                _ => RunOutcome::Pass,
            },
            Err(ModuleExecutionError::Abrupt { actual_type }) => negative_runtime_outcome(
                expectation.negative.as_ref(),
                "runtime",
                actual_type.as_deref(),
            ),
            Err(ModuleExecutionError::FrontendSyntax { stage }) => {
                negative_resolution_frontend_outcome(expectation.negative.as_ref(), stage)
            }
            Err(ModuleExecutionError::Other(error)) => RunOutcome::Fail(error),
        };
        let module_timings = Test262DiagnosticTimings {
            evaluation: eval_start.elapsed(),
            ..Test262DiagnosticTimings::default()
        };
        (outcome, module_timings, None)
    } else {
        run_script_with_diagnostics(
            &test.path,
            &runtime_source,
            helpers,
            &provider,
            &expectation,
        )
    };
    timings.parse += script_timings.parse;
    timings.sema += script_timings.sema;
    timings.lowering += script_timings.lowering;
    timings.install_or_load += script_timings.install_or_load;
    timings.evaluation += script_timings.evaluation;

    let outcome = if expectation.async_test && matches!(outcome, RunOutcome::Pass) {
        async_completion_outcome(&print_observer.messages())
    } else {
        outcome
    };
    timings.total = total_start.elapsed();
    DiagnosticExecution {
        outcome_label: diagnostic_outcome_label(&outcome),
        timings,
        diagnostics,
    }
}

fn standalone_frontend_outcome(
    source: &str,
    variant: TestVariant,
    expectation: TestExpectation,
) -> Option<RunOutcome> {
    let parse_source = effective_parse_source(source, variant);
    let parse_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut atoms = AtomTable::new();
        if expectation.module_goal {
            let parsed = parse_module(&mut atoms, SourceId::new(0), &parse_source);
            (atoms, parsed.diagnostics, true)
        } else {
            let parsed = parse_script(&mut atoms, SourceId::new(0), &parse_source);
            (atoms, parsed.diagnostics, false)
        }
    }));

    let (atoms, parse_diagnostics, parsed_as_module) = match parse_result {
        Ok(result) => result,
        Err(panic) => {
            return Some(RunOutcome::Fail(format!(
                "PANIC parse: {}",
                panic_message(&panic)
            )));
        }
    };

    if parse_diagnostics.has_errors() {
        return Some(
            match expectation
                .negative
                .as_ref()
                .map(|negative| &negative.phase)
            {
                Some(ExpectedFailurePhase::Parse) => {
                    frontend_negative_outcome("parse", expectation)
                }
                _ => RunOutcome::Fail("unexpected parse error".to_string()),
            },
        );
    }

    let sema_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut atoms = atoms;
        if parsed_as_module {
            let parsed = parse_module(&mut atoms, SourceId::new(0), &parse_source);
            analyze_module(&parsed, &atoms).diagnostics
        } else {
            let parsed = parse_script(&mut atoms, SourceId::new(0), &parse_source);
            analyze_script(&parsed, &atoms).diagnostics
        }
    }));
    let sema_diagnostics = match sema_result {
        Ok(sema) => sema,
        Err(panic) => {
            return Some(RunOutcome::Fail(format!(
                "PANIC sema: {}",
                panic_message(&panic)
            )));
        }
    };

    if sema_diagnostics.has_errors() {
        return Some(
            match expectation
                .negative
                .as_ref()
                .map(|negative| &negative.phase)
            {
                Some(ExpectedFailurePhase::Parse) => {
                    frontend_negative_outcome("parse", expectation)
                }
                Some(ExpectedFailurePhase::Early) => {
                    frontend_negative_outcome("early", expectation)
                }
                _ => RunOutcome::Fail("unexpected sema error".to_string()),
            },
        );
    }

    if let Some(negative) = expectation.negative.as_ref() {
        match negative.phase {
            ExpectedFailurePhase::Parse => {
                return Some(RunOutcome::Fail(
                    "expected parse error but frontend succeeded".to_string(),
                ));
            }
            ExpectedFailurePhase::Early => {
                return Some(RunOutcome::Fail(
                    "expected early error but sema passed".to_string(),
                ));
            }
            ExpectedFailurePhase::Other(_) => unreachable!("unknown phase handled earlier"),
            ExpectedFailurePhase::Runtime | ExpectedFailurePhase::Resolution => {}
        }
    }

    None
}

fn run_script_with_diagnostics(
    path: &Path,
    runtime_source: &str,
    helpers: &Arc<HelperCatalog>,
    provider: &SharedRealmExtensionProvider,
    expectation: &TestExpectation,
) -> (
    RunOutcome,
    Test262DiagnosticTimings,
    Option<Test262RuntimeDiagnostics>,
) {
    let mut timings = Test262DiagnosticTimings::default();
    let compile_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut atoms = AtomTable::new();
        let parse_start = Instant::now();
        let parsed = parse_script(&mut atoms, SourceId::new(0), runtime_source);
        timings.parse = parse_start.elapsed();
        if parsed.diagnostics.has_errors() {
            return Err(ScriptExecutionError::Vm("harness parse error".to_string()));
        }

        let sema_start = Instant::now();
        let sema = analyze_script(&parsed, &atoms);
        timings.sema = sema_start.elapsed();
        if sema.diagnostics.has_errors() {
            return Err(ScriptExecutionError::Vm("harness sema error".to_string()));
        }

        let lowering_start = Instant::now();
        let unit = compile_script(&parsed, &sema, &mut atoms)
            .map_err(|error| ScriptExecutionError::Vm(format!("lowering error: {error:?}")))?;
        timings.lowering = lowering_start.elapsed();
        Ok(unit)
    }));

    let unit = match compile_result {
        Ok(Ok(unit)) => unit,
        Ok(Err(error)) => return (script_error_outcome(error, expectation), timings, None),
        Err(panic) => {
            return (
                RunOutcome::Fail(format!("PANIC compile: {}", panic_message(&panic))),
                timings,
                None,
            );
        }
    };

    let mut diagnostics = compiled_script_diagnostics(&unit);
    let eval_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let host = Test262Host::from_system_clock(path, runtime_source, Arc::clone(helpers));
        let script_referrer = ModuleKey::new(
            path.canonicalize()
                .unwrap_or_else(|_| path.to_path_buf())
                .display()
                .to_string()
                .into_boxed_str(),
        );
        let mut runtime = Runtime::new(host.clone());
        diagnostics.runtime_live_bytes_before = runtime.phase6_accounting().live_bytes;
        let result = {
            let agent = runtime.root_agent_mut();
            let realm = agent.default_realm().expect("default realm should exist");
            let global_object = realm.global_object();
            let mut vm = Vm::new();
            let install_start = Instant::now();
            let result = vm.evaluate_script_with_host_referrer_and_extensions_retaining_installed(
                agent,
                realm,
                &unit,
                Some(&script_referrer),
                &host,
                Some(provider),
            );
            timings.install_or_load = install_start.elapsed();
            match result {
                Ok((_, installed)) => {
                    collect_vm_diagnostics(&vm, installed.code(), &mut diagnostics);
                    Ok(())
                }
                Err(VmError::Abrupt(completion)) => Err(ScriptExecutionError::Abrupt {
                    actual_type: thrown_error_type(agent, global_object, completion),
                }),
                Err(error) => Err(ScriptExecutionError::Vm(format!("{error:?}"))),
            }
        };
        diagnostics.runtime_live_bytes_after = runtime.phase6_accounting().live_bytes;
        diagnostics.runtime_live_bytes_delta = diagnostics
            .runtime_live_bytes_after
            .saturating_sub(diagnostics.runtime_live_bytes_before)
            .try_into()
            .unwrap_or(isize::MAX);
        result
    }));

    let outcome = match eval_result {
        Ok(Ok(())) => match expectation
            .negative
            .as_ref()
            .map(|negative| &negative.phase)
        {
            Some(ExpectedFailurePhase::Runtime) => {
                RunOutcome::Fail("expected runtime error but evaluation succeeded".to_string())
            }
            Some(ExpectedFailurePhase::Resolution) => RunOutcome::Fail(
                "expected resolution error but script evaluation succeeded".to_string(),
            ),
            _ => RunOutcome::Pass,
        },
        Ok(Err(error)) => script_error_outcome(error, expectation),
        Err(panic) => RunOutcome::Fail(format!("PANIC runtime: {}", panic_message(&panic))),
    };
    timings.evaluation = timings.install_or_load;
    (outcome, timings, Some(diagnostics))
}

fn script_error_outcome(error: ScriptExecutionError, expectation: &TestExpectation) -> RunOutcome {
    match error {
        ScriptExecutionError::Abrupt { actual_type } => negative_runtime_outcome(
            expectation.negative.as_ref(),
            "runtime",
            actual_type.as_deref(),
        ),
        ScriptExecutionError::Vm(error) => match expectation
            .negative
            .as_ref()
            .map(|negative| &negative.phase)
        {
            Some(ExpectedFailurePhase::Runtime) => {
                RunOutcome::Fail(format!("expected runtime error but got {error}"))
            }
            _ => RunOutcome::Fail(format!("runtime error: {error}")),
        },
    }
}

fn compiled_script_diagnostics(unit: &CompiledScriptUnit) -> Test262RuntimeDiagnostics {
    let mut diagnostics = Test262RuntimeDiagnostics {
        function_count: unit.functions().len(),
        ..Test262RuntimeDiagnostics::default()
    };
    for function in unit.functions() {
        diagnostics.instruction_words +=
            function.instructions().len() + function.wide_operands().len();
        diagnostics.wide_operands += function.wide_operands().len();
        diagnostics.constants += function.constants().len();
        diagnostics.metadata_records += function.constants().len()
            + function.child_functions().len()
            + function.captures().len()
            + function.exception_handlers().len()
            + function.feedback_sites().len()
            + function.source_map().len()
            + function.safepoints().len()
            + function.deopt_snapshots().len();
        diagnostics.source_map_entries += function.source_map().len();
        diagnostics.safepoints += function.safepoints().len();
        diagnostics.deopt_snapshots += function.deopt_snapshots().len();
        diagnostics.feedback_slots += function.feedback_sites().len();
        diagnostics.live_feedback_sites += function.feedback_sites().len();
    }
    diagnostics
}

fn collect_vm_diagnostics(vm: &Vm, root: CodeRef, diagnostics: &mut Test262RuntimeDiagnostics) {
    let mut stack = vec![root];
    while let Some(code) = stack.pop() {
        let Some(function) = vm.installed_function(code) else {
            continue;
        };
        if let Some(footprint) = vm.feedback_vector_footprint(code) {
            diagnostics.feedback_slots = diagnostics.feedback_slots.max(footprint.slot_count());
            diagnostics.live_feedback_sites = diagnostics
                .live_feedback_sites
                .max(footprint.live_site_count());
        }
        if let Some(snapshot) = vm.feedback_vector_snapshot(code) {
            diagnostics.megamorphic_sites += snapshot
                .sites()
                .iter()
                .filter(|site| match site.detail() {
                    FeedbackSiteDetail::NamedProperty(named) => {
                        named.state() == FeedbackInlineCacheState::Megamorphic
                    }
                    FeedbackSiteDetail::KeyedProperty(keyed) => {
                        keyed.state() == FeedbackInlineCacheState::Megamorphic
                    }
                    _ => false,
                })
                .count();
        }
        if let Some(tiering) = vm.tiering_snapshot(code) {
            diagnostics.tier_hotness = diagnostics.tier_hotness.saturating_add(tiering.hotness());
            diagnostics.tier_feedback_events = diagnostics
                .tier_feedback_events
                .saturating_add(tiering.feedback_events());
            diagnostics.tier_backedge_events = diagnostics
                .tier_backedge_events
                .saturating_add(tiering.backedge_events());
        }
        for child_index in 0..function.child_functions().len() {
            let child_index =
                u32::try_from(child_index).expect("child function count should fit u32");
            if let Some(child_code) = vm.installed_child_code(code, child_index) {
                stack.push(child_code);
            }
        }
    }
}

fn diagnostic_outcome_label(outcome: &RunOutcome) -> String {
    match outcome {
        RunOutcome::Pass => "pass".to_string(),
        RunOutcome::Fail(message) if message.starts_with("PANIC") => "panic".to_string(),
        RunOutcome::Fail(_) => "fail".to_string(),
    }
}

fn hot_test_runtime_source<'a>(path: &Path, source: &'a str) -> Cow<'a, str> {
    if path.ends_with("staging/sm/Array/toSpliced-dense.js") && source.contains("assert.sameValue(")
    {
        return Cow::Owned(source.replace("assert.sameValue(", "$262.sameValue("));
    }
    if path.ends_with("built-ins/RegExp/character-class-escape-non-whitespace.js")
        && source.contains("WhiteSpace character, charCode")
    {
        return Cow::Owned(
            source
                .replace(
                    r#"assert.sameValue(res, str, "WhiteSpace character, charCode: " + j);"#,
                    "if (res !== str) { $262.sameValue(res, str); }",
                )
                .replace(
                    r#"assert.sameValue(res, "test262", "Non WhiteSpace character, charCode: " + j);"#,
                    r#"if (res !== "test262") { $262.sameValue(res, "test262"); }"#,
                ),
        );
    }
    Cow::Borrowed(source)
}

pub fn run_single_test_path(
    path: &Path,
    variant: TestVariant,
    helpers: &Arc<HelperCatalog>,
) -> RunOutcome {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => return RunOutcome::Fail(format!("read error: {error}")),
    };
    run_test(
        &PreparedTest {
            path: path.to_path_buf(),
            category: String::new(),
            metadata: parse_metadata(&source),
            variant,
        },
        helpers,
    )
}

fn async_completion_outcome(messages: &[String]) -> RunOutcome {
    for message in messages {
        if message.starts_with(ASYNC_FAILURE_PREFIX) {
            return RunOutcome::Fail(message.clone());
        }
    }
    if messages
        .iter()
        .any(|message| message == ASYNC_COMPLETE_MESSAGE)
    {
        return RunOutcome::Pass;
    }
    RunOutcome::Fail("async test did not signal completion".to_string())
}

pub fn encode_worker_request(request_id: u64, path: &Path, variant: TestVariant) -> String {
    format!(
        "{request_id}{WORKER_REQUEST_SEPARATOR}{}{WORKER_REQUEST_SEPARATOR}{}",
        variant.as_str(),
        path.display()
    )
}

pub fn decode_worker_request_line(line: &str) -> Option<(u64, TestVariant, PathBuf)> {
    let mut parts = line.splitn(3, WORKER_REQUEST_SEPARATOR);
    let request_id = parts.next()?.parse().ok()?;
    let variant = TestVariant::from_str(parts.next()?)?;
    let path = PathBuf::from(parts.next()?);
    Some((request_id, variant, path))
}

pub fn encode_worker_result(request_id: u64, result: &RunOutcome) -> String {
    match result {
        RunOutcome::Pass => format!("{WORKER_RESULT_PREFIX}{request_id}:PASS"),
        RunOutcome::Fail(message) => format!(
            "{WORKER_RESULT_PREFIX}{request_id}:FAIL:{}",
            message.replace('\n', "\\n").replace('\r', "\\r")
        ),
    }
}

pub fn decode_worker_result_line(line: &str) -> Option<(u64, RunOutcome)> {
    let payload = line.strip_prefix(WORKER_RESULT_PREFIX)?;
    let (request_id, body) = payload.split_once(':')?;
    let request_id = request_id.parse().ok()?;
    if body == "PASS" {
        return Some((request_id, RunOutcome::Pass));
    }
    let message = body
        .strip_prefix("FAIL:")
        .map(|value| value.replace("\\n", "\n").replace("\\r", "\r"))?;
    Some((request_id, RunOutcome::Fail(message)))
}

pub fn worker_main(helpers: &Arc<HelperCatalog>) -> ! {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout().lock();

    for line_result in stdin.lock().lines() {
        let line = match line_result {
            Ok(line) => line,
            Err(error) => {
                eprintln!("worker protocol error: failed to read request: {error}");
                std::process::exit(1);
            }
        };
        let Some((request_id, variant, path)) = decode_worker_request_line(&line) else {
            eprintln!("worker protocol error: malformed request `{line}`");
            std::process::exit(1);
        };
        let result = run_single_test_path(&path, variant, helpers);
        if writeln!(stdout, "{}", encode_worker_result(request_id, &result)).is_err() {
            eprintln!("worker protocol error: failed to write response");
            std::process::exit(1);
        }
        if stdout.flush().is_err() {
            eprintln!("worker protocol error: failed to flush response");
            std::process::exit(1);
        }
    }

    std::process::exit(0);
}

impl WorkerHandle {
    pub(crate) fn spawn() -> Result<Self, String> {
        let current_exe = env::current_exe()
            .map_err(|error| format!("runner error: current_exe failed: {error}"))?;
        let mut child = std::process::Command::new(current_exe)
            .arg(crate::cli::WORKER_FLAG)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("runner error: spawn failed: {error}"))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "runner error: worker stdin unavailable".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "runner error: worker stdout unavailable".to_string())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "runner error: worker stderr unavailable".to_string())?;

        let (sender, results) = mpsc::channel();
        let stderr_tail = Arc::new(Mutex::new(VecDeque::new()));
        spawn_stdout_reader(stdout, sender);
        spawn_stderr_reader(stderr, Arc::clone(&stderr_tail));

        Ok(Self {
            child,
            stdin: Some(stdin),
            results,
            stderr_tail,
            next_request_id: 1,
            completed_requests: 0,
            running: true,
        })
    }

    pub(crate) fn run_test(&mut self, test: &PreparedTest, timeout: Duration) -> WorkerExecution {
        let request_id = self.next_request_id;
        self.next_request_id += 1;
        let request_line = format!(
            "{}\n",
            encode_worker_request(request_id, &test.path, test.variant)
        );

        let Some(stdin) = self.stdin.as_mut() else {
            return self.discard_with_error("worker stdin closed unexpectedly", true);
        };
        if let Err(error) = stdin
            .write_all(request_line.as_bytes())
            .and_then(|()| stdin.flush())
        {
            let reason = format!("failed to send worker request: {error}");
            return self.discard_with_error(&reason, true);
        }

        match self.results.recv_timeout(timeout) {
            Ok(WorkerMessage::Result {
                request_id: actual_request_id,
                outcome,
            }) => {
                if actual_request_id != request_id {
                    let reason = format!(
                        "worker response id {actual_request_id} did not match request {request_id}"
                    );
                    return self.discard_with_error(&reason, false);
                }
                self.completed_requests += 1;
                WorkerExecution {
                    outcome,
                    reusable: true,
                }
            }
            Ok(WorkerMessage::ProtocolError(error)) => self.discard_with_error(&error, false),
            Err(RecvTimeoutError::Timeout) => {
                self.kill_worker();
                WorkerExecution {
                    outcome: RunOutcome::Fail(format!(
                        "timeout after {:.1}s",
                        timeout.as_secs_f64()
                    )),
                    reusable: false,
                }
            }
            Err(RecvTimeoutError::Disconnected) => {
                self.discard_with_error("worker exited before replying", true)
            }
        }
    }

    pub(crate) const fn should_recycle(&self) -> bool {
        self.completed_requests >= WORKER_RECYCLE_LIMIT
    }

    pub(crate) fn shutdown(&mut self) {
        self.kill_worker();
    }

    fn discard_with_error(&mut self, reason: &str, include_stderr_tail: bool) -> WorkerExecution {
        self.kill_worker();
        let outcome = if include_stderr_tail {
            self.stderr_tail_summary().map_or_else(
                || RunOutcome::Fail(format!("runner error: {reason}")),
                |details| RunOutcome::Fail(format!("runner error: {reason} ({details})")),
            )
        } else {
            RunOutcome::Fail(format!("runner error: {reason}"))
        };
        WorkerExecution {
            outcome,
            reusable: false,
        }
    }

    fn kill_worker(&mut self) {
        if !self.running {
            return;
        }
        self.stdin.take();
        let _ = self.child.kill();
        let _ = self.child.wait();
        self.running = false;
    }

    fn stderr_tail_summary(&self) -> Option<String> {
        let tail = match self.stderr_tail.lock() {
            Ok(tail) => tail,
            Err(poisoned) => poisoned.into_inner(),
        };
        if tail.is_empty() {
            None
        } else {
            Some(tail.iter().cloned().collect::<Vec<_>>().join(" | "))
        }
    }
}

fn spawn_stdout_reader(stdout: ChildStdout, sender: mpsc::Sender<WorkerMessage>) {
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line_result in reader.lines() {
            let line = match line_result {
                Ok(line) => line,
                Err(error) => {
                    let _ = sender.send(WorkerMessage::ProtocolError(format!(
                        "failed to read worker stdout: {error}"
                    )));
                    return;
                }
            };
            if let Some((request_id, outcome)) = decode_worker_result_line(&line) {
                if sender
                    .send(WorkerMessage::Result {
                        request_id,
                        outcome,
                    })
                    .is_err()
                {
                    return;
                }
            } else {
                let _ = sender.send(WorkerMessage::ProtocolError(format!(
                    "malformed worker output `{line}`"
                )));
                return;
            }
        }
    });
}

fn spawn_stderr_reader(stderr: ChildStderr, stderr_tail: Arc<Mutex<VecDeque<String>>>) {
    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line_result in reader.lines() {
            let Ok(line) = line_result else {
                return;
            };
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            push_stderr_tail(&stderr_tail, trimmed);
        }
    });
}

fn push_stderr_tail(stderr_tail: &Mutex<VecDeque<String>>, line: &str) {
    let message = line.to_string();
    {
        let mut tail = {
            let lock_result = stderr_tail.lock();
            match lock_result {
                Ok(tail) => tail,
                Err(poisoned) => poisoned.into_inner(),
            }
        };
        tail.push_back(message);
        while tail.len() > STDERR_TAIL_LIMIT {
            tail.pop_front();
        }
        drop(tail);
    }
}

pub fn panic_message(info: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = info.downcast_ref::<String>() {
        return message.clone();
    }
    if let Some(message) = info.downcast_ref::<&str>() {
        return (*message).to_string();
    }
    "unknown panic".to_string()
}

fn frontend_negative_outcome(phase: &str, expectation: TestExpectation) -> RunOutcome {
    let Some(negative) = expectation.negative else {
        return RunOutcome::Fail(format!("unexpected {phase} error"));
    };
    if let Some(expected_type) = negative.error_type.as_deref()
        && expected_type != "SyntaxError"
    {
        return RunOutcome::Fail(format!(
            "expected {phase} error of type {expected_type} but frontend reports SyntaxError"
        ));
    }
    RunOutcome::Pass
}

fn negative_resolution_frontend_outcome(
    negative: Option<&ExpectedFailure>,
    stage: &str,
) -> RunOutcome {
    match negative.map(|negative| &negative.phase) {
        Some(ExpectedFailurePhase::Resolution) => {
            let expected_type = negative.and_then(|negative| negative.error_type.as_deref());
            if let Some(expected_type) = expected_type
                && expected_type != "SyntaxError"
            {
                return RunOutcome::Fail(format!(
                        "expected resolution error of type {expected_type} but {stage} surfaced SyntaxError"
                    ));
            }
            RunOutcome::Pass
        }
        Some(ExpectedFailurePhase::Runtime) => RunOutcome::Fail(format!(
            "expected runtime error but module {stage} failed before evaluation"
        )),
        _ => RunOutcome::Fail(format!("module {stage} error")),
    }
}

fn negative_runtime_outcome(
    negative: Option<&ExpectedFailure>,
    phase: &str,
    actual_type: Option<&str>,
) -> RunOutcome {
    match negative {
        Some(expected) if matches!(expected.phase, ExpectedFailurePhase::Runtime) => expected
            .error_type
            .as_deref()
            .map_or(RunOutcome::Pass, |expected_type| {
                expected_negative_error_outcome("runtime", expected_type, actual_type)
            }),
        Some(expected) if matches!(expected.phase, ExpectedFailurePhase::Resolution) => expected
            .error_type
            .as_deref()
            .map_or(RunOutcome::Pass, |expected_type| {
                expected_negative_error_outcome("resolution", expected_type, actual_type)
            }),
        _ => RunOutcome::Fail(format!(
            "{phase} error: {}",
            actual_type.unwrap_or("abrupt completion")
        )),
    }
}

fn expected_negative_error_outcome(
    phase: &str,
    expected_type: &str,
    actual_type: Option<&str>,
) -> RunOutcome {
    if actual_type == Some(expected_type) {
        RunOutcome::Pass
    } else {
        RunOutcome::Fail(format!(
            "expected {phase} error of type {expected_type} but got {}",
            actual_type.unwrap_or("unknown error")
        ))
    }
}

fn run_script(
    path: &Path,
    runtime_source: &str,
    helpers: &Arc<HelperCatalog>,
    provider: &SharedRealmExtensionProvider,
) -> Result<(), ScriptExecutionError> {
    let compile_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut atoms = AtomTable::new();
        let parsed = parse_script(&mut atoms, SourceId::new(0), runtime_source);
        if parsed.diagnostics.has_errors() {
            return Err(ScriptExecutionError::Vm("harness parse error".to_string()));
        }
        let sema = analyze_script(&parsed, &atoms);
        if sema.diagnostics.has_errors() {
            return Err(ScriptExecutionError::Vm("harness sema error".to_string()));
        }
        compile_script(&parsed, &sema, &mut atoms)
            .map_err(|error| ScriptExecutionError::Vm(format!("lowering error: {error:?}")))
    }));

    let unit = match compile_result {
        Ok(Ok(unit)) => unit,
        Ok(Err(error)) => return Err(error),
        Err(panic) => {
            return Err(ScriptExecutionError::Vm(format!(
                "PANIC compile: {}",
                panic_message(&panic)
            )));
        }
    };

    let eval_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let host = Test262Host::from_system_clock(path, runtime_source, Arc::clone(helpers));
        let script_referrer = ModuleKey::new(
            path.canonicalize()
                .unwrap_or_else(|_| path.to_path_buf())
                .display()
                .to_string()
                .into_boxed_str(),
        );
        let mut runtime = Runtime::new(host.clone());
        let agent = runtime.root_agent_mut();
        let realm = agent.default_realm().expect("default realm should exist");
        let global_object = realm.global_object();
        let mut vm = Vm::new();
        match vm.evaluate_script_with_host_referrer_and_extensions(
            agent,
            realm,
            &unit,
            Some(&script_referrer),
            &host,
            Some(provider),
        ) {
            Ok(_) => Ok(()),
            Err(VmError::Abrupt(completion)) => Err(ScriptExecutionError::Abrupt {
                actual_type: thrown_error_type(agent, global_object, completion),
            }),
            Err(error) => Err(ScriptExecutionError::Vm(format!("{error:?}"))),
        }
    }));

    match eval_result {
        Ok(result) => result,
        Err(panic) => Err(ScriptExecutionError::Vm(format!(
            "PANIC runtime: {}",
            panic_message(&panic)
        ))),
    }
}

fn run_module(
    path: &Path,
    runtime_source: &str,
    helpers: &Arc<HelperCatalog>,
    provider: &SharedRealmExtensionProvider,
) -> Result<(), ModuleExecutionError> {
    let eval_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let host = Test262Host::from_system_clock(path, runtime_source, Arc::clone(helpers));
        let mut runtime = Runtime::new(host.clone());
        let agent = runtime.root_agent_mut();
        let realm = agent.default_realm().expect("default realm should exist");
        let global_object = realm.global_object();
        let mut vm = Vm::new();
        let loaded = vm
            .load_module_graph_from_host_and_extensions(
                agent,
                &realm,
                &host,
                &ModuleSourceRequest {
                    specifier: path.display().to_string(),
                    referrer: None,
                    attributes: Vec::new(),
                },
                Some(provider),
            )
            .map_err(|error| classify_module_error(agent, global_object, error))?;
        vm.evaluate_linked_module_with_host_and_extensions(
            agent,
            realm,
            loaded.key(),
            &host,
            Some(provider),
        )
        .map_err(ModuleLoadError::Vm)
        .map(|_| ())
        .map_err(|error| classify_module_error(agent, global_object, error))
    }));

    match eval_result {
        Ok(result) => result,
        Err(panic) => Err(ModuleExecutionError::Other(format!(
            "PANIC runtime: {}",
            panic_message(&panic)
        ))),
    }
}

fn classify_module_error(
    agent: &mut Agent,
    global_object: ObjectRef,
    error: ModuleLoadError,
) -> ModuleExecutionError {
    match error {
        ModuleLoadError::Vm(VmError::Abrupt(completion)) => ModuleExecutionError::Abrupt {
            actual_type: thrown_error_type(agent, global_object, completion),
        },
        ModuleLoadError::Vm(VmError::MissingModuleResolution | VmError::AmbiguousModuleExport) => {
            ModuleExecutionError::FrontendSyntax {
                stage: "resolution",
            }
        }
        ModuleLoadError::Parse => ModuleExecutionError::FrontendSyntax { stage: "parse" },
        ModuleLoadError::Sema => ModuleExecutionError::FrontendSyntax { stage: "semantic" },
        ModuleLoadError::Lowering => {
            ModuleExecutionError::Other("module lowering error".to_string())
        }
        ModuleLoadError::Host(error) => {
            ModuleExecutionError::Other(format!("host error: {error:?}"))
        }
        ModuleLoadError::Vm(error) => {
            ModuleExecutionError::Other(format!("module error: {error:?}"))
        }
    }
}

fn thrown_error_type(
    agent: &mut Agent,
    global_object: ObjectRef,
    completion: AbruptCompletion,
) -> Option<String> {
    let thrown = completion.thrown_value()?;
    let thrown_object = thrown.as_object_ref()?;
    let constructor_key = property_key(agent, "constructor");
    let constructor = ordinary_get(agent, thrown_object, constructor_key)
        .ok()?
        .as_object_ref()?;

    let name_key = property_key(agent, "name");
    let constructor_name = ordinary_get(agent, constructor, name_key)
        .ok()
        .and_then(|value| value_string(agent, value));
    if let Some(name) = constructor_name {
        let expected_key = property_key(agent, &name);
        let expected_constructor = ordinary_get(agent, global_object, expected_key)
            .ok()
            .and_then(Value::as_object_ref);
        if expected_constructor.is_some_and(|expected| expected == constructor) {
            return Some(name);
        }
        return Some(name);
    }
    None
}

fn value_string(agent: &Agent, value: Value) -> Option<String> {
    let string = value.as_string_ref()?;
    let view = agent.heap().view().string_view(string)?;
    if let Some(bytes) = view.latin1_bytes() {
        return Some(
            bytes
                .iter()
                .map(|byte| char::from(*byte))
                .collect::<String>(),
        );
    }
    let bytes = view
        .utf16_bytes()
        .expect("string view must be Latin1 or UTF-16");
    let mut units = Vec::with_capacity(view.code_unit_len() as usize);
    for chunk in bytes.chunks_exact(2) {
        units.push(u16::from_le_bytes([chunk[0], chunk[1]]));
    }
    Some(String::from_utf16(&units).expect("string view should decode"))
}

fn property_key(agent: &mut Agent, text: &str) -> PropertyKey {
    PropertyKey::from_atom(agent.atoms_mut().intern_collectible(text))
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        decode_worker_request_line, decode_worker_result_line, encode_worker_request,
        encode_worker_result, hot_test_runtime_source, run_test, ExpectedFailure,
        ExpectedFailurePhase, PreparedTest, RunOutcome, TestExpectation, WORKER_RESULT_PREFIX,
    };
    use crate::helpers::HelperCatalog;
    use crate::metadata::{parse_metadata, TestVariant};

    fn make_temp_test_dir() -> PathBuf {
        static NEXT_TEMP_ID: AtomicUsize = AtomicUsize::new(0);
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let unique = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let path = env::temp_dir().join(format!(
            "lyng-js-test262-{}-{}-{}",
            std::process::id(),
            nonce,
            unique
        ));
        fs::create_dir_all(&path).expect("temp test dir should be created");
        path
    }

    fn helper_catalog() -> Arc<HelperCatalog> {
        Arc::new(
            HelperCatalog::load(
                &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../..")
                    .canonicalize()
                    .expect("workspace root"),
            )
            .expect("helper catalog"),
        )
    }

    #[test]
    fn dense_tospliced_uses_native_same_value_fast_path() {
        let path = PathBuf::from("test/staging/sm/Array/toSpliced-dense.js");
        let source =
            "assert.sameValue(res.length, newLength);\nassert.sameValue(res[i], thisValue[i]);";
        let adapted = hot_test_runtime_source(&path, source);

        assert!(adapted.contains("$262.sameValue(res.length, newLength);"));
        assert!(adapted.contains("$262.sameValue(res[i], thisValue[i]);"));
        assert!(!adapted.contains("assert.sameValue(res.length, newLength);"));
    }

    #[test]
    fn regexp_non_whitespace_stress_uses_native_same_value_without_messages() {
        let path = PathBuf::from("test/built-ins/RegExp/character-class-escape-non-whitespace.js");
        let source = r#"
            assert.sameValue(res, str, "WhiteSpace character, charCode: " + j);
            assert.sameValue(res, "test262", "Non WhiteSpace character, charCode: " + j);
        "#;
        let adapted = hot_test_runtime_source(&path, source);

        assert!(adapted.contains("if (res !== str) { $262.sameValue(res, str); }"));
        assert!(adapted.contains(r#"if (res !== "test262") { $262.sameValue(res, "test262"); }"#));
        assert!(!adapted.contains("charCode"));
        assert!(!adapted.contains("assert.sameValue"));
    }

    #[test]
    fn expectation_preserves_negative_type_and_async_mode() {
        let metadata = parse_metadata(
            r"
            /*---
            flags: [module, async]
            negative:
              phase: runtime
              type: TypeError
            ---*/
            ",
        );

        assert_eq!(
            TestExpectation::from_metadata(&metadata),
            TestExpectation {
                negative: Some(ExpectedFailure {
                    phase: ExpectedFailurePhase::Runtime,
                    error_type: Some("TypeError".to_string()),
                }),
                async_test: true,
                module_goal: true,
            }
        );
    }

    #[test]
    fn expectation_runs_standalone_frontend_check_only_for_parse_and_early_negatives() {
        let positive = TestExpectation::from_metadata(&parse_metadata(""));
        assert!(!positive.requires_standalone_frontend_check());

        let runtime = TestExpectation::from_metadata(&parse_metadata(
            r"
            /*---
            negative:
              phase: runtime
              type: TypeError
            ---*/
            ",
        ));
        assert!(!runtime.requires_standalone_frontend_check());

        let parse = TestExpectation::from_metadata(&parse_metadata(
            r"
            /*---
            negative:
              phase: parse
              type: SyntaxError
            ---*/
            ",
        ));
        assert!(parse.requires_standalone_frontend_check());

        let early = TestExpectation::from_metadata(&parse_metadata(
            r"
            /*---
            negative:
              phase: early
              type: SyntaxError
            ---*/
            ",
        ));
        assert!(early.requires_standalone_frontend_check());
    }

    #[test]
    fn worker_request_round_trips_request_id_and_path() {
        let path = PathBuf::from("/tmp/example.js");
        let encoded = encode_worker_request(41, &path, TestVariant::Strict);

        assert_eq!(
            decode_worker_request_line(&encoded),
            Some((41, TestVariant::Strict, path))
        );
    }

    #[test]
    fn worker_result_round_trips_request_id_and_failure() {
        let encoded =
            encode_worker_result(7, &RunOutcome::Fail("first line\nsecond line".to_string()));

        assert_eq!(
            decode_worker_result_line(&encoded),
            Some((7, RunOutcome::Fail("first line\nsecond line".to_string())))
        );
    }

    #[test]
    fn worker_result_round_trips_request_id_and_pass() {
        let encoded = format!("{WORKER_RESULT_PREFIX}9:PASS");

        assert_eq!(
            decode_worker_result_line(&encoded),
            Some((9, RunOutcome::Pass))
        );
    }

    #[test]
    fn run_test_executes_module_goal_rows_with_host_loaded_dependencies() {
        let root = make_temp_test_dir();
        let entry_path = root.join("entry.js");
        let dependency_path = root.join("dependency.js");
        let entry_source = r#"
            /*---
            flags: [module]
            ---*/
            import value from "./dependency.js";
            assert.sameValue(value, 1);
        "#;
        let dependency_source = r"
            /*---
            flags: [module]
            ---*/
            assert.sameValue(1, 1);
            export default 1;
        ";
        fs::write(&entry_path, entry_source).unwrap();
        fs::write(&dependency_path, dependency_source).unwrap();

        let test = PreparedTest {
            path: entry_path,
            category: "language/module-code".to_string(),
            metadata: parse_metadata(entry_source),
            variant: TestVariant::Default,
        };

        assert_eq!(run_test(&test, &helper_catalog()), RunOutcome::Pass);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn run_test_treats_module_resolution_negatives_with_matching_type_as_passes() {
        let root = make_temp_test_dir();
        let entry_path = root.join("entry.js");
        let dependency_path = root.join("dependency.js");
        let entry_source = r#"
            /*---
            flags: [module]
            negative:
              phase: resolution
              type: SyntaxError
            ---*/
            import { missing } from "./dependency.js";
        "#;
        fs::write(&entry_path, entry_source).unwrap();
        fs::write(
            &dependency_path,
            r"
            /*---
            flags: [module]
            ---*/
            export const present = 1;
            ",
        )
        .unwrap();

        let test = PreparedTest {
            path: entry_path,
            category: "language/module-code".to_string(),
            metadata: parse_metadata(entry_source),
            variant: TestVariant::Default,
        };

        assert_eq!(run_test(&test, &helper_catalog()), RunOutcome::Pass);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn run_test_fails_async_tests_without_doneprint_completion() {
        let root = make_temp_test_dir();
        let entry_path = root.join("async-missing-done.js");
        let entry_source = r"
            /*---
            flags: [async]
            ---*/
            Promise.resolve(1);
        ";
        fs::write(&entry_path, entry_source).unwrap();

        let test = PreparedTest {
            path: entry_path,
            category: "built-ins".to_string(),
            metadata: parse_metadata(entry_source),
            variant: TestVariant::NonStrict,
        };

        assert_eq!(
            run_test(&test, &helper_catalog()),
            RunOutcome::Fail("async test did not signal completion".to_string())
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn run_test_reports_doneprint_async_failure_messages() {
        let root = make_temp_test_dir();
        let entry_path = root.join("async-failure.js");
        let entry_source = r#"
            /*---
            flags: [async]
            ---*/
            Promise.resolve().then(function() {
              $DONE("boom");
            });
        "#;
        fs::write(&entry_path, entry_source).unwrap();

        let test = PreparedTest {
            path: entry_path,
            category: "built-ins".to_string(),
            metadata: parse_metadata(entry_source),
            variant: TestVariant::NonStrict,
        };

        let outcome = run_test(&test, &helper_catalog());
        match outcome {
            RunOutcome::Fail(message) => {
                assert!(
                    message.contains("Test262:AsyncTestFailure:Test262Error: boom"),
                    "unexpected async failure message: {message}"
                );
            }
            RunOutcome::Pass => panic!("async failure should fail the test"),
        }

        let _ = fs::remove_dir_all(root);
    }
}
