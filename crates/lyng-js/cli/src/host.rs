use lyng_js_host::{
    DiagnosticReportRequest, HostError, HostHooks, HostResult, ImportMetaProperties,
    ImportMetaProperty, ImportMetaRequest, ImportMetaValue, LoadedModuleSource, LoadedSourceText,
    ModuleKey, ModuleSourceRequest, ScriptSourceRequest, UncaughtExceptionReport,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct CliHostSnapshot {
    diagnostics: Vec<DiagnosticReportRequest>,
    uncaught_exceptions: Vec<UncaughtExceptionReport>,
}

impl CliHostSnapshot {
    #[inline]
    pub(crate) fn diagnostics(&self) -> &[DiagnosticReportRequest] {
        &self.diagnostics
    }

    #[inline]
    pub(crate) fn uncaught_exceptions(&self) -> &[UncaughtExceptionReport] {
        &self.uncaught_exceptions
    }
}

#[derive(Clone, Default)]
pub(crate) struct CliHost {
    state: Arc<Mutex<CliHostState>>,
}

#[derive(Default)]
struct CliHostState {
    diagnostics: Vec<DiagnosticReportRequest>,
    uncaught_exceptions: Vec<UncaughtExceptionReport>,
}

impl CliHost {
    #[inline]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn snapshot(&self) -> CliHostSnapshot {
        let state = self.state.lock().expect("cli host mutex poisoned");
        CliHostSnapshot {
            diagnostics: state.diagnostics.clone(),
            uncaught_exceptions: state.uncaught_exceptions.clone(),
        }
    }

    fn read_utf8_file(operation: &'static str, path: &Path) -> HostResult<String> {
        let bytes = fs::read(path).map_err(|error| match error.kind() {
            std::io::ErrorKind::NotFound => HostError::not_found(
                operation,
                format!("file `{}` was not found", path.display()),
            ),
            _ => HostError::internal(
                operation,
                format!("failed to read `{}`: {error}", path.display()),
            ),
        })?;
        String::from_utf8(bytes).map_err(|error| {
            HostError::invalid_request(
                operation,
                format!("file `{}` is not valid UTF-8: {error}", path.display()),
            )
        })
    }

    fn resolve_module_path(request: &ModuleSourceRequest) -> HostResult<PathBuf> {
        let specifier = Path::new(&request.specifier);
        let candidate = if specifier.is_absolute() {
            specifier.to_path_buf()
        } else if let Some(referrer) = &request.referrer {
            Path::new(referrer.as_str())
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(specifier)
        } else {
            std::env::current_dir()
                .map_err(|error| {
                    HostError::internal(
                        "load_module_source",
                        format!("failed to read current directory: {error}"),
                    )
                })?
                .join(specifier)
        };

        candidate
            .canonicalize()
            .map_err(|error| match error.kind() {
                std::io::ErrorKind::NotFound => HostError::not_found(
                    "load_module_source",
                    format!("module file `{}` was not found", candidate.display()),
                ),
                _ => HostError::internal(
                    "load_module_source",
                    format!("failed to canonicalize `{}`: {error}", candidate.display()),
                ),
            })
    }
}

impl HostHooks for CliHost {
    fn report_diagnostic(&self, request: &DiagnosticReportRequest) -> HostResult<()> {
        self.state
            .lock()
            .expect("cli host mutex poisoned")
            .diagnostics
            .push(request.clone());
        Ok(())
    }

    fn report_uncaught_exception(&self, request: &UncaughtExceptionReport) -> HostResult<()> {
        self.state
            .lock()
            .expect("cli host mutex poisoned")
            .uncaught_exceptions
            .push(request.clone());
        Ok(())
    }

    fn load_script_source(&self, request: &ScriptSourceRequest) -> HostResult<LoadedSourceText> {
        let path = Path::new(&request.path);
        let source_text = Self::read_utf8_file("load_script_source", path)?;

        Ok(LoadedSourceText::new(
            path.display().to_string(),
            source_text,
        ))
    }

    fn load_module_source(&self, request: &ModuleSourceRequest) -> HostResult<LoadedModuleSource> {
        let path = Self::resolve_module_path(request)?;
        let source_text = Self::read_utf8_file("load_module_source", &path)?;
        let key = ModuleKey::new(path.display().to_string().into_boxed_str());

        Ok(LoadedModuleSource::new(
            key,
            path.display().to_string(),
            source_text,
        ))
    }

    fn resolve_import_meta(&self, request: &ImportMetaRequest) -> HostResult<ImportMetaProperties> {
        Ok(ImportMetaProperties::new(vec![ImportMetaProperty {
            key: "url".into(),
            value: ImportMetaValue::String(format!("file://{}", request.module.as_str())),
        }]))
    }
}
