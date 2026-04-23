use lyng_js_common::AtomId;
use lyng_js_types::{EnvironmentRef, Value};

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
    target: ModuleNamespaceExportTarget,
}

impl ModuleNamespaceExport {
    #[inline]
    pub const fn new(export_name: AtomId, target: ModuleNamespaceExportTarget) -> Self {
        Self {
            export_name,
            target,
        }
    }

    #[inline]
    pub const fn export_name(self) -> AtomId {
        self.export_name
    }

    #[inline]
    pub const fn target(self) -> ModuleNamespaceExportTarget {
        self.target
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ModuleNamespaceObject {
    exports: Vec<ModuleNamespaceExport>,
}

impl ModuleNamespaceObject {
    #[inline]
    pub(crate) fn new(exports: Vec<ModuleNamespaceExport>) -> Self {
        Self { exports }
    }

    #[inline]
    pub(crate) fn exports(&self) -> &[ModuleNamespaceExport] {
        &self.exports
    }

    #[inline]
    pub(crate) fn export(&self, export_name: AtomId) -> Option<ModuleNamespaceExport> {
        self.exports
            .iter()
            .copied()
            .find(|entry| entry.export_name() == export_name)
    }
}
