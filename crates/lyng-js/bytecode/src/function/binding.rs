use lyng_js_common::AtomId;

/// Frozen per-slot flags carried alongside one installable bytecode environment binding.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[allow(clippy::struct_excessive_bools)]
pub struct BytecodeEnvironmentSlotFlags {
    mutable: bool,
    lexical: bool,
    needs_tdz: bool,
    dynamic: bool,
    scoped: bool,
    sloppy_immutable_assign_silent: bool,
}

impl BytecodeEnvironmentSlotFlags {
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

/// One compiler-derived binding slot in an installable bytecode environment layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BytecodeEnvironmentBinding {
    name: Option<AtomId>,
    flags: BytecodeEnvironmentSlotFlags,
}

impl BytecodeEnvironmentBinding {
    #[inline]
    pub const fn new(name: Option<AtomId>, flags: BytecodeEnvironmentSlotFlags) -> Self {
        Self { name, flags }
    }

    #[inline]
    pub const fn name(self) -> Option<AtomId> {
        self.name
    }

    #[inline]
    pub const fn flags(self) -> BytecodeEnvironmentSlotFlags {
        self.flags
    }
}
