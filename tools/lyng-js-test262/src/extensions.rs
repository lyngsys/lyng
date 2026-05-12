use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use lyng_js_host::{
    HostError, HostHooks, HostResult, ImportMetaProperties, ImportMetaProperty, ImportMetaRequest,
    ImportMetaValue, LoadedModuleSource, ModuleKey, ModuleSourceRequest, ParkAgentRequest,
    ParkAgentResult, ParkAgentStatus, TemporalCurrentInstantRequest, TemporalDefaultTimeZone,
    TemporalDefaultTimeZoneRequest, TemporalInstant, UnparkAgentRequest, UnparkAgentResult,
};

use crate::helpers::HelperCatalog;
use crate::metadata::parse_metadata;

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Clone)]
pub enum Test262TemporalCurrentInstantSource {
    Fixed(TemporalInstant),
    SystemClock,
}

#[derive(Clone)]
pub struct Test262Host {
    entry_path: PathBuf,
    entry_key: ModuleKey,
    entry_source: String,
    helpers: Arc<HelperCatalog>,
    temporal_current_instant: Test262TemporalCurrentInstantSource,
    temporal_default_time_zone: String,
}

impl Test262Host {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn new(entry_path: &Path, entry_source: &str, helpers: Arc<HelperCatalog>) -> Self {
        Self::with_temporal_defaults(
            entry_path,
            entry_source,
            helpers,
            TemporalInstant::new(0),
            "UTC".to_string(),
        )
    }

    pub(crate) fn from_system_clock(
        entry_path: &Path,
        entry_source: &str,
        helpers: Arc<HelperCatalog>,
    ) -> Self {
        let canonical_entry = entry_path
            .canonicalize()
            .unwrap_or_else(|_| entry_path.to_path_buf());
        Self {
            entry_key: ModuleKey::new(canonical_entry.display().to_string().into_boxed_str()),
            entry_path: canonical_entry,
            entry_source: entry_source.to_owned(),
            helpers,
            temporal_current_instant: Test262TemporalCurrentInstantSource::SystemClock,
            temporal_default_time_zone: "UTC".to_string(),
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn with_temporal_defaults(
        entry_path: &Path,
        entry_source: &str,
        helpers: Arc<HelperCatalog>,
        temporal_current_instant: TemporalInstant,
        temporal_default_time_zone: String,
    ) -> Self {
        let canonical_entry = entry_path
            .canonicalize()
            .unwrap_or_else(|_| entry_path.to_path_buf());
        Self {
            entry_key: ModuleKey::new(canonical_entry.display().to_string().into_boxed_str()),
            entry_path: canonical_entry,
            entry_source: entry_source.to_owned(),
            helpers,
            temporal_current_instant: Test262TemporalCurrentInstantSource::Fixed(
                temporal_current_instant,
            ),
            temporal_default_time_zone,
        }
    }

    fn system_temporal_current_instant() -> TemporalInstant {
        let epoch_nanoseconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| {
                i128::from(duration.as_secs()) * 1_000_000_000 + i128::from(duration.subsec_nanos())
            })
            .unwrap_or(0);
        TemporalInstant::new(epoch_nanoseconds)
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

    fn recognized_module_type(attributes: &[lyng_js_host::ModuleImportAttribute]) -> Option<&str> {
        attributes
            .iter()
            .find(|attribute| attribute.key == "type")
            .and_then(|attribute| match attribute.value.as_str() {
                "json" | "text" => Some(attribute.value.as_str()),
                _ => None,
            })
    }

    fn module_key_for_import(
        path: &Path,
        attributes: &[lyng_js_host::ModuleImportAttribute],
    ) -> ModuleKey {
        let path = path.display();
        if let Some(module_type) = Self::recognized_module_type(attributes) {
            return ModuleKey::new(format!("{path}#with:type={module_type}").into_boxed_str());
        }
        ModuleKey::new(path.to_string().into_boxed_str())
    }

    fn source_text_from_import_attributes(
        raw_source: &str,
        attributes: &[lyng_js_host::ModuleImportAttribute],
    ) -> Option<String> {
        let module_type = Self::recognized_module_type(attributes)?;
        match module_type {
            "json" => {
                if serde_json::from_str::<serde_json::Value>(raw_source).is_err() {
                    return Some("export default ;".to_string());
                }
                Some(format!("export default ({raw_source});"))
            }
            "text" => Some(format!(
                "export default {};",
                Self::js_string_literal(raw_source)
            )),
            _ => None,
        }
    }

    fn js_string_literal(source: &str) -> String {
        let mut literal = String::with_capacity(source.len() + 2);
        literal.push('"');
        for ch in source.chars() {
            match ch {
                '"' => literal.push_str("\\\""),
                '\\' => literal.push_str("\\\\"),
                '\n' => literal.push_str("\\n"),
                '\r' => literal.push_str("\\r"),
                '\t' => literal.push_str("\\t"),
                '\u{2028}' => literal.push_str("\\u2028"),
                '\u{2029}' => literal.push_str("\\u2029"),
                ch if ch.is_control() => {
                    use std::fmt::Write as _;
                    let _ = write!(literal, "\\u{:04x}", ch as u32);
                }
                ch => literal.push(ch),
            }
        }
        literal.push('"');
        literal
    }
}

impl HostHooks for Test262Host {
    fn load_module_source(&self, request: &ModuleSourceRequest) -> HostResult<LoadedModuleSource> {
        let path = Self::resolve_module_path(request)?;
        let raw_source = if path == self.entry_path {
            self.entry_source.clone()
        } else {
            Self::read_utf8_file("load_module_source", &path)?
        };
        let source_text = if let Some(source_text) =
            Self::source_text_from_import_attributes(&raw_source, &request.attributes)
        {
            source_text
        } else if path != self.entry_path && raw_source.contains("/*---") {
            self.helpers
                .build_runtime_source(&parse_metadata(&raw_source), &raw_source)
                .map_err(|error| HostError::internal("load_module_source", error))?
        } else {
            raw_source
        };
        let module_key = Self::module_key_for_import(&path, &request.attributes);

        Ok(LoadedModuleSource::new(
            module_key,
            path.display().to_string(),
            source_text,
        ))
    }

    fn resolve_import_meta(&self, request: &ImportMetaRequest) -> HostResult<ImportMetaProperties> {
        let mut properties = vec![ImportMetaProperty {
            key: "url".into(),
            value: ImportMetaValue::String(format!("file://{}", request.module.as_str())),
        }];
        if request.module == self.entry_key {
            properties.push(ImportMetaProperty {
                key: "test262".into(),
                value: ImportMetaValue::Boolean(true),
            });
        }
        Ok(ImportMetaProperties::new(properties))
    }

    fn park_agent(&self, request: &ParkAgentRequest) -> HostResult<ParkAgentResult> {
        Ok(ParkAgentResult {
            status: if request.timeout_ns.is_some() {
                ParkAgentStatus::TimedOut
            } else {
                ParkAgentStatus::Parked
            },
        })
    }

    fn unpark_agent(&self, request: &UnparkAgentRequest) -> HostResult<UnparkAgentResult> {
        Ok(UnparkAgentResult {
            woken_agents: request.max_count.min(1),
            remaining_waiters: false,
        })
    }

    fn temporal_current_instant(
        &self,
        _request: &TemporalCurrentInstantRequest,
    ) -> HostResult<TemporalInstant> {
        Ok(match self.temporal_current_instant {
            Test262TemporalCurrentInstantSource::Fixed(instant) => instant,
            Test262TemporalCurrentInstantSource::SystemClock => {
                Self::system_temporal_current_instant()
            }
        })
    }

    fn temporal_default_time_zone(
        &self,
        _request: &TemporalDefaultTimeZoneRequest,
    ) -> HostResult<TemporalDefaultTimeZone> {
        Ok(TemporalDefaultTimeZone::new(
            self.temporal_default_time_zone.clone(),
        ))
    }

    fn temporal_default_time_zone_is_utc(
        &self,
        _request: &TemporalDefaultTimeZoneRequest,
    ) -> HostResult<bool> {
        Ok(self.temporal_default_time_zone == "UTC")
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use lyng_js_host::{HostHooks, ModuleImportAttribute, ModuleSourceRequest, TemporalInstant};

    use crate::helpers::HelperCatalog;

    use super::{Test262Host, Test262TemporalCurrentInstantSource};

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("workspace root should exist")
    }

    #[test]
    fn new_host_uses_fixed_epoch_and_utc_defaults() {
        let helpers = Arc::new(HelperCatalog::load(&workspace_root()).expect("helper catalog"));
        let entry_path = PathBuf::from("/tmp/test262.js");
        let host = Test262Host::new(&entry_path, "", helpers);

        match host.temporal_current_instant {
            Test262TemporalCurrentInstantSource::Fixed(instant) => {
                assert_eq!(instant, TemporalInstant::new(0));
            }
            Test262TemporalCurrentInstantSource::SystemClock => {
                panic!("expected fixed temporal instant for test host")
            }
        }
        assert_eq!(host.temporal_default_time_zone, "UTC");
    }

    #[test]
    fn host_can_override_temporal_defaults_for_deterministic_tests() {
        let helpers = Arc::new(HelperCatalog::load(&workspace_root()).expect("helper catalog"));
        let entry_path = PathBuf::from("/tmp/test262.js");
        let instant = TemporalInstant::new(123_456);
        let host = Test262Host::with_temporal_defaults(
            &entry_path,
            "",
            helpers,
            instant,
            "Europe/Berlin".to_string(),
        );

        match host.temporal_current_instant {
            Test262TemporalCurrentInstantSource::Fixed(actual) => {
                assert_eq!(actual, instant);
            }
            Test262TemporalCurrentInstantSource::SystemClock => {
                panic!("expected fixed temporal instant for overridden test host")
            }
        }
        assert_eq!(host.temporal_default_time_zone, "Europe/Berlin");
    }

    #[test]
    fn import_attribute_source_transform_wraps_json_and_text_modules() {
        let json = Test262Host::source_text_from_import_attributes(
            "262",
            &[ModuleImportAttribute {
                key: "type".to_string(),
                value: "json".to_string(),
            }],
        )
        .expect("json import attribute should transform source");
        assert_eq!(json, "export default (262);");

        let text = Test262Host::source_text_from_import_attributes(
            "line \"one\"\nline \\two",
            &[ModuleImportAttribute {
                key: "type".to_string(),
                value: "text".to_string(),
            }],
        )
        .expect("text import attribute should transform source");
        assert_eq!(text, "export default \"line \\\"one\\\"\\nline \\\\two\";");
    }

    #[test]
    fn json_import_attribute_rejects_invalid_json_during_module_parse() {
        let source_text = Test262Host::source_text_from_import_attributes(
            "{\n  notJson: 0\n}\n",
            &[ModuleImportAttribute {
                key: "type".to_string(),
                value: "json".to_string(),
            }],
        )
        .expect("json import attribute should transform source");

        assert_eq!(source_text, "export default ;");
    }

    #[test]
    fn load_module_source_applies_import_attributes_to_entry_self_imports() {
        let helpers = Arc::new(HelperCatalog::load(&workspace_root()).expect("helper catalog"));
        let entry_path = workspace_root()
            .join("testdata/test262/test/language/import/import-attributes/text-self.js");
        let host = Test262Host::new(&entry_path, "entry source", helpers);

        let loaded = host
            .load_module_source(&ModuleSourceRequest {
                specifier: entry_path.display().to_string(),
                referrer: None,
                attributes: vec![ModuleImportAttribute {
                    key: "type".to_string(),
                    value: "text".to_string(),
                }],
            })
            .expect("entry self import should load");

        assert_eq!(loaded.source_text, "export default \"entry source\";");
        assert_ne!(
            loaded.key, host.entry_key,
            "text-module self imports need a distinct module identity"
        );
    }
}
