use crate::errors::internal_method_error;
use lyng_js_env::Agent;
use lyng_js_types::{Completion, ObjectRef, PropertyKey};
use std::collections::HashSet;

/// Baseline ordinary-object `for-in` state for Phase 4 script execution.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ForInEnumerator {
    keys: Vec<EnumeratedKey>,
    next_index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct EnumeratedKey {
    owner: ObjectRef,
    key: PropertyKey,
}

impl ForInEnumerator {
    #[inline]
    pub fn new(keys: Vec<(ObjectRef, PropertyKey)>) -> Self {
        Self {
            keys: keys
                .into_iter()
                .map(|(owner, key)| EnumeratedKey { owner, key })
                .collect(),
            next_index: 0,
        }
    }

    #[inline]
    pub fn keys(&self) -> Vec<PropertyKey> {
        self.keys.iter().map(|entry| entry.key).collect()
    }

    #[inline]
    pub fn is_done(&self) -> bool {
        self.next_index >= self.keys.len()
    }

    #[inline]
    pub fn remaining(&self) -> usize {
        self.keys.len().saturating_sub(self.next_index)
    }

    pub fn next_key(&mut self, agent: &mut Agent) -> Completion<Option<PropertyKey>> {
        while let Some(entry) = self.keys.get(self.next_index).copied() {
            self.next_index += 1;
            if agent.objects().is_proxy_object(entry.owner) {
                return Ok(Some(entry.key));
            }
            let descriptor = crate::object::get_own_property(agent, entry.owner, entry.key)?;
            if descriptor.is_some_and(|descriptor| descriptor.enumerable() == Some(true)) {
                return Ok(Some(entry.key));
            }
        }
        Ok(None)
    }
}

/// Collects the baseline ordinary-object `for-in` key order for one object and
/// returns a resumable enumerator.
///
/// # Errors
/// Returns an abrupt completion when key collection, descriptor lookup, or
/// prototype traversal fails through the object internal-method surface.
pub fn create_for_in_enumerator(
    agent: &mut Agent,
    object: ObjectRef,
) -> Completion<ForInEnumerator> {
    let mut visited = HashSet::new();
    let mut keys = Vec::new();
    let mut current = Some(object);

    while let Some(object) = current {
        let own_keys = agent
            .objects()
            .own_property_keys(agent.heap().view(), object)
            .map_err(|error| internal_method_error(agent, error))?;

        for key in own_keys {
            if key.is_symbol() || !visited.insert(key) {
                continue;
            }

            let descriptor = crate::object::get_own_property(agent, object, key)?;
            if descriptor.is_some_and(|descriptor| descriptor.enumerable() == Some(true)) {
                keys.push((object, key));
            }
        }

        current = agent
            .objects()
            .get_prototype_of(agent.heap().view(), object)
            .map_err(|error| internal_method_error(agent, error))?;
    }

    Ok(ForInEnumerator::new(keys))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lyng_js_common::AtomId;
    use lyng_js_env::Runtime;
    use lyng_js_gc::AllocationLifetime;
    use lyng_js_host::NoopHostHooks;
    use lyng_js_objects::ObjectAllocation;
    use lyng_js_types::{PropertyDescriptor, SymbolRef, Value};

    fn data_descriptor(value: Value, enumerable: bool) -> PropertyDescriptor {
        let mut descriptor = PropertyDescriptor::new();
        descriptor.set_value(value);
        descriptor.set_writable(true);
        descriptor.set_enumerable(enumerable);
        descriptor.set_configurable(true);
        descriptor
    }

    #[test]
    fn for_in_enumerator_skips_symbols_and_suppresses_shadowed_prototype_keys() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let default_realm = agent.default_realm().expect("default realm should exist");
        let root_shape = default_realm
            .root_shape()
            .expect("default realm should expose a root shape");

        let own_key = PropertyKey::from_atom(AtomId::from_raw(701));
        let shadowed_key = PropertyKey::from_atom(AtomId::from_raw(702));
        let index_key = PropertyKey::Index(2);
        let symbol_key = PropertyKey::from_symbol(SymbolRef::from_raw(17).unwrap());
        let prototype = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            let prototype = objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape),
                AllocationLifetime::Default,
            );
            assert!(objects
                .define_own_property(
                    &mut mutator,
                    prototype,
                    shadowed_key,
                    data_descriptor(Value::from_smi(1), true),
                    AllocationLifetime::Default,
                )
                .unwrap());
            prototype
        });

        let object = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            let object = objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape).with_prototype(Some(prototype)),
                AllocationLifetime::Default,
            );
            assert!(objects
                .define_own_property(
                    &mut mutator,
                    object,
                    own_key,
                    data_descriptor(Value::from_smi(2), true),
                    AllocationLifetime::Default,
                )
                .unwrap());
            assert!(objects
                .define_own_property(
                    &mut mutator,
                    object,
                    shadowed_key,
                    data_descriptor(Value::from_smi(3), false),
                    AllocationLifetime::Default,
                )
                .unwrap());
            assert!(objects
                .define_own_property(
                    &mut mutator,
                    object,
                    index_key,
                    data_descriptor(Value::from_smi(4), true),
                    AllocationLifetime::Default,
                )
                .unwrap());
            assert!(objects
                .define_own_property(
                    &mut mutator,
                    object,
                    symbol_key,
                    data_descriptor(Value::from_smi(5), true),
                    AllocationLifetime::Default,
                )
                .unwrap());
            object
        });

        let mut enumerator = create_for_in_enumerator(agent, object).unwrap();

        assert_eq!(enumerator.keys(), vec![index_key, own_key]);
        assert_eq!(enumerator.remaining(), 2);
        assert_eq!(enumerator.next_key(agent).unwrap(), Some(index_key));
        assert_eq!(enumerator.next_key(agent).unwrap(), Some(own_key));
        assert!(enumerator.is_done());
        assert_eq!(enumerator.next_key(agent).unwrap(), None);
    }
}
