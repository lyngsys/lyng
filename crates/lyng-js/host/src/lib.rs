//! Cold-path host integration boundary for the lyng-js runtime substrate.
//!
//! Ownership: `lyng_js_host` owns host-facing traits, typed request and response
//! shells, host errors, and default host test doubles only. It does not own
//! runtime state, queue order, or object semantics.

#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::return_self_not_must_use
)]

mod error;
mod hooks;
mod ids;
mod test_host;

pub use error::{HostError, HostErrorKind, HostResult};
pub use hooks::{
    AgentSpawnKind, AgentThreadStartKind, ArrayBufferTransferRequest, ArrayBufferTransferResponse,
    CreateAgentRequest, CreateAgentResponse, DiagnosticReportRequest, HostHooks, HostJobKind,
    HostJobPhase, ImportMetaProperties, ImportMetaProperty, ImportMetaRequest, ImportMetaValue,
    JobObservation, LoadedModuleSource, LoadedSourceText, ModuleImportAttribute, ModuleKey,
    ModuleSourceRequest, ParkAgentRequest, ParkAgentResult, ParkAgentStatus, ScriptSourceRequest,
    SharedArrayBufferShareRequest, SharedArrayBufferShareResponse, StartAgentThreadRequest,
    StartAgentThreadResponse, TemporalCivilDateTime, TemporalCivilTime,
    TemporalCivilToInstantRequest, TemporalCurrentInstantRequest, TemporalDefaultTimeZone,
    TemporalDefaultTimeZoneRequest, TemporalDisambiguation, TemporalInstant,
    TemporalInstantToCivilRequest, TemporalInstantWithOffset, UncaughtExceptionReport,
    UnparkAgentRequest, UnparkAgentResult, WaitLocation,
};
pub use ids::{HostAgentId, HostJobId, HostSharedBufferId, HostThreadId, HostTransferredBufferId};
pub use test_host::{HostCall, NoopHostHooks, TestHost, TestHostSnapshot};

#[cfg(test)]
mod tests {
    use super::*;
    use lyng_js_common::{Severity, SourceId, Span};
    use lyng_js_types::{BackingStoreRef, BuiltinFunctionId, RealmRef, Value};

    fn assert_host_hooks<T: HostHooks>() {}

    #[test]
    fn noop_host_hooks_satisfies_host_hooks_trait() {
        assert_host_hooks::<NoopHostHooks>();
        let noop = NoopHostHooks;
        let diagnostic = DiagnosticReportRequest {
            severity: Severity::Warning,
            source: Some(SourceId::new(1)),
            span: Some(Span::from_offsets(SourceId::new(1), 0, 1)),
            message: "warn".into(),
        };

        assert!(noop.report_diagnostic(&diagnostic).is_ok());
        let err = noop
            .load_script_source(&ScriptSourceRequest {
                path: "main.js".into(),
                referrer: None,
                is_entry: true,
            })
            .unwrap_err();
        assert_eq!(err.kind(), HostErrorKind::Unsupported);
        assert_eq!(err.operation(), "load_script_source");
    }

    #[test]
    fn noop_host_hooks_provides_deterministic_temporal_defaults() {
        let noop = NoopHostHooks;

        let instant = noop
            .temporal_current_instant(&TemporalCurrentInstantRequest {})
            .unwrap();
        let time_zone = noop
            .temporal_default_time_zone(&TemporalDefaultTimeZoneRequest {})
            .unwrap();
        let default_is_utc = noop
            .temporal_default_time_zone_is_utc(&TemporalDefaultTimeZoneRequest {})
            .unwrap();
        let civil = noop
            .temporal_instant_to_civil_time(&TemporalInstantToCivilRequest {
                time_zone_id: "UTC".into(),
                epoch_nanoseconds: 0,
            })
            .unwrap();
        let round_trip = noop
            .temporal_civil_time_to_instant(&TemporalCivilToInstantRequest {
                time_zone_id: "UTC".into(),
                date_time: TemporalCivilDateTime::new(1970, 1, 1, 0, 0, 0, 0, 0, 0),
                disambiguation: TemporalDisambiguation::Compatible,
            })
            .unwrap();

        assert_eq!(instant.epoch_nanoseconds, 0);
        assert_eq!(time_zone.time_zone_id, "UTC");
        assert!(default_is_utc);
        assert_eq!(
            civil,
            TemporalCivilTime {
                date_time: TemporalCivilDateTime::new(1970, 1, 1, 0, 0, 0, 0, 0, 0),
                offset_nanoseconds: 0,
            }
        );
        assert_eq!(
            round_trip,
            TemporalInstantWithOffset {
                epoch_nanoseconds: 0,
                offset_nanoseconds: 0,
            }
        );
        let offset_civil = noop
            .temporal_instant_to_civil_time(&TemporalInstantToCivilRequest {
                time_zone_id: "+01:00".into(),
                epoch_nanoseconds: 0,
            })
            .unwrap();
        let offset_round_trip = noop
            .temporal_civil_time_to_instant(&TemporalCivilToInstantRequest {
                time_zone_id: "+01:00".into(),
                date_time: offset_civil.date_time,
                disambiguation: TemporalDisambiguation::Compatible,
            })
            .unwrap();
        assert_eq!(
            offset_civil,
            TemporalCivilTime {
                date_time: TemporalCivilDateTime::new(1970, 1, 1, 1, 0, 0, 0, 0, 0),
                offset_nanoseconds: 3_600_000_000_000,
            }
        );
        assert_eq!(
            offset_round_trip,
            TemporalInstantWithOffset {
                epoch_nanoseconds: 0,
                offset_nanoseconds: 3_600_000_000_000,
            }
        );

        let err = noop
            .temporal_instant_to_civil_time(&TemporalInstantToCivilRequest {
                time_zone_id: "Europe/Berlin".into(),
                epoch_nanoseconds: 0,
            })
            .unwrap_err();
        assert_eq!(err.kind(), HostErrorKind::Unsupported);
        assert_eq!(err.operation(), "temporal_instant_to_civil_time");
    }

    #[test]
    fn test_host_records_diagnostics_exceptions_and_source_loading() {
        let host = TestHost::new();
        host.define_script_source("main.js", LoadedSourceText::new("main.js", "print('ok');"));
        host.define_module_source(
            "dep",
            LoadedModuleSource::new(
                ModuleKey::new("/tmp/dep.js"),
                "dep.js",
                "export const value = 1;",
            ),
        );
        host.define_import_meta(
            ModuleKey::new("/tmp/dep.js"),
            ImportMetaProperties::new(vec![ImportMetaProperty {
                key: "url".into(),
                value: ImportMetaValue::String("file:///tmp/dep.js".into()),
            }]),
        );

        let diagnostic = DiagnosticReportRequest {
            severity: Severity::Error,
            source: Some(SourceId::new(9)),
            span: Some(Span::from_offsets(SourceId::new(9), 2, 4)),
            message: "bad token".into(),
        };
        let exception = UncaughtExceptionReport {
            source: Some(SourceId::new(9)),
            realm: Some(RealmRef::from_raw(3).unwrap()),
            thrown_value: Value::from_smi(17),
            message: "boom".into(),
        };
        let script = host
            .load_script_source(&ScriptSourceRequest {
                path: "main.js".into(),
                referrer: None,
                is_entry: true,
            })
            .unwrap();
        let module = host
            .load_module_source(&ModuleSourceRequest {
                specifier: "dep".into(),
                referrer: Some(ModuleKey::new("/tmp/main.js")),
                attributes: vec![ModuleImportAttribute {
                    key: "type".into(),
                    value: "js".into(),
                }],
            })
            .unwrap();
        let import_meta = host
            .resolve_import_meta(&ImportMetaRequest {
                module: module.key.clone(),
            })
            .unwrap();
        host.report_diagnostic(&diagnostic).unwrap();
        host.report_uncaught_exception(&exception).unwrap();

        let snapshot = host.snapshot();
        assert_eq!(script.display_name, "main.js");
        assert_eq!(module.display_name, "dep.js");
        assert_eq!(module.key, ModuleKey::new("/tmp/dep.js"));
        assert_eq!(
            import_meta,
            ImportMetaProperties::new(vec![ImportMetaProperty {
                key: "url".into(),
                value: ImportMetaValue::String("file:///tmp/dep.js".into()),
            }])
        );
        assert_eq!(
            snapshot.calls,
            vec![
                HostCall::LoadScript(ScriptSourceRequest {
                    path: "main.js".into(),
                    referrer: None,
                    is_entry: true,
                }),
                HostCall::LoadModule(ModuleSourceRequest {
                    specifier: "dep".into(),
                    referrer: Some(ModuleKey::new("/tmp/main.js")),
                    attributes: vec![ModuleImportAttribute {
                        key: "type".into(),
                        value: "js".into(),
                    }],
                }),
                HostCall::ResolveImportMeta(ImportMetaRequest {
                    module: ModuleKey::new("/tmp/dep.js"),
                }),
                HostCall::Diagnostic(diagnostic),
                HostCall::UncaughtException(exception),
            ]
        );
    }

    #[test]
    fn test_host_records_job_and_agent_lifecycle_requests() {
        let host = TestHost::new();
        let agent = host
            .create_agent(&CreateAgentRequest {
                parent_agent: None,
                kind: AgentSpawnKind::Harness,
                debug_name: Some("worker".into()),
            })
            .unwrap();
        let thread = host
            .start_agent_thread(&StartAgentThreadRequest {
                agent_id: agent.agent_id,
                kind: AgentThreadStartKind::AgentMain,
                debug_name: Some("worker-thread".into()),
            })
            .unwrap();
        host.observe_job(&JobObservation {
            agent: Some(agent.agent_id),
            job_id: HostJobId::from_raw(1).unwrap(),
            phase: HostJobPhase::Enqueued,
            kind: HostJobKind::Native(BuiltinFunctionId::from_raw(4).unwrap()),
        })
        .unwrap();

        let snapshot = host.snapshot();
        assert_eq!(agent.agent_id.get(), 1);
        assert_eq!(thread.thread_id.get(), 1);
        assert!(thread.started);
        assert!(matches!(snapshot.calls[0], HostCall::CreateAgent(_)));
        assert!(matches!(snapshot.calls[1], HostCall::StartAgentThread(_)));
        assert!(matches!(snapshot.calls[2], HostCall::ObserveJob(_)));
    }

    #[test]
    fn test_host_handles_buffer_boundaries_and_parking_primitives() {
        let host = TestHost::new();
        host.push_park_result(Ok(ParkAgentResult {
            status: ParkAgentStatus::TimedOut,
        }));
        host.push_unpark_result(Ok(UnparkAgentResult {
            woken_agents: 2,
            remaining_waiters: true,
        }));

        let source_agent = HostAgentId::from_raw(1).unwrap();
        let target_agent = HostAgentId::from_raw(2).unwrap();
        let transfer = host
            .transfer_array_buffer(&ArrayBufferTransferRequest {
                source_agent,
                target_agent,
                buffer_id: HostTransferredBufferId::from_raw(7).unwrap(),
                byte_length: 64,
                detach_on_success: true,
            })
            .unwrap();
        let shared = host
            .share_array_buffer(&SharedArrayBufferShareRequest {
                source_agent,
                target_agent,
                backing_store: BackingStoreRef::from_raw(9).unwrap(),
                byte_length: 128,
            })
            .unwrap();
        let location = WaitLocation::new(BackingStoreRef::from_raw(9).unwrap(), 16);
        let parked = host
            .park_agent(&ParkAgentRequest {
                agent_id: target_agent,
                thread_id: Some(HostThreadId::from_raw(3).unwrap()),
                location,
                timeout_ns: Some(50),
                allow_async: false,
            })
            .unwrap();
        let unparked = host
            .unpark_agent(&UnparkAgentRequest {
                location,
                max_count: 3,
            })
            .unwrap();

        let snapshot = host.snapshot();
        assert_eq!(transfer.buffer_id.get(), 1);
        assert!(transfer.detached);
        assert_eq!(shared.shared_buffer.get(), 1);
        assert_eq!(parked.status, ParkAgentStatus::TimedOut);
        assert_eq!(unparked.woken_agents, 2);
        assert!(unparked.remaining_waiters);
        assert!(matches!(
            snapshot.calls[0],
            HostCall::TransferArrayBuffer(_)
        ));
        assert!(matches!(snapshot.calls[1], HostCall::ShareArrayBuffer(_)));
        assert!(matches!(snapshot.calls[2], HostCall::ParkAgent(_)));
        assert!(matches!(snapshot.calls[3], HostCall::UnparkAgent(_)));
    }

    #[test]
    fn test_host_records_temporal_requests_and_uses_defined_responses() {
        let host = TestHost::new();
        host.set_temporal_current_instant(TemporalInstant::new(123));
        host.set_temporal_default_time_zone("Europe/Berlin");

        let instant_request = TemporalInstantToCivilRequest {
            time_zone_id: "Europe/Berlin".into(),
            epoch_nanoseconds: 123,
        };
        let civil_response = TemporalCivilTime {
            date_time: TemporalCivilDateTime::new(1970, 1, 1, 1, 0, 0, 0, 0, 123),
            offset_nanoseconds: 3_600_000_000_000,
        };
        host.define_temporal_instant_to_civil(instant_request.clone(), Ok(civil_response));

        let civil_request = TemporalCivilToInstantRequest {
            time_zone_id: "Europe/Berlin".into(),
            date_time: civil_response.date_time,
            disambiguation: TemporalDisambiguation::Compatible,
        };
        let instant_response = TemporalInstantWithOffset {
            epoch_nanoseconds: 123,
            offset_nanoseconds: 3_600_000_000_000,
        };
        host.define_temporal_civil_to_instant(civil_request.clone(), Ok(instant_response));

        let instant = host
            .temporal_current_instant(&TemporalCurrentInstantRequest {})
            .unwrap();
        let time_zone = host
            .temporal_default_time_zone(&TemporalDefaultTimeZoneRequest {})
            .unwrap();
        let default_is_utc = host
            .temporal_default_time_zone_is_utc(&TemporalDefaultTimeZoneRequest {})
            .unwrap();
        let civil = host
            .temporal_instant_to_civil_time(&instant_request)
            .unwrap();
        let round_trip = host.temporal_civil_time_to_instant(&civil_request).unwrap();

        assert_eq!(instant, TemporalInstant::new(123));
        assert_eq!(
            time_zone,
            TemporalDefaultTimeZone {
                time_zone_id: "Europe/Berlin".into(),
            }
        );
        assert!(!default_is_utc);
        assert_eq!(civil, civil_response);
        assert_eq!(round_trip, instant_response);

        let snapshot = host.snapshot();
        assert!(matches!(
            snapshot.calls[0],
            HostCall::TemporalCurrentInstant(_)
        ));
        assert!(matches!(
            snapshot.calls[1],
            HostCall::TemporalDefaultTimeZone(_)
        ));
        assert!(matches!(
            snapshot.calls[2],
            HostCall::TemporalDefaultTimeZoneIsUtc(_)
        ));
        assert!(matches!(
            snapshot.calls[3],
            HostCall::TemporalInstantToCivil(_)
        ));
        assert!(matches!(
            snapshot.calls[4],
            HostCall::TemporalCivilToInstant(_)
        ));
    }
}
