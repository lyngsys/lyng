use std::collections::HashMap;

use lyng_js_types::{EnvironmentRef, ObjectRef};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MappedArgumentsObject {
    environment: EnvironmentRef,
    mapped_slots: Vec<Option<u32>>,
}

impl MappedArgumentsObject {
    #[inline]
    pub(crate) fn new(environment: EnvironmentRef, mapped_slots: Vec<Option<u32>>) -> Self {
        Self {
            environment,
            mapped_slots,
        }
    }

    #[inline]
    pub(crate) const fn environment(&self) -> EnvironmentRef {
        self.environment
    }

    #[inline]
    pub(crate) fn slot_for_index(&self, index: u32) -> Option<u32> {
        self.mapped_slots
            .get(usize::try_from(index).ok()?)
            .copied()
            .flatten()
    }

    #[inline]
    pub(crate) fn mapped_indices(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        self.mapped_slots
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| {
                let slot = (*slot)?;
                Some((u32::try_from(index).ok()?, slot))
            })
    }

    #[inline]
    pub(crate) fn detach_index(&mut self, index: u32) -> bool {
        let Some(slot) = self
            .mapped_slots
            .get_mut(usize::try_from(index).ok().unwrap_or(usize::MAX))
        else {
            return false;
        };
        let detached = slot.is_some();
        *slot = None;
        detached
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ActivationSideTables {
    mapped_arguments: HashMap<ObjectRef, MappedArgumentsObject>,
}

impl ActivationSideTables {
    #[inline]
    pub(crate) fn track_mapped_arguments(
        &mut self,
        object: ObjectRef,
        mapped: MappedArgumentsObject,
    ) {
        self.mapped_arguments.insert(object, mapped);
    }

    #[inline]
    pub(crate) fn mapped_argument_slot(
        &self,
        object: ObjectRef,
        index: u32,
    ) -> Option<(EnvironmentRef, u32)> {
        let mapped = self.mapped_arguments.get(&object)?;
        Some((mapped.environment(), mapped.slot_for_index(index)?))
    }

    #[inline]
    pub(crate) fn detach_mapped_argument(&mut self, object: ObjectRef, index: u32) -> bool {
        self.mapped_arguments
            .get_mut(&object)
            .is_some_and(|mapped| mapped.detach_index(index))
    }

    pub(crate) fn drain_mapped_arguments_for_environment(
        &mut self,
        environment: EnvironmentRef,
    ) -> Vec<(ObjectRef, MappedArgumentsObject)> {
        let drained = self
            .mapped_arguments
            .iter()
            .filter_map(|(object, mapped)| {
                (mapped.environment() == environment).then_some((*object, mapped.clone()))
            })
            .collect::<Vec<_>>();
        for (object, _) in &drained {
            self.mapped_arguments.remove(object);
        }
        drained
    }
}
