use super::{ExecutableId, JobId, PromiseCapabilityId, PromiseReactionId, RuntimeDomainAccounting};
use lyng_js_common::AtomId;
use lyng_js_gc::{PrimitiveTracer, TraceHeapEdges};
use lyng_js_host::{HostJobId, HostJobKind, WaitLocation};
use lyng_js_types::{ObjectRef, RealmRef, Value};
use std::collections::VecDeque;
use std::mem::size_of;

use crate::WaiterToken;

/// Frozen queue families owned by one `Agent`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JobQueueKind {
    Promise,
    Script,
    Module,
    Harness,
    Native,
}

impl From<HostJobKind> for JobQueueKind {
    fn from(value: HostJobKind) -> Self {
        match value {
            HostJobKind::Promise => Self::Promise,
            HostJobKind::Script => Self::Script,
            HostJobKind::Module => Self::Module,
            HostJobKind::Harness => Self::Harness,
            HostJobKind::Native(_) => Self::Native,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeJobPayload {
    Executable,
    PromiseReaction {
        reaction: PromiseReactionId,
        argument: Value,
    },
    PromiseThenableResolve {
        promise: ObjectRef,
        thenable: ObjectRef,
        then: ObjectRef,
    },
    DynamicImportEvaluate {
        request: u32,
        script_or_module_referrer: Option<AtomId>,
    },
    DynamicImportSettle {
        capability: PromiseCapabilityId,
        value: Value,
        rejected: bool,
        script_or_module_referrer: Option<AtomId>,
    },
    AtomicsWaitAsyncTimeout {
        location: WaitLocation,
        token: WaiterToken,
        promise: ObjectRef,
    },
    FinalizationCleanup {
        registry: ObjectRef,
    },
}

/// One runtime-owned queued job record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeJob {
    pub(crate) id: JobId,
    pub(crate) kind: HostJobKind,
    pub(crate) executable: ExecutableId,
    pub(crate) payload: RuntimeJobPayload,
    pub(crate) realm: Option<RealmRef>,
    pub(crate) debug_name: Option<String>,
}

impl RuntimeJob {
    #[inline]
    pub const fn id(&self) -> JobId {
        self.id
    }

    #[inline]
    pub const fn kind(&self) -> HostJobKind {
        self.kind
    }

    #[inline]
    pub fn queue_kind(&self) -> JobQueueKind {
        JobQueueKind::from(self.kind)
    }

    #[inline]
    pub const fn executable(&self) -> ExecutableId {
        self.executable
    }

    #[inline]
    pub const fn payload(&self) -> RuntimeJobPayload {
        self.payload
    }

    #[inline]
    pub const fn realm(&self) -> Option<RealmRef> {
        self.realm
    }

    #[inline]
    pub fn debug_name(&self) -> Option<&str> {
        self.debug_name.as_deref()
    }

    #[inline]
    pub const fn host_job_id(&self) -> HostJobId {
        HostJobId::new(self.id.raw())
    }
}

#[derive(Clone, Default)]
pub(crate) struct AgentJobQueues {
    promise: VecDeque<RuntimeJob>,
    script: VecDeque<RuntimeJob>,
    module: VecDeque<RuntimeJob>,
    harness: VecDeque<RuntimeJob>,
    native: VecDeque<RuntimeJob>,
}

impl AgentJobQueues {
    pub(crate) fn enqueue(&mut self, job: RuntimeJob) {
        self.queue_mut(job.queue_kind()).push_back(job);
    }

    pub(crate) fn dequeue(&mut self, kind: JobQueueKind) -> Option<RuntimeJob> {
        self.queue_mut(kind).pop_front()
    }

    pub(crate) fn snapshot(&self, kind: JobQueueKind) -> Vec<RuntimeJob> {
        self.queue(kind).iter().cloned().collect()
    }

    pub(crate) fn len(&self, kind: JobQueueKind) -> usize {
        self.queue(kind).len()
    }

    pub(crate) fn total_len(&self) -> usize {
        self.promise.len()
            + self.script.len()
            + self.module.len()
            + self.harness.len()
            + self.native.len()
    }

    pub(crate) fn promise_job_accounting(&self) -> RuntimeDomainAccounting {
        let records = self.promise.len();
        let metadata_bytes = size_of::<RuntimeJob>() * records;
        let payload_bytes = self
            .promise
            .iter()
            .filter_map(|job| job.debug_name.as_ref())
            .map(String::len)
            .sum::<usize>();
        RuntimeDomainAccounting {
            records,
            metadata_bytes,
            payload_bytes,
            live_bytes: metadata_bytes + payload_bytes,
        }
    }

    fn queue(&self, kind: JobQueueKind) -> &VecDeque<RuntimeJob> {
        match kind {
            JobQueueKind::Promise => &self.promise,
            JobQueueKind::Script => &self.script,
            JobQueueKind::Module => &self.module,
            JobQueueKind::Harness => &self.harness,
            JobQueueKind::Native => &self.native,
        }
    }

    fn queue_mut(&mut self, kind: JobQueueKind) -> &mut VecDeque<RuntimeJob> {
        match kind {
            JobQueueKind::Promise => &mut self.promise,
            JobQueueKind::Script => &mut self.script,
            JobQueueKind::Module => &mut self.module,
            JobQueueKind::Harness => &mut self.harness,
            JobQueueKind::Native => &mut self.native,
        }
    }
}

impl TraceHeapEdges for RuntimeJobPayload {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        match self {
            Self::Executable => {}
            Self::PromiseReaction { argument, .. } => argument.trace_heap_edges(tracer),
            Self::PromiseThenableResolve {
                promise,
                thenable,
                then,
            } => {
                promise.trace_heap_edges(tracer);
                thenable.trace_heap_edges(tracer);
                then.trace_heap_edges(tracer);
            }
            Self::DynamicImportEvaluate { .. } => {}
            Self::DynamicImportSettle { value, .. } => value.trace_heap_edges(tracer),
            Self::AtomicsWaitAsyncTimeout { promise, .. } => promise.trace_heap_edges(tracer),
            Self::FinalizationCleanup { registry } => registry.trace_heap_edges(tracer),
        }
    }
}

impl TraceHeapEdges for RuntimeJob {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.executable.trace_heap_edges(tracer);
        self.payload.trace_heap_edges(tracer);
        self.realm.trace_heap_edges(tracer);
    }
}

impl TraceHeapEdges for AgentJobQueues {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        for queue in [
            &self.promise,
            &self.script,
            &self.module,
            &self.harness,
            &self.native,
        ] {
            for job in queue {
                job.trace_heap_edges(tracer);
            }
        }
    }
}
