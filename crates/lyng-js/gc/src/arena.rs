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
use std::collections::BTreeMap;

mod records;
mod storage;
mod store_helpers;
mod weak_state;

pub use records::*;
use storage::*;

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

    pub(crate) fn latin1_concat_string_payload_len(
        &self,
        left: StringRef,
        right: StringRef,
    ) -> Option<usize> {
        let left = self.string(left)?;
        let right = self.string(right)?;
        if left.encoding() != StringEncoding::Latin1 || right.encoding() != StringEncoding::Latin1 {
            return None;
        }
        usize::try_from(left.code_unit_len())
            .ok()?
            .checked_add(usize::try_from(right.code_unit_len()).ok()?)
    }

    pub(crate) fn utf16_concat_string_payload_len(
        &self,
        left: StringRef,
        right: StringRef,
    ) -> Option<usize> {
        let left = self.string(left)?;
        let right = self.string(right)?;
        if left.encoding() == StringEncoding::Latin1 && right.encoding() == StringEncoding::Latin1 {
            return None;
        }
        usize::try_from(left.code_unit_len().checked_add(right.code_unit_len())?)
            .ok()?
            .checked_mul(2)
    }

    pub(crate) fn alloc_latin1_concat_string(
        &mut self,
        left: StringRef,
        right: StringRef,
        lifetime: AllocationLifetime,
    ) -> Option<StringRef> {
        let left_record = self.string(left)?;
        let right_record = self.string(right)?;
        if left_record.encoding() != StringEncoding::Latin1
            || right_record.encoding() != StringEncoding::Latin1
        {
            return None;
        }

        let code_unit_len = left_record
            .code_unit_len()
            .checked_add(right_record.code_unit_len())?;
        if left_record.code_unit_len() == 0 {
            return Some(right);
        }
        if right_record.code_unit_len() == 0 {
            return Some(left);
        }

        let payload = self.string_payloads.allocate_concat(
            left_record.payload()?,
            usize::try_from(left_record.code_unit_len()).ok()?,
            right_record.payload()?,
            usize::try_from(right_record.code_unit_len()).ok()?,
            lifetime,
        )?;
        let record = PrimitiveStringRecord::with_payload(
            StringEncoding::Latin1,
            code_unit_len,
            None,
            payload,
        );
        Some(self.strings.allocate(record, lifetime))
    }

    pub(crate) fn alloc_utf16_concat_string(
        &mut self,
        left: StringRef,
        right: StringRef,
        lifetime: AllocationLifetime,
    ) -> Option<StringRef> {
        let left_record = self.string(left)?;
        let right_record = self.string(right)?;
        if left_record.encoding() == StringEncoding::Latin1
            && right_record.encoding() == StringEncoding::Latin1
        {
            return None;
        }

        let code_unit_len = left_record
            .code_unit_len()
            .checked_add(right_record.code_unit_len())?;
        if left_record.code_unit_len() == 0 {
            return Some(right);
        }
        if right_record.code_unit_len() == 0 {
            return Some(left);
        }

        let payload = self.string_payloads.allocate_utf16_concat(
            left_record.payload()?,
            left_record.encoding(),
            left_record.code_unit_len(),
            right_record.payload()?,
            right_record.encoding(),
            right_record.code_unit_len(),
            lifetime,
        )?;
        let record = PrimitiveStringRecord::with_payload(
            StringEncoding::Utf16,
            code_unit_len,
            None,
            payload,
        );
        Some(self.strings.allocate(record, lifetime))
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
