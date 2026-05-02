use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use lyng_js_env::Agent;
use lyng_js_host::{
    HostError, HostHooks, HostResult, ImportMetaProperties, ImportMetaProperty, ImportMetaRequest,
    ImportMetaValue, LoadedModuleSource, ModuleKey, ModuleSourceRequest, ParkAgentRequest,
    ParkAgentResult, ParkAgentStatus, TemporalCurrentInstantRequest, TemporalDefaultTimeZone,
    TemporalDefaultTimeZoneRequest, TemporalInstant, UnparkAgentRequest, UnparkAgentResult,
};
use lyng_js_ops::{errors, read};
use lyng_js_types::{
    abstract_module_source_builtin, EmbeddingFunctionId, ObjectRef, PropertyKey, Value,
};
use lyng_js_vm::{
    EmbeddingFunctionContext, EmbeddingFunctionMetadata, EmbeddingInvocation,
    RealmExtensionInstallation, RealmExtensionProvider, VmError,
};

use crate::helpers::HelperCatalog;
use crate::metadata::parse_metadata;

const TEST262_EVAL_SCRIPT_RAW: u32 = 1;
const TEST262_CREATE_REALM_RAW: u32 = 2;
const TEST262_DETACH_ARRAY_BUFFER_RAW: u32 = 3;
const TEST262_GC_RAW: u32 = 4;
const TEST262_PRINT_RAW: u32 = 5;
const TEST262_SAME_VALUE_RAW: u32 = 6;

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Clone)]
pub(crate) enum Test262TemporalCurrentInstantSource {
    Fixed(TemporalInstant),
    SystemClock,
}

#[derive(Clone)]
pub(crate) struct Test262Host {
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

fn test262_eval_script_entry() -> EmbeddingFunctionId {
    EmbeddingFunctionId::from_raw(TEST262_EVAL_SCRIPT_RAW)
        .expect("test262 embedding function ids should stay non-zero")
}

fn test262_create_realm_entry() -> EmbeddingFunctionId {
    EmbeddingFunctionId::from_raw(TEST262_CREATE_REALM_RAW)
        .expect("test262 embedding function ids should stay non-zero")
}

fn test262_detach_array_buffer_entry() -> EmbeddingFunctionId {
    EmbeddingFunctionId::from_raw(TEST262_DETACH_ARRAY_BUFFER_RAW)
        .expect("test262 embedding function ids should stay non-zero")
}

fn test262_gc_entry() -> EmbeddingFunctionId {
    EmbeddingFunctionId::from_raw(TEST262_GC_RAW)
        .expect("test262 embedding function ids should stay non-zero")
}

fn test262_print_entry() -> EmbeddingFunctionId {
    EmbeddingFunctionId::from_raw(TEST262_PRINT_RAW)
        .expect("test262 embedding function ids should stay non-zero")
}

fn test262_same_value_entry() -> EmbeddingFunctionId {
    EmbeddingFunctionId::from_raw(TEST262_SAME_VALUE_RAW)
        .expect("test262 embedding function ids should stay non-zero")
}

fn test262_property_key(agent: &mut Agent, text: &str) -> PropertyKey {
    PropertyKey::from_atom(agent.atoms_mut().intern_collectible(text))
}

fn read_test262_object(agent: &mut Agent, global_object: ObjectRef) -> Result<ObjectRef, VmError> {
    let key = test262_property_key(agent, "$262");
    agent
        .objects()
        .get_own_property(agent.heap().view(), global_object, key)
        .map_err(|_| VmError::Abrupt(errors::throw_type_error(agent)))?
        .and_then(lyng_js_types::PropertyDescriptor::value)
        .and_then(Value::as_object_ref)
        .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))
}

#[derive(Clone, Default)]
pub(crate) struct Test262PrintObserver {
    messages: Arc<Mutex<Vec<String>>>,
}

impl Test262PrintObserver {
    pub(crate) fn record(&self, message: String) {
        match self.messages.lock() {
            Ok(mut messages) => messages.push(message),
            Err(poisoned) => poisoned.into_inner().push(message),
        }
    }

    pub(crate) fn messages(&self) -> Vec<String> {
        match self.messages.lock() {
            Ok(messages) => messages.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct Test262RealmExtension {
    print_observer: Test262PrintObserver,
}

impl Test262RealmExtension {
    pub(crate) fn new(print_observer: Test262PrintObserver) -> Self {
        Self { print_observer }
    }
}

impl RealmExtensionProvider for Test262RealmExtension {
    fn embedding_function_metadata(
        &self,
        entry: EmbeddingFunctionId,
    ) -> Option<EmbeddingFunctionMetadata> {
        if entry == test262_eval_script_entry() {
            return Some(EmbeddingFunctionMetadata::new(
                "evalScript",
                1,
                false,
                false,
            ));
        }
        if entry == test262_create_realm_entry() {
            return Some(EmbeddingFunctionMetadata::new(
                "createRealm",
                0,
                false,
                false,
            ));
        }
        if entry == test262_detach_array_buffer_entry() {
            return Some(EmbeddingFunctionMetadata::new(
                "detachArrayBuffer",
                1,
                false,
                false,
            ));
        }
        if entry == test262_gc_entry() {
            return Some(EmbeddingFunctionMetadata::new("gc", 0, false, false));
        }
        if entry == test262_print_entry() {
            return Some(EmbeddingFunctionMetadata::new("print", 1, false, false));
        }
        if entry == test262_same_value_entry() {
            return Some(EmbeddingFunctionMetadata::new("sameValue", 2, false, false));
        }
        None
    }

    fn install_realm_extensions(
        &self,
        installation: &mut RealmExtensionInstallation<'_>,
    ) -> Result<(), VmError> {
        let realm = installation.realm();
        let object_prototype = installation
            .agent()
            .realm(realm)
            .and_then(|realm| realm.intrinsics().object_prototype())
            .ok_or(VmError::MissingRootShape(realm))?;
        let harness = installation.allocate_ordinary_object(Some(object_prototype))?;

        let harness_key = test262_property_key(installation.agent(), "$262");
        let global_key = test262_property_key(installation.agent(), "global");
        let eval_script_key = test262_property_key(installation.agent(), "evalScript");
        let create_realm_key = test262_property_key(installation.agent(), "createRealm");
        let detach_key = test262_property_key(installation.agent(), "detachArrayBuffer");
        let gc_key = test262_property_key(installation.agent(), "gc");
        let print_key = test262_property_key(installation.agent(), "print");
        let same_value_key = test262_property_key(installation.agent(), "sameValue");
        let abstract_module_source_key =
            test262_property_key(installation.agent(), "AbstractModuleSource");

        installation.define_data_property(
            installation.global_object(),
            harness_key,
            Value::from_object_ref(harness),
            true,
            false,
            true,
        )?;
        installation.define_data_property(
            harness,
            global_key,
            Value::from_object_ref(installation.global_object()),
            true,
            false,
            true,
        )?;
        let _ = installation.define_function_property(
            harness,
            eval_script_key,
            test262_eval_script_entry(),
            true,
            false,
            true,
        )?;
        let _ = installation.define_function_property(
            harness,
            create_realm_key,
            test262_create_realm_entry(),
            true,
            false,
            true,
        )?;
        let _ = installation.define_function_property(
            harness,
            detach_key,
            test262_detach_array_buffer_entry(),
            true,
            false,
            true,
        )?;
        let _ = installation.define_function_property(
            harness,
            gc_key,
            test262_gc_entry(),
            true,
            false,
            true,
        )?;
        let _ = installation.define_function_property(
            installation.global_object(),
            print_key,
            test262_print_entry(),
            true,
            false,
            true,
        )?;
        let _ = installation.define_function_property(
            harness,
            same_value_key,
            test262_same_value_entry(),
            true,
            false,
            true,
        )?;
        let abstract_module_source =
            installation.builtin_constant(abstract_module_source_builtin())?;
        installation.define_data_property(
            harness,
            abstract_module_source_key,
            abstract_module_source,
            true,
            false,
            true,
        )?;
        Ok(())
    }

    fn call_embedding_function(
        &self,
        context: &mut EmbeddingFunctionContext<'_>,
        entry: EmbeddingFunctionId,
        invocation: EmbeddingInvocation<'_>,
    ) -> Result<Value, VmError> {
        if entry == test262_eval_script_entry() {
            let source = invocation
                .arguments()
                .first()
                .copied()
                .unwrap_or(Value::undefined());
            let source_text = context.value_to_string_text(source)?;
            return context.evaluate_script_in_realm(context.function_realm(), &source_text);
        }
        if entry == test262_create_realm_entry() {
            let artifacts = context.create_embedding_realm()?;
            return read_test262_object(context.agent(), artifacts.global_object())
                .map(Value::from_object_ref);
        }
        if entry == test262_detach_array_buffer_entry() {
            let value = invocation
                .arguments()
                .first()
                .copied()
                .unwrap_or(Value::undefined());
            let Some(object) = value.as_object_ref() else {
                return Err(VmError::Abrupt(errors::throw_type_error(context.agent())));
            };
            let Some(array_buffer) = context.agent().objects().array_buffer(object) else {
                return Err(VmError::Abrupt(errors::throw_type_error(context.agent())));
            };
            let _ = context
                .agent()
                .detach_backing_store(array_buffer.backing_store());
            return Ok(Value::undefined());
        }
        if entry == test262_gc_entry() {
            let _ = context.agent().force_collect();
            return Ok(Value::undefined());
        }
        if entry == test262_print_entry() {
            let value = invocation
                .arguments()
                .first()
                .copied()
                .unwrap_or(Value::undefined());
            let message = context.value_to_string_text(value)?;
            self.print_observer.record(message);
            return Ok(Value::undefined());
        }
        if entry == test262_same_value_entry() {
            let actual = invocation
                .arguments()
                .first()
                .copied()
                .unwrap_or(Value::undefined());
            let expected = invocation
                .arguments()
                .get(1)
                .copied()
                .unwrap_or(Value::undefined());
            let same = {
                let agent = context.agent();
                read::same_value(agent.heap().view(), actual, expected).map_err(VmError::Abrupt)?
            };
            if same {
                return Ok(Value::undefined());
            }
            return Err(VmError::Abrupt(errors::throw_type_error(context.agent())));
        }
        Err(VmError::MissingEmbeddingFunction(entry))
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
