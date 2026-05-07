use super::Agent;
use crate::{ExecutableId, JobId, JobQueueKind, RuntimeJob, RuntimeJobPayload};
use lyng_js_host::HostJobKind;
use lyng_js_types::RealmRef;

impl Agent {
    #[cfg(test)]
    pub(crate) const fn set_next_job_id_for_test(&mut self, next_job_id: u32) {
        self.next_job_id = next_job_id;
    }

    /// Enqueues one runtime job on this agent.
    ///
    /// # Panics
    /// Panics if the monotonic job id overflows the supported non-zero `u32` range.
    pub fn enqueue_job(
        &mut self,
        kind: HostJobKind,
        executable: ExecutableId,
        realm: Option<RealmRef>,
        debug_name: Option<String>,
    ) -> RuntimeJob {
        self.enqueue_job_with_payload(
            kind,
            executable,
            RuntimeJobPayload::Executable,
            realm,
            debug_name,
        )
    }

    pub fn enqueue_job_with_payload(
        &mut self,
        kind: HostJobKind,
        executable: ExecutableId,
        payload: RuntimeJobPayload,
        realm: Option<RealmRef>,
        debug_name: Option<String>,
    ) -> RuntimeJob {
        let raw_id = self.next_job_id.max(1);
        self.next_job_id = raw_id
            .checked_add(1)
            .expect("runtime job id overflowed supported u32 range");
        let id = JobId::from_raw(raw_id).expect("runtime job id must stay non-zero");
        let job = RuntimeJob {
            id,
            kind,
            executable,
            payload,
            realm,
            debug_name,
        };
        self.job_queues.enqueue(job.clone());
        job
    }

    pub fn dequeue_job(&mut self, kind: JobQueueKind) -> Option<RuntimeJob> {
        self.job_queues.dequeue(kind)
    }

    pub fn queued_jobs(&self, kind: JobQueueKind) -> Vec<RuntimeJob> {
        self.job_queues.snapshot(kind)
    }

    #[inline]
    pub fn queued_job_count(&self, kind: JobQueueKind) -> usize {
        self.job_queues.len(kind)
    }

    #[inline]
    pub fn total_queued_jobs(&self) -> usize {
        self.job_queues.total_len()
    }
}
