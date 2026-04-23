use lyng_js_bytecode::BytecodeFunctionId;
use lyng_js_env::ExecutableId;
use lyng_js_types::CodeRef;

/// Minimal installed-code shell proving the VM owns the transition into runtime code storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InstalledCode {
    code: CodeRef,
    entry: BytecodeFunctionId,
}

impl InstalledCode {
    #[inline]
    pub const fn new(code: CodeRef, entry: BytecodeFunctionId) -> Self {
        Self { code, entry }
    }

    #[inline]
    pub const fn code(self) -> CodeRef {
        self.code
    }

    #[inline]
    pub const fn entry(self) -> BytecodeFunctionId {
        self.entry
    }

    #[inline]
    pub const fn executable(self) -> ExecutableId {
        ExecutableId::Bytecode(self.code)
    }
}
