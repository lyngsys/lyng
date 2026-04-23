use lyng_js_bytecode::BytecodeMarker;
use lyng_js_env::ExecutionContext;

use crate::FrameRecord;

/// Minimal VM-owned marker linking bytecode installation to execution-context state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VmMarker {
    bytecode: BytecodeMarker,
    context: ExecutionContext,
    frame: FrameRecord,
}

impl VmMarker {
    #[inline]
    pub const fn new(
        bytecode: BytecodeMarker,
        context: ExecutionContext,
        frame: FrameRecord,
    ) -> Self {
        Self {
            bytecode,
            context,
            frame,
        }
    }

    #[inline]
    pub const fn bytecode(self) -> BytecodeMarker {
        self.bytecode
    }

    #[inline]
    pub const fn context(self) -> ExecutionContext {
        self.context
    }

    #[inline]
    pub const fn frame(self) -> FrameRecord {
        self.frame
    }
}
