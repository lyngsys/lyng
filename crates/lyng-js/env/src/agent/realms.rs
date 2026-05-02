use super::Agent;
use crate::{
    realm_index, AllocationLifetime, EnvironmentLayout, EnvironmentLayoutKind, Intrinsics,
    RealmBootstrapState, RealmMetadata, RealmRecord, RegExpLegacyStaticState, RuntimeRealmRecord,
};
use lyng_js_objects::ObjectAllocation;
use lyng_js_types::{ObjectRef, RealmRef, ShapeId};

impl Agent {
    #[inline]
    pub fn realm_refs(&self) -> &[RealmRef] {
        &self.realms
    }

    #[inline]
    pub const fn default_realm_id(&self) -> Option<RealmRef> {
        self.default_realm
    }

    pub fn default_realm(&self) -> Option<RealmRecord> {
        self.default_realm.and_then(|realm| self.realm(realm))
    }

    /// Allocates the default realm shell used to bootstrap the runtime.
    ///
    /// # Panics
    /// Panics if the bootstrap global environment allocation fails unexpectedly.
    pub fn create_default_realm_shell(&mut self, lifetime: AllocationLifetime) -> RealmRef {
        let global_layout = self.alloc_environment_layout(EnvironmentLayout::empty(
            EnvironmentLayoutKind::Global,
            true,
        ));
        let (global_object, root_shape) = self.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            let root_shape = objects.root_shape(&mut mutator, None, lifetime);
            let global_object = objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape),
                lifetime,
            );
            (global_object, root_shape)
        });
        let global_env = self
            .alloc_global_environment(None, global_layout, global_object, lifetime)
            .expect("default realm global environment should allocate");
        let realm = self.heap.mutator().alloc_realm(
            RuntimeRealmRecord::new(
                Some(global_object),
                Some(global_env),
                None,
                Some(root_shape),
            ),
            lifetime,
        );

        self.store_realm_metadata(
            realm,
            RealmMetadata {
                intrinsics: Intrinsics::new(),
                bootstrap_state: RealmBootstrapState::new(),
                is_default: self.default_realm.is_none(),
                regexp_legacy_static_state: RegExpLegacyStaticState::default(),
            },
        );
        if !self.realms.contains(&realm) {
            self.realms.push(realm);
        }
        if self.default_realm.is_none() {
            self.default_realm = Some(realm);
        }
        debug_assert_eq!(
            self.realm(realm),
            Some(RealmRecord {
                id: realm,
                global_object,
                global_env,
                bootstrap_code: None,
                root_shape: Some(root_shape),
                intrinsics: Intrinsics::new(),
                bootstrap_state: RealmBootstrapState::new(),
                is_default: self.default_realm == Some(realm),
            })
        );
        realm
    }

    pub fn realm(&self, realm: RealmRef) -> Option<RealmRecord> {
        let record = self.heap.view().realm(realm)?;
        let metadata = self.realm_metadata(realm)?;
        Some(RealmRecord {
            id: realm,
            global_object: record.global_object()?,
            global_env: record.global_env()?,
            bootstrap_code: record.bootstrap_code(),
            root_shape: record.root_shape(),
            intrinsics: metadata.intrinsics,
            bootstrap_state: metadata.bootstrap_state,
            is_default: metadata.is_default,
        })
    }

    pub fn realm_intrinsics(&self, realm: RealmRef) -> Option<&Intrinsics> {
        Some(&self.realm_metadata(realm)?.intrinsics)
    }

    pub fn realm_global_object(&self, realm: RealmRef) -> Option<ObjectRef> {
        self.heap.view().realm(realm)?.global_object()
    }

    pub fn realm_root_shape(&self, realm: RealmRef) -> Option<ShapeId> {
        self.heap.view().realm(realm)?.root_shape()
    }

    pub fn set_realm_intrinsics(&mut self, realm: RealmRef, intrinsics: Intrinsics) -> bool {
        let Some(metadata) = self.realm_metadata_mut(realm) else {
            return false;
        };
        metadata.intrinsics = intrinsics;
        true
    }

    pub fn realm_bootstrap_state(&self, realm: RealmRef) -> Option<RealmBootstrapState> {
        Some(self.realm_metadata(realm)?.bootstrap_state)
    }

    pub fn set_realm_bootstrap_state(
        &mut self,
        realm: RealmRef,
        bootstrap_state: RealmBootstrapState,
    ) -> bool {
        let Some(metadata) = self.realm_metadata_mut(realm) else {
            return false;
        };
        metadata.bootstrap_state = bootstrap_state;
        true
    }

    pub fn mark_realm_spec_bootstrapped(&mut self, realm: RealmRef) -> bool {
        let Some(metadata) = self.realm_metadata_mut(realm) else {
            return false;
        };
        metadata.bootstrap_state = metadata.bootstrap_state.with_spec_ready(true);
        true
    }

    pub fn mark_realm_embedding_bootstrapped(&mut self, realm: RealmRef) -> bool {
        let Some(metadata) = self.realm_metadata_mut(realm) else {
            return false;
        };
        metadata.bootstrap_state = metadata
            .bootstrap_state
            .with_spec_ready(true)
            .with_embedding_ready(true);
        true
    }

    pub fn regexp_legacy_static_state(&self, realm: RealmRef) -> Option<&RegExpLegacyStaticState> {
        Some(&self.realm_metadata(realm)?.regexp_legacy_static_state)
    }

    pub fn regexp_legacy_static_state_mut(
        &mut self,
        realm: RealmRef,
    ) -> Option<&mut RegExpLegacyStaticState> {
        Some(&mut self.realm_metadata_mut(realm)?.regexp_legacy_static_state)
    }

    fn store_realm_metadata(&mut self, realm: RealmRef, metadata: RealmMetadata) {
        let index = realm_index(realm);
        if self.realm_metadata.len() <= index {
            self.realm_metadata.resize_with(index + 1, || None);
        }
        self.realm_metadata[index] = Some(metadata);
    }

    fn realm_metadata(&self, realm: RealmRef) -> Option<&RealmMetadata> {
        self.realm_metadata.get(realm_index(realm))?.as_ref()
    }

    fn realm_metadata_mut(&mut self, realm: RealmRef) -> Option<&mut RealmMetadata> {
        self.realm_metadata.get_mut(realm_index(realm))?.as_mut()
    }
}
