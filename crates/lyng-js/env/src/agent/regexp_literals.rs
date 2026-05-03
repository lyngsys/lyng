use super::{Agent, RegExpLiteralCacheKey};
use crate::RuntimeDomainAccounting;
use lyng_js_objects::RegExpPayload;
use lyng_js_types::{CodeRef, RealmRef};
use std::collections::hash_map::Entry;
use std::mem::size_of;

impl Agent {
    pub fn regexp_literal_cached_payload(
        &self,
        realm: RealmRef,
        code: CodeRef,
        site: u32,
    ) -> Option<&RegExpPayload> {
        self.regexp_literal_cache
            .get(&RegExpLiteralCacheKey::new(realm, code, site))
    }

    pub fn cache_regexp_literal_payload(
        &mut self,
        realm: RealmRef,
        code: CodeRef,
        site: u32,
        payload: RegExpPayload,
    ) -> bool {
        match self
            .regexp_literal_cache
            .entry(RegExpLiteralCacheKey::new(realm, code, site))
        {
            Entry::Occupied(_) => false,
            Entry::Vacant(entry) => {
                entry.insert(payload);
                true
            }
        }
    }

    pub(crate) fn regexp_literal_cache_accounting(&self) -> RuntimeDomainAccounting {
        let records = self.regexp_literal_cache.len();
        let metadata_bytes =
            records * (size_of::<RegExpLiteralCacheKey>() + size_of::<RegExpPayload>());
        let payload_bytes = self
            .regexp_literal_cache
            .values()
            .map(RegExpPayload::payload_bytes)
            .sum();
        RuntimeDomainAccounting {
            records,
            metadata_bytes,
            payload_bytes,
            live_bytes: metadata_bytes + payload_bytes,
        }
    }
}
