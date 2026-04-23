use super::template::BytecodeFunction;
use crate::ids::BytecodeFunctionId;
use lyng_js_common::{AtomId, SourceId};

/// Frozen compiled-unit atom payload preserved across compiler -> VM install.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompiledAtom {
    Utf8(Box<str>),
    Utf16(Box<[u16]>),
}

impl CompiledAtom {
    #[inline]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Utf8(text) => Some(text),
            Self::Utf16(_) => None,
        }
    }

    #[inline]
    pub fn as_utf16(&self) -> Option<&[u16]> {
        match self {
            Self::Utf8(_) => None,
            Self::Utf16(units) => Some(units),
        }
    }
}

impl From<&str> for CompiledAtom {
    #[inline]
    fn from(value: &str) -> Self {
        Self::Utf8(value.into())
    }
}

impl From<String> for CompiledAtom {
    #[inline]
    fn from(value: String) -> Self {
        Self::Utf8(value.into_boxed_str())
    }
}

impl From<Box<str>> for CompiledAtom {
    #[inline]
    fn from(value: Box<str>) -> Self {
        Self::Utf8(value)
    }
}

impl From<Vec<u16>> for CompiledAtom {
    #[inline]
    fn from(value: Vec<u16>) -> Self {
        Self::Utf16(value.into_boxed_slice())
    }
}

impl From<Box<[u16]>> for CompiledAtom {
    #[inline]
    fn from(value: Box<[u16]>) -> Self {
        Self::Utf16(value)
    }
}

/// Frozen global-instantiation metadata derived from one compiled script.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GlobalScriptInstantiationPlan {
    lexical_names: Vec<Box<str>>,
    lexical_bindings: Vec<GlobalLexicalBindingPlan>,
    function_names: Vec<Box<str>>,
    var_names: Vec<Box<str>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlobalLexicalBindingPlan {
    name: Box<str>,
    slot: u32,
}

impl GlobalLexicalBindingPlan {
    #[inline]
    pub fn new(name: impl Into<Box<str>>, slot: u32) -> Self {
        Self {
            name: name.into(),
            slot,
        }
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub const fn slot(&self) -> u32 {
        self.slot
    }
}

impl GlobalScriptInstantiationPlan {
    #[inline]
    pub fn new(
        lexical_names: Vec<Box<str>>,
        lexical_bindings: Vec<GlobalLexicalBindingPlan>,
        function_names: Vec<Box<str>>,
        var_names: Vec<Box<str>>,
    ) -> Self {
        Self {
            lexical_names,
            lexical_bindings,
            function_names,
            var_names,
        }
    }

    #[inline]
    pub fn lexical_names(&self) -> &[Box<str>] {
        &self.lexical_names
    }

    #[inline]
    pub fn lexical_bindings(&self) -> &[GlobalLexicalBindingPlan] {
        &self.lexical_bindings
    }

    #[inline]
    pub fn function_names(&self) -> &[Box<str>] {
        &self.function_names
    }

    #[inline]
    pub fn var_names(&self) -> &[Box<str>] {
        &self.var_names
    }
}

/// Installable script artifact produced by the compiler before runtime installation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledScriptUnit {
    source: SourceId,
    entry: BytecodeFunctionId,
    functions: Vec<BytecodeFunction>,
    atoms: Vec<(AtomId, CompiledAtom)>,
    source_text: Option<Box<str>>,
    instantiation_plan: GlobalScriptInstantiationPlan,
}

impl CompiledScriptUnit {
    #[inline]
    pub fn new(
        source: SourceId,
        entry: BytecodeFunctionId,
        functions: Vec<BytecodeFunction>,
    ) -> Self {
        Self {
            source,
            entry,
            functions,
            atoms: Vec::new(),
            source_text: None,
            instantiation_plan: GlobalScriptInstantiationPlan::default(),
        }
    }

    #[inline]
    pub const fn source(&self) -> SourceId {
        self.source
    }

    #[inline]
    pub const fn entry(&self) -> BytecodeFunctionId {
        self.entry
    }

    #[inline]
    pub fn functions(&self) -> &[BytecodeFunction] {
        &self.functions
    }

    #[inline]
    pub fn atoms(&self) -> &[(AtomId, CompiledAtom)] {
        &self.atoms
    }

    #[inline]
    pub fn source_text(&self) -> Option<&str> {
        self.source_text.as_deref()
    }

    #[inline]
    pub fn instantiation_plan(&self) -> &GlobalScriptInstantiationPlan {
        &self.instantiation_plan
    }

    #[inline]
    pub fn atom_text(&self, atom: AtomId) -> Option<&str> {
        self.atoms
            .iter()
            .find_map(|(candidate, text)| (*candidate == atom).then_some(text.as_str()).flatten())
    }

    #[inline]
    pub fn atom_utf16(&self, atom: AtomId) -> Option<&[u16]> {
        self.atoms
            .iter()
            .find_map(|(candidate, text)| (*candidate == atom).then_some(text.as_utf16()).flatten())
    }

    #[inline]
    pub fn function(&self, id: BytecodeFunctionId) -> Option<&BytecodeFunction> {
        self.functions.iter().find(|function| function.id() == id)
    }

    #[inline]
    pub fn with_atoms(mut self, atoms: Vec<(AtomId, CompiledAtom)>) -> Self {
        self.atoms = atoms;
        self
    }

    #[inline]
    pub fn with_source_text(mut self, source_text: impl Into<Box<str>>) -> Self {
        self.source_text = Some(source_text.into());
        self
    }

    #[inline]
    pub fn with_instantiation_plan(
        mut self,
        instantiation_plan: GlobalScriptInstantiationPlan,
    ) -> Self {
        self.instantiation_plan = instantiation_plan;
        self
    }
}

/// Installable standalone function artifact produced by the compiler before runtime installation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledFunctionUnit {
    source: SourceId,
    entry: BytecodeFunctionId,
    functions: Vec<BytecodeFunction>,
    atoms: Vec<(AtomId, CompiledAtom)>,
    source_text: Option<Box<str>>,
}

impl CompiledFunctionUnit {
    #[inline]
    pub fn new(
        source: SourceId,
        entry: BytecodeFunctionId,
        functions: Vec<BytecodeFunction>,
    ) -> Self {
        Self {
            source,
            entry,
            functions,
            atoms: Vec::new(),
            source_text: None,
        }
    }

    #[inline]
    pub const fn source(&self) -> SourceId {
        self.source
    }

    #[inline]
    pub const fn entry(&self) -> BytecodeFunctionId {
        self.entry
    }

    #[inline]
    pub fn functions(&self) -> &[BytecodeFunction] {
        &self.functions
    }

    #[inline]
    pub fn atoms(&self) -> &[(AtomId, CompiledAtom)] {
        &self.atoms
    }

    #[inline]
    pub fn source_text(&self) -> Option<&str> {
        self.source_text.as_deref()
    }

    #[inline]
    pub fn atom_text(&self, atom: AtomId) -> Option<&str> {
        self.atoms
            .iter()
            .find_map(|(candidate, text)| (*candidate == atom).then_some(text.as_str()).flatten())
    }

    #[inline]
    pub fn atom_utf16(&self, atom: AtomId) -> Option<&[u16]> {
        self.atoms
            .iter()
            .find_map(|(candidate, text)| (*candidate == atom).then_some(text.as_utf16()).flatten())
    }

    #[inline]
    pub fn function(&self, id: BytecodeFunctionId) -> Option<&BytecodeFunction> {
        self.functions.iter().find(|function| function.id() == id)
    }

    #[inline]
    pub fn with_atoms(mut self, atoms: Vec<(AtomId, CompiledAtom)>) -> Self {
        self.atoms = atoms;
        self
    }

    #[inline]
    pub fn with_source_text(mut self, source_text: impl Into<Box<str>>) -> Self {
        self.source_text = Some(source_text.into());
        self
    }
}
