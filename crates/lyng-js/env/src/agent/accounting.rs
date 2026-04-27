use super::Agent;
use crate::{total_live_bytes, AgentPhase6Accounting, RuntimeDomainAccounting};

impl Agent {
    pub fn phase6_accounting(&self) -> AgentPhase6Accounting {
        let heap = self.heap.accounting();
        let iterator_records = Default::default();
        let regexp_payloads = {
            let accounting = self.objects.regexp_payload_accounting(self.heap.view());
            RuntimeDomainAccounting {
                records: accounting.records,
                metadata_bytes: accounting.metadata_bytes,
                payload_bytes: accounting.payload_bytes,
                live_bytes: accounting.live_bytes,
            }
        };
        let module_caches = Default::default();
        let promise_jobs = self.job_queues.promise_job_accounting();
        AgentPhase6Accounting {
            heap,
            iterator_records,
            regexp_payloads,
            module_caches,
            promise_jobs,
            live_bytes: total_live_bytes(
                heap,
                iterator_records,
                regexp_payloads,
                module_caches,
                promise_jobs,
                Default::default(),
            ),
        }
    }
}
