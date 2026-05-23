use super::{EnvironmentLayoutId, ThisBindingStatus};
use lyng_js_common::AtomId;
use lyng_js_gc::EnvironmentSlotsRef;
use lyng_js_types::{EnvironmentRef, ObjectRef, Value};
use std::collections::HashSet;

/// Read-only typed view over one declarative environment record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeclarativeEnvironmentRecord {
    pub(crate) id: EnvironmentRef,
    pub(crate) outer: Option<EnvironmentRef>,
    pub(crate) layout: EnvironmentLayoutId,
    pub(crate) slots: Option<EnvironmentSlotsRef>,
}

impl DeclarativeEnvironmentRecord {
    #[inline]
    pub const fn id(self) -> EnvironmentRef {
        self.id
    }

    #[inline]
    pub const fn outer(self) -> Option<EnvironmentRef> {
        self.outer
    }

    #[inline]
    pub const fn layout(self) -> EnvironmentLayoutId {
        self.layout
    }

    #[inline]
    pub const fn slots(self) -> Option<EnvironmentSlotsRef> {
        self.slots
    }
}

/// Read-only typed view over one private environment record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrivateEnvironmentRecord {
    pub(crate) id: EnvironmentRef,
    pub(crate) outer: Option<EnvironmentRef>,
    pub(crate) layout: EnvironmentLayoutId,
    pub(crate) slots: Option<EnvironmentSlotsRef>,
}

impl PrivateEnvironmentRecord {
    #[inline]
    pub const fn id(self) -> EnvironmentRef {
        self.id
    }

    #[inline]
    pub const fn outer(self) -> Option<EnvironmentRef> {
        self.outer
    }

    #[inline]
    pub const fn layout(self) -> EnvironmentLayoutId {
        self.layout
    }

    #[inline]
    pub const fn slots(self) -> Option<EnvironmentSlotsRef> {
        self.slots
    }
}

/// Read-only typed view over one function environment record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FunctionEnvironmentRecord {
    pub(crate) declarative: DeclarativeEnvironmentRecord,
    pub(crate) function_object: ObjectRef,
    pub(crate) this_binding_status: ThisBindingStatus,
    pub(crate) this_value: Value,
    pub(crate) new_target: Option<ObjectRef>,
    pub(crate) home_object: Option<ObjectRef>,
}

impl FunctionEnvironmentRecord {
    #[inline]
    pub const fn declarative(self) -> DeclarativeEnvironmentRecord {
        self.declarative
    }

    #[inline]
    pub const fn function_object(self) -> ObjectRef {
        self.function_object
    }

    #[inline]
    pub const fn this_binding_status(self) -> ThisBindingStatus {
        self.this_binding_status
    }

    #[inline]
    pub const fn this_value(self) -> Value {
        self.this_value
    }

    #[inline]
    pub const fn new_target(self) -> Option<ObjectRef> {
        self.new_target
    }

    #[inline]
    pub const fn home_object(self) -> Option<ObjectRef> {
        self.home_object
    }
}

/// Read-only typed view over one module environment record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleBindingAlias {
    pub(crate) environment: EnvironmentRef,
    pub(crate) slot: u32,
}

impl ModuleBindingAlias {
    #[inline]
    pub const fn new(environment: EnvironmentRef, slot: u32) -> Self {
        Self { environment, slot }
    }

    #[inline]
    pub const fn environment(self) -> EnvironmentRef {
        self.environment
    }

    #[inline]
    pub const fn slot(self) -> u32 {
        self.slot
    }
}

/// Read-only typed view over one module environment record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleEnvironmentRecord {
    pub(crate) declarative: DeclarativeEnvironmentRecord,
}

impl ModuleEnvironmentRecord {
    #[inline]
    pub const fn declarative(self) -> DeclarativeEnvironmentRecord {
        self.declarative
    }

    #[inline]
    pub const fn id(self) -> EnvironmentRef {
        self.declarative.id()
    }

    #[inline]
    pub const fn outer(self) -> Option<EnvironmentRef> {
        self.declarative.outer()
    }

    #[inline]
    pub const fn layout(self) -> EnvironmentLayoutId {
        self.declarative.layout()
    }

    #[inline]
    pub const fn slots(self) -> Option<EnvironmentSlotsRef> {
        self.declarative.slots()
    }
}

/// Read-only typed view over one global environment record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlobalEnvironmentRecord {
    pub(crate) id: EnvironmentRef,
    pub(crate) outer: Option<EnvironmentRef>,
    pub(crate) layout: EnvironmentLayoutId,
    pub(crate) lexical_slots: Option<EnvironmentSlotsRef>,
    pub(crate) global_object: ObjectRef,
    pub(crate) lexical_names: HashSet<AtomId>,
    pub(crate) lexical_bindings: Vec<GlobalLexicalBindingRecord>,
    pub(crate) var_names: HashSet<AtomId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlobalLexicalBindingRecord {
    pub(crate) name: AtomId,
    pub(crate) environment: EnvironmentRef,
    pub(crate) slot: u32,
}

impl GlobalLexicalBindingRecord {
    #[inline]
    pub const fn new(name: AtomId, environment: EnvironmentRef, slot: u32) -> Self {
        Self {
            name,
            environment,
            slot,
        }
    }

    #[inline]
    pub const fn name(self) -> AtomId {
        self.name
    }

    #[inline]
    pub const fn environment(self) -> EnvironmentRef {
        self.environment
    }

    #[inline]
    pub const fn slot(self) -> u32 {
        self.slot
    }
}

impl GlobalEnvironmentRecord {
    #[inline]
    pub const fn id(&self) -> EnvironmentRef {
        self.id
    }

    #[inline]
    pub const fn outer(&self) -> Option<EnvironmentRef> {
        self.outer
    }

    #[inline]
    pub const fn layout(&self) -> EnvironmentLayoutId {
        self.layout
    }

    #[inline]
    pub const fn lexical_slots(&self) -> Option<EnvironmentSlotsRef> {
        self.lexical_slots
    }

    #[inline]
    pub const fn global_object(&self) -> ObjectRef {
        self.global_object
    }

    #[inline]
    pub const fn var_names(&self) -> &HashSet<AtomId> {
        &self.var_names
    }

    #[inline]
    pub const fn lexical_names(&self) -> &HashSet<AtomId> {
        &self.lexical_names
    }

    #[inline]
    pub fn lexical_bindings(&self) -> &[GlobalLexicalBindingRecord] {
        &self.lexical_bindings
    }

    #[inline]
    pub fn has_lexical_name(&self, name: AtomId) -> bool {
        self.lexical_names.contains(&name)
    }

    #[inline]
    pub fn lexical_binding(&self, name: AtomId) -> Option<GlobalLexicalBindingRecord> {
        self.lexical_bindings
            .iter()
            .copied()
            .find(|binding| binding.name() == name)
    }

    #[inline]
    pub fn has_var_name(&self, name: AtomId) -> bool {
        self.var_names.contains(&name)
    }
}

/// Read-only typed view over one slow-path object environment record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectEnvironmentRecord {
    pub(crate) id: EnvironmentRef,
    pub(crate) outer: Option<EnvironmentRef>,
    pub(crate) binding_object: ObjectRef,
    pub(crate) with_environment: bool,
}

impl ObjectEnvironmentRecord {
    #[inline]
    pub const fn id(self) -> EnvironmentRef {
        self.id
    }

    #[inline]
    pub const fn outer(self) -> Option<EnvironmentRef> {
        self.outer
    }

    #[inline]
    pub const fn binding_object(self) -> ObjectRef {
        self.binding_object
    }

    #[inline]
    pub const fn with_environment(self) -> bool {
        self.with_environment
    }
}

/// Public typed read model for runtime environment families.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EnvironmentRecord {
    Declarative(DeclarativeEnvironmentRecord),
    Private(PrivateEnvironmentRecord),
    Function(FunctionEnvironmentRecord),
    Module(ModuleEnvironmentRecord),
    Global(GlobalEnvironmentRecord),
    Object(ObjectEnvironmentRecord),
}

impl EnvironmentRecord {
    #[inline]
    pub const fn id(&self) -> EnvironmentRef {
        match self {
            Self::Declarative(record) => record.id(),
            Self::Private(record) => record.id(),
            Self::Function(record) => record.declarative().id(),
            Self::Module(record) => record.id(),
            Self::Global(record) => record.id(),
            Self::Object(record) => record.id(),
        }
    }

    #[inline]
    pub const fn outer(&self) -> Option<EnvironmentRef> {
        match self {
            Self::Declarative(record) => record.outer(),
            Self::Private(record) => record.outer(),
            Self::Function(record) => record.declarative().outer(),
            Self::Module(record) => record.outer(),
            Self::Global(record) => record.outer(),
            Self::Object(record) => record.outer(),
        }
    }

    #[inline]
    pub const fn layout(&self) -> Option<EnvironmentLayoutId> {
        match self {
            Self::Declarative(record) => Some(record.layout()),
            Self::Private(record) => Some(record.layout()),
            Self::Function(record) => Some(record.declarative().layout()),
            Self::Module(record) => Some(record.layout()),
            Self::Global(record) => Some(record.layout()),
            Self::Object(_) => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EnvironmentMetadata {
    Declarative {
        layout: EnvironmentLayoutId,
    },
    Private {
        layout: EnvironmentLayoutId,
    },
    Function {
        layout: EnvironmentLayoutId,
        this_binding_status: ThisBindingStatus,
    },
    Module {
        layout: EnvironmentLayoutId,
        import_aliases: Vec<Option<ModuleBindingAlias>>,
    },
    Global {
        layout: EnvironmentLayoutId,
        lexical_names: HashSet<AtomId>,
        lexical_bindings: Vec<GlobalLexicalBindingRecord>,
        var_names: HashSet<AtomId>,
    },
    Object {
        with_environment: bool,
    },
}
