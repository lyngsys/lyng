use std::collections::HashMap;

use lyng_js_common::AtomId;
use lyng_js_env::ObjectEnvironmentRecord;
use lyng_js_types::EnvironmentRef;

use crate::RegisterWindow;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapturedNameTarget {
    EnvironmentSlot {
        environment: EnvironmentRef,
        slot: u32,
    },
    ObjectProperty {
        record: ObjectEnvironmentRecord,
    },
    GlobalProperty {
        environment: EnvironmentRef,
    },
    Unresolvable {
        global_environment: EnvironmentRef,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CapturedNameReference {
    name: AtomId,
    target: CapturedNameTarget,
}

impl CapturedNameReference {
    #[inline]
    pub(crate) const fn new(name: AtomId, target: CapturedNameTarget) -> Self {
        Self { name, target }
    }

    #[inline]
    pub(crate) const fn name(self) -> AtomId {
        self.name
    }

    #[inline]
    pub(crate) const fn target(self) -> CapturedNameTarget {
        self.target
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CapturedNameReferenceTable {
    states: HashMap<u32, CapturedNameReference>,
}

impl CapturedNameReferenceTable {
    #[inline]
    pub(crate) fn insert(
        &mut self,
        register_base: u32,
        reference_register: u16,
        reference: CapturedNameReference,
    ) {
        self.states
            .insert(slot_key(register_base, reference_register), reference);
    }

    #[inline]
    pub(crate) fn get(
        &self,
        register_base: u32,
        reference_register: u16,
    ) -> Option<CapturedNameReference> {
        self.states
            .get(&slot_key(register_base, reference_register))
            .copied()
    }

    pub(crate) fn clear_window(&mut self, window: RegisterWindow) {
        let start = window.base();
        let end = window.end();
        self.states
            .retain(|register, _| *register < start || *register >= end);
    }

    pub(crate) fn drain_window(
        &mut self,
        window: RegisterWindow,
    ) -> Vec<(u16, CapturedNameReference)> {
        let start = window.base();
        let end = window.end();
        let mut drained = Vec::new();
        self.states.retain(|register, reference| {
            let keep = *register < start || *register >= end;
            if !keep {
                drained.push((
                    u16::try_from(register.saturating_sub(start))
                        .expect("captured name register offset should fit into u16"),
                    *reference,
                ));
            }
            keep
        });
        drained
    }

    pub(crate) fn restore_window(
        &mut self,
        window: RegisterWindow,
        states: Vec<(u16, CapturedNameReference)>,
    ) {
        for (register, reference) in states {
            self.insert(window.base(), register, reference);
        }
    }
}

#[inline]
fn slot_key(register_base: u32, reference_register: u16) -> u32 {
    register_base + u32::from(reference_register)
}
