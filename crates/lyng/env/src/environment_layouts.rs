use lyng_js_common::AtomId;
use std::fmt;
use std::num::NonZeroU32;

/// Stable typed ID for environment-layout metadata records.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EnvironmentLayoutId(NonZeroU32);

impl EnvironmentLayoutId {
    #[inline]
    pub const fn new(raw: NonZeroU32) -> Self {
        Self(raw)
    }

    #[inline]
    pub const fn from_raw(raw: u32) -> Option<Self> {
        match NonZeroU32::new(raw) {
            Some(raw) => Some(Self(raw)),
            None => None,
        }
    }

    #[inline]
    pub const fn raw(self) -> NonZeroU32 {
        self.0
    }

    #[inline]
    pub const fn get(self) -> u32 {
        self.0.get()
    }
}

impl fmt::Debug for EnvironmentLayoutId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EnvironmentLayoutId({})", self.get())
    }
}

/// Frozen runtime layout categories reserved by the environment model.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EnvironmentLayoutKind {
    Declarative,
    Function,
    Global,
    Object,
    Module,
    Private,
}

/// Per-slot metadata flags carried by one immutable environment layout.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[allow(clippy::struct_excessive_bools)]
pub struct EnvironmentSlotFlags {
    mutable: bool,
    lexical: bool,
    needs_tdz: bool,
    dynamic: bool,
    scoped: bool,
    sloppy_immutable_assign_silent: bool,
}

impl EnvironmentSlotFlags {
    #[inline]
    #[allow(clippy::fn_params_excessive_bools)]
    pub const fn new(mutable: bool, lexical: bool, needs_tdz: bool, dynamic: bool) -> Self {
        Self {
            mutable,
            lexical,
            needs_tdz,
            dynamic,
            scoped: false,
            sloppy_immutable_assign_silent: false,
        }
    }

    #[inline]
    pub const fn mutable_lexical() -> Self {
        Self::new(true, true, true, false)
    }

    #[inline]
    pub const fn immutable_lexical() -> Self {
        Self::new(false, true, true, false)
    }

    #[inline]
    pub const fn var_like() -> Self {
        Self::new(true, false, false, false)
    }

    #[inline]
    pub const fn is_mutable(self) -> bool {
        self.mutable
    }

    #[inline]
    pub const fn is_lexical(self) -> bool {
        self.lexical
    }

    #[inline]
    pub const fn needs_tdz(self) -> bool {
        self.needs_tdz
    }

    #[inline]
    pub const fn is_dynamic(self) -> bool {
        self.dynamic
    }

    #[inline]
    pub const fn is_scoped(self) -> bool {
        self.scoped
    }

    #[inline]
    pub const fn with_dynamic(mut self, dynamic: bool) -> Self {
        self.dynamic = dynamic;
        self
    }

    #[inline]
    pub const fn with_scoped(mut self, scoped: bool) -> Self {
        self.scoped = scoped;
        self
    }

    #[inline]
    pub const fn sloppy_immutable_assign_silent(self) -> bool {
        self.sloppy_immutable_assign_silent
    }

    #[inline]
    pub const fn with_sloppy_immutable_assign_silent(mut self, silent: bool) -> Self {
        self.sloppy_immutable_assign_silent = silent;
        self
    }
}

/// One binding slot in a frozen environment layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EnvironmentBindingLayout {
    name: Option<AtomId>,
    flags: EnvironmentSlotFlags,
}

impl EnvironmentBindingLayout {
    #[inline]
    pub const fn new(name: Option<AtomId>, flags: EnvironmentSlotFlags) -> Self {
        Self { name, flags }
    }

    #[inline]
    pub const fn name(self) -> Option<AtomId> {
        self.name
    }

    #[inline]
    pub const fn flags(self) -> EnvironmentSlotFlags {
        self.flags
    }
}

/// Immutable metadata record that maps sema-produced slot order to runtime storage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnvironmentLayout {
    kind: EnvironmentLayoutKind,
    bindings: Vec<EnvironmentBindingLayout>,
    needs_environment: bool,
}

impl EnvironmentLayout {
    pub fn new(
        kind: EnvironmentLayoutKind,
        bindings: impl Into<Vec<EnvironmentBindingLayout>>,
        needs_environment: bool,
    ) -> Self {
        Self {
            kind,
            bindings: bindings.into(),
            needs_environment,
        }
    }

    #[inline]
    pub fn empty(kind: EnvironmentLayoutKind, needs_environment: bool) -> Self {
        Self::new(kind, Vec::new(), needs_environment)
    }

    #[inline]
    pub const fn kind(&self) -> EnvironmentLayoutKind {
        self.kind
    }

    #[inline]
    /// Returns the number of storage slots described by this layout.
    ///
    /// # Panics
    /// Panics if the binding count does not fit into `u32`.
    pub fn slot_count(&self) -> u32 {
        u32::try_from(self.bindings.len()).expect("environment layout slot count must fit into u32")
    }

    #[inline]
    pub fn bindings(&self) -> &[EnvironmentBindingLayout] {
        &self.bindings
    }

    #[inline]
    pub fn binding(&self, index: u32) -> Option<EnvironmentBindingLayout> {
        self.bindings.get(index as usize).copied()
    }

    #[inline]
    pub const fn needs_environment(&self) -> bool {
        self.needs_environment
    }
}

/// Function-environment ownership state for `this`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ThisBindingStatus {
    Lexical,
    Uninitialized,
    Initialized,
}
