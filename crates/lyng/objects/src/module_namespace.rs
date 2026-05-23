use lyng_js_common::AtomId;
use lyng_js_types::{EnvironmentRef, PropertyKey, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModuleNamespaceExportTarget {
    Binding {
        environment: EnvironmentRef,
        slot: u32,
    },
    Value(Value),
}

impl ModuleNamespaceExportTarget {
    #[inline]
    pub const fn environment(self) -> Option<EnvironmentRef> {
        match self {
            Self::Binding { environment, .. } => Some(environment),
            Self::Value(_) => None,
        }
    }

    #[inline]
    pub const fn slot(self) -> Option<u32> {
        match self {
            Self::Binding { slot, .. } => Some(slot),
            Self::Value(_) => None,
        }
    }

    #[inline]
    pub const fn value(self) -> Option<Value> {
        match self {
            Self::Binding { .. } => None,
            Self::Value(value) => Some(value),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleNamespaceExport {
    export_name: AtomId,
    export_index: Option<u32>,
    target: ModuleNamespaceExportTarget,
}

impl ModuleNamespaceExport {
    #[inline]
    pub const fn new(export_name: AtomId, target: ModuleNamespaceExportTarget) -> Self {
        Self {
            export_name,
            export_index: None,
            target,
        }
    }

    #[inline]
    pub const fn with_array_index(mut self, export_index: Option<u32>) -> Self {
        self.export_index = export_index;
        self
    }

    #[inline]
    pub const fn export_name(self) -> AtomId {
        self.export_name
    }

    #[inline]
    pub const fn export_key(self) -> PropertyKey {
        match self.export_index {
            Some(index) => PropertyKey::Index(index),
            None => PropertyKey::Atom(self.export_name),
        }
    }

    #[inline]
    pub fn matches_key(self, key: PropertyKey) -> bool {
        match key {
            PropertyKey::Index(index) => {
                matches!(self.export_index, Some(export_index) if export_index == index)
            }
            PropertyKey::Atom(atom) => self.export_name == atom,
            PropertyKey::Symbol(_) => false,
        }
    }

    #[inline]
    pub const fn target(self) -> ModuleNamespaceExportTarget {
        self.target
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleNamespaceObject {
    exports: Vec<ModuleNamespaceExport>,
}

impl ModuleNamespaceObject {
    #[inline]
    pub(crate) const fn new(exports: Vec<ModuleNamespaceExport>) -> Self {
        Self { exports }
    }

    #[inline]
    pub(crate) fn exports(&self) -> &[ModuleNamespaceExport] {
        &self.exports
    }

    #[inline]
    pub(crate) fn export_for_key(&self, key: PropertyKey) -> Option<ModuleNamespaceExport> {
        self.exports
            .iter()
            .copied()
            .find(|entry| entry.matches_key(key))
    }
}
