use super::{
    CodeRef, EnvironmentRef, ObjectRef, PrimitiveStringView, RealmRef, ShapeId, StringRef,
    SymbolRef, Value,
};
use std::num::NonZeroU32;

pub const PRIMITIVE_SLOTS_PER_PAGE: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum AllocationLifetime {
    #[default]
    Default,
    LongLived,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum HeapGeneration {
    Young,
    #[default]
    Old,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct SideAllocationClass(usize);

impl SideAllocationClass {
    #[inline]
    pub const fn for_payload_len(payload_len: usize) -> Self {
        match payload_len {
            0..=16 => Self(16),
            17..=32 => Self(32),
            33..=64 => Self(64),
            65..=128 => Self(128),
            129..=256 => Self(256),
            257..=512 => Self(512),
            513..=1024 => Self(1024),
            _ => Self(payload_len.next_power_of_two()),
        }
    }

    #[inline]
    pub const fn slot_bytes(self) -> usize {
        self.0
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SideAllocationRef(NonZeroU32);

impl SideAllocationRef {
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
    pub const fn get(self) -> u32 {
        self.0.get()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
#[repr(transparent)]
pub struct SymbolFlags(u8);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PrimitiveSymbolClass {
    Ordinary,
    WellKnown,
}

impl SymbolFlags {
    const WELL_KNOWN: u8 = 1 << 0;

    #[inline]
    pub const fn ordinary() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn well_known() -> Self {
        Self(Self::WELL_KNOWN)
    }

    #[inline]
    pub const fn for_class(class: PrimitiveSymbolClass) -> Self {
        match class {
            PrimitiveSymbolClass::Ordinary => Self::ordinary(),
            PrimitiveSymbolClass::WellKnown => Self::well_known(),
        }
    }

    #[inline]
    pub const fn class(self) -> PrimitiveSymbolClass {
        if self.is_well_known() {
            PrimitiveSymbolClass::WellKnown
        } else {
            PrimitiveSymbolClass::Ordinary
        }
    }

    #[inline]
    pub const fn is_ordinary(self) -> bool {
        !self.is_well_known()
    }

    #[inline]
    pub const fn is_well_known(self) -> bool {
        (self.0 & Self::WELL_KNOWN) != 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrimitiveSymbolRecord {
    pub(super) description: Option<StringRef>,
    pub(super) flags: SymbolFlags,
}

#[derive(Debug)]
pub struct PrimitiveSymbolView<'a> {
    pub(super) id: SymbolRef,
    pub(super) record: PrimitiveSymbolRecord,
    pub(super) description: Option<PrimitiveStringView<'a>>,
}

impl PrimitiveSymbolRecord {
    #[inline]
    pub const fn new(description: Option<StringRef>, flags: SymbolFlags) -> Self {
        Self { description, flags }
    }

    #[inline]
    pub const fn description(self) -> Option<StringRef> {
        self.description
    }

    #[inline]
    pub const fn flags(self) -> SymbolFlags {
        self.flags
    }

    #[inline]
    pub const fn class(self) -> PrimitiveSymbolClass {
        self.flags.class()
    }

    #[inline]
    pub const fn is_ordinary(self) -> bool {
        self.flags.is_ordinary()
    }

    #[inline]
    pub const fn is_well_known(self) -> bool {
        self.flags.is_well_known()
    }
}

impl<'a> PrimitiveSymbolView<'a> {
    pub(crate) const fn new(
        id: SymbolRef,
        record: PrimitiveSymbolRecord,
        description: Option<PrimitiveStringView<'a>>,
    ) -> Self {
        Self {
            id,
            record,
            description,
        }
    }

    #[inline]
    pub const fn identity(&self) -> SymbolRef {
        self.id
    }

    #[inline]
    pub const fn record(&self) -> PrimitiveSymbolRecord {
        self.record
    }

    #[inline]
    pub const fn description(&self) -> Option<StringRef> {
        self.record.description()
    }

    #[inline]
    pub const fn description_view(&self) -> Option<&PrimitiveStringView<'a>> {
        self.description.as_ref()
    }

    #[inline]
    pub const fn flags(&self) -> SymbolFlags {
        self.record.flags()
    }

    #[inline]
    pub const fn class(&self) -> PrimitiveSymbolClass {
        self.record.class()
    }

    #[inline]
    pub const fn is_ordinary(&self) -> bool {
        self.record.is_ordinary()
    }

    #[inline]
    pub const fn is_well_known(&self) -> bool {
        self.record.is_well_known()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BigIntSign {
    NonNegative,
    Negative,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrimitiveBigIntRecord {
    pub(super) sign: BigIntSign,
    pub(super) limb_count: u32,
    pub(super) limbs: Option<SideAllocationRef>,
}

#[derive(Clone, Copy, Debug)]
pub struct PrimitiveBigIntView<'a> {
    pub(super) record: PrimitiveBigIntRecord,
    pub(super) limb_bytes: &'a [u8],
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PrimitiveValueCellRef(NonZeroU32);

impl PrimitiveValueCellRef {
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
    pub const fn get(self) -> u32 {
        self.0.get()
    }
}

macro_rules! define_gc_buffer_ref {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[repr(transparent)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub struct $name(NonZeroU32);

        impl $name {
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
            pub const fn get(self) -> u32 {
                self.0.get()
            }
        }
    };
}

define_gc_buffer_ref!(
    /// Slot-buffer handle for object named-slot or element storage.
    ObjectSlotsRef
);
define_gc_buffer_ref!(
    /// Slot-buffer handle for environment dense binding storage.
    EnvironmentSlotsRef
);
define_gc_buffer_ref!(
    /// Slot-buffer handle for code-owned value metadata or constant storage.
    CodeSlotsRef
);
define_gc_buffer_ref!(
    /// Auxiliary handle for one object-owned function payload record.
    FunctionPayloadRef
);
define_gc_buffer_ref!(
    /// Slot-buffer handle for suspended frame register state.
    SuspendedRegistersRef
);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrimitiveValueCellRecord {
    pub(super) stored_value: Value,
    pub(super) linked_string: Option<StringRef>,
}

impl PrimitiveValueCellRecord {
    #[inline]
    pub const fn new(stored_value: Value, linked_string: Option<StringRef>) -> Self {
        Self {
            stored_value,
            linked_string,
        }
    }

    #[inline]
    pub const fn stored_value(self) -> Value {
        self.stored_value
    }

    #[inline]
    pub const fn linked_string(self) -> Option<StringRef> {
        self.linked_string
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeObjectRecord {
    pub(super) prototype: Option<ObjectRef>,
    pub(super) shape: Option<ShapeId>,
    pub(super) named_slots: Option<ObjectSlotsRef>,
    pub(super) elements: Option<ObjectSlotsRef>,
    pub(super) private_slots: Option<ObjectSlotsRef>,
    pub(super) function_payload: Option<FunctionPayloadRef>,
    pub(super) ordinary_payload: Option<PrimitiveValueCellRef>,
}

impl RuntimeObjectRecord {
    #[inline]
    pub const fn new(
        prototype: Option<ObjectRef>,
        shape: Option<ShapeId>,
        named_slots: Option<ObjectSlotsRef>,
        elements: Option<ObjectSlotsRef>,
        function_payload: Option<FunctionPayloadRef>,
    ) -> Self {
        Self {
            prototype,
            shape,
            named_slots,
            elements,
            private_slots: None,
            function_payload,
            ordinary_payload: None,
        }
    }

    #[inline]
    pub const fn prototype(self) -> Option<ObjectRef> {
        self.prototype
    }

    #[inline]
    pub const fn shape(self) -> Option<ShapeId> {
        self.shape
    }

    #[inline]
    pub const fn named_slots(self) -> Option<ObjectSlotsRef> {
        self.named_slots
    }

    #[inline]
    pub const fn elements(self) -> Option<ObjectSlotsRef> {
        self.elements
    }

    #[inline]
    pub const fn private_slots(self) -> Option<ObjectSlotsRef> {
        self.private_slots
    }

    #[inline]
    pub const fn function_payload(self) -> Option<FunctionPayloadRef> {
        self.function_payload
    }

    #[inline]
    pub const fn ordinary_payload(self) -> Option<PrimitiveValueCellRef> {
        self.ordinary_payload
    }

    #[inline]
    pub const fn with_ordinary_payload(
        mut self,
        ordinary_payload: Option<PrimitiveValueCellRef>,
    ) -> Self {
        self.ordinary_payload = ordinary_payload;
        self
    }

    #[inline]
    pub const fn with_private_slots(mut self, private_slots: Option<ObjectSlotsRef>) -> Self {
        self.private_slots = private_slots;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeBoundFunctionRecord {
    pub(super) target: ObjectRef,
    pub(super) this_value: Value,
    pub(super) arguments: Option<ObjectSlotsRef>,
}

impl RuntimeBoundFunctionRecord {
    #[inline]
    pub const fn new(
        target: ObjectRef,
        this_value: Value,
        arguments: Option<ObjectSlotsRef>,
    ) -> Self {
        Self {
            target,
            this_value,
            arguments,
        }
    }

    #[inline]
    pub const fn target(self) -> ObjectRef {
        self.target
    }

    #[inline]
    pub const fn this_value(self) -> Value {
        self.this_value
    }

    #[inline]
    pub const fn arguments(self) -> Option<ObjectSlotsRef> {
        self.arguments
    }

    #[inline]
    pub const fn with_arguments(mut self, arguments: Option<ObjectSlotsRef>) -> Self {
        self.arguments = arguments;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeFunctionRecord {
    pub(super) realm: Option<RealmRef>,
    pub(super) environment: Option<EnvironmentRef>,
    pub(super) private_env: Option<EnvironmentRef>,
    pub(super) home_object: Option<ObjectRef>,
    pub(super) bytecode: Option<CodeRef>,
    pub(super) bound: Option<RuntimeBoundFunctionRecord>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeSuspendedExecutionRecord {
    pub(super) realm: RealmRef,
    pub(super) code: CodeRef,
    pub(super) instruction_offset: u32,
    pub(super) lexical_env: EnvironmentRef,
    pub(super) variable_env: EnvironmentRef,
    pub(super) private_env: Option<EnvironmentRef>,
    pub(super) this_value: Value,
    pub(super) this_state_kind: u8,
    pub(super) construct_this: Option<ObjectRef>,
    pub(super) new_target: Option<ObjectRef>,
    pub(super) callee: Option<ObjectRef>,
    pub(super) handler_cursor: u16,
    pub(super) frame_flags_raw: u8,
    pub(super) context_kind_raw: u8,
    pub(super) registers: Option<SuspendedRegistersRef>,
}

impl RuntimeSuspendedExecutionRecord {
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        realm: RealmRef,
        code: CodeRef,
        instruction_offset: u32,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
        private_env: Option<EnvironmentRef>,
        this_value: Value,
        this_state_kind: u8,
        construct_this: Option<ObjectRef>,
        new_target: Option<ObjectRef>,
        callee: Option<ObjectRef>,
        handler_cursor: u16,
        frame_flags_raw: u8,
        context_kind_raw: u8,
        registers: Option<SuspendedRegistersRef>,
    ) -> Self {
        Self {
            realm,
            code,
            instruction_offset,
            lexical_env,
            variable_env,
            private_env,
            this_value,
            this_state_kind,
            construct_this,
            new_target,
            callee,
            handler_cursor,
            frame_flags_raw,
            context_kind_raw,
            registers,
        }
    }

    #[inline]
    pub const fn code(self) -> CodeRef {
        self.code
    }

    #[inline]
    pub const fn realm(self) -> RealmRef {
        self.realm
    }

    #[inline]
    pub const fn instruction_offset(self) -> u32 {
        self.instruction_offset
    }

    #[inline]
    pub const fn lexical_env(self) -> EnvironmentRef {
        self.lexical_env
    }

    #[inline]
    pub const fn variable_env(self) -> EnvironmentRef {
        self.variable_env
    }

    #[inline]
    pub const fn private_env(self) -> Option<EnvironmentRef> {
        self.private_env
    }

    #[inline]
    pub const fn this_value(self) -> Value {
        self.this_value
    }

    #[inline]
    pub const fn this_state_kind(self) -> u8 {
        self.this_state_kind
    }

    #[inline]
    pub const fn construct_this(self) -> Option<ObjectRef> {
        self.construct_this
    }

    #[inline]
    pub const fn new_target(self) -> Option<ObjectRef> {
        self.new_target
    }

    #[inline]
    pub const fn callee(self) -> Option<ObjectRef> {
        self.callee
    }

    #[inline]
    pub const fn handler_cursor(self) -> u16 {
        self.handler_cursor
    }

    #[inline]
    pub const fn frame_flags_raw(self) -> u8 {
        self.frame_flags_raw
    }

    #[inline]
    pub const fn context_kind_raw(self) -> u8 {
        self.context_kind_raw
    }

    #[inline]
    pub const fn registers(self) -> Option<SuspendedRegistersRef> {
        self.registers
    }
}

impl RuntimeFunctionRecord {
    #[inline]
    pub const fn new(
        realm: Option<RealmRef>,
        environment: Option<EnvironmentRef>,
        private_env: Option<EnvironmentRef>,
        home_object: Option<ObjectRef>,
        bytecode: Option<CodeRef>,
    ) -> Self {
        Self {
            realm,
            environment,
            private_env,
            home_object,
            bytecode,
            bound: None,
        }
    }

    #[inline]
    pub const fn realm(self) -> Option<RealmRef> {
        self.realm
    }

    #[inline]
    pub const fn environment(self) -> Option<EnvironmentRef> {
        self.environment
    }

    #[inline]
    pub const fn private_env(self) -> Option<EnvironmentRef> {
        self.private_env
    }

    #[inline]
    pub const fn home_object(self) -> Option<ObjectRef> {
        self.home_object
    }

    #[inline]
    pub const fn bytecode(self) -> Option<CodeRef> {
        self.bytecode
    }

    #[inline]
    pub const fn bound(self) -> Option<RuntimeBoundFunctionRecord> {
        self.bound
    }

    #[inline]
    pub const fn with_bound(mut self, bound: Option<RuntimeBoundFunctionRecord>) -> Self {
        self.bound = bound;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeEnvironmentRecord {
    pub(super) outer: Option<EnvironmentRef>,
    pub(super) slots: Option<EnvironmentSlotsRef>,
    pub(super) function_object: Option<ObjectRef>,
    pub(super) this_value: Value,
    pub(super) new_target: Option<ObjectRef>,
    pub(super) home_object: Option<ObjectRef>,
}

impl RuntimeEnvironmentRecord {
    #[inline]
    pub const fn new(
        outer: Option<EnvironmentRef>,
        slots: Option<EnvironmentSlotsRef>,
        function_object: Option<ObjectRef>,
        this_value: Value,
        new_target: Option<ObjectRef>,
        home_object: Option<ObjectRef>,
    ) -> Self {
        Self {
            outer,
            slots,
            function_object,
            this_value,
            new_target,
            home_object,
        }
    }

    #[inline]
    pub const fn outer(self) -> Option<EnvironmentRef> {
        self.outer
    }

    #[inline]
    pub const fn slots(self) -> Option<EnvironmentSlotsRef> {
        self.slots
    }

    #[inline]
    pub const fn function_object(self) -> Option<ObjectRef> {
        self.function_object
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeCodeRecord {
    pub(super) parent: Option<CodeRef>,
    pub(super) realm: Option<RealmRef>,
    pub(super) constants: Option<CodeSlotsRef>,
}

impl RuntimeCodeRecord {
    #[inline]
    pub const fn new(
        parent: Option<CodeRef>,
        realm: Option<RealmRef>,
        constants: Option<CodeSlotsRef>,
    ) -> Self {
        Self {
            parent,
            realm,
            constants,
        }
    }

    #[inline]
    pub const fn parent(self) -> Option<CodeRef> {
        self.parent
    }

    #[inline]
    pub const fn realm(self) -> Option<RealmRef> {
        self.realm
    }

    #[inline]
    pub const fn constants(self) -> Option<CodeSlotsRef> {
        self.constants
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeRealmRecord {
    pub(super) global_object: Option<ObjectRef>,
    pub(super) global_env: Option<EnvironmentRef>,
    pub(super) bootstrap_code: Option<CodeRef>,
    pub(super) root_shape: Option<ShapeId>,
}

impl RuntimeRealmRecord {
    #[inline]
    pub const fn new(
        global_object: Option<ObjectRef>,
        global_env: Option<EnvironmentRef>,
        bootstrap_code: Option<CodeRef>,
        root_shape: Option<ShapeId>,
    ) -> Self {
        Self {
            global_object,
            global_env,
            bootstrap_code,
            root_shape,
        }
    }

    #[inline]
    pub const fn global_object(self) -> Option<ObjectRef> {
        self.global_object
    }

    #[inline]
    pub const fn global_env(self) -> Option<EnvironmentRef> {
        self.global_env
    }

    #[inline]
    pub const fn bootstrap_code(self) -> Option<CodeRef> {
        self.bootstrap_code
    }

    #[inline]
    pub const fn root_shape(self) -> Option<ShapeId> {
        self.root_shape
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeShapeRecord {
    pub(super) parent: Option<ShapeId>,
    pub(super) prototype_guard: Option<ObjectRef>,
    pub(super) slot_count: u32,
}

impl RuntimeShapeRecord {
    #[inline]
    pub const fn new(
        parent: Option<ShapeId>,
        prototype_guard: Option<ObjectRef>,
        slot_count: u32,
    ) -> Self {
        Self {
            parent,
            prototype_guard,
            slot_count,
        }
    }

    #[inline]
    pub const fn parent(self) -> Option<ShapeId> {
        self.parent
    }

    #[inline]
    pub const fn prototype_guard(self) -> Option<ObjectRef> {
        self.prototype_guard
    }

    #[inline]
    pub const fn slot_count(self) -> u32 {
        self.slot_count
    }
}

impl PrimitiveBigIntRecord {
    #[inline]
    pub const fn new(sign: BigIntSign, limb_count: u32, limbs: Option<SideAllocationRef>) -> Self {
        Self {
            sign,
            limb_count,
            limbs,
        }
    }

    #[inline]
    pub const fn sign(self) -> BigIntSign {
        self.sign
    }

    #[inline]
    pub const fn limb_count(self) -> u32 {
        self.limb_count
    }

    #[inline]
    pub const fn limb_storage(self) -> Option<SideAllocationRef> {
        self.limbs
    }

    #[inline]
    pub const fn is_zero(self) -> bool {
        self.limb_count == 0
    }
}

impl<'a> PrimitiveBigIntView<'a> {
    pub(crate) fn new(record: PrimitiveBigIntRecord, limb_bytes: &'a [u8]) -> Self {
        debug_assert_eq!(
            limb_bytes.len(),
            record.limb_count() as usize * std::mem::size_of::<u64>(),
            "bigint limb bytes must match the normalized limb count"
        );

        Self { record, limb_bytes }
    }

    #[inline]
    pub const fn record(self) -> PrimitiveBigIntRecord {
        self.record
    }

    #[inline]
    pub const fn sign(self) -> BigIntSign {
        self.record.sign()
    }

    #[inline]
    pub const fn limb_count(self) -> u32 {
        self.record.limb_count()
    }

    #[inline]
    pub const fn is_zero(self) -> bool {
        self.record.is_zero()
    }

    #[inline]
    pub const fn limb_bytes_le(self) -> &'a [u8] {
        self.limb_bytes
    }

    pub fn limb_at(self, index: usize) -> Option<u64> {
        if index >= self.limb_count() as usize {
            return None;
        }

        let start = index.checked_mul(std::mem::size_of::<u64>())?;
        let end = start + std::mem::size_of::<u64>();
        let chunk = self.limb_bytes.get(start..end)?;
        let mut raw = [0_u8; std::mem::size_of::<u64>()];
        raw.copy_from_slice(chunk);
        Some(u64::from_le_bytes(raw))
    }

    pub fn to_limbs(self) -> Vec<u64> {
        self.limb_bytes
            .chunks_exact(std::mem::size_of::<u64>())
            .map(|chunk| {
                let mut raw = [0_u8; std::mem::size_of::<u64>()];
                raw.copy_from_slice(chunk);
                u64::from_le_bytes(raw)
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct SideAllocationStats {
    pub live_allocations: usize,
    pub reusable_allocations: usize,
    pub live_payload_bytes: usize,
    pub young_live_payload_bytes: usize,
    pub old_live_payload_bytes: usize,
    pub reserved_bytes: usize,
    pub reusable_reserved_bytes: usize,
    pub young_allocations: usize,
    pub old_allocations: usize,
    pub default_allocations: usize,
    pub long_lived_allocations: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct PrimitiveDomainStats {
    pub pages: usize,
    pub occupied_slots: usize,
    pub reusable_slots: usize,
    pub marked_slots: usize,
    pub young_slots: usize,
    pub old_slots: usize,
    pub default_slots: usize,
    pub long_lived_slots: usize,
    pub side_allocations: SideAllocationStats,
}
