use super::{
    merge_primitive_heap_accounting, total_live_bytes, Agent, AgentCluster, AgentId, ExecutableId,
    HostHooks, HostJobKind, HostJobPhase, HostResult, HostThreadId, JobId, JobObservation,
    RealmRef, RuntimeDomainAccounting, RuntimePhase6Accounting,
};
use lyng_js_gc::PrimitiveHeapAccounting;
use lyng_js_host::{
    AgentSpawnKind, AgentThreadStartKind, CreateAgentRequest, HostError, StartAgentThreadRequest,
};

/// Public embedding entrypoint for the Lyng JS runtime substrate.
pub struct Runtime {
    host: Box<dyn HostHooks>,
    root_cluster: AgentCluster,
}

impl Runtime {
    #[inline]
    pub fn new(host: impl HostHooks + 'static) -> Self {
        Self::from_boxed_host(Box::new(host))
    }

    #[inline]
    pub fn from_boxed_host(host: Box<dyn HostHooks>) -> Self {
        Self {
            host,
            root_cluster: AgentCluster::new(),
        }
    }

    #[inline]
    pub fn host(&self) -> &dyn HostHooks {
        self.host.as_ref()
    }

    #[inline]
    pub const fn root_cluster(&self) -> &AgentCluster {
        &self.root_cluster
    }

    #[inline]
    pub const fn root_cluster_mut(&mut self) -> &mut AgentCluster {
        &mut self.root_cluster
    }

    #[inline]
    pub const fn root_agent_id(&self) -> AgentId {
        self.root_cluster.root_agent_id()
    }

    #[inline]
    pub fn root_agent(&self) -> &Agent {
        self.root_cluster.root_agent()
    }

    #[inline]
    pub fn root_agent_mut(&mut self) -> &mut Agent {
        self.root_cluster.root_agent_mut()
    }

    pub fn phase6_accounting(&self) -> RuntimePhase6Accounting {
        let mut heap = PrimitiveHeapAccounting::default();
        let mut iterator_records = RuntimeDomainAccounting::default();
        let mut regexp_payloads = RuntimeDomainAccounting::default();
        let mut regexp_literal_cache = RuntimeDomainAccounting::default();
        let mut module_caches = RuntimeDomainAccounting::default();
        let mut promise_jobs = RuntimeDomainAccounting::default();

        for agent_id in self.root_cluster.agent_ids() {
            let Some(agent) = self.root_cluster.agent(agent_id) else {
                continue;
            };
            let accounting = agent.phase6_accounting();
            heap = merge_primitive_heap_accounting(heap, accounting.heap);
            iterator_records = iterator_records.merge(accounting.iterator_records);
            regexp_payloads = regexp_payloads.merge(accounting.regexp_payloads);
            regexp_literal_cache = regexp_literal_cache.merge(accounting.regexp_literal_cache);
            module_caches = module_caches.merge(accounting.module_caches);
            promise_jobs = promise_jobs.merge(accounting.promise_jobs);
        }

        let backing_stores = self.root_cluster.backing_store_accounting();

        RuntimePhase6Accounting {
            heap,
            iterator_records,
            regexp_payloads,
            regexp_literal_cache,
            module_caches,
            promise_jobs,
            backing_stores,
            live_bytes: total_live_bytes(
                heap,
                iterator_records,
                regexp_payloads,
                regexp_literal_cache,
                module_caches,
                promise_jobs,
                backing_stores,
            ),
        }
    }

    /// Spawns one additional runtime agent through the host.
    ///
    /// # Errors
    /// Returns an error when the host rejects the request or cannot provision another agent.
    pub fn spawn_agent(
        &mut self,
        kind: AgentSpawnKind,
        debug_name: Option<String>,
    ) -> HostResult<AgentId> {
        let response = self.host.create_agent(&CreateAgentRequest {
            parent_agent: self.root_agent().host_id(),
            kind,
            debug_name: debug_name.clone(),
        })?;
        Ok(self
            .root_cluster
            .add_agent(Some(response.agent_id), debug_name))
    }

    /// Starts a host thread for one runtime agent.
    ///
    /// # Errors
    /// Returns an error when the runtime agent is unknown, has no host id, or the host cannot
    /// start the thread.
    pub fn start_agent_thread(
        &mut self,
        agent: AgentId,
        kind: AgentThreadStartKind,
        debug_name: Option<String>,
    ) -> HostResult<HostThreadId> {
        let host_id = self
            .root_cluster
            .agent(agent)
            .ok_or_else(|| HostError::not_found("start_agent_thread", "unknown runtime agent"))?
            .host_id()
            .ok_or_else(|| {
                HostError::invalid_request(
                    "start_agent_thread",
                    "runtime agent is not associated with a host agent id",
                )
            })?;

        let response = self.host.start_agent_thread(&StartAgentThreadRequest {
            agent_id: host_id,
            kind,
            debug_name,
        })?;
        let Some(agent) = self.root_cluster.agent_mut(agent) else {
            return Err(HostError::not_found(
                "start_agent_thread",
                "runtime agent disappeared before thread binding",
            ));
        };
        agent.bind_thread(response.thread_id);
        Ok(response.thread_id)
    }

    /// Enqueues one host-observable job for the selected runtime agent.
    ///
    /// # Errors
    /// Returns an error when the runtime agent is unknown or when the host rejects the job
    /// observation.
    pub fn enqueue_job(
        &mut self,
        agent: AgentId,
        kind: HostJobKind,
        executable: ExecutableId,
        realm: Option<RealmRef>,
        debug_name: Option<String>,
    ) -> HostResult<JobId> {
        let Some(agent_record) = self.root_cluster.agent_mut(agent) else {
            return Err(HostError::not_found(
                "enqueue_job",
                "unknown runtime agent for job enqueue",
            ));
        };
        let job = agent_record.enqueue_job(kind, executable, realm, debug_name);
        self.host.observe_job(&JobObservation {
            agent: agent_record.host_id(),
            job_id: job.host_job_id(),
            phase: HostJobPhase::Enqueued,
            kind,
        })?;
        Ok(job.id())
    }
}
