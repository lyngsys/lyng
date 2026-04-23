use crate::{
    collection::DEFAULT_COLLECTION_BUDGET_BYTES, weak::FinalizationRegistryState,
    weak::WeakMapState, weak::WeakRefState, weak::WeakSetState, PrimitiveStringRecord,
    PrimitiveStringView, StringEncoding, WeakHeapRef,
};
use lyng_js_common::AtomId;
use lyng_js_types::{
    BigIntRef, CodeRef, EnvironmentRef, ObjectRef, RealmRef, ShapeId, StringRef,
    SuspendedExecutionRef, SymbolRef, Value,
};
use std::array::from_fn;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::num::NonZeroU32;

pub const PRIMITIVE_SLOTS_PER_PAGE: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum AllocationLifetime {
    #[default]
    Default,
    LongLived,
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
    description: Option<StringRef>,
    flags: SymbolFlags,
}

#[derive(Clone, Copy, Debug)]
pub struct PrimitiveSymbolView<'a> {
    id: SymbolRef,
    record: PrimitiveSymbolRecord,
    description: Option<PrimitiveStringView<'a>>,
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
    pub const fn identity(self) -> SymbolRef {
        self.id
    }

    #[inline]
    pub const fn record(self) -> PrimitiveSymbolRecord {
        self.record
    }

    #[inline]
    pub const fn description(self) -> Option<StringRef> {
        self.record.description()
    }

    #[inline]
    pub const fn description_view(self) -> Option<PrimitiveStringView<'a>> {
        self.description
    }

    #[inline]
    pub const fn flags(self) -> SymbolFlags {
        self.record.flags()
    }

    #[inline]
    pub const fn class(self) -> PrimitiveSymbolClass {
        self.record.class()
    }

    #[inline]
    pub const fn is_ordinary(self) -> bool {
        self.record.is_ordinary()
    }

    #[inline]
    pub const fn is_well_known(self) -> bool {
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
    sign: BigIntSign,
    limb_count: u32,
    limbs: Option<SideAllocationRef>,
}

#[derive(Clone, Copy, Debug)]
pub struct PrimitiveBigIntView<'a> {
    record: PrimitiveBigIntRecord,
    limb_bytes: &'a [u8],
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
    stored_value: Value,
    linked_string: Option<StringRef>,
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
    prototype: Option<ObjectRef>,
    shape: Option<ShapeId>,
    named_slots: Option<ObjectSlotsRef>,
    elements: Option<ObjectSlotsRef>,
    private_slots: Option<ObjectSlotsRef>,
    function_payload: Option<FunctionPayloadRef>,
    ordinary_payload: Option<PrimitiveValueCellRef>,
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
    target: ObjectRef,
    this_value: Value,
    arguments: Option<ObjectSlotsRef>,
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
    realm: Option<RealmRef>,
    environment: Option<EnvironmentRef>,
    private_env: Option<EnvironmentRef>,
    home_object: Option<ObjectRef>,
    bytecode: Option<CodeRef>,
    bound: Option<RuntimeBoundFunctionRecord>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeSuspendedExecutionRecord {
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
    outer: Option<EnvironmentRef>,
    slots: Option<EnvironmentSlotsRef>,
    function_object: Option<ObjectRef>,
    this_value: Value,
    new_target: Option<ObjectRef>,
    home_object: Option<ObjectRef>,
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
    parent: Option<CodeRef>,
    realm: Option<RealmRef>,
    constants: Option<CodeSlotsRef>,
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
    global_object: Option<ObjectRef>,
    global_env: Option<EnvironmentRef>,
    bootstrap_code: Option<CodeRef>,
    root_shape: Option<ShapeId>,
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
    parent: Option<ShapeId>,
    prototype_guard: Option<ObjectRef>,
    slot_count: u32,
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
    pub reserved_bytes: usize,
    pub reusable_reserved_bytes: usize,
    pub default_allocations: usize,
    pub long_lived_allocations: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct PrimitiveDomainStats {
    pub pages: usize,
    pub occupied_slots: usize,
    pub reusable_slots: usize,
    pub marked_slots: usize,
    pub default_slots: usize,
    pub long_lived_slots: usize,
    pub side_allocations: SideAllocationStats,
}

pub struct PrimitiveHeap {
    strings: SlotArena<PrimitiveStringRecord, StringRef>,
    string_payloads: SideAllocator,
    symbols: SlotArena<PrimitiveSymbolRecord, SymbolRef>,
    bigints: SlotArena<PrimitiveBigIntRecord, BigIntRef>,
    bigint_payloads: SideAllocator,
    value_cells: SlotArena<PrimitiveValueCellRecord, PrimitiveValueCellRef>,
    objects: SlotArena<RuntimeObjectRecord, ObjectRef>,
    function_payloads: SlotArena<RuntimeFunctionRecord, FunctionPayloadRef>,
    object_slots: ValueSlotAllocator<ObjectSlotsRef>,
    suspended_executions: SlotArena<RuntimeSuspendedExecutionRecord, SuspendedExecutionRef>,
    suspended_registers: ValueSlotAllocator<SuspendedRegistersRef>,
    environments: SlotArena<RuntimeEnvironmentRecord, EnvironmentRef>,
    environment_slots: ValueSlotAllocator<EnvironmentSlotsRef>,
    codes: SlotArena<RuntimeCodeRecord, CodeRef>,
    code_slots: ValueSlotAllocator<CodeSlotsRef>,
    realms: SlotArena<RuntimeRealmRecord, RealmRef>,
    shapes: SlotArena<RuntimeShapeRecord, ShapeId>,
    weak_maps: BTreeMap<ObjectRef, WeakMapState>,
    weak_sets: BTreeMap<ObjectRef, WeakSetState>,
    weak_refs: BTreeMap<ObjectRef, WeakRefState>,
    finalization_registries: BTreeMap<ObjectRef, FinalizationRegistryState>,
    pending_finalization_registries: Vec<ObjectRef>,
    pub(crate) collection_budget_bytes: usize,
}

impl PrimitiveHeap {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn alloc_string(
        &mut self,
        encoding: StringEncoding,
        code_unit_len: u32,
        payload_bytes: &[u8],
        cached_atom: Option<AtomId>,
        lifetime: AllocationLifetime,
    ) -> StringRef {
        let expected_len = expected_string_payload_len(encoding, code_unit_len);
        assert_eq!(
            payload_bytes.len(),
            expected_len,
            "string payload length must match encoding and code unit count"
        );

        let record = if payload_bytes.is_empty() {
            match cached_atom {
                Some(atom) => {
                    PrimitiveStringRecord::with_cached_atom(encoding, code_unit_len, atom)
                }
                None => PrimitiveStringRecord::new(encoding, code_unit_len),
            }
        } else {
            let payload = self.string_payloads.allocate(payload_bytes, lifetime);
            PrimitiveStringRecord::with_payload(encoding, code_unit_len, cached_atom, payload)
        };

        self.strings.allocate(record, lifetime)
    }

    #[inline]
    pub(crate) fn string_allocation_requires_growth(&self, payload_len: usize) -> bool {
        self.strings.allocation_requires_growth()
            || (payload_len != 0
                && self
                    .string_payloads
                    .allocation_requires_growth(SideAllocationClass::for_payload_len(payload_len)))
    }

    #[inline]
    pub(crate) fn string(&self, id: StringRef) -> Option<PrimitiveStringRecord> {
        self.strings.get(id)
    }

    pub(crate) fn string_view(&self, id: StringRef) -> Option<PrimitiveStringView<'_>> {
        let record = self.string(id)?;
        let payload = match record.payload() {
            Some(payload) => self.string_payloads.get(payload)?,
            None if record.code_unit_len() == 0 => &[],
            None => return None,
        };

        Some(PrimitiveStringView::new(record, payload))
    }

    pub(crate) fn string_payload(&self, id: StringRef) -> Option<&[u8]> {
        let record = self.string(id)?;
        let payload = record.payload()?;
        self.string_payloads.get(payload)
    }

    pub(crate) fn free_string(&mut self, id: StringRef) -> Option<PrimitiveStringRecord> {
        let record = self.strings.free(id)?;
        if let Some(payload) = record.payload() {
            self.string_payloads.free(payload);
        }
        Some(record)
    }

    #[inline]
    pub(crate) fn mark_string(&mut self, id: StringRef) -> bool {
        self.strings.mark(id)
    }

    #[inline]
    pub(crate) fn clear_string_marks(&mut self) {
        self.strings.clear_marks();
    }

    #[inline]
    pub(crate) fn string_stats(&self) -> PrimitiveDomainStats {
        self.strings.stats(self.string_payloads.stats())
    }

    pub(crate) fn sweep_unmarked_strings(&mut self) -> usize {
        self.strings.sweep(|record| {
            if let Some(payload) = record.payload() {
                self.string_payloads.free(payload);
            }
        })
    }

    pub(crate) fn cache_string_hash(&mut self, id: StringRef) -> Option<u32> {
        if let Some(hash) = self.string(id)?.cached_hash() {
            return Some(hash);
        }

        let hash = {
            let view = self.string_view(id)?;
            view.compute_hash()
        };

        if self.strings.update(id, |record| {
            record.cached_hash = Some(hash);
        }) {
            Some(hash)
        } else {
            None
        }
    }

    pub(crate) fn memoize_string_atom(&mut self, id: StringRef, atom: AtomId) -> bool {
        self.strings.update(id, |record| match record.cached_atom {
            Some(existing) => debug_assert_eq!(
                existing, atom,
                "string atom cache should not change to a different AtomId"
            ),
            None => {
                record.cached_atom = Some(atom);
            }
        })
    }

    #[inline]
    pub(crate) fn alloc_symbol(
        &mut self,
        description: Option<StringRef>,
        flags: SymbolFlags,
        lifetime: AllocationLifetime,
    ) -> SymbolRef {
        self.symbols
            .allocate(PrimitiveSymbolRecord::new(description, flags), lifetime)
    }

    #[inline]
    pub(crate) fn symbol_allocation_requires_growth(&self) -> bool {
        self.symbols.allocation_requires_growth()
    }

    #[inline]
    pub(crate) fn symbol(&self, id: SymbolRef) -> Option<PrimitiveSymbolRecord> {
        self.symbols.get(id)
    }

    pub(crate) fn symbol_view(&self, id: SymbolRef) -> Option<PrimitiveSymbolView<'_>> {
        let record = self.symbol(id)?;
        let description = match record.description() {
            Some(description) => Some(self.string_view(description)?),
            None => None,
        };
        Some(PrimitiveSymbolView::new(id, record, description))
    }

    #[inline]
    pub(crate) fn free_symbol(&mut self, id: SymbolRef) -> Option<PrimitiveSymbolRecord> {
        self.symbols.free(id)
    }

    #[inline]
    pub(crate) fn mark_symbol(&mut self, id: SymbolRef) -> bool {
        self.symbols.mark(id)
    }

    #[inline]
    pub(crate) fn clear_symbol_marks(&mut self) {
        self.symbols.clear_marks();
    }

    #[inline]
    pub(crate) fn symbol_stats(&self) -> PrimitiveDomainStats {
        self.symbols.stats(SideAllocationStats::default())
    }

    pub(crate) fn sweep_unmarked_symbols(&mut self) -> usize {
        self.symbols.sweep(|_| {})
    }

    pub(crate) fn alloc_bigint(
        &mut self,
        sign: BigIntSign,
        limbs: &[u64],
        lifetime: AllocationLifetime,
    ) -> BigIntRef {
        let normalized_len = normalized_bigint_limb_count(limbs);

        let (sign, limb_storage) = if normalized_len == 0 {
            (BigIntSign::NonNegative, None)
        } else {
            let mut bytes = Vec::with_capacity(normalized_len * std::mem::size_of::<u64>());
            for limb in &limbs[..normalized_len] {
                bytes.extend_from_slice(&limb.to_le_bytes());
            }
            (sign, Some(self.bigint_payloads.allocate(&bytes, lifetime)))
        };

        let record = PrimitiveBigIntRecord::new(
            sign,
            u32::try_from(normalized_len).expect("normalized bigint limb count must fit into u32"),
            limb_storage,
        );
        self.bigints.allocate(record, lifetime)
    }

    #[inline]
    pub(crate) fn bigint_allocation_requires_growth(&self, limbs: &[u64]) -> bool {
        let normalized_len = normalized_bigint_limb_count(limbs);
        self.bigints.allocation_requires_growth()
            || (normalized_len != 0
                && self.bigint_payloads.allocation_requires_growth(
                    SideAllocationClass::for_payload_len(
                        normalized_len * std::mem::size_of::<u64>(),
                    ),
                ))
    }

    #[inline]
    pub(crate) fn bigint(&self, id: BigIntRef) -> Option<PrimitiveBigIntRecord> {
        self.bigints.get(id)
    }

    pub(crate) fn bigint_view(&self, id: BigIntRef) -> Option<PrimitiveBigIntView<'_>> {
        let record = self.bigint(id)?;
        let limb_bytes = match record.limb_storage() {
            Some(storage) => self.bigint_payloads.get(storage)?,
            None if record.limb_count() == 0 => &[],
            None => return None,
        };

        Some(PrimitiveBigIntView::new(record, limb_bytes))
    }

    pub(crate) fn bigint_limbs(&self, id: BigIntRef) -> Option<Vec<u64>> {
        Some(self.bigint_view(id)?.to_limbs())
    }

    pub(crate) fn free_bigint(&mut self, id: BigIntRef) -> Option<PrimitiveBigIntRecord> {
        let record = self.bigints.free(id)?;
        if let Some(storage) = record.limb_storage() {
            self.bigint_payloads.free(storage);
        }
        Some(record)
    }

    #[inline]
    pub(crate) fn mark_bigint(&mut self, id: BigIntRef) -> bool {
        self.bigints.mark(id)
    }

    #[inline]
    pub(crate) fn clear_bigint_marks(&mut self) {
        self.bigints.clear_marks();
    }

    #[inline]
    pub(crate) fn bigint_stats(&self) -> PrimitiveDomainStats {
        self.bigints.stats(self.bigint_payloads.stats())
    }

    #[inline]
    pub(crate) fn alloc_value_cell(
        &mut self,
        lifetime: AllocationLifetime,
    ) -> PrimitiveValueCellRef {
        self.value_cells.allocate(
            PrimitiveValueCellRecord::new(Value::empty_internal_slot(), None),
            lifetime,
        )
    }

    #[inline]
    pub(crate) fn value_cell_allocation_requires_growth(&self) -> bool {
        self.value_cells.allocation_requires_growth()
    }

    #[inline]
    pub(crate) fn value_cell(&self, id: PrimitiveValueCellRef) -> Option<PrimitiveValueCellRecord> {
        self.value_cells.get(id)
    }

    #[inline]
    pub(crate) fn free_value_cell(
        &mut self,
        id: PrimitiveValueCellRef,
    ) -> Option<PrimitiveValueCellRecord> {
        self.value_cells.free(id)
    }

    #[inline]
    pub(crate) fn mark_value_cell(&mut self, id: PrimitiveValueCellRef) -> bool {
        self.value_cells.mark(id)
    }

    #[inline]
    pub(crate) fn clear_value_cell_marks(&mut self) {
        self.value_cells.clear_marks();
    }

    #[inline]
    pub(crate) fn value_cell_stats(&self) -> PrimitiveDomainStats {
        self.value_cells.stats(SideAllocationStats::default())
    }

    #[inline]
    pub(crate) fn alloc_object(
        &mut self,
        record: RuntimeObjectRecord,
        lifetime: AllocationLifetime,
    ) -> ObjectRef {
        self.objects.allocate(record, lifetime)
    }

    #[inline]
    pub(crate) fn object_allocation_requires_growth(&self) -> bool {
        self.objects.allocation_requires_growth()
    }

    #[inline]
    pub(crate) fn object(&self, id: ObjectRef) -> Option<RuntimeObjectRecord> {
        self.objects.get(id)
    }

    #[inline]
    pub(crate) fn function_payload(&self, id: FunctionPayloadRef) -> Option<RuntimeFunctionRecord> {
        self.function_payloads.get(id)
    }

    #[inline]
    pub(crate) fn function_payload_allocation_requires_growth(&self) -> bool {
        self.function_payloads.allocation_requires_growth()
    }

    #[inline]
    pub(crate) fn alloc_function_payload(
        &mut self,
        record: RuntimeFunctionRecord,
        lifetime: AllocationLifetime,
    ) -> FunctionPayloadRef {
        self.function_payloads.allocate(record, lifetime)
    }

    #[inline]
    pub(crate) fn free_function_payload(
        &mut self,
        id: FunctionPayloadRef,
    ) -> Option<RuntimeFunctionRecord> {
        self.function_payloads.free(id)
    }

    #[inline]
    pub(crate) fn set_function_payload_home_object(
        &mut self,
        id: FunctionPayloadRef,
        home_object: Option<ObjectRef>,
    ) -> bool {
        self.function_payloads.update(id, |record| {
            record.home_object = home_object;
        })
    }

    #[inline]
    pub(crate) fn set_function_payload_environment(
        &mut self,
        id: FunctionPayloadRef,
        environment: Option<EnvironmentRef>,
    ) -> bool {
        self.function_payloads.update(id, |record| {
            record.environment = environment;
        })
    }

    #[inline]
    pub(crate) fn set_function_payload_private_env(
        &mut self,
        id: FunctionPayloadRef,
        private_env: Option<EnvironmentRef>,
    ) -> bool {
        self.function_payloads.update(id, |record| {
            record.private_env = private_env;
        })
    }

    #[inline]
    pub(crate) fn free_object(&mut self, id: ObjectRef) -> Option<RuntimeObjectRecord> {
        let record = self.objects.free(id)?;
        if let Some(slots) = record.named_slots() {
            self.object_slots.free(slots);
        }
        if let Some(elements) = record.elements() {
            self.object_slots.free(elements);
        }
        if let Some(function_payload) = record.function_payload() {
            self.function_payloads.free(function_payload);
        }
        if let Some(ordinary_payload) = record.ordinary_payload() {
            self.value_cells.free(ordinary_payload);
        }
        Some(record)
    }

    #[inline]
    pub(crate) fn mark_object(&mut self, id: ObjectRef) -> bool {
        self.objects.mark(id)
    }

    #[inline]
    pub(crate) fn clear_object_marks(&mut self) {
        self.objects.clear_marks();
    }

    #[inline]
    pub(crate) fn object_stats(&self) -> PrimitiveDomainStats {
        self.objects.stats(self.object_slots.stats())
    }

    #[inline]
    pub(crate) fn function_payload_stats(&self) -> PrimitiveDomainStats {
        self.function_payloads.stats(SideAllocationStats::default())
    }

    #[inline]
    pub(crate) fn alloc_suspended_execution(
        &mut self,
        record: RuntimeSuspendedExecutionRecord,
        lifetime: AllocationLifetime,
    ) -> SuspendedExecutionRef {
        self.suspended_executions.allocate(record, lifetime)
    }

    #[inline]
    pub(crate) fn suspended_execution_allocation_requires_growth(&self) -> bool {
        self.suspended_executions.allocation_requires_growth()
    }

    #[inline]
    pub(crate) fn suspended_execution(
        &self,
        id: SuspendedExecutionRef,
    ) -> Option<RuntimeSuspendedExecutionRecord> {
        self.suspended_executions.get(id)
    }

    #[inline]
    pub(crate) fn free_suspended_execution(
        &mut self,
        id: SuspendedExecutionRef,
    ) -> Option<RuntimeSuspendedExecutionRecord> {
        let record = self.suspended_executions.free(id)?;
        if let Some(registers) = record.registers() {
            self.suspended_registers.free(registers);
        }
        Some(record)
    }

    #[inline]
    pub(crate) fn mark_suspended_execution(&mut self, id: SuspendedExecutionRef) -> bool {
        self.suspended_executions.mark(id)
    }

    #[inline]
    pub(crate) fn clear_suspended_execution_marks(&mut self) {
        self.suspended_executions.clear_marks();
    }

    #[inline]
    pub(crate) fn suspended_execution_stats(&self) -> PrimitiveDomainStats {
        self.suspended_executions
            .stats(self.suspended_registers.stats())
    }

    #[inline]
    pub(crate) fn alloc_suspended_registers(
        &mut self,
        slot_count: usize,
        fill: Value,
        lifetime: AllocationLifetime,
    ) -> SuspendedRegistersRef {
        self.suspended_registers
            .allocate(slot_count, fill, lifetime)
    }

    #[inline]
    pub(crate) fn suspended_registers_allocation_requires_growth(&self, slot_count: usize) -> bool {
        self.suspended_registers
            .allocation_requires_growth(slot_count)
    }

    #[inline]
    pub(crate) fn suspended_registers(&self, id: SuspendedRegistersRef) -> Option<&[Value]> {
        self.suspended_registers.get(id)
    }

    pub(crate) fn write_suspended_register(
        &mut self,
        id: SuspendedRegistersRef,
        index: u32,
        value: Value,
    ) -> bool {
        self.suspended_registers.write(id, index, value)
    }

    pub(crate) fn alloc_object_slots(
        &mut self,
        slot_count: usize,
        fill: Value,
        lifetime: AllocationLifetime,
    ) -> ObjectSlotsRef {
        self.object_slots.allocate(slot_count, fill, lifetime)
    }

    #[inline]
    pub(crate) fn object_slots_allocation_requires_growth(&self, slot_count: usize) -> bool {
        self.object_slots.allocation_requires_growth(slot_count)
    }

    #[inline]
    pub(crate) fn object_slots(&self, id: ObjectSlotsRef) -> Option<&[Value]> {
        self.object_slots.get(id)
    }

    pub(crate) fn write_object_slot(
        &mut self,
        id: ObjectSlotsRef,
        index: u32,
        value: Value,
    ) -> bool {
        self.object_slots.write(id, index, value)
    }

    #[inline]
    pub(crate) fn alloc_environment(
        &mut self,
        record: RuntimeEnvironmentRecord,
        lifetime: AllocationLifetime,
    ) -> EnvironmentRef {
        self.environments.allocate(record, lifetime)
    }

    #[inline]
    pub(crate) fn environment_allocation_requires_growth(&self) -> bool {
        self.environments.allocation_requires_growth()
    }

    #[inline]
    pub(crate) fn environment(&self, id: EnvironmentRef) -> Option<RuntimeEnvironmentRecord> {
        self.environments.get(id)
    }

    #[inline]
    pub(crate) fn free_environment(
        &mut self,
        id: EnvironmentRef,
    ) -> Option<RuntimeEnvironmentRecord> {
        let record = self.environments.free(id)?;
        if let Some(slots) = record.slots() {
            self.environment_slots.free(slots);
        }
        Some(record)
    }

    #[inline]
    pub(crate) fn mark_environment(&mut self, id: EnvironmentRef) -> bool {
        self.environments.mark(id)
    }

    #[inline]
    pub(crate) fn clear_environment_marks(&mut self) {
        self.environments.clear_marks();
    }

    #[inline]
    pub(crate) fn environment_stats(&self) -> PrimitiveDomainStats {
        self.environments.stats(self.environment_slots.stats())
    }

    pub(crate) fn alloc_environment_slots(
        &mut self,
        slot_count: usize,
        fill: Value,
        lifetime: AllocationLifetime,
    ) -> EnvironmentSlotsRef {
        self.environment_slots.allocate(slot_count, fill, lifetime)
    }

    #[inline]
    pub(crate) fn environment_slots_allocation_requires_growth(&self, slot_count: usize) -> bool {
        self.environment_slots
            .allocation_requires_growth(slot_count)
    }

    #[inline]
    pub(crate) fn environment_slots(&self, id: EnvironmentSlotsRef) -> Option<&[Value]> {
        self.environment_slots.get(id)
    }

    pub(crate) fn write_environment_slot(
        &mut self,
        id: EnvironmentSlotsRef,
        index: u32,
        value: Value,
    ) -> bool {
        self.environment_slots.write(id, index, value)
    }

    #[inline]
    pub(crate) fn alloc_code(
        &mut self,
        record: RuntimeCodeRecord,
        lifetime: AllocationLifetime,
    ) -> CodeRef {
        self.codes.allocate(record, lifetime)
    }

    #[inline]
    pub(crate) fn code_allocation_requires_growth(&self) -> bool {
        self.codes.allocation_requires_growth()
    }

    #[inline]
    pub(crate) fn code(&self, id: CodeRef) -> Option<RuntimeCodeRecord> {
        self.codes.get(id)
    }

    #[inline]
    pub(crate) fn free_code(&mut self, id: CodeRef) -> Option<RuntimeCodeRecord> {
        let record = self.codes.free(id)?;
        if let Some(constants) = record.constants() {
            self.code_slots.free(constants);
        }
        Some(record)
    }

    #[inline]
    pub(crate) fn mark_code(&mut self, id: CodeRef) -> bool {
        self.codes.mark(id)
    }

    #[inline]
    pub(crate) fn clear_code_marks(&mut self) {
        self.codes.clear_marks();
    }

    #[inline]
    pub(crate) fn code_stats(&self) -> PrimitiveDomainStats {
        self.codes.stats(self.code_slots.stats())
    }

    pub(crate) fn alloc_code_slots(
        &mut self,
        slot_count: usize,
        fill: Value,
        lifetime: AllocationLifetime,
    ) -> CodeSlotsRef {
        self.code_slots.allocate(slot_count, fill, lifetime)
    }

    #[inline]
    pub(crate) fn code_slots_allocation_requires_growth(&self, slot_count: usize) -> bool {
        self.code_slots.allocation_requires_growth(slot_count)
    }

    #[inline]
    pub(crate) fn code_slots(&self, id: CodeSlotsRef) -> Option<&[Value]> {
        self.code_slots.get(id)
    }

    pub(crate) fn write_code_slot(&mut self, id: CodeSlotsRef, index: u32, value: Value) -> bool {
        self.code_slots.write(id, index, value)
    }

    #[inline]
    pub(crate) fn alloc_realm(
        &mut self,
        record: RuntimeRealmRecord,
        lifetime: AllocationLifetime,
    ) -> RealmRef {
        self.realms.allocate(record, lifetime)
    }

    #[inline]
    pub(crate) fn realm_allocation_requires_growth(&self) -> bool {
        self.realms.allocation_requires_growth()
    }

    #[inline]
    pub(crate) fn realm(&self, id: RealmRef) -> Option<RuntimeRealmRecord> {
        self.realms.get(id)
    }

    #[inline]
    pub(crate) fn free_realm(&mut self, id: RealmRef) -> Option<RuntimeRealmRecord> {
        self.realms.free(id)
    }

    #[inline]
    pub(crate) fn mark_realm(&mut self, id: RealmRef) -> bool {
        self.realms.mark(id)
    }

    #[inline]
    pub(crate) fn clear_realm_marks(&mut self) {
        self.realms.clear_marks();
    }

    #[inline]
    pub(crate) fn realm_stats(&self) -> PrimitiveDomainStats {
        self.realms.stats(SideAllocationStats::default())
    }

    #[inline]
    pub(crate) fn alloc_shape(
        &mut self,
        record: RuntimeShapeRecord,
        lifetime: AllocationLifetime,
    ) -> ShapeId {
        self.shapes.allocate(record, lifetime)
    }

    #[inline]
    pub(crate) fn shape_allocation_requires_growth(&self) -> bool {
        self.shapes.allocation_requires_growth()
    }

    #[inline]
    pub(crate) fn shape(&self, id: ShapeId) -> Option<RuntimeShapeRecord> {
        self.shapes.get(id)
    }

    #[inline]
    pub(crate) fn free_shape(&mut self, id: ShapeId) -> Option<RuntimeShapeRecord> {
        self.shapes.free(id)
    }

    #[inline]
    pub(crate) fn mark_shape(&mut self, id: ShapeId) -> bool {
        self.shapes.mark(id)
    }

    #[inline]
    pub(crate) fn clear_shape_marks(&mut self) {
        self.shapes.clear_marks();
    }

    #[inline]
    pub(crate) fn shape_stats(&self) -> PrimitiveDomainStats {
        self.shapes.stats(SideAllocationStats::default())
    }

    #[inline]
    pub(crate) fn init_weak_map(&mut self, owner: ObjectRef) -> bool {
        self.objects.get(owner).is_some()
            && self.weak_maps.insert(owner, WeakMapState::new()).is_none()
    }

    #[inline]
    pub(crate) fn weak_map_get(&self, owner: ObjectRef, key: WeakHeapRef) -> Option<Option<Value>> {
        Some(self.weak_maps.get(&owner)?.get(key))
    }

    pub(crate) fn weak_map_set(
        &mut self,
        owner: ObjectRef,
        key: WeakHeapRef,
        value: Value,
    ) -> bool {
        let Some(state) = self.weak_maps.get_mut(&owner) else {
            return false;
        };
        state.set(key, value);
        true
    }

    pub(crate) fn weak_map_delete(&mut self, owner: ObjectRef, key: WeakHeapRef) -> Option<bool> {
        Some(self.weak_maps.get_mut(&owner)?.delete(key))
    }

    pub(crate) fn weak_map_snapshots(&self) -> Vec<(ObjectRef, Vec<(WeakHeapRef, Value)>)> {
        self.weak_maps
            .iter()
            .map(|(owner, state)| {
                (
                    *owner,
                    state
                        .entries()
                        .iter()
                        .map(|entry| (entry.key(), entry.value()))
                        .collect(),
                )
            })
            .collect()
    }

    #[inline]
    pub(crate) fn init_weak_set(&mut self, owner: ObjectRef) -> bool {
        self.objects.get(owner).is_some()
            && self.weak_sets.insert(owner, WeakSetState::new()).is_none()
    }

    #[inline]
    pub(crate) fn weak_set_contains(&self, owner: ObjectRef, value: WeakHeapRef) -> Option<bool> {
        Some(self.weak_sets.get(&owner)?.contains(value))
    }

    pub(crate) fn weak_set_insert(&mut self, owner: ObjectRef, value: WeakHeapRef) -> bool {
        let Some(state) = self.weak_sets.get_mut(&owner) else {
            return false;
        };
        state.insert(value);
        true
    }

    pub(crate) fn weak_set_delete(&mut self, owner: ObjectRef, value: WeakHeapRef) -> Option<bool> {
        Some(self.weak_sets.get_mut(&owner)?.delete(value))
    }

    #[inline]
    pub(crate) fn init_weak_ref(&mut self, owner: ObjectRef, target: WeakHeapRef) -> bool {
        self.objects.get(owner).is_some()
            && self
                .weak_refs
                .insert(owner, WeakRefState::new(target))
                .is_none()
    }

    #[inline]
    pub(crate) fn weak_ref_target(&self, owner: ObjectRef) -> Option<Option<WeakHeapRef>> {
        Some(self.weak_refs.get(&owner)?.target())
    }

    #[inline]
    pub(crate) fn init_finalization_registry(&mut self, owner: ObjectRef) -> bool {
        self.objects.get(owner).is_some()
            && self
                .finalization_registries
                .insert(owner, FinalizationRegistryState::new())
                .is_none()
    }

    pub(crate) fn finalization_registry_register(
        &mut self,
        owner: ObjectRef,
        target: WeakHeapRef,
        holdings: Value,
        unregister_token: Option<WeakHeapRef>,
    ) -> bool {
        let Some(state) = self.finalization_registries.get_mut(&owner) else {
            return false;
        };
        state.register(target, holdings, unregister_token);
        true
    }

    pub(crate) fn finalization_registry_unregister(
        &mut self,
        owner: ObjectRef,
        unregister_token: WeakHeapRef,
    ) -> Option<bool> {
        Some(
            self.finalization_registries
                .get_mut(&owner)?
                .unregister(unregister_token),
        )
    }

    pub(crate) fn finalization_registry_snapshots(
        &self,
    ) -> Vec<(ObjectRef, Vec<Value>, Vec<Value>)> {
        self.finalization_registries
            .iter()
            .map(|(owner, state)| {
                (
                    *owner,
                    state.cells().iter().map(|cell| cell.holdings()).collect(),
                    state.pending_holdings().to_vec(),
                )
            })
            .collect()
    }

    #[inline]
    pub(crate) fn pending_finalization_registries(&self) -> &[ObjectRef] {
        &self.pending_finalization_registries
    }

    pub(crate) fn take_finalization_cleanup_holdings(&mut self, owner: ObjectRef) -> Vec<Value> {
        let Some(state) = self.finalization_registries.get_mut(&owner) else {
            return Vec::new();
        };
        let holdings = state.take_pending_holdings();
        self.pending_finalization_registries
            .retain(|pending| *pending != owner);
        holdings
    }

    pub(crate) fn set_finalization_cleanup_active(
        &mut self,
        owner: ObjectRef,
        active: bool,
    ) -> bool {
        let Some(state) = self.finalization_registries.get_mut(&owner) else {
            return false;
        };
        state.set_cleanup_active(active);
        true
    }

    #[inline]
    pub(crate) fn finalization_cleanup_pending(&self, owner: ObjectRef) -> Option<bool> {
        Some(self.finalization_registries.get(&owner)?.cleanup_pending())
    }

    #[inline]
    pub(crate) fn is_object_marked(&self, id: ObjectRef) -> bool {
        self.objects.is_marked(id)
    }

    #[inline]
    pub(crate) fn is_symbol_marked(&self, id: SymbolRef) -> bool {
        self.symbols.is_marked(id)
    }

    #[inline]
    pub(crate) fn is_weak_ref_marked(&self, id: WeakHeapRef) -> bool {
        match id {
            WeakHeapRef::Object(object) => self.is_object_marked(object),
            WeakHeapRef::Symbol(symbol) => self.is_symbol_marked(symbol),
        }
    }

    pub(crate) fn sweep_weak_state(&mut self) -> (usize, usize, usize) {
        let mut weak_refs_cleared = 0;
        let mut finalization_cells_queued = 0;
        let objects = &self.objects;
        let symbols = &self.symbols;

        self.pending_finalization_registries.clear();
        self.weak_maps.retain(|owner, _| objects.is_marked(*owner));
        self.weak_sets.retain(|owner, _| objects.is_marked(*owner));
        self.weak_refs.retain(|owner, state| {
            if !objects.is_marked(*owner) {
                return false;
            }
            weak_refs_cleared += usize::from(state.clear_if_dead(|value| match value {
                WeakHeapRef::Object(object) => objects.is_marked(object),
                WeakHeapRef::Symbol(symbol) => symbols.is_marked(symbol),
            }));
            true
        });
        self.finalization_registries.retain(|owner, state| {
            if !objects.is_marked(*owner) {
                return false;
            }
            finalization_cells_queued += state.queue_dead_targets(|value| match value {
                WeakHeapRef::Object(object) => objects.is_marked(object),
                WeakHeapRef::Symbol(symbol) => symbols.is_marked(symbol),
            });
            if state.cleanup_pending() && !state.cleanup_active() {
                self.pending_finalization_registries.push(*owner);
            }
            true
        });
        for state in self.weak_maps.values_mut() {
            state.retain_live_keys(|key| match key {
                WeakHeapRef::Object(object) => objects.is_marked(object),
                WeakHeapRef::Symbol(symbol) => symbols.is_marked(symbol),
            });
        }
        for state in self.weak_sets.values_mut() {
            state.retain_live_values(|value| match value {
                WeakHeapRef::Object(object) => objects.is_marked(object),
                WeakHeapRef::Symbol(symbol) => symbols.is_marked(symbol),
            });
        }

        (
            weak_refs_cleared,
            finalization_cells_queued,
            self.pending_finalization_registries.len(),
        )
    }

    pub(crate) fn set_symbol_description(
        &mut self,
        id: SymbolRef,
        description: Option<StringRef>,
    ) -> bool {
        self.symbols.update(id, |record| {
            record.description = description;
        })
    }

    pub(crate) fn set_value_cell_value(&mut self, id: PrimitiveValueCellRef, value: Value) -> bool {
        self.value_cells.update(id, |record| {
            record.stored_value = value;
        })
    }

    pub(crate) fn set_value_cell_linked_string(
        &mut self,
        id: PrimitiveValueCellRef,
        linked_string: Option<StringRef>,
    ) -> bool {
        self.value_cells.update(id, |record| {
            record.linked_string = linked_string;
        })
    }

    pub(crate) fn set_object_prototype(
        &mut self,
        id: ObjectRef,
        prototype: Option<ObjectRef>,
    ) -> bool {
        self.objects.update(id, |record| {
            record.prototype = prototype;
        })
    }

    pub(crate) fn set_object_shape(&mut self, id: ObjectRef, shape: Option<ShapeId>) -> bool {
        self.objects.update(id, |record| {
            record.shape = shape;
        })
    }

    pub(crate) fn set_object_named_slots(
        &mut self,
        id: ObjectRef,
        named_slots: Option<ObjectSlotsRef>,
    ) -> bool {
        self.objects.update(id, |record| {
            record.named_slots = named_slots;
        })
    }

    pub(crate) fn set_object_elements(
        &mut self,
        id: ObjectRef,
        elements: Option<ObjectSlotsRef>,
    ) -> bool {
        self.objects.update(id, |record| {
            record.elements = elements;
        })
    }

    pub(crate) fn set_object_private_slots(
        &mut self,
        id: ObjectRef,
        private_slots: Option<ObjectSlotsRef>,
    ) -> bool {
        self.objects.update(id, |record| {
            record.private_slots = private_slots;
        })
    }

    pub(crate) fn set_environment_outer(
        &mut self,
        id: EnvironmentRef,
        outer: Option<EnvironmentRef>,
    ) -> bool {
        self.environments.update(id, |record| {
            record.outer = outer;
        })
    }

    pub(crate) fn set_environment_function_object(
        &mut self,
        id: EnvironmentRef,
        function_object: Option<ObjectRef>,
    ) -> bool {
        self.environments.update(id, |record| {
            record.function_object = function_object;
        })
    }

    pub(crate) fn set_environment_this_value(
        &mut self,
        id: EnvironmentRef,
        this_value: Value,
    ) -> bool {
        self.environments.update(id, |record| {
            record.this_value = this_value;
        })
    }

    pub(crate) fn set_environment_new_target(
        &mut self,
        id: EnvironmentRef,
        new_target: Option<ObjectRef>,
    ) -> bool {
        self.environments.update(id, |record| {
            record.new_target = new_target;
        })
    }

    pub(crate) fn set_environment_home_object(
        &mut self,
        id: EnvironmentRef,
        home_object: Option<ObjectRef>,
    ) -> bool {
        self.environments.update(id, |record| {
            record.home_object = home_object;
        })
    }

    pub(crate) fn set_code_parent(&mut self, id: CodeRef, parent: Option<CodeRef>) -> bool {
        self.codes.update(id, |record| {
            record.parent = parent;
        })
    }

    pub(crate) fn set_code_realm(&mut self, id: CodeRef, realm: Option<RealmRef>) -> bool {
        self.codes.update(id, |record| {
            record.realm = realm;
        })
    }

    pub(crate) fn set_realm_global_object(
        &mut self,
        id: RealmRef,
        global_object: Option<ObjectRef>,
    ) -> bool {
        self.realms.update(id, |record| {
            record.global_object = global_object;
        })
    }

    pub(crate) fn set_realm_global_env(
        &mut self,
        id: RealmRef,
        global_env: Option<EnvironmentRef>,
    ) -> bool {
        self.realms.update(id, |record| {
            record.global_env = global_env;
        })
    }

    pub(crate) fn set_realm_bootstrap_code(
        &mut self,
        id: RealmRef,
        bootstrap_code: Option<CodeRef>,
    ) -> bool {
        self.realms.update(id, |record| {
            record.bootstrap_code = bootstrap_code;
        })
    }

    pub(crate) fn set_realm_root_shape(
        &mut self,
        id: RealmRef,
        root_shape: Option<ShapeId>,
    ) -> bool {
        self.realms.update(id, |record| {
            record.root_shape = root_shape;
        })
    }

    pub(crate) fn set_shape_parent(&mut self, id: ShapeId, parent: Option<ShapeId>) -> bool {
        self.shapes.update(id, |record| {
            record.parent = parent;
        })
    }

    pub(crate) fn set_shape_prototype_guard(
        &mut self,
        id: ShapeId,
        prototype_guard: Option<ObjectRef>,
    ) -> bool {
        self.shapes.update(id, |record| {
            record.prototype_guard = prototype_guard;
        })
    }

    pub(crate) fn sweep_unmarked_bigints(&mut self) -> usize {
        self.bigints.sweep(|record| {
            if let Some(storage) = record.limb_storage() {
                self.bigint_payloads.free(storage);
            }
        })
    }

    pub(crate) fn sweep_unmarked_value_cells(&mut self) -> usize {
        self.value_cells.sweep(|_| {})
    }

    pub(crate) fn sweep_unmarked_objects(&mut self) -> usize {
        self.objects.sweep(|record| {
            if let Some(slots) = record.named_slots() {
                self.object_slots.free(slots);
            }
            if let Some(elements) = record.elements() {
                self.object_slots.free(elements);
            }
            if let Some(function_payload) = record.function_payload() {
                self.function_payloads.free(function_payload);
            }
            if let Some(ordinary_payload) = record.ordinary_payload() {
                self.value_cells.free(ordinary_payload);
            }
        })
    }

    pub(crate) fn sweep_unmarked_suspended_executions(&mut self) -> usize {
        self.suspended_executions.sweep(|record| {
            if let Some(registers) = record.registers() {
                self.suspended_registers.free(registers);
            }
        })
    }

    pub(crate) fn sweep_unmarked_environments(&mut self) -> usize {
        self.environments.sweep(|record| {
            if let Some(slots) = record.slots() {
                self.environment_slots.free(slots);
            }
        })
    }

    pub(crate) fn sweep_unmarked_codes(&mut self) -> usize {
        self.codes.sweep(|record| {
            if let Some(constants) = record.constants() {
                self.code_slots.free(constants);
            }
        })
    }

    pub(crate) fn sweep_unmarked_realms(&mut self) -> usize {
        self.realms.sweep(|_| {})
    }

    pub(crate) fn sweep_unmarked_shapes(&mut self) -> usize {
        self.shapes.sweep(|_| {})
    }
}

impl Default for PrimitiveHeap {
    fn default() -> Self {
        Self {
            strings: SlotArena::default(),
            string_payloads: SideAllocator::default(),
            symbols: SlotArena::default(),
            bigints: SlotArena::default(),
            bigint_payloads: SideAllocator::default(),
            value_cells: SlotArena::default(),
            objects: SlotArena::default(),
            function_payloads: SlotArena::default(),
            object_slots: ValueSlotAllocator::default(),
            suspended_executions: SlotArena::default(),
            suspended_registers: ValueSlotAllocator::default(),
            environments: SlotArena::default(),
            environment_slots: ValueSlotAllocator::default(),
            codes: SlotArena::default(),
            code_slots: ValueSlotAllocator::default(),
            realms: SlotArena::default(),
            shapes: SlotArena::default(),
            weak_maps: BTreeMap::new(),
            weak_sets: BTreeMap::new(),
            weak_refs: BTreeMap::new(),
            finalization_registries: BTreeMap::new(),
            pending_finalization_registries: Vec::new(),
            collection_budget_bytes: DEFAULT_COLLECTION_BUDGET_BYTES,
        }
    }
}

trait ArenaHandle: Copy {
    fn from_raw(raw: u32) -> Option<Self>;
    fn get(self) -> u32;
}

impl ArenaHandle for StringRef {
    fn from_raw(raw: u32) -> Option<Self> {
        Self::from_raw(raw)
    }

    fn get(self) -> u32 {
        self.get()
    }
}

impl ArenaHandle for SymbolRef {
    fn from_raw(raw: u32) -> Option<Self> {
        Self::from_raw(raw)
    }

    fn get(self) -> u32 {
        self.get()
    }
}

impl ArenaHandle for BigIntRef {
    fn from_raw(raw: u32) -> Option<Self> {
        Self::from_raw(raw)
    }

    fn get(self) -> u32 {
        self.get()
    }
}

impl ArenaHandle for PrimitiveValueCellRef {
    fn from_raw(raw: u32) -> Option<Self> {
        Self::from_raw(raw)
    }

    fn get(self) -> u32 {
        self.get()
    }
}

impl ArenaHandle for ObjectRef {
    fn from_raw(raw: u32) -> Option<Self> {
        Self::from_raw(raw)
    }

    fn get(self) -> u32 {
        self.get()
    }
}

impl ArenaHandle for EnvironmentRef {
    fn from_raw(raw: u32) -> Option<Self> {
        Self::from_raw(raw)
    }

    fn get(self) -> u32 {
        self.get()
    }
}

impl ArenaHandle for CodeRef {
    fn from_raw(raw: u32) -> Option<Self> {
        Self::from_raw(raw)
    }

    fn get(self) -> u32 {
        self.get()
    }
}

impl ArenaHandle for RealmRef {
    fn from_raw(raw: u32) -> Option<Self> {
        Self::from_raw(raw)
    }

    fn get(self) -> u32 {
        self.get()
    }
}

impl ArenaHandle for SuspendedExecutionRef {
    fn from_raw(raw: u32) -> Option<Self> {
        Self::from_raw(raw)
    }

    fn get(self) -> u32 {
        self.get()
    }
}

impl ArenaHandle for ShapeId {
    fn from_raw(raw: u32) -> Option<Self> {
        Self::from_raw(raw)
    }

    fn get(self) -> u32 {
        self.get()
    }
}

impl ArenaHandle for ObjectSlotsRef {
    fn from_raw(raw: u32) -> Option<Self> {
        Self::from_raw(raw)
    }

    fn get(self) -> u32 {
        self.get()
    }
}

impl ArenaHandle for EnvironmentSlotsRef {
    fn from_raw(raw: u32) -> Option<Self> {
        Self::from_raw(raw)
    }

    fn get(self) -> u32 {
        self.get()
    }
}

impl ArenaHandle for CodeSlotsRef {
    fn from_raw(raw: u32) -> Option<Self> {
        Self::from_raw(raw)
    }

    fn get(self) -> u32 {
        self.get()
    }
}

impl ArenaHandle for FunctionPayloadRef {
    fn from_raw(raw: u32) -> Option<Self> {
        Self::from_raw(raw)
    }

    fn get(self) -> u32 {
        self.get()
    }
}

impl ArenaHandle for SuspendedRegistersRef {
    fn from_raw(raw: u32) -> Option<Self> {
        Self::from_raw(raw)
    }

    fn get(self) -> u32 {
        self.get()
    }
}

struct SlotArena<Record, Handle> {
    pages: Vec<SlotPage<Record>>,
    pages_with_available_slots: usize,
    first_page_with_available_slot: Option<usize>,
    marker: PhantomData<Handle>,
}

impl<Record, Handle> Default for SlotArena<Record, Handle> {
    fn default() -> Self {
        Self {
            pages: Vec::new(),
            pages_with_available_slots: 0,
            first_page_with_available_slot: None,
            marker: PhantomData,
        }
    }
}

impl<Record: Copy, Handle: ArenaHandle> SlotArena<Record, Handle> {
    fn allocation_requires_growth(&self) -> bool {
        !self.pages.is_empty() && self.pages_with_available_slots == 0
    }

    fn allocate(&mut self, record: Record, lifetime: AllocationLifetime) -> Handle {
        if let Some(page_index) = self.first_page_with_available_slot {
            let slot_index = self.pages[page_index]
                .allocate(record, lifetime)
                .expect("page availability hint must point at a page with a free slot");
            if !self.pages[page_index].has_available_slot() {
                self.pages_with_available_slots -= 1;
                self.first_page_with_available_slot =
                    self.find_available_page_after(page_index + 1);
            }
            return make_handle::<Handle>(page_index, slot_index);
        }

        let mut page = SlotPage::new();
        let slot_index = page
            .allocate(record, lifetime)
            .expect("fresh primitive page must accept its first record");
        let page_has_available_slot = page.has_available_slot();
        self.pages.push(page);
        let page_index = self.pages.len() - 1;
        if page_has_available_slot {
            self.pages_with_available_slots += 1;
            if self.first_page_with_available_slot.is_none() {
                self.first_page_with_available_slot = Some(page_index);
            }
        }
        make_handle::<Handle>(page_index, slot_index)
    }

    fn get(&self, handle: Handle) -> Option<Record> {
        let (page_index, slot_index) = locate::<Handle>(handle)?;
        self.pages.get(page_index)?.get(slot_index)
    }

    fn free(&mut self, handle: Handle) -> Option<Record> {
        let (page_index, slot_index) = locate::<Handle>(handle)?;
        let (was_available, is_available, record) = {
            let page = self.pages.get_mut(page_index)?;
            let was_available = page.has_available_slot();
            let record = page.free(slot_index)?;
            (was_available, page.has_available_slot(), record)
        };
        self.update_page_availability(page_index, was_available, is_available);
        Some(record)
    }

    fn mark(&mut self, handle: Handle) -> bool {
        let Some((page_index, slot_index)) = locate::<Handle>(handle) else {
            return false;
        };
        match self.pages.get_mut(page_index) {
            Some(page) => page.mark(slot_index),
            None => false,
        }
    }

    fn is_marked(&self, handle: Handle) -> bool {
        let Some((page_index, slot_index)) = locate::<Handle>(handle) else {
            return false;
        };
        self.pages
            .get(page_index)
            .is_some_and(|page| page.is_marked(slot_index))
    }

    fn clear_marks(&mut self) {
        for page in &mut self.pages {
            page.clear_marks();
        }
    }

    fn stats(&self, side_allocations: SideAllocationStats) -> PrimitiveDomainStats {
        let mut stats = PrimitiveDomainStats {
            pages: self.pages.len(),
            side_allocations,
            ..PrimitiveDomainStats::default()
        };

        for page in &self.pages {
            stats.occupied_slots += page.occupied;
            stats.reusable_slots += page.free_list.len();
            stats.marked_slots += page.marked_slots();
            stats.default_slots += page.default_slots();
            stats.long_lived_slots += page.long_lived_slots();
        }

        stats
    }

    fn sweep(&mut self, mut reclaim: impl FnMut(Record)) -> usize {
        let mut reclaimed = 0;

        for page_index in 0..self.pages.len() {
            let (was_available, is_available, page_reclaimed) = {
                let page = &mut self.pages[page_index];
                let was_available = page.has_available_slot();
                let page_reclaimed = page.sweep(&mut reclaim);
                (was_available, page.has_available_slot(), page_reclaimed)
            };
            self.update_page_availability(page_index, was_available, is_available);
            reclaimed += page_reclaimed;
        }

        reclaimed
    }

    fn update(&mut self, handle: Handle, mut update: impl FnMut(&mut Record)) -> bool {
        let Some((page_index, slot_index)) = locate::<Handle>(handle) else {
            return false;
        };

        match self.pages.get_mut(page_index) {
            Some(page) => page.update(slot_index, &mut update),
            None => false,
        }
    }

    fn find_available_page_after(&self, start: usize) -> Option<usize> {
        self.pages
            .iter()
            .enumerate()
            .skip(start)
            .find_map(|(page_index, page)| page.has_available_slot().then_some(page_index))
    }

    fn update_page_availability(
        &mut self,
        page_index: usize,
        was_available: bool,
        is_available: bool,
    ) {
        match (was_available, is_available) {
            (false, true) => {
                self.pages_with_available_slots += 1;
                if self
                    .first_page_with_available_slot
                    .is_none_or(|first_page| page_index < first_page)
                {
                    self.first_page_with_available_slot = Some(page_index);
                }
            }
            (true, false) => {
                self.pages_with_available_slots -= 1;
                if self.first_page_with_available_slot == Some(page_index) {
                    self.first_page_with_available_slot =
                        self.find_available_page_after(page_index + 1);
                }
            }
            _ => {}
        }

        debug_assert_eq!(
            self.pages_with_available_slots,
            self.pages
                .iter()
                .filter(|page| page.has_available_slot())
                .count(),
            "slot arena availability metadata must track page capacity exactly"
        );
        debug_assert_eq!(
            self.first_page_with_available_slot,
            self.pages
                .iter()
                .enumerate()
                .find_map(|(index, page)| page.has_available_slot().then_some(index)),
            "slot arena availability hint must target the first page with capacity"
        );
    }
}

struct SlotPage<Record> {
    slots: [Option<Record>; PRIMITIVE_SLOTS_PER_PAGE],
    marks: [bool; PRIMITIVE_SLOTS_PER_PAGE],
    lifetimes: [AllocationLifetime; PRIMITIVE_SLOTS_PER_PAGE],
    occupied: usize,
    next_uninitialized: usize,
    free_list: Vec<u16>,
}

impl<Record: Copy> SlotPage<Record> {
    fn new() -> Self {
        Self {
            slots: from_fn(|_| None),
            marks: [false; PRIMITIVE_SLOTS_PER_PAGE],
            lifetimes: [AllocationLifetime::Default; PRIMITIVE_SLOTS_PER_PAGE],
            occupied: 0,
            next_uninitialized: 0,
            free_list: Vec::new(),
        }
    }

    fn allocate(&mut self, record: Record, lifetime: AllocationLifetime) -> Option<usize> {
        let slot_index = if let Some(slot) = self.free_list.pop() {
            usize::from(slot)
        } else if self.next_uninitialized < PRIMITIVE_SLOTS_PER_PAGE {
            let slot_index = self.next_uninitialized;
            self.next_uninitialized += 1;
            slot_index
        } else {
            return None;
        };

        self.slots[slot_index] = Some(record);
        self.marks[slot_index] = false;
        self.lifetimes[slot_index] = lifetime;
        self.occupied += 1;
        Some(slot_index)
    }

    fn has_available_slot(&self) -> bool {
        !self.free_list.is_empty() || self.next_uninitialized < PRIMITIVE_SLOTS_PER_PAGE
    }

    fn get(&self, slot_index: usize) -> Option<Record> {
        self.slots.get(slot_index).copied().flatten()
    }

    fn update(&mut self, slot_index: usize, update: &mut impl FnMut(&mut Record)) -> bool {
        match self.slots.get_mut(slot_index) {
            Some(Some(record)) => {
                update(record);
                true
            }
            _ => false,
        }
    }

    fn free(&mut self, slot_index: usize) -> Option<Record> {
        let record = self.slots.get_mut(slot_index)?.take()?;
        self.marks[slot_index] = false;
        self.lifetimes[slot_index] = AllocationLifetime::Default;
        self.occupied -= 1;
        self.free_list
            .push(u16::try_from(slot_index).expect("primitive page slot index must fit into u16"));
        Some(record)
    }

    fn mark(&mut self, slot_index: usize) -> bool {
        match self.slots.get(slot_index) {
            Some(Some(_)) => {
                let was_marked = self.marks[slot_index];
                self.marks[slot_index] = true;
                !was_marked
            }
            _ => false,
        }
    }

    #[inline]
    fn is_marked(&self, slot_index: usize) -> bool {
        self.slots.get(slot_index).is_some_and(Option::is_some) && self.marks[slot_index]
    }

    fn clear_marks(&mut self) {
        for slot_index in 0..self.next_uninitialized {
            if self.slots[slot_index].is_some() {
                self.marks[slot_index] = false;
            }
        }
    }

    fn sweep(&mut self, reclaim: &mut impl FnMut(Record)) -> usize {
        let mut reclaimed = 0;

        for slot_index in 0..self.next_uninitialized {
            match self.slots[slot_index] {
                Some(record) if self.marks[slot_index] => {
                    self.marks[slot_index] = false;
                }
                Some(record) => {
                    self.slots[slot_index] = None;
                    self.marks[slot_index] = false;
                    self.lifetimes[slot_index] = AllocationLifetime::Default;
                    self.occupied -= 1;
                    self.free_list.push(
                        u16::try_from(slot_index)
                            .expect("primitive page slot index must fit into u16"),
                    );
                    reclaim(record);
                    reclaimed += 1;
                }
                None => {}
            }
        }

        reclaimed
    }

    fn marked_slots(&self) -> usize {
        (0..self.next_uninitialized)
            .filter(|&slot_index| self.slots[slot_index].is_some() && self.marks[slot_index])
            .count()
    }

    fn default_slots(&self) -> usize {
        self.count_slots_with_lifetime(AllocationLifetime::Default)
    }

    fn long_lived_slots(&self) -> usize {
        self.count_slots_with_lifetime(AllocationLifetime::LongLived)
    }

    fn count_slots_with_lifetime(&self, lifetime: AllocationLifetime) -> usize {
        (0..self.next_uninitialized)
            .filter(|&slot_index| {
                self.slots[slot_index].is_some() && self.lifetimes[slot_index] == lifetime
            })
            .count()
    }
}

#[derive(Default)]
struct SideAllocator {
    slots: Vec<SideAllocationSlot>,
    free_by_class: BTreeMap<SideAllocationClass, Vec<u32>>,
}

struct SideAllocationSlot {
    class: SideAllocationClass,
    lifetime: AllocationLifetime,
    payload_len: usize,
    occupied: bool,
    bytes: Box<[u8]>,
}

impl SideAllocator {
    fn allocation_requires_growth(&self, class: SideAllocationClass) -> bool {
        !self.slots.is_empty()
            && self
                .free_by_class
                .get(&class)
                .is_none_or(std::vec::Vec::is_empty)
    }

    fn allocate(&mut self, payload: &[u8], lifetime: AllocationLifetime) -> SideAllocationRef {
        let class = SideAllocationClass::for_payload_len(payload.len());

        if let Some(id) = self
            .free_by_class
            .get_mut(&class)
            .and_then(std::vec::Vec::pop)
        {
            let slot = &mut self.slots[(id - 1) as usize];
            slot.lifetime = lifetime;
            slot.payload_len = payload.len();
            slot.occupied = true;
            slot.bytes[..payload.len()].copy_from_slice(payload);
            return SideAllocationRef::from_raw(id).unwrap();
        }

        let mut bytes = vec![0_u8; class.slot_bytes()].into_boxed_slice();
        bytes[..payload.len()].copy_from_slice(payload);
        self.slots.push(SideAllocationSlot {
            class,
            lifetime,
            payload_len: payload.len(),
            occupied: true,
            bytes,
        });
        SideAllocationRef::from_raw(
            u32::try_from(self.slots.len())
                .expect("side allocation handle count must fit into u32"),
        )
        .unwrap()
    }

    fn get(&self, id: SideAllocationRef) -> Option<&[u8]> {
        let slot = self.slots.get((id.get() - 1) as usize)?;
        if slot.occupied {
            Some(&slot.bytes[..slot.payload_len])
        } else {
            None
        }
    }

    fn free(&mut self, id: SideAllocationRef) -> bool {
        let Some(slot) = self.slots.get_mut((id.get() - 1) as usize) else {
            return false;
        };
        if !slot.occupied {
            return false;
        }

        slot.occupied = false;
        slot.payload_len = 0;
        self.free_by_class
            .entry(slot.class)
            .or_default()
            .push(id.get());
        true
    }

    fn stats(&self) -> SideAllocationStats {
        let mut stats = SideAllocationStats::default();

        for slot in &self.slots {
            stats.reserved_bytes += slot.class.slot_bytes();
            if slot.occupied {
                stats.live_allocations += 1;
                stats.live_payload_bytes += slot.payload_len;
                match slot.lifetime {
                    AllocationLifetime::Default => stats.default_allocations += 1,
                    AllocationLifetime::LongLived => stats.long_lived_allocations += 1,
                }
            } else {
                stats.reusable_allocations += 1;
                stats.reusable_reserved_bytes += slot.class.slot_bytes();
            }
        }

        stats
    }
}

struct ValueSlotAllocator<Handle> {
    slots: Vec<ValueSlotBufferSlot>,
    free_by_len: BTreeMap<usize, Vec<u32>>,
    marker: PhantomData<Handle>,
}

impl<Handle> Default for ValueSlotAllocator<Handle> {
    fn default() -> Self {
        Self {
            slots: Vec::new(),
            free_by_len: BTreeMap::new(),
            marker: PhantomData,
        }
    }
}

struct ValueSlotBufferSlot {
    lifetime: AllocationLifetime,
    occupied: bool,
    values: Box<[Value]>,
}

impl<Handle: ArenaHandle> ValueSlotAllocator<Handle> {
    fn allocation_requires_growth(&self, slot_count: usize) -> bool {
        !self.slots.is_empty()
            && self
                .free_by_len
                .get(&slot_count)
                .is_none_or(std::vec::Vec::is_empty)
    }

    fn allocate(&mut self, slot_count: usize, fill: Value, lifetime: AllocationLifetime) -> Handle {
        if let Some(id) = self
            .free_by_len
            .get_mut(&slot_count)
            .and_then(std::vec::Vec::pop)
        {
            let slot = &mut self.slots[(id - 1) as usize];
            slot.lifetime = lifetime;
            slot.occupied = true;
            for value in &mut slot.values {
                *value = fill;
            }
            return Handle::from_raw(id).unwrap();
        }

        self.slots.push(ValueSlotBufferSlot {
            lifetime,
            occupied: true,
            values: vec![fill; slot_count].into_boxed_slice(),
        });
        Handle::from_raw(
            u32::try_from(self.slots.len()).expect("value slot buffer count must fit into u32"),
        )
        .unwrap()
    }

    fn get(&self, id: Handle) -> Option<&[Value]> {
        let slot = self.slots.get((id.get() - 1) as usize)?;
        if slot.occupied {
            Some(&slot.values)
        } else {
            None
        }
    }

    fn write(&mut self, id: Handle, index: u32, value: Value) -> bool {
        let Some(slot) = self.slots.get_mut((id.get() - 1) as usize) else {
            return false;
        };
        if !slot.occupied {
            return false;
        }
        let Some(target) = slot.values.get_mut(index as usize) else {
            return false;
        };
        *target = value;
        true
    }

    fn free(&mut self, id: Handle) -> bool {
        let Some(slot) = self.slots.get_mut((id.get() - 1) as usize) else {
            return false;
        };
        if !slot.occupied {
            return false;
        }

        slot.occupied = false;
        self.free_by_len
            .entry(slot.values.len())
            .or_default()
            .push(id.get());
        true
    }

    fn stats(&self) -> SideAllocationStats {
        let mut stats = SideAllocationStats::default();
        let value_bytes = std::mem::size_of::<Value>();

        for slot in &self.slots {
            let reserved_bytes = slot.values.len() * value_bytes;
            stats.reserved_bytes += reserved_bytes;
            if slot.occupied {
                stats.live_allocations += 1;
                stats.live_payload_bytes += reserved_bytes;
                match slot.lifetime {
                    AllocationLifetime::Default => stats.default_allocations += 1,
                    AllocationLifetime::LongLived => stats.long_lived_allocations += 1,
                }
            } else {
                stats.reusable_allocations += 1;
                stats.reusable_reserved_bytes += reserved_bytes;
            }
        }

        stats
    }
}

fn make_handle<Handle: ArenaHandle>(page_index: usize, slot_index: usize) -> Handle {
    let raw = u32::try_from(page_index * PRIMITIVE_SLOTS_PER_PAGE + slot_index + 1)
        .expect("primitive arena handle IDs must fit into u32");
    Handle::from_raw(raw).expect("primitive arena handle IDs must stay non-zero")
}

fn locate<Handle: ArenaHandle>(handle: Handle) -> Option<(usize, usize)> {
    let raw = handle.get().checked_sub(1)? as usize;
    Some((
        raw / PRIMITIVE_SLOTS_PER_PAGE,
        raw % PRIMITIVE_SLOTS_PER_PAGE,
    ))
}

fn normalized_bigint_limb_count(limbs: &[u64]) -> usize {
    limbs
        .iter()
        .rposition(|limb| *limb != 0)
        .map_or(0, |index| index + 1)
}

fn expected_string_payload_len(encoding: StringEncoding, code_unit_len: u32) -> usize {
    match encoding {
        StringEncoding::Latin1 => code_unit_len as usize,
        StringEncoding::Utf16 => (code_unit_len as usize)
            .checked_mul(2)
            .expect("Phase 2 UTF-16 strings must fit in addressable side storage"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lyng_js_common::AtomId;

    #[test]
    fn string_slots_and_side_allocations_reuse_freed_storage() {
        let mut heap = PrimitiveHeap::new();
        let first = heap.alloc_string(
            StringEncoding::Latin1,
            3,
            b"one",
            Some(AtomId::from_raw(7)),
            AllocationLifetime::Default,
        );
        let second = heap.alloc_string(
            StringEncoding::Latin1,
            3,
            b"two",
            None,
            AllocationLifetime::Default,
        );

        let first_payload = heap.string(first).unwrap().payload().unwrap();
        let freed = heap.free_string(first).unwrap();
        let replacement = heap.alloc_string(
            StringEncoding::Latin1,
            3,
            b"red",
            None,
            AllocationLifetime::Default,
        );

        assert_eq!(freed.cached_atom(), Some(AtomId::from_raw(7)));
        assert_eq!(replacement, first);
        assert_eq!(heap.string_payload(replacement), Some(&b"red"[..]));
        assert_eq!(
            heap.string(replacement).unwrap().payload(),
            Some(first_payload)
        );
        assert_eq!(heap.string(second).unwrap().code_unit_len(), 3);
    }

    #[test]
    fn lifetime_hints_flow_through_domain_and_side_allocation_stats() {
        let mut heap = PrimitiveHeap::new();
        let description = heap.alloc_string(
            StringEncoding::Latin1,
            4,
            b"desc",
            None,
            AllocationLifetime::LongLived,
        );
        let _ = heap.alloc_symbol(
            Some(description),
            SymbolFlags::well_known(),
            AllocationLifetime::LongLived,
        );
        let _ = heap.alloc_bigint(
            BigIntSign::Negative,
            &[9, 8, 0],
            AllocationLifetime::Default,
        );

        let string_stats = heap.string_stats();
        let symbol_stats = heap.symbol_stats();
        let bigint_stats = heap.bigint_stats();

        assert_eq!(string_stats.long_lived_slots, 1);
        assert_eq!(string_stats.side_allocations.long_lived_allocations, 1);
        assert_eq!(symbol_stats.long_lived_slots, 1);
        assert_eq!(bigint_stats.default_slots, 1);
        assert_eq!(bigint_stats.side_allocations.default_allocations, 1);
    }

    #[test]
    fn symbol_pages_grow_and_mark_bits_remain_domain_local() {
        let mut heap = PrimitiveHeap::new();
        let mut last = None;

        for _ in 0..=PRIMITIVE_SLOTS_PER_PAGE {
            last =
                Some(heap.alloc_symbol(None, SymbolFlags::ordinary(), AllocationLifetime::Default));
        }

        let first = SymbolRef::from_raw(1).unwrap();
        let last = last.unwrap();

        assert_eq!(heap.symbol_stats().pages, 2);
        assert!(heap.mark_symbol(first));
        assert!(heap.mark_symbol(last));
        assert_eq!(heap.symbol_stats().marked_slots, 2);

        heap.clear_symbol_marks();
        assert_eq!(heap.symbol_stats().marked_slots, 0);
    }

    #[test]
    fn slot_arena_capacity_metadata_tracks_cross_page_reuse() {
        let mut heap = PrimitiveHeap::new();
        let mut handles = Vec::new();

        for _ in 0..(PRIMITIVE_SLOTS_PER_PAGE * 2) {
            handles.push(heap.alloc_symbol(
                None,
                SymbolFlags::ordinary(),
                AllocationLifetime::Default,
            ));
        }

        let freed = handles[0];
        assert!(heap.symbol_allocation_requires_growth());
        assert!(heap.free_symbol(freed).is_some());
        assert!(!heap.symbol_allocation_requires_growth());

        let replacement =
            heap.alloc_symbol(None, SymbolFlags::ordinary(), AllocationLifetime::Default);
        assert_eq!(replacement, freed);
        assert!(heap.symbol_allocation_requires_growth());
        assert_eq!(heap.symbol_stats().pages, 2);
    }

    #[test]
    fn bigint_normalizes_zero_and_round_trips_limb_storage() {
        let mut heap = PrimitiveHeap::new();
        let zero = heap.alloc_bigint(
            BigIntSign::Negative,
            &[0, 0, 0],
            AllocationLifetime::Default,
        );
        let value = heap.alloc_bigint(
            BigIntSign::Negative,
            &[1, 2, 0, 0],
            AllocationLifetime::LongLived,
        );

        assert_eq!(heap.bigint(zero).unwrap().sign(), BigIntSign::NonNegative);
        assert_eq!(heap.bigint(zero).unwrap().limb_count(), 0);
        assert_eq!(heap.bigint_limbs(zero), Some(Vec::new()));

        assert_eq!(heap.bigint(value).unwrap().sign(), BigIntSign::Negative);
        assert_eq!(heap.bigint(value).unwrap().limb_count(), 2);
        assert_eq!(heap.bigint_limbs(value), Some(vec![1, 2]));
    }
}
