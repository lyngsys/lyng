use crate::error::{HostError, HostResult};
use crate::hooks::{
    ArrayBufferTransferRequest, ArrayBufferTransferResponse, CreateAgentRequest,
    CreateAgentResponse, DiagnosticReportRequest, HostHooks, ImportMetaProperties,
    ImportMetaRequest, JobObservation, LoadedModuleSource, LoadedSourceText, ModuleKey,
    ModuleSourceRequest, ParkAgentRequest, ParkAgentResult, ParkAgentStatus, ScriptSourceRequest,
    SharedArrayBufferShareRequest, SharedArrayBufferShareResponse, StartAgentThreadRequest,
    StartAgentThreadResponse, TemporalCivilTime, TemporalCivilToInstantRequest,
    TemporalCurrentInstantRequest, TemporalDefaultTimeZone, TemporalDefaultTimeZoneRequest,
    TemporalInstant, TemporalInstantToCivilRequest, TemporalInstantWithOffset, UnparkAgentRequest,
    UnparkAgentResult,
};
use crate::ids::{HostAgentId, HostSharedBufferId, HostThreadId, HostTransferredBufferId};
use lyng_js_types::BackingStoreRef;
use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex};

/// Minimal no-op host hook implementation used by early compile smoke tests.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NoopHostHooks;

impl HostHooks for NoopHostHooks {}

/// Recorded host-boundary call made against [`TestHost`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostCall {
    Diagnostic(DiagnosticReportRequest),
    UncaughtException(crate::UncaughtExceptionReport),
    LoadScript(ScriptSourceRequest),
    LoadModule(ModuleSourceRequest),
    ResolveImportMeta(ImportMetaRequest),
    ObserveJob(JobObservation),
    CreateAgent(CreateAgentRequest),
    StartAgentThread(StartAgentThreadRequest),
    TransferArrayBuffer(ArrayBufferTransferRequest),
    ShareArrayBuffer(SharedArrayBufferShareRequest),
    ParkAgent(ParkAgentRequest),
    UnparkAgent(UnparkAgentRequest),
    TemporalCurrentInstant(TemporalCurrentInstantRequest),
    TemporalDefaultTimeZone(TemporalDefaultTimeZoneRequest),
    TemporalInstantToCivil(TemporalInstantToCivilRequest),
    TemporalCivilToInstant(TemporalCivilToInstantRequest),
}

/// Immutable snapshot of the recorded default test-host activity.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TestHostSnapshot {
    pub calls: Vec<HostCall>,
}

/// Default test host for Phase 3 runtime-boundary tests.
#[derive(Clone, Default)]
pub struct TestHost {
    state: Arc<Mutex<TestHostState>>,
}

struct TestHostState {
    calls: Vec<HostCall>,
    script_sources: BTreeMap<String, LoadedSourceText>,
    module_sources: BTreeMap<String, LoadedModuleSource>,
    import_meta: BTreeMap<ModuleKey, ImportMetaProperties>,
    create_agent_results: VecDeque<HostResult<CreateAgentResponse>>,
    start_thread_results: VecDeque<HostResult<StartAgentThreadResponse>>,
    transfer_results: VecDeque<HostResult<ArrayBufferTransferResponse>>,
    share_results: VecDeque<HostResult<SharedArrayBufferShareResponse>>,
    park_results: VecDeque<HostResult<ParkAgentResult>>,
    unpark_results: VecDeque<HostResult<UnparkAgentResult>>,
    next_agent_id: u32,
    next_thread_id: u32,
    next_transfer_buffer_id: u32,
    next_shared_buffer_id: u32,
    shared_buffer_by_backing_store: BTreeMap<BackingStoreRef, HostSharedBufferId>,
    temporal_current_instant: TemporalInstant,
    temporal_default_time_zone: String,
    temporal_instant_to_civil:
        BTreeMap<TemporalInstantToCivilRequest, HostResult<TemporalCivilTime>>,
    temporal_civil_to_instant:
        BTreeMap<TemporalCivilToInstantRequest, HostResult<TemporalInstantWithOffset>>,
}

impl Default for TestHostState {
    fn default() -> Self {
        Self {
            calls: Vec::new(),
            script_sources: BTreeMap::new(),
            module_sources: BTreeMap::new(),
            import_meta: BTreeMap::new(),
            create_agent_results: VecDeque::new(),
            start_thread_results: VecDeque::new(),
            transfer_results: VecDeque::new(),
            share_results: VecDeque::new(),
            park_results: VecDeque::new(),
            unpark_results: VecDeque::new(),
            next_agent_id: 0,
            next_thread_id: 0,
            next_transfer_buffer_id: 0,
            next_shared_buffer_id: 0,
            shared_buffer_by_backing_store: BTreeMap::new(),
            temporal_current_instant: TemporalInstant::new(0),
            temporal_default_time_zone: "UTC".into(),
            temporal_instant_to_civil: BTreeMap::new(),
            temporal_civil_to_instant: BTreeMap::new(),
        }
    }
}

impl TestHost {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Installs one script source fixture for later host lookups.
    ///
    /// # Panics
    /// Panics if the internal test-host mutex is poisoned.
    pub fn define_script_source(&self, path: impl Into<String>, source: LoadedSourceText) {
        self.state
            .lock()
            .expect("test host mutex poisoned")
            .script_sources
            .insert(path.into(), source);
    }

    /// Installs one module source fixture for later host lookups.
    ///
    /// # Panics
    /// Panics if the internal test-host mutex is poisoned.
    pub fn define_module_source(&self, specifier: impl Into<String>, source: LoadedModuleSource) {
        self.state
            .lock()
            .expect("test host mutex poisoned")
            .module_sources
            .insert(specifier.into(), source);
    }

    /// Installs one import.meta fixture for later host lookups.
    ///
    /// # Panics
    /// Panics if the internal test-host mutex is poisoned.
    pub fn define_import_meta(&self, key: ModuleKey, properties: ImportMetaProperties) {
        self.state
            .lock()
            .expect("test host mutex poisoned")
            .import_meta
            .insert(key, properties);
    }

    /// Overrides the default Temporal current-instant response.
    ///
    /// # Panics
    /// Panics if the internal test-host mutex is poisoned.
    pub fn set_temporal_current_instant(&self, instant: TemporalInstant) {
        self.state
            .lock()
            .expect("test host mutex poisoned")
            .temporal_current_instant = instant;
    }

    /// Overrides the default Temporal default time-zone identifier.
    ///
    /// # Panics
    /// Panics if the internal test-host mutex is poisoned.
    pub fn set_temporal_default_time_zone(&self, time_zone_id: impl Into<String>) {
        self.state
            .lock()
            .expect("test host mutex poisoned")
            .temporal_default_time_zone = time_zone_id.into();
    }

    /// Installs one instant-to-civil Temporal host response.
    ///
    /// # Panics
    /// Panics if the internal test-host mutex is poisoned.
    pub fn define_temporal_instant_to_civil(
        &self,
        request: TemporalInstantToCivilRequest,
        result: HostResult<TemporalCivilTime>,
    ) {
        self.state
            .lock()
            .expect("test host mutex poisoned")
            .temporal_instant_to_civil
            .insert(request, result);
    }

    /// Installs one civil-to-instant Temporal host response.
    ///
    /// # Panics
    /// Panics if the internal test-host mutex is poisoned.
    pub fn define_temporal_civil_to_instant(
        &self,
        request: TemporalCivilToInstantRequest,
        result: HostResult<TemporalInstantWithOffset>,
    ) {
        self.state
            .lock()
            .expect("test host mutex poisoned")
            .temporal_civil_to_instant
            .insert(request, result);
    }

    /// Queues one `create_agent` host result.
    ///
    /// # Panics
    /// Panics if the internal test-host mutex is poisoned.
    pub fn push_create_agent_result(&self, result: HostResult<CreateAgentResponse>) {
        self.state
            .lock()
            .expect("test host mutex poisoned")
            .create_agent_results
            .push_back(result);
    }

    /// Queues one `start_agent_thread` host result.
    ///
    /// # Panics
    /// Panics if the internal test-host mutex is poisoned.
    pub fn push_start_thread_result(&self, result: HostResult<StartAgentThreadResponse>) {
        self.state
            .lock()
            .expect("test host mutex poisoned")
            .start_thread_results
            .push_back(result);
    }

    /// Queues one `transfer_array_buffer` host result.
    ///
    /// # Panics
    /// Panics if the internal test-host mutex is poisoned.
    pub fn push_transfer_result(&self, result: HostResult<ArrayBufferTransferResponse>) {
        self.state
            .lock()
            .expect("test host mutex poisoned")
            .transfer_results
            .push_back(result);
    }

    /// Queues one `share_array_buffer` host result.
    ///
    /// # Panics
    /// Panics if the internal test-host mutex is poisoned.
    pub fn push_share_result(&self, result: HostResult<SharedArrayBufferShareResponse>) {
        self.state
            .lock()
            .expect("test host mutex poisoned")
            .share_results
            .push_back(result);
    }

    /// Queues one `park_agent` host result.
    ///
    /// # Panics
    /// Panics if the internal test-host mutex is poisoned.
    pub fn push_park_result(&self, result: HostResult<ParkAgentResult>) {
        self.state
            .lock()
            .expect("test host mutex poisoned")
            .park_results
            .push_back(result);
    }

    /// Queues one `unpark_agent` host result.
    ///
    /// # Panics
    /// Panics if the internal test-host mutex is poisoned.
    pub fn push_unpark_result(&self, result: HostResult<UnparkAgentResult>) {
        self.state
            .lock()
            .expect("test host mutex poisoned")
            .unpark_results
            .push_back(result);
    }

    /// Snapshots the recorded host-call log for assertions.
    ///
    /// # Panics
    /// Panics if the internal test-host mutex is poisoned.
    pub fn snapshot(&self) -> TestHostSnapshot {
        let state = self.state.lock().expect("test host mutex poisoned");
        TestHostSnapshot {
            calls: state.calls.clone(),
        }
    }
}

impl HostHooks for TestHost {
    fn report_diagnostic(&self, request: &DiagnosticReportRequest) -> HostResult<()> {
        self.state
            .lock()
            .expect("test host mutex poisoned")
            .calls
            .push(HostCall::Diagnostic(request.clone()));
        Ok(())
    }

    fn report_uncaught_exception(
        &self,
        request: &crate::UncaughtExceptionReport,
    ) -> HostResult<()> {
        self.state
            .lock()
            .expect("test host mutex poisoned")
            .calls
            .push(HostCall::UncaughtException(request.clone()));
        Ok(())
    }

    fn load_script_source(&self, request: &ScriptSourceRequest) -> HostResult<LoadedSourceText> {
        let mut state = self.state.lock().expect("test host mutex poisoned");
        state.calls.push(HostCall::LoadScript(request.clone()));
        state
            .script_sources
            .get(&request.path)
            .cloned()
            .ok_or_else(|| {
                HostError::not_found(
                    "load_script_source",
                    format!("no scripted test source for `{}`", request.path),
                )
            })
    }

    fn load_module_source(&self, request: &ModuleSourceRequest) -> HostResult<LoadedModuleSource> {
        let mut state = self.state.lock().expect("test host mutex poisoned");
        state.calls.push(HostCall::LoadModule(request.clone()));
        state
            .module_sources
            .get(&request.specifier)
            .cloned()
            .ok_or_else(|| {
                HostError::not_found(
                    "load_module_source",
                    format!("no scripted test module for `{}`", request.specifier),
                )
            })
    }

    fn resolve_import_meta(&self, request: &ImportMetaRequest) -> HostResult<ImportMetaProperties> {
        let mut state = self.state.lock().expect("test host mutex poisoned");
        state
            .calls
            .push(HostCall::ResolveImportMeta(request.clone()));
        Ok(state
            .import_meta
            .get(&request.module)
            .cloned()
            .unwrap_or_default())
    }

    fn temporal_current_instant(
        &self,
        request: &TemporalCurrentInstantRequest,
    ) -> HostResult<TemporalInstant> {
        let mut state = self.state.lock().expect("test host mutex poisoned");
        state.calls.push(HostCall::TemporalCurrentInstant(*request));
        Ok(state.temporal_current_instant)
    }

    fn temporal_default_time_zone(
        &self,
        request: &TemporalDefaultTimeZoneRequest,
    ) -> HostResult<TemporalDefaultTimeZone> {
        let mut state = self.state.lock().expect("test host mutex poisoned");
        state
            .calls
            .push(HostCall::TemporalDefaultTimeZone(*request));
        Ok(TemporalDefaultTimeZone::new(
            state.temporal_default_time_zone.clone(),
        ))
    }

    fn temporal_instant_to_civil_time(
        &self,
        request: &TemporalInstantToCivilRequest,
    ) -> HostResult<TemporalCivilTime> {
        let mut state = self.state.lock().expect("test host mutex poisoned");
        state
            .calls
            .push(HostCall::TemporalInstantToCivil(request.clone()));
        if let Some(result) = state.temporal_instant_to_civil.get(request) {
            return result.clone();
        }
        drop(state);
        NoopHostHooks.temporal_instant_to_civil_time(request)
    }

    fn temporal_civil_time_to_instant(
        &self,
        request: &TemporalCivilToInstantRequest,
    ) -> HostResult<TemporalInstantWithOffset> {
        let mut state = self.state.lock().expect("test host mutex poisoned");
        state
            .calls
            .push(HostCall::TemporalCivilToInstant(request.clone()));
        if let Some(result) = state.temporal_civil_to_instant.get(request) {
            return result.clone();
        }
        drop(state);
        NoopHostHooks.temporal_civil_time_to_instant(request)
    }

    fn observe_job(&self, request: &JobObservation) -> HostResult<()> {
        self.state
            .lock()
            .expect("test host mutex poisoned")
            .calls
            .push(HostCall::ObserveJob(request.clone()));
        Ok(())
    }

    fn create_agent(&self, request: &CreateAgentRequest) -> HostResult<CreateAgentResponse> {
        let mut state = self.state.lock().expect("test host mutex poisoned");
        state.calls.push(HostCall::CreateAgent(request.clone()));
        if let Some(result) = state.create_agent_results.pop_front() {
            return result;
        }

        let next_raw = state.next_agent_id.max(1);
        state.next_agent_id = next_raw
            .checked_add(1)
            .expect("test host agent id overflowed supported u32 range");
        Ok(CreateAgentResponse {
            agent_id: HostAgentId::from_raw(next_raw)
                .expect("test host agent id must stay non-zero"),
        })
    }

    fn start_agent_thread(
        &self,
        request: &StartAgentThreadRequest,
    ) -> HostResult<StartAgentThreadResponse> {
        let mut state = self.state.lock().expect("test host mutex poisoned");
        state
            .calls
            .push(HostCall::StartAgentThread(request.clone()));
        if let Some(result) = state.start_thread_results.pop_front() {
            return result;
        }

        let next_raw = state.next_thread_id.max(1);
        state.next_thread_id = next_raw
            .checked_add(1)
            .expect("test host thread id overflowed supported u32 range");
        Ok(StartAgentThreadResponse {
            thread_id: HostThreadId::from_raw(next_raw)
                .expect("test host thread id must stay non-zero"),
            started: true,
        })
    }

    fn transfer_array_buffer(
        &self,
        request: &ArrayBufferTransferRequest,
    ) -> HostResult<ArrayBufferTransferResponse> {
        let mut state = self.state.lock().expect("test host mutex poisoned");
        state.calls.push(HostCall::TransferArrayBuffer(*request));
        if let Some(result) = state.transfer_results.pop_front() {
            return result;
        }

        let next_raw = state.next_transfer_buffer_id.max(1);
        state.next_transfer_buffer_id = next_raw
            .checked_add(1)
            .expect("test host transfer buffer id overflowed supported u32 range");
        Ok(ArrayBufferTransferResponse {
            buffer_id: HostTransferredBufferId::from_raw(next_raw)
                .expect("test host buffer id must stay non-zero"),
            detached: request.detach_on_success,
        })
    }

    fn share_array_buffer(
        &self,
        request: &SharedArrayBufferShareRequest,
    ) -> HostResult<SharedArrayBufferShareResponse> {
        let mut state = self.state.lock().expect("test host mutex poisoned");
        state.calls.push(HostCall::ShareArrayBuffer(*request));
        if let Some(result) = state.share_results.pop_front() {
            return result;
        }

        if let Some(shared_buffer) = state
            .shared_buffer_by_backing_store
            .get(&request.backing_store)
            .copied()
        {
            return Ok(SharedArrayBufferShareResponse { shared_buffer });
        }

        let next_raw = state.next_shared_buffer_id.max(1);
        state.next_shared_buffer_id = next_raw
            .checked_add(1)
            .expect("test host shared-buffer id overflowed supported u32 range");
        let shared_buffer = HostSharedBufferId::from_raw(next_raw)
            .expect("test host shared-buffer id must stay non-zero");
        state
            .shared_buffer_by_backing_store
            .insert(request.backing_store, shared_buffer);
        Ok(SharedArrayBufferShareResponse { shared_buffer })
    }

    fn park_agent(&self, request: &ParkAgentRequest) -> HostResult<ParkAgentResult> {
        let mut state = self.state.lock().expect("test host mutex poisoned");
        state.calls.push(HostCall::ParkAgent(*request));
        if let Some(result) = state.park_results.pop_front() {
            return result;
        }

        Ok(ParkAgentResult {
            status: ParkAgentStatus::Parked,
        })
    }

    fn unpark_agent(&self, request: &UnparkAgentRequest) -> HostResult<UnparkAgentResult> {
        let mut state = self.state.lock().expect("test host mutex poisoned");
        state.calls.push(HostCall::UnparkAgent(*request));
        if let Some(result) = state.unpark_results.pop_front() {
            return result;
        }

        Ok(UnparkAgentResult {
            woken_agents: request.max_count.min(1),
            remaining_waiters: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::{AgentSpawnKind, AgentThreadStartKind};

    #[test]
    #[should_panic(expected = "test host agent id overflowed supported u32 range")]
    fn create_agent_ids_fail_loudly_on_overflow() {
        let host = TestHost::new();
        host.state
            .lock()
            .expect("test host mutex poisoned")
            .next_agent_id = u32::MAX;

        let _ = host.create_agent(&CreateAgentRequest {
            parent_agent: None,
            kind: AgentSpawnKind::Harness,
            debug_name: Some("overflow".into()),
        });
    }

    #[test]
    #[should_panic(expected = "test host thread id overflowed supported u32 range")]
    fn thread_ids_fail_loudly_on_overflow() {
        let host = TestHost::new();
        host.state
            .lock()
            .expect("test host mutex poisoned")
            .next_thread_id = u32::MAX;

        let _ = host.start_agent_thread(&StartAgentThreadRequest {
            agent_id: HostAgentId::from_raw(1).unwrap(),
            kind: AgentThreadStartKind::Harness,
            debug_name: Some("overflow".into()),
        });
    }

    #[test]
    #[should_panic(expected = "test host transfer buffer id overflowed supported u32 range")]
    fn transferred_buffer_ids_fail_loudly_on_overflow() {
        let host = TestHost::new();
        host.state
            .lock()
            .expect("test host mutex poisoned")
            .next_transfer_buffer_id = u32::MAX;

        let _ = host.transfer_array_buffer(&ArrayBufferTransferRequest {
            source_agent: HostAgentId::from_raw(1).unwrap(),
            target_agent: HostAgentId::from_raw(2).unwrap(),
            buffer_id: HostTransferredBufferId::from_raw(3).unwrap(),
            byte_length: 16,
            detach_on_success: true,
        });
    }

    #[test]
    #[should_panic(expected = "test host shared-buffer id overflowed supported u32 range")]
    fn shared_buffer_ids_fail_loudly_on_overflow() {
        let host = TestHost::new();
        host.state
            .lock()
            .expect("test host mutex poisoned")
            .next_shared_buffer_id = u32::MAX;

        let _ = host.share_array_buffer(&SharedArrayBufferShareRequest {
            source_agent: HostAgentId::from_raw(1).unwrap(),
            target_agent: HostAgentId::from_raw(2).unwrap(),
            backing_store: BackingStoreRef::from_raw(3).unwrap(),
            byte_length: 32,
        });
    }

    #[test]
    fn share_array_buffer_reuses_host_handle_for_existing_backing_store() {
        let host = TestHost::new();
        let source_agent = HostAgentId::from_raw(1).unwrap();
        let target_agent = HostAgentId::from_raw(2).unwrap();
        let backing_store = BackingStoreRef::from_raw(7).unwrap();

        let first = host
            .share_array_buffer(&SharedArrayBufferShareRequest {
                source_agent,
                target_agent,
                backing_store,
                byte_length: 32,
            })
            .unwrap();
        let second = host
            .share_array_buffer(&SharedArrayBufferShareRequest {
                source_agent,
                target_agent,
                backing_store,
                byte_length: 32,
            })
            .unwrap();

        assert_eq!(first, second);
    }
}
