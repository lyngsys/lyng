use crate::error::CliError;
use crate::extensions::CliRealmExtension;
use crate::host::{CliHost, CliHostSnapshot};
use crate::CliInvocation;
use lyng_js_common::{Diagnostic, SourceId, WellKnownAtom};
use lyng_js_compiler::compile_script;
use lyng_js_env::{Agent, Runtime};
use lyng_js_host::{
    DiagnosticReportRequest, HostHooks, ModuleKey, ModuleSourceRequest, ScriptSourceRequest,
    UncaughtExceptionReport,
};
use lyng_js_ops::{number_to_string, object};
use lyng_js_parser::parse_script;
use lyng_js_sema::analyze_script;
use lyng_js_types::{ObjectRef, PropertyKey, Value};
use lyng_js_vm::{ModuleLoadError, SharedRealmExtensionProvider, Vm, VmError};
use std::fmt::Write as _;
use std::io::Write;
use std::sync::Arc;

const ENTRY_SOURCE_ID: SourceId = SourceId::new(1);

pub(crate) fn run_script(
    invocation: &CliInvocation,
    stderr: &mut dyn Write,
) -> Result<i32, CliError> {
    let host = CliHost::new();
    let outcome = if invocation.is_module_entry() {
        execute_module(invocation, host.clone())?
    } else {
        execute_script(invocation, host.clone())?
    };
    write_reports(stderr, &outcome.display_name, &outcome.snapshot)?;
    Ok(outcome.exit_code)
}

struct ScriptOutcome {
    exit_code: i32,
    display_name: String,
    snapshot: CliHostSnapshot,
}

fn execute_script(invocation: &CliInvocation, host: CliHost) -> Result<ScriptOutcome, CliError> {
    let loaded = host
        .load_script_source(&ScriptSourceRequest {
            path: invocation.script_path().display().to_string(),
            referrer: None,
            is_entry: true,
        })
        .map_err(CliError::host)?;

    let mut runtime = Runtime::new(host.clone());
    let agent = runtime.root_agent_mut();
    let realm = agent
        .default_realm()
        .ok_or_else(|| CliError::internal("default realm is missing from the runtime shell"))?;

    let parsed = parse_script(agent.atoms_mut(), ENTRY_SOURCE_ID, &loaded.source_text);
    report_diagnostics(&host, parsed.diagnostics.as_slice()).map_err(CliError::host)?;
    if parsed.diagnostics.has_errors() {
        return Ok(ScriptOutcome {
            exit_code: 1,
            display_name: loaded.display_name,
            snapshot: host.snapshot(),
        });
    }

    let sema = analyze_script(&parsed, agent.atoms());
    report_diagnostics(&host, sema.diagnostics.as_slice()).map_err(CliError::host)?;
    if sema.diagnostics.has_errors() {
        return Ok(ScriptOutcome {
            exit_code: 1,
            display_name: loaded.display_name,
            snapshot: host.snapshot(),
        });
    }

    let unit = compile_script(&parsed, &sema, agent.atoms_mut())
        .map_err(|error| CliError::lowering(format!("script lowering failed: {error:?}")))?;

    let mut vm = Vm::new();
    vm.bootstrap_realm(agent, realm.id(), invocation.bootstrap_mode())
        .map_err(|error| CliError::vm(format!("realm bootstrap failed: {error:?}")))?;
    let script_referrer = ModuleKey::new(
        invocation
            .script_path()
            .display()
            .to_string()
            .into_boxed_str(),
    );
    let provider = shell_extension_provider(invocation);
    let execution_result = if let Some(provider) = provider.as_ref() {
        vm.evaluate_script_with_host_referrer_and_extensions(
            agent,
            realm,
            &unit,
            Some(&script_referrer),
            &host,
            Some(provider),
        )
    } else {
        vm.evaluate_script_with_host_referrer(agent, realm, &unit, Some(&script_referrer), &host)
    };
    let exit_code = match execution_result {
        Ok(_) => 0,
        Err(VmError::Abrupt(completion)) => {
            let thrown_value = completion.thrown_value().unwrap_or(Value::undefined());
            let message = describe_uncaught_exception(agent, thrown_value);
            host.report_uncaught_exception(&UncaughtExceptionReport {
                source: Some(unit.source()),
                realm: Some(realm.id()),
                thrown_value,
                message,
            })
            .map_err(CliError::host)?;
            1
        }
        Err(error) => {
            return Err(CliError::vm(format!("script execution failed: {error:?}")));
        }
    };

    Ok(ScriptOutcome {
        exit_code,
        display_name: loaded.display_name,
        snapshot: host.snapshot(),
    })
}

fn execute_module(invocation: &CliInvocation, host: CliHost) -> Result<ScriptOutcome, CliError> {
    let mut runtime = Runtime::new(host.clone());
    let agent = runtime.root_agent_mut();
    let realm = agent
        .default_realm()
        .ok_or_else(|| CliError::internal("default realm is missing from the runtime shell"))?;
    let mut vm = Vm::new();
    let _ = vm
        .bootstrap_realm(agent, realm.id(), invocation.bootstrap_mode())
        .map_err(|error| CliError::vm(format!("realm bootstrap failed: {error:?}")))?;

    let module_request = ModuleSourceRequest {
        specifier: invocation.script_path().display().to_string(),
        referrer: None,
        attributes: Vec::new(),
    };
    let provider = shell_extension_provider(invocation);
    let load_result = if let Some(provider) = provider.as_ref() {
        vm.load_module_graph_from_host_and_extensions(
            agent,
            realm,
            &host,
            &module_request,
            Some(provider),
        )
    } else {
        vm.load_module_graph_from_host(agent, realm, &host, &module_request)
    };
    let loaded = match load_result {
        Ok(loaded) => loaded,
        Err(ModuleLoadError::Host(error)) => return Err(CliError::host(error)),
        Err(ModuleLoadError::Lowering) => return Err(CliError::lowering("module lowering failed")),
        Err(ModuleLoadError::Vm(error)) => {
            return Err(CliError::vm(format!("module loading failed: {error:?}")))
        }
        Err(ModuleLoadError::Parse | ModuleLoadError::Sema) => {
            return Ok(ScriptOutcome {
                exit_code: 1,
                display_name: invocation.script_path().display().to_string(),
                snapshot: host.snapshot(),
            })
        }
    };

    let execution_result = if let Some(provider) = provider.as_ref() {
        vm.evaluate_linked_module_with_host_and_extensions(
            agent,
            realm,
            loaded.key(),
            &host,
            Some(provider),
        )
    } else {
        vm.evaluate_linked_module_with_host(agent, realm, loaded.key(), &host)
    };
    let exit_code = match execution_result {
        Ok(_) => 0,
        Err(VmError::Abrupt(completion)) => {
            let thrown_value = completion.thrown_value().unwrap_or(Value::undefined());
            let message = describe_uncaught_exception(agent, thrown_value);
            host.report_uncaught_exception(&UncaughtExceptionReport {
                source: None,
                realm: Some(realm.id()),
                thrown_value,
                message,
            })
            .map_err(CliError::host)?;
            1
        }
        Err(error) => {
            return Err(CliError::vm(format!("module execution failed: {error:?}")));
        }
    };

    Ok(ScriptOutcome {
        exit_code,
        display_name: loaded.display_name().to_owned(),
        snapshot: host.snapshot(),
    })
}

fn shell_extension_provider(invocation: &CliInvocation) -> Option<SharedRealmExtensionProvider> {
    if invocation.shell_mode() {
        Some(Arc::new(CliRealmExtension))
    } else {
        None
    }
}

fn report_diagnostics(
    host: &dyn HostHooks,
    diagnostics: &[Diagnostic],
) -> Result<(), lyng_js_host::HostError> {
    for diagnostic in diagnostics {
        host.report_diagnostic(&DiagnosticReportRequest {
            severity: diagnostic.severity,
            source: Some(diagnostic.span.source),
            span: Some(diagnostic.span),
            message: diagnostic.message.clone(),
        })?;
    }
    Ok(())
}

fn write_reports(
    stderr: &mut dyn Write,
    display_name: &str,
    snapshot: &CliHostSnapshot,
) -> Result<(), CliError> {
    for diagnostic in snapshot.diagnostics() {
        write_diagnostic(stderr, display_name, diagnostic)?;
    }
    for report in snapshot.uncaught_exceptions() {
        writeln!(stderr, "Uncaught exception: {}", report.message).map_err(CliError::io)?;
    }
    Ok(())
}

fn write_diagnostic(
    stderr: &mut dyn Write,
    display_name: &str,
    diagnostic: &DiagnosticReportRequest,
) -> Result<(), CliError> {
    let mut location = display_name.to_owned();
    if let Some(span) = diagnostic.span {
        let _ = write!(
            &mut location,
            "@{}..{}",
            span.range.start.raw(),
            span.range.end.raw()
        );
    }
    writeln!(
        stderr,
        "{}: {} ({location})",
        diagnostic.severity, diagnostic.message
    )
    .map_err(CliError::io)
}

fn describe_uncaught_exception(agent: &mut Agent, thrown: Value) -> String {
    if let Some(text) = primitive_value_text(agent, thrown) {
        return text;
    }

    let Some(object) = thrown.as_object_ref() else {
        return "uncaught exception".to_owned();
    };

    format_error_like_object(agent, object).unwrap_or_else(|| "uncaught exception".to_owned())
}

fn format_error_like_object(agent: &mut Agent, object: ObjectRef) -> Option<String> {
    let name = lookup_data_property_text(
        agent,
        object,
        PropertyKey::from_atom(WellKnownAtom::name.id()),
    )
    .filter(|text| !text.is_empty());
    let message = lookup_data_property_text(
        agent,
        object,
        PropertyKey::from_atom(agent.bootstrap_atoms().message()),
    )
    .filter(|text| !text.is_empty());

    match (name, message) {
        (Some(name), Some(message)) => Some(format!("{name}: {message}")),
        (Some(name), None) => Some(name),
        (None, Some(message)) => Some(message),
        (None, None) => None,
    }
}

fn lookup_data_property_text(
    agent: &mut Agent,
    mut object: ObjectRef,
    key: PropertyKey,
) -> Option<String> {
    loop {
        match object::ordinary_get_own_property(agent, object, key).ok()? {
            Some(descriptor) if descriptor.has_value() => {
                return primitive_value_text(agent, descriptor.value()?);
            }
            Some(_) => return None,
            None => {
                let prototype = object::ordinary_get_prototype_of(agent, object)
                    .ok()
                    .flatten()?;
                object = prototype;
            }
        }
    }
}

fn primitive_value_text(agent: &Agent, value: Value) -> Option<String> {
    if value.is_undefined() {
        return Some("undefined".to_owned());
    }
    if value.is_null() {
        return Some("null".to_owned());
    }
    if let Some(boolean) = value.as_bool() {
        return Some(if boolean { "true" } else { "false" }.to_owned());
    }
    if let Some(number) = value.as_f64() {
        return Some(number_to_string(number));
    }
    if let Some(string) = value.as_string_ref() {
        return decode_string(agent, string);
    }
    if let Some(symbol) = value.as_symbol_ref() {
        let view = agent.heap().view().symbol_view(symbol)?;
        let description = view.description_view().and_then(|description| {
            decode_string_units(
                description.latin1_bytes(),
                description.utf16_bytes(),
                description.code_unit_len(),
            )
        });
        return Some(match description {
            Some(description) => format!("Symbol({description})"),
            None => "Symbol()".to_owned(),
        });
    }
    None
}

fn decode_string(agent: &Agent, string: lyng_js_types::StringRef) -> Option<String> {
    let view = agent.heap().view().string_view(string)?;
    decode_string_units(
        view.latin1_bytes(),
        view.utf16_bytes(),
        view.code_unit_len(),
    )
}

fn decode_string_units(
    latin1: Option<&[u8]>,
    utf16: Option<&[u8]>,
    code_unit_len: u32,
) -> Option<String> {
    if let Some(bytes) = latin1 {
        return Some(bytes.iter().map(|byte| char::from(*byte)).collect());
    }

    let bytes = utf16?;
    let mut units = Vec::with_capacity(code_unit_len as usize);
    for chunk in bytes.chunks_exact(2) {
        units.push(u16::from_le_bytes([chunk[0], chunk[1]]));
    }
    Some(String::from_utf16_lossy(&units))
}
