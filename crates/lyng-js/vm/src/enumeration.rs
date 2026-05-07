use std::collections::HashMap;

use lyng_js_env::Agent;
use lyng_js_ops::enumeration::ForInEnumerator;
use lyng_js_ops::iterator::IteratorRecord;
use lyng_js_types::PropertyKey;

use crate::error::VmResult;
use crate::{RegisterWindow, VmError};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ForInStateTable {
    states: HashMap<u32, ForInEnumerator>,
}

impl ForInStateTable {
    #[inline]
    pub(crate) fn insert(
        &mut self,
        register_base: u32,
        enumerator_register: u16,
        enumerator: ForInEnumerator,
    ) {
        self.states
            .insert(slot_key(register_base, enumerator_register), enumerator);
    }

    #[inline]
    pub(crate) fn advance(
        &mut self,
        agent: &mut Agent,
        register_base: u32,
        enumerator_register: u16,
    ) -> VmResult<Option<PropertyKey>> {
        let Some(enumerator) = self
            .states
            .get_mut(&slot_key(register_base, enumerator_register))
        else {
            return Ok(None);
        };
        enumerator.next_key(agent).map_err(VmError::Abrupt)
    }

    #[inline]
    pub(crate) fn remove(
        &mut self,
        register_base: u32,
        enumerator_register: u16,
    ) -> Option<ForInEnumerator> {
        self.states
            .remove(&slot_key(register_base, enumerator_register))
    }

    pub(crate) fn clear_window(&mut self, window: RegisterWindow) {
        let start = window.base();
        let end = window.end();
        self.states
            .retain(|register, _| *register < start || *register >= end);
    }

    pub(crate) fn drain_window(&mut self, window: RegisterWindow) -> Vec<(u16, ForInEnumerator)> {
        let start = window.base();
        let end = window.end();
        let mut drained = Vec::new();
        self.states.retain(|register, enumerator| {
            let keep = *register < start || *register >= end;
            if !keep {
                drained.push((
                    u16::try_from(register.saturating_sub(start))
                        .expect("for-in register offset should fit into u16"),
                    enumerator.clone(),
                ));
            }
            keep
        });
        drained
    }

    pub(crate) fn restore_window(
        &mut self,
        window: RegisterWindow,
        states: Vec<(u16, ForInEnumerator)>,
    ) {
        for (register, enumerator) in states {
            self.insert(window.base(), register, enumerator);
        }
    }

    #[cfg(test)]
    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.states.len()
    }
}

#[inline]
fn slot_key(register_base: u32, enumerator_register: u16) -> u32 {
    register_base + u32::from(enumerator_register)
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IteratorStateTable {
    states: HashMap<u32, IteratorRecord>,
}

impl IteratorStateTable {
    #[inline]
    pub(crate) fn insert(
        &mut self,
        register_base: u32,
        iterator_register: u16,
        iterator: IteratorRecord,
    ) {
        self.states
            .insert(slot_key(register_base, iterator_register), iterator);
    }

    #[inline]
    pub(crate) fn remove(
        &mut self,
        register_base: u32,
        iterator_register: u16,
    ) -> Option<IteratorRecord> {
        self.states
            .remove(&slot_key(register_base, iterator_register))
    }

    pub(crate) fn clear_window(&mut self, window: RegisterWindow) {
        let start = window.base();
        let end = window.end();
        self.states
            .retain(|register, _| *register < start || *register >= end);
    }

    pub(crate) fn drain_window(&mut self, window: RegisterWindow) -> Vec<(u16, IteratorRecord)> {
        let start = window.base();
        let end = window.end();
        let mut drained = Vec::new();
        self.states.retain(|register, iterator| {
            let keep = *register < start || *register >= end;
            if !keep {
                drained.push((
                    u16::try_from(register.saturating_sub(start))
                        .expect("iterator register offset should fit into u16"),
                    *iterator,
                ));
            }
            keep
        });
        drained
    }

    pub(crate) fn restore_window(
        &mut self,
        window: RegisterWindow,
        states: Vec<(u16, IteratorRecord)>,
    ) {
        for (register, iterator) in states {
            self.insert(window.base(), register, iterator);
        }
    }
}
