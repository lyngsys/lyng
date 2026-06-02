use lyng_env::{ExecutionContextKind, ThisState};
use lyng_types::{CodeRef, EnvironmentRef, ObjectRef, Value};

/// Number of 64-bit slots the header occupies at the front of every frame.
pub const HEADER_SLOTS: usize = 7;

const ROOT_CFR: u32 = u32::MAX;

mod flag {
    pub const HAS_RETURN_REGISTER: u8 = 1 << 4; // bits 0..3 mirror FrameFlags
}

mod this_tag {
    pub const UNINITIALIZED: u8 = 0;
    pub const LEXICAL: u8 = 1;
    pub const VALUE: u8 = 2;
}

/// asm-addressable, GC-traced per-frame header.
///
/// Overlaid as POD on the first `HEADER_SLOTS` `Value`-sized slots of a frame in
/// the `FrameArena`. Field order is the ABI (locked by `frame_header_offsets_stable`);
/// slots 0-3 are the asm-hot cluster, 4-6 the interpreter-warm cluster.
/// realm/referrer/executable/geometry are NOT stored (derived elsewhere).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FrameHeader {
    // slot 0
    pub(crate) caller_cfr: u32,
    pub(crate) saved_pc: u32,
    // slot 1
    pub(crate) code: u32,
    pub(crate) callee: u32, // 0 = None
    // slot 2
    pub(crate) this_value: Value,
    // slot 3
    pub(crate) arg_count: u16,
    pub(crate) return_register: u16, // valid iff flags & HAS_RETURN_REGISTER
    pub(crate) flags: u8,
    pub(crate) this_state_tag: u8,
    pub(crate) kind: u8,
    pub(crate) _pad0: u8,
    // slot 4
    pub(crate) variable_env: u32,
    pub(crate) lexical_env: u32,
    // slot 5
    pub(crate) private_env: u32, // 0 = None
    pub(crate) new_target: u32,  // 0 = None
    // slot 6
    pub(crate) construct_this: u32, // 0 = None
    pub(crate) _pad1: u32,
}

impl FrameHeader {
    /// Construct an all-zero header with the root-frame sentinel in `caller_cfr`.
    #[inline]
    pub const fn zeroed() -> Self {
        Self {
            caller_cfr: ROOT_CFR,
            saved_pc: 0,
            code: 0,
            callee: 0,
            // The only header slot GC reads as a Value (the rest are packed ints,
            // traced via typed accessors). Must be a valid Value, not raw-zero bits.
            this_value: Value::undefined(),
            arg_count: 0,
            return_register: 0,
            flags: 0,
            this_state_tag: this_tag::UNINITIALIZED,
            kind: 0,
            _pad0: 0,
            variable_env: 0,
            lexical_env: 0,
            private_env: 0,
            new_target: 0,
            construct_this: 0,
            _pad1: 0,
        }
    }

    // ---- caller_cfr / saved_pc ----

    /// `None` means this is the root frame (no caller).
    #[inline]
    pub const fn caller_cfr(&self) -> Option<u32> {
        if self.caller_cfr == ROOT_CFR {
            None
        } else {
            Some(self.caller_cfr)
        }
    }

    #[inline]
    pub fn set_caller_cfr(&mut self, cfr: Option<u32>) {
        self.caller_cfr = cfr.unwrap_or(ROOT_CFR);
    }

    #[inline]
    pub const fn saved_pc(&self) -> u32 {
        self.saved_pc
    }

    #[inline]
    pub const fn set_saved_pc(&mut self, pc: u32) {
        self.saved_pc = pc;
    }

    // ---- return_register ----

    /// `Some` iff the `HAS_RETURN_REGISTER` flag bit is set.
    #[inline]
    pub const fn return_register(&self) -> Option<u16> {
        if self.flags & flag::HAS_RETURN_REGISTER != 0 {
            Some(self.return_register)
        } else {
            None
        }
    }

    #[inline]
    pub const fn set_return_register(&mut self, reg: Option<u16>) {
        if let Some(r) = reg {
            self.return_register = r;
            self.flags |= flag::HAS_RETURN_REGISTER;
        } else {
            self.return_register = 0;
            self.flags &= !flag::HAS_RETURN_REGISTER;
        }
    }

    // ---- code ----

    /// The frame's code reference.
    ///
    /// # Panics
    /// Panics if the `code` slot is zero — never the case for a live frame.
    #[inline]
    pub const fn code(&self) -> CodeRef {
        CodeRef::from_raw(self.code).expect("FrameHeader::code field is always non-zero")
    }

    #[inline]
    pub const fn set_code(&mut self, code: CodeRef) {
        self.code = code.get();
    }

    // ---- callee ----

    #[inline]
    pub const fn callee(&self) -> Option<ObjectRef> {
        ObjectRef::from_raw(self.callee)
    }

    #[inline]
    pub fn set_callee(&mut self, callee: Option<ObjectRef>) {
        self.callee = callee.map_or(0, lyng_types::ObjectRef::get);
    }

    // ---- this ----

    #[inline]
    pub const fn this_value(&self) -> Value {
        self.this_value
    }

    #[inline]
    pub const fn this_state(&self) -> ThisState {
        match self.this_state_tag {
            this_tag::LEXICAL => ThisState::Lexical,
            this_tag::VALUE => ThisState::Value(self.this_value),
            _ => ThisState::Uninitialized,
        }
    }

    /// Store both the `ThisState` tag and the accompanying `Value` slot.
    /// For `Lexical` and `Uninitialized`, `this_value` is still stored so the
    /// slot is deterministic (avoids stale reads after a state transition).
    /// The separate `this_value` param is intentional: Lexical/Uninitialized
    /// frames carry their `this` binding independently of the state tag.
    #[inline]
    pub fn set_this(&mut self, state: ThisState, this_value: Value) {
        debug_assert!(
            !matches!(state, ThisState::Value(v) if v != this_value),
            "set_this: ThisState::Value payload must equal the this_value argument",
        );
        self.this_value = this_value;
        self.this_state_tag = match state {
            ThisState::Uninitialized => this_tag::UNINITIALIZED,
            ThisState::Lexical => this_tag::LEXICAL,
            ThisState::Value(_) => this_tag::VALUE,
        };
    }

    /// Update the raw `this_value` slot without touching the tag.
    #[inline]
    pub const fn set_this_value(&mut self, this_value: Value) {
        self.this_value = this_value;
    }

    // ---- variable_env / lexical_env ----

    /// The frame's variable environment.
    ///
    /// # Panics
    /// Panics if the `variable_env` slot is zero — never the case for a live frame.
    #[inline]
    pub const fn variable_env(&self) -> EnvironmentRef {
        EnvironmentRef::from_raw(self.variable_env)
            .expect("FrameHeader::variable_env field is always non-zero")
    }

    #[inline]
    pub const fn set_variable_env(&mut self, env: EnvironmentRef) {
        self.variable_env = env.get();
    }

    /// The frame's lexical environment.
    ///
    /// # Panics
    /// Panics if the `lexical_env` slot is zero — never the case for a live frame.
    #[inline]
    pub const fn lexical_env(&self) -> EnvironmentRef {
        EnvironmentRef::from_raw(self.lexical_env)
            .expect("FrameHeader::lexical_env field is always non-zero")
    }

    #[inline]
    pub const fn set_lexical_env(&mut self, env: EnvironmentRef) {
        self.lexical_env = env.get();
    }

    // ---- private_env (optional) ----

    #[inline]
    pub const fn private_env(&self) -> Option<EnvironmentRef> {
        EnvironmentRef::from_raw(self.private_env)
    }

    #[inline]
    pub fn set_private_env(&mut self, env: Option<EnvironmentRef>) {
        self.private_env = env.map_or(0, lyng_types::EnvironmentRef::get);
    }

    // ---- new_target (optional ObjectRef) ----

    #[inline]
    pub const fn new_target(&self) -> Option<ObjectRef> {
        ObjectRef::from_raw(self.new_target)
    }

    #[inline]
    pub fn set_new_target(&mut self, target: Option<ObjectRef>) {
        self.new_target = target.map_or(0, lyng_types::ObjectRef::get);
    }

    // ---- construct_this (optional ObjectRef) ----

    #[inline]
    pub const fn construct_this(&self) -> Option<ObjectRef> {
        ObjectRef::from_raw(self.construct_this)
    }

    #[inline]
    pub fn set_construct_this(&mut self, obj: Option<ObjectRef>) {
        self.construct_this = obj.map_or(0, lyng_types::ObjectRef::get);
    }

    // ---- arg_count ----

    #[inline]
    pub const fn arg_count(&self) -> u16 {
        self.arg_count
    }

    #[inline]
    pub const fn set_arg_count(&mut self, count: u16) {
        self.arg_count = count;
    }

    // ---- flags (low nibble mirrors FrameFlags; bit 4 = HAS_RETURN_REGISTER) ----

    /// Returns the low 4 bits (`FrameFlags` portion).
    #[inline]
    pub const fn flags_bits(&self) -> u8 {
        self.flags & 0x0F
    }

    /// Sets the low 4 bits without disturbing the high bits (e.g. `HAS_RETURN_REGISTER`).
    #[inline]
    pub const fn set_flags_bits(&mut self, bits: u8) {
        self.flags = (self.flags & !0x0F) | (bits & 0x0F);
    }

    // ---- kind (ExecutionContextKind raw byte) ----

    #[inline]
    pub const fn kind_raw(&self) -> u8 {
        self.kind
    }

    #[inline]
    pub const fn set_kind_raw(&mut self, kind: u8) {
        self.kind = kind;
    }

    /// Decode the stored `kind_raw()` byte back into [`ExecutionContextKind`].
    ///
    /// Inverts the `ExecutionContextKind as u8` mapping `write_header_from_record`
    /// stores via `set_kind_raw` (the enum's default discriminants). The match
    /// arms reference `as u8` of each variant so this stays a single source of
    /// truth with the encode side; an unrecognized byte falls back to `Function`
    /// (the generic activation kind) so a corrupt slot can never masquerade as
    /// the synthetic `Job` root.
    #[inline]
    pub const fn kind(&self) -> ExecutionContextKind {
        const SCRIPT: u8 = ExecutionContextKind::Script as u8;
        const MODULE: u8 = ExecutionContextKind::Module as u8;
        const BUILTIN: u8 = ExecutionContextKind::Builtin as u8;
        const EVAL: u8 = ExecutionContextKind::Eval as u8;
        const JOB: u8 = ExecutionContextKind::Job as u8;
        match self.kind {
            SCRIPT => ExecutionContextKind::Script,
            MODULE => ExecutionContextKind::Module,
            BUILTIN => ExecutionContextKind::Builtin,
            EVAL => ExecutionContextKind::Eval,
            JOB => ExecutionContextKind::Job,
            _ => ExecutionContextKind::Function,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::{align_of, offset_of, size_of};
    use lyng_env::ThisState;
    use lyng_types::{CodeRef, EnvironmentRef, ObjectRef, Value};
    use std::num::NonZeroU32;

    const fn id(raw: u32) -> NonZeroU32 {
        match NonZeroU32::new(raw) {
            Some(v) => v,
            None => panic!("non-zero"),
        }
    }

    #[test]
    fn frame_header_offsets_stable() {
        assert_eq!(offset_of!(FrameHeader, caller_cfr), 0);
        assert_eq!(offset_of!(FrameHeader, saved_pc), 4);
        assert_eq!(offset_of!(FrameHeader, code), 8);
        assert_eq!(offset_of!(FrameHeader, callee), 12);
        assert_eq!(offset_of!(FrameHeader, this_value), 16);
        assert_eq!(offset_of!(FrameHeader, arg_count), 24);
        assert_eq!(offset_of!(FrameHeader, return_register), 26);
        assert_eq!(offset_of!(FrameHeader, flags), 28);
        assert_eq!(offset_of!(FrameHeader, this_state_tag), 29);
        assert_eq!(offset_of!(FrameHeader, kind), 30);
        assert_eq!(offset_of!(FrameHeader, variable_env), 32);
        assert_eq!(offset_of!(FrameHeader, lexical_env), 36);
        assert_eq!(offset_of!(FrameHeader, private_env), 40);
        assert_eq!(offset_of!(FrameHeader, new_target), 44);
        assert_eq!(offset_of!(FrameHeader, construct_this), 48);
        assert_eq!(size_of::<FrameHeader>(), HEADER_SLOTS * size_of::<Value>());
        assert_eq!(size_of::<FrameHeader>(), 56);
        assert_eq!(align_of::<FrameHeader>(), align_of::<Value>());
    }

    /// Asserts that every `FRAME_HEADER_*` constant in `reg_convention`
    /// matches the corresponding `offset_of!(FrameHeader, <field>)` value.
    /// Guards the asm-ABI contract when reading header fields at constant
    /// offsets from the frame pointer (cfr).
    #[test]
    fn frame_header_abi_offsets_match_reg_convention() {
        use crate::dsl::reg_convention as r;
        assert_eq!(
            r::FRAME_HEADER_CALLER_CFR,
            offset_of!(FrameHeader, caller_cfr)
        );
        assert_eq!(r::FRAME_HEADER_SAVED_PC, offset_of!(FrameHeader, saved_pc));
        assert_eq!(r::FRAME_HEADER_CODE, offset_of!(FrameHeader, code));
        assert_eq!(r::FRAME_HEADER_CALLEE, offset_of!(FrameHeader, callee));
        assert_eq!(
            r::FRAME_HEADER_THIS_VALUE,
            offset_of!(FrameHeader, this_value)
        );
        assert_eq!(
            r::FRAME_HEADER_ARG_COUNT,
            offset_of!(FrameHeader, arg_count)
        );
        assert_eq!(r::FRAME_HEADER_FLAGS, offset_of!(FrameHeader, flags));
        // Cross-check the literal offsets from frame_header_offsets_stable:
        assert_eq!(r::FRAME_HEADER_CALLER_CFR, 0);
        assert_eq!(r::FRAME_HEADER_SAVED_PC, 4);
        assert_eq!(r::FRAME_HEADER_CODE, 8);
        assert_eq!(r::FRAME_HEADER_CALLEE, 12);
        assert_eq!(r::FRAME_HEADER_THIS_VALUE, 16);
        assert_eq!(r::FRAME_HEADER_ARG_COUNT, 24);
        assert_eq!(r::FRAME_HEADER_FLAGS, 28);
        // FRAME_HEADER_SLOTS mirrors HEADER_SLOTS.
        assert_eq!(r::FRAME_HEADER_SLOTS, HEADER_SLOTS);
    }

    #[test]
    fn typed_accessors_round_trip() {
        let mut h = FrameHeader::zeroed();
        h.set_code(CodeRef::new(id(7)));
        h.set_callee(Some(ObjectRef::new(id(3))));
        h.set_callee(None); // 0 sentinel
        h.set_variable_env(EnvironmentRef::new(id(5)));
        h.set_this(ThisState::Value(Value::from_smi(11)), Value::from_smi(11));
        h.set_return_register(Some(4));
        assert_eq!(h.code(), CodeRef::new(id(7)));
        assert_eq!(h.callee(), None);
        assert_eq!(h.variable_env(), EnvironmentRef::new(id(5)));
        assert_eq!(h.this_state(), ThisState::Value(Value::from_smi(11)));
        assert_eq!(h.this_value(), Value::from_smi(11));
        assert_eq!(h.return_register(), Some(4));
        assert_eq!(FrameHeader::zeroed().return_register(), None);

        let mut h2 = FrameHeader::zeroed();
        h2.set_this(ThisState::Lexical, Value::undefined());
        assert_eq!(h2.this_state(), ThisState::Lexical);
        h2.set_this(ThisState::Uninitialized, Value::undefined());
        assert_eq!(h2.this_state(), ThisState::Uninitialized);
    }

    #[test]
    fn caller_cfr_round_trips() {
        let mut h = FrameHeader::zeroed();
        assert_eq!(h.caller_cfr(), None); // ROOT sentinel
        h.set_caller_cfr(Some(42));
        assert_eq!(h.caller_cfr(), Some(42));
        h.set_caller_cfr(None);
        assert_eq!(h.caller_cfr(), None);
    }

    #[test]
    fn flags_bits_and_return_register_do_not_collide() {
        let mut h = FrameHeader::zeroed();
        h.set_flags_bits(0x0F); // all four FrameFlags bits
        h.set_return_register(Some(9)); // sets bit 4
        assert_eq!(h.flags_bits(), 0x0F); // FrameFlags bits intact
        assert_eq!(h.return_register(), Some(9));
        h.set_return_register(None); // clears bit 4 only
        assert_eq!(h.flags_bits(), 0x0F); // still intact
        assert_eq!(h.return_register(), None);
    }
}
