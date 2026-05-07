use super::{
    FinalizationRegistryState, ObjectRef, PrimitiveHeap, SymbolRef, Value, WeakHeapRef,
    WeakMapState, WeakRefState, WeakSetState,
};

impl PrimitiveHeap {
    #[inline]
    pub(crate) fn init_weak_map(&mut self, owner: ObjectRef) -> bool {
        self.objects.get(owner).is_some()
            && self.weak_maps.insert(owner, WeakMapState::new()).is_none()
    }

    #[inline]
    #[allow(
        clippy::option_option,
        reason = "outer None means the WeakMap object is missing; inner None means the key is absent"
    )]
    pub(crate) fn weak_map_get(&self, owner: ObjectRef, key: WeakHeapRef) -> Option<Option<Value>> {
        Some(self.weak_maps.get(&owner)?.get(key))
    }

    pub(crate) fn weak_map_set(
        &mut self,
        owner: ObjectRef,
        key: WeakHeapRef,
        value: Value,
    ) -> bool {
        let Some(state) = self.weak_maps.get_mut(&owner) else {
            return false;
        };
        state.set(key, value);
        true
    }

    pub(crate) fn weak_map_delete(&mut self, owner: ObjectRef, key: WeakHeapRef) -> Option<bool> {
        Some(self.weak_maps.get_mut(&owner)?.delete(key))
    }

    pub(crate) fn weak_map_snapshots(&self) -> Vec<(ObjectRef, Vec<(WeakHeapRef, Value)>)> {
        self.weak_maps
            .iter()
            .map(|(owner, state)| {
                (
                    *owner,
                    state
                        .entries()
                        .map(|entry| (entry.key(), entry.value()))
                        .collect(),
                )
            })
            .collect()
    }

    #[inline]
    pub(crate) fn init_weak_set(&mut self, owner: ObjectRef) -> bool {
        self.objects.get(owner).is_some()
            && self.weak_sets.insert(owner, WeakSetState::new()).is_none()
    }

    #[inline]
    pub(crate) fn weak_set_contains(&self, owner: ObjectRef, value: WeakHeapRef) -> Option<bool> {
        Some(self.weak_sets.get(&owner)?.contains(value))
    }

    pub(crate) fn weak_set_insert(&mut self, owner: ObjectRef, value: WeakHeapRef) -> bool {
        let Some(state) = self.weak_sets.get_mut(&owner) else {
            return false;
        };
        state.insert(value);
        true
    }

    pub(crate) fn weak_set_delete(&mut self, owner: ObjectRef, value: WeakHeapRef) -> Option<bool> {
        Some(self.weak_sets.get_mut(&owner)?.delete(value))
    }

    #[inline]
    pub(crate) fn init_weak_ref(&mut self, owner: ObjectRef, target: WeakHeapRef) -> bool {
        self.objects.get(owner).is_some()
            && self
                .weak_refs
                .insert(owner, WeakRefState::new(target))
                .is_none()
    }

    #[inline]
    #[allow(
        clippy::option_option,
        reason = "outer None means the WeakRef object is missing; inner None means its target was cleared"
    )]
    pub(crate) fn weak_ref_target(&self, owner: ObjectRef) -> Option<Option<WeakHeapRef>> {
        Some(self.weak_refs.get(&owner)?.target())
    }

    #[inline]
    pub(crate) fn init_finalization_registry(&mut self, owner: ObjectRef) -> bool {
        self.objects.get(owner).is_some()
            && self
                .finalization_registries
                .insert(owner, FinalizationRegistryState::new())
                .is_none()
    }

    pub(crate) fn finalization_registry_register(
        &mut self,
        owner: ObjectRef,
        target: WeakHeapRef,
        holdings: Value,
        unregister_token: Option<WeakHeapRef>,
    ) -> bool {
        let Some(state) = self.finalization_registries.get_mut(&owner) else {
            return false;
        };
        state.register(target, holdings, unregister_token);
        true
    }

    pub(crate) fn finalization_registry_unregister(
        &mut self,
        owner: ObjectRef,
        unregister_token: WeakHeapRef,
    ) -> Option<bool> {
        Some(
            self.finalization_registries
                .get_mut(&owner)?
                .unregister(unregister_token),
        )
    }

    pub(crate) fn finalization_registry_snapshots(
        &self,
    ) -> Vec<(ObjectRef, Vec<Value>, Vec<Value>)> {
        self.finalization_registries
            .iter()
            .map(|(owner, state)| {
                (
                    *owner,
                    state.cells().iter().map(|cell| cell.holdings()).collect(),
                    state.pending_holdings().to_vec(),
                )
            })
            .collect()
    }

    #[inline]
    pub(crate) fn pending_finalization_registries(&self) -> &[ObjectRef] {
        &self.pending_finalization_registries
    }

    pub(crate) fn take_finalization_cleanup_holdings(&mut self, owner: ObjectRef) -> Vec<Value> {
        let Some(state) = self.finalization_registries.get_mut(&owner) else {
            return Vec::new();
        };
        let holdings = state.take_pending_holdings();
        self.pending_finalization_registries
            .retain(|pending| *pending != owner);
        holdings
    }

    pub(crate) fn set_finalization_cleanup_active(
        &mut self,
        owner: ObjectRef,
        active: bool,
    ) -> bool {
        let Some(state) = self.finalization_registries.get_mut(&owner) else {
            return false;
        };
        state.set_cleanup_active(active);
        true
    }

    #[inline]
    pub(crate) fn finalization_cleanup_pending(&self, owner: ObjectRef) -> Option<bool> {
        Some(self.finalization_registries.get(&owner)?.cleanup_pending())
    }

    #[inline]
    pub(crate) fn is_object_marked(&self, id: ObjectRef) -> bool {
        self.objects.is_marked(id)
    }

    #[inline]
    pub(crate) fn is_symbol_marked(&self, id: SymbolRef) -> bool {
        self.symbols.is_marked(id)
    }

    #[inline]
    pub(crate) fn is_weak_ref_marked(&self, id: WeakHeapRef) -> bool {
        match id {
            WeakHeapRef::Object(object) => self.is_object_marked(object),
            WeakHeapRef::Symbol(symbol) => self.is_symbol_marked(symbol),
        }
    }

    pub(crate) fn sweep_weak_state(&mut self) -> (usize, usize, usize) {
        let mut weak_refs_cleared = 0;
        let mut finalization_cells_queued = 0;
        let objects = &self.objects;
        let symbols = &self.symbols;

        self.pending_finalization_registries.clear();
        self.weak_maps.retain(|owner, _| objects.is_marked(*owner));
        self.weak_sets.retain(|owner, _| objects.is_marked(*owner));
        self.weak_refs.retain(|owner, state| {
            if !objects.is_marked(*owner) {
                return false;
            }
            weak_refs_cleared += usize::from(state.clear_if_dead(|value| match value {
                WeakHeapRef::Object(object) => objects.is_marked(object),
                WeakHeapRef::Symbol(symbol) => symbols.is_marked(symbol),
            }));
            true
        });
        self.finalization_registries.retain(|owner, state| {
            if !objects.is_marked(*owner) {
                return false;
            }
            finalization_cells_queued += state.queue_dead_targets(|value| match value {
                WeakHeapRef::Object(object) => objects.is_marked(object),
                WeakHeapRef::Symbol(symbol) => symbols.is_marked(symbol),
            });
            if state.cleanup_pending() && !state.cleanup_active() {
                self.pending_finalization_registries.push(*owner);
            }
            true
        });
        for state in self.weak_maps.values_mut() {
            state.retain_live_keys(|key| match key {
                WeakHeapRef::Object(object) => objects.is_marked(object),
                WeakHeapRef::Symbol(symbol) => symbols.is_marked(symbol),
            });
        }
        for state in self.weak_sets.values_mut() {
            state.retain_live_values(|value| match value {
                WeakHeapRef::Object(object) => objects.is_marked(object),
                WeakHeapRef::Symbol(symbol) => symbols.is_marked(symbol),
            });
        }

        (
            weak_refs_cleared,
            finalization_cells_queued,
            self.pending_finalization_registries.len(),
        )
    }
}
