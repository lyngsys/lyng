use crate::nursery::{Nursery, NurseryDomain};
use crate::{
    card_table::{CardDomain, CardKey, CardTable},
    collection::{DEFAULT_COLLECTION_BUDGET_BYTES, DEFAULT_MAJOR_MARK_SLICE_BUDGET},
    rooting::PrimitiveIncrementalMark,
    weak::FinalizationRegistryState,
    weak::WeakMapState,
    weak::WeakRefState,
    weak::WeakSetState,
    NurseryStats, PrimitiveAllocationProfile, PrimitiveStringRecord, PrimitiveStringView,
    StringEncoding, WeakHeapRef,
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
use storage::{SideAllocator, SlotArena, ValueSlotAllocator, YoungSweepStats};

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
    pub(crate) major_mark_slice_budget: usize,
    pub(crate) last_major_mark_slices: usize,
    pub(crate) last_major_mark_work_items: usize,
    pub(crate) last_major_max_mark_slice_work_items: usize,
    pub(crate) last_major_total_mark_pause_ns: u128,
    pub(crate) last_major_max_mark_pause_ns: u128,
    pub(crate) last_major_mark_finish_work_items: usize,
    pub(crate) last_major_mark_finish_pause_ns: u128,
    pub(crate) last_major_gray_work_items_after_finish: usize,
    pub(crate) active_major_mark: Option<PrimitiveIncrementalMark>,
    nursery: Nursery,
    card_table: CardTable,
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

        let record_bytes = std::mem::size_of::<PrimitiveStringRecord>();
        let record_generation =
            self.allocation_generation(NurseryDomain::String, record_bytes, lifetime);

        let record = if payload_bytes.is_empty() {
            cached_atom.map_or_else(
                || PrimitiveStringRecord::new(encoding, code_unit_len),
                |atom| PrimitiveStringRecord::with_cached_atom(encoding, code_unit_len, atom),
            )
        } else {
            let payload_generation = self.allocation_generation(
                NurseryDomain::StringPayload,
                payload_bytes.len(),
                lifetime,
            );
            let payload =
                self.string_payloads
                    .allocate(payload_bytes, lifetime, payload_generation);
            PrimitiveStringRecord::with_payload(encoding, code_unit_len, cached_atom, payload)
        };

        self.strings.allocate(record, lifetime, record_generation)
    }

    #[inline]
    pub const fn nursery_stats(&self) -> NurseryStats {
        self.nursery.stats()
    }

    #[inline]
    pub const fn allocation_profile(&self) -> PrimitiveAllocationProfile {
        self.nursery.profile()
    }

    #[inline]
    pub fn set_nursery_capacity_bytes(&mut self, bytes: usize) {
        self.nursery.set_capacity_bytes(bytes);
    }

    #[inline]
    pub fn set_nursery_tenuring_threshold(&mut self, threshold: u8) {
        self.nursery.set_tenuring_threshold(threshold);
    }

    #[inline]
    pub(crate) const fn nursery_tenuring_threshold(&self) -> u8 {
        self.nursery.tenuring_threshold()
    }

    #[inline]
    pub(crate) const fn nursery_can_fit(&self, bytes: usize) -> bool {
        self.nursery.can_fit(bytes)
    }

    #[inline]
    pub(crate) const fn should_allocate_in_nursery(
        domain: NurseryDomain,
        lifetime: AllocationLifetime,
    ) -> bool {
        Nursery::is_nursery_eligible(domain, lifetime)
    }

    const fn allocation_generation(
        &mut self,
        domain: NurseryDomain,
        bytes: usize,
        lifetime: AllocationLifetime,
    ) -> HeapGeneration {
        if Self::should_allocate_in_nursery(domain, lifetime) && self.nursery.reserve(bytes) {
            HeapGeneration::Young
        } else {
            self.nursery.note_old_allocation();
            HeapGeneration::Old
        }
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

    #[inline]
    pub(crate) const fn string_record_allocation_requires_growth(&self) -> bool {
        self.strings.allocation_requires_growth()
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

        let record =
            PrimitiveStringRecord::with_cons(StringEncoding::Latin1, code_unit_len, left, right);
        let generation = self.allocation_generation(
            NurseryDomain::String,
            std::mem::size_of::<PrimitiveStringRecord>(),
            lifetime,
        );
        let id = self.strings.allocate(record, lifetime, generation);
        self.mark_string_card_if_old_points_to_young(id, record);
        Some(id)
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

        let record =
            PrimitiveStringRecord::with_cons(StringEncoding::Utf16, code_unit_len, left, right);
        let generation = self.allocation_generation(
            NurseryDomain::String,
            std::mem::size_of::<PrimitiveStringRecord>(),
            lifetime,
        );
        let id = self.strings.allocate(record, lifetime, generation);
        self.mark_string_card_if_old_points_to_young(id, record);
        Some(id)
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
        match record.payload() {
            Some(payload) => Some(PrimitiveStringView::new(
                record,
                self.string_payloads.get(payload)?,
            )),
            None if record.code_unit_len() == 0 => Some(PrimitiveStringView::new(record, &[])),
            None if record.is_cons() => Some(PrimitiveStringView::with_heap(record, None, self)),
            None => None,
        }
    }

    pub(crate) fn flatten_string_payload(&self, record: PrimitiveStringRecord) -> Option<Vec<u8>> {
        let mut payload = Vec::with_capacity(expected_string_payload_len(
            record.encoding(),
            record.code_unit_len(),
        ));
        self.append_string_payload(record, record.encoding(), &mut payload)?;
        Some(payload)
    }

    fn append_string_payload(
        &self,
        record: PrimitiveStringRecord,
        target_encoding: StringEncoding,
        output: &mut Vec<u8>,
    ) -> Option<()> {
        if let Some(payload) = record.payload() {
            let payload = self.string_payloads.get(payload)?;
            append_flat_string_payload(record.encoding(), target_encoding, payload, output);
            return Some(());
        }

        if record.code_unit_len() == 0 {
            return Some(());
        }

        let (left, right) = record.cons_children()?;
        self.append_string_payload(self.string(left)?, target_encoding, output)?;
        self.append_string_payload(self.string(right)?, target_encoding, output)
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
    pub(crate) fn is_string_marked(&self, id: StringRef) -> bool {
        self.strings.is_marked(id)
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
        let generation = self.allocation_generation(
            NurseryDomain::Symbol,
            std::mem::size_of::<PrimitiveSymbolRecord>(),
            lifetime,
        );
        let id = self.symbols.allocate(
            PrimitiveSymbolRecord::new(description, flags),
            lifetime,
            generation,
        );
        self.mark_symbol_card_if_old_points_to_young(id, description);
        id
    }

    #[inline]
    pub(crate) const fn symbol_allocation_requires_growth(&self) -> bool {
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
            let generation =
                self.allocation_generation(NurseryDomain::BigIntPayload, bytes.len(), lifetime);
            (
                sign,
                Some(self.bigint_payloads.allocate(&bytes, lifetime, generation)),
            )
        };

        let record = PrimitiveBigIntRecord::new(
            sign,
            u32::try_from(normalized_len).expect("normalized bigint limb count must fit into u32"),
            limb_storage,
        );
        let generation = self.allocation_generation(
            NurseryDomain::BigInt,
            std::mem::size_of::<PrimitiveBigIntRecord>(),
            lifetime,
        );
        self.bigints.allocate(record, lifetime, generation)
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
    pub(crate) fn is_bigint_marked(&self, id: BigIntRef) -> bool {
        self.bigints.is_marked(id)
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
        let generation = self.allocation_generation(
            NurseryDomain::ValueCell,
            std::mem::size_of::<PrimitiveValueCellRecord>(),
            lifetime,
        );
        self.value_cells.allocate(
            PrimitiveValueCellRecord::new(Value::empty_internal_slot(), None),
            lifetime,
            generation,
        )
    }

    #[inline]
    pub(crate) const fn value_cell_allocation_requires_growth(&self) -> bool {
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
    pub(crate) fn is_value_cell_marked(&self, id: PrimitiveValueCellRef) -> bool {
        self.value_cells.is_marked(id)
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
        let generation = self.allocation_generation(
            NurseryDomain::Object,
            std::mem::size_of::<RuntimeObjectRecord>(),
            lifetime,
        );
        let id = self.objects.allocate(record, lifetime, generation);
        self.mark_object_card_if_old_points_to_young(id, record);
        id
    }

    #[inline]
    pub(crate) const fn object_allocation_requires_growth(&self) -> bool {
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
    pub(crate) const fn function_payload_allocation_requires_growth(&self) -> bool {
        self.function_payloads.allocation_requires_growth()
    }

    #[inline]
    pub(crate) fn alloc_function_payload(
        &mut self,
        record: RuntimeFunctionRecord,
        lifetime: AllocationLifetime,
    ) -> FunctionPayloadRef {
        let generation = self.allocation_generation(
            NurseryDomain::FunctionPayload,
            std::mem::size_of::<RuntimeFunctionRecord>(),
            lifetime,
        );
        let id = self
            .function_payloads
            .allocate(record, lifetime, generation);
        self.mark_function_payload_card_if_old_points_to_young(id, record);
        id
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
        let generation = self.allocation_generation(
            NurseryDomain::SuspendedExecution,
            std::mem::size_of::<RuntimeSuspendedExecutionRecord>(),
            lifetime,
        );
        let id = self
            .suspended_executions
            .allocate(record, lifetime, generation);
        self.mark_suspended_execution_card_if_old_points_to_young(id, record);
        id
    }

    #[inline]
    pub(crate) const fn suspended_execution_allocation_requires_growth(&self) -> bool {
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
    pub(crate) fn is_suspended_execution_marked(&self, id: SuspendedExecutionRef) -> bool {
        self.suspended_executions.is_marked(id)
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
        let generation = self.allocation_generation(
            NurseryDomain::SuspendedRegisters,
            slot_count.saturating_mul(std::mem::size_of::<Value>()),
            lifetime,
        );
        let id = self
            .suspended_registers
            .allocate(slot_count, fill, lifetime, generation);
        self.mark_value_slot_card_if_old_points_to_young(
            CardDomain::SuspendedRegisters,
            id,
            generation,
            fill,
        );
        id
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
        let generation = self.allocation_generation(
            NurseryDomain::ObjectSlots,
            slot_count.saturating_mul(std::mem::size_of::<Value>()),
            lifetime,
        );
        let id = self
            .object_slots
            .allocate(slot_count, fill, lifetime, generation);
        self.mark_value_slot_card_if_old_points_to_young(
            CardDomain::ObjectSlots,
            id,
            generation,
            fill,
        );
        id
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
        let generation = self.allocation_generation(
            NurseryDomain::Environment,
            std::mem::size_of::<RuntimeEnvironmentRecord>(),
            lifetime,
        );
        let id = self.environments.allocate(record, lifetime, generation);
        self.mark_environment_card_if_old_points_to_young(id, record);
        id
    }

    #[inline]
    pub(crate) const fn environment_allocation_requires_growth(&self) -> bool {
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
    pub(crate) fn is_environment_marked(&self, id: EnvironmentRef) -> bool {
        self.environments.is_marked(id)
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
        let generation = self.allocation_generation(
            NurseryDomain::EnvironmentSlots,
            slot_count.saturating_mul(std::mem::size_of::<Value>()),
            lifetime,
        );
        let id = self
            .environment_slots
            .allocate(slot_count, fill, lifetime, generation);
        self.mark_value_slot_card_if_old_points_to_young(
            CardDomain::EnvironmentSlots,
            id,
            generation,
            fill,
        );
        id
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
        let generation = self.allocation_generation(
            NurseryDomain::Code,
            std::mem::size_of::<RuntimeCodeRecord>(),
            lifetime,
        );
        self.codes.allocate(record, lifetime, generation)
    }

    #[inline]
    pub(crate) const fn code_allocation_requires_growth(&self) -> bool {
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
    pub(crate) fn is_code_marked(&self, id: CodeRef) -> bool {
        self.codes.is_marked(id)
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
        let generation = self.allocation_generation(
            NurseryDomain::CodeSlots,
            slot_count.saturating_mul(std::mem::size_of::<Value>()),
            lifetime,
        );
        let id = self
            .code_slots
            .allocate(slot_count, fill, lifetime, generation);
        self.mark_value_slot_card_if_old_points_to_young(
            CardDomain::CodeSlots,
            id,
            generation,
            fill,
        );
        id
    }

    #[inline]
    pub(crate) fn code_slots_allocation_requires_growth(&self, slot_count: usize) -> bool {
        self.code_slots.allocation_requires_growth(slot_count)
    }

    #[inline]
    pub(crate) fn code_slots(&self, id: CodeSlotsRef) -> Option<&[Value]> {
        self.code_slots.get(id)
    }

    pub(crate) fn mark_code_slots(&mut self, id: CodeSlotsRef) -> bool {
        self.code_slots.mark(id)
    }

    #[inline]
    pub(crate) fn is_code_slots_marked(&self, id: CodeSlotsRef) -> bool {
        self.code_slots.is_marked(id)
    }

    #[inline]
    pub(crate) fn alloc_realm(
        &mut self,
        record: RuntimeRealmRecord,
        lifetime: AllocationLifetime,
    ) -> RealmRef {
        let generation = self.allocation_generation(
            NurseryDomain::Realm,
            std::mem::size_of::<RuntimeRealmRecord>(),
            lifetime,
        );
        let id = self.realms.allocate(record, lifetime, generation);
        self.mark_realm_card_if_old_points_to_young(id, record);
        id
    }

    #[inline]
    pub(crate) const fn realm_allocation_requires_growth(&self) -> bool {
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
    pub(crate) fn is_realm_marked(&self, id: RealmRef) -> bool {
        self.realms.is_marked(id)
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
        let generation = self.allocation_generation(
            NurseryDomain::Shape,
            std::mem::size_of::<RuntimeShapeRecord>(),
            lifetime,
        );
        let id = self.shapes.allocate(record, lifetime, generation);
        self.mark_shape_card_if_old_points_to_young(id, record);
        id
    }

    #[inline]
    pub(crate) const fn shape_allocation_requires_growth(&self) -> bool {
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
    pub(crate) fn is_shape_marked(&self, id: ShapeId) -> bool {
        self.shapes.is_marked(id)
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

    #[inline]
    pub(crate) fn is_young_object(&self, id: ObjectRef) -> bool {
        self.objects.generation(id) == Some(HeapGeneration::Young)
    }

    #[inline]
    pub(crate) fn is_young_function_payload(&self, id: FunctionPayloadRef) -> bool {
        self.function_payloads.generation(id) == Some(HeapGeneration::Young)
    }

    #[inline]
    pub(crate) fn is_young_string(&self, id: StringRef) -> bool {
        self.strings.generation(id) == Some(HeapGeneration::Young)
    }

    #[inline]
    pub(crate) fn is_young_symbol(&self, id: SymbolRef) -> bool {
        self.symbols.generation(id) == Some(HeapGeneration::Young)
    }

    #[inline]
    pub(crate) fn is_young_bigint(&self, id: BigIntRef) -> bool {
        self.bigints.generation(id) == Some(HeapGeneration::Young)
    }

    #[inline]
    pub(crate) fn is_young_value_cell(&self, id: PrimitiveValueCellRef) -> bool {
        self.value_cells.generation(id) == Some(HeapGeneration::Young)
    }

    #[inline]
    pub(crate) fn is_young_environment(&self, id: EnvironmentRef) -> bool {
        self.environments.generation(id) == Some(HeapGeneration::Young)
    }

    #[inline]
    pub(crate) fn is_young_suspended_execution(&self, id: SuspendedExecutionRef) -> bool {
        self.suspended_executions.generation(id) == Some(HeapGeneration::Young)
    }

    #[inline]
    pub(crate) fn is_young_object_slots(&self, id: ObjectSlotsRef) -> bool {
        self.object_slots.generation(id) == Some(HeapGeneration::Young)
    }

    #[inline]
    pub(crate) fn is_young_environment_slots(&self, id: EnvironmentSlotsRef) -> bool {
        self.environment_slots.generation(id) == Some(HeapGeneration::Young)
    }

    #[inline]
    pub(crate) fn is_young_suspended_registers(&self, id: SuspendedRegistersRef) -> bool {
        self.suspended_registers.generation(id) == Some(HeapGeneration::Young)
    }

    pub fn nursery_age(&self, id: ObjectRef) -> Option<u8> {
        if self.is_young_object(id) {
            self.objects.age(id)
        } else {
            None
        }
    }

    pub(crate) fn mark_object_slots(&mut self, id: ObjectSlotsRef) -> bool {
        self.object_slots.mark(id)
    }

    #[inline]
    pub(crate) fn is_object_slots_marked(&self, id: ObjectSlotsRef) -> bool {
        self.object_slots.is_marked(id)
    }

    pub(crate) fn mark_environment_slots(&mut self, id: EnvironmentSlotsRef) -> bool {
        self.environment_slots.mark(id)
    }

    #[inline]
    pub(crate) fn is_environment_slots_marked(&self, id: EnvironmentSlotsRef) -> bool {
        self.environment_slots.is_marked(id)
    }

    pub(crate) fn mark_suspended_registers(&mut self, id: SuspendedRegistersRef) -> bool {
        self.suspended_registers.mark(id)
    }

    #[inline]
    pub(crate) fn is_suspended_registers_marked(&self, id: SuspendedRegistersRef) -> bool {
        self.suspended_registers.is_marked(id)
    }

    pub(crate) fn mark_function_payload(&mut self, id: FunctionPayloadRef) -> bool {
        self.function_payloads.mark(id)
    }

    #[inline]
    pub(crate) fn is_function_payload_marked(&self, id: FunctionPayloadRef) -> bool {
        self.function_payloads.is_marked(id)
    }

    pub(crate) fn scan_object_slots_card(&self, card_index: usize, scan: impl FnMut(&[Value])) {
        self.object_slots.scan_card(card_index, scan);
    }

    pub(crate) fn scan_environment_slots_card(
        &self,
        card_index: usize,
        scan: impl FnMut(&[Value]),
    ) {
        self.environment_slots.scan_card(card_index, scan);
    }

    pub(crate) fn scan_code_slots_card(&self, card_index: usize, scan: impl FnMut(&[Value])) {
        self.code_slots.scan_card(card_index, scan);
    }

    pub(crate) fn scan_suspended_registers_card(
        &self,
        card_index: usize,
        scan: impl FnMut(&[Value]),
    ) {
        self.suspended_registers.scan_card(card_index, scan);
    }

    pub(crate) fn scan_string_card(
        &self,
        card_index: usize,
        scan: impl FnMut(PrimitiveStringRecord),
    ) {
        self.strings.scan_card(
            card_index,
            std::mem::size_of::<PrimitiveStringRecord>(),
            scan,
        );
    }

    pub(crate) fn scan_symbol_card(
        &self,
        card_index: usize,
        scan: impl FnMut(PrimitiveSymbolRecord),
    ) {
        self.symbols.scan_card(
            card_index,
            std::mem::size_of::<PrimitiveSymbolRecord>(),
            scan,
        );
    }

    pub(crate) fn scan_object_card(
        &self,
        card_index: usize,
        scan: impl FnMut(RuntimeObjectRecord),
    ) {
        self.objects
            .scan_card(card_index, std::mem::size_of::<RuntimeObjectRecord>(), scan);
    }

    pub(crate) fn scan_environment_card(
        &self,
        card_index: usize,
        scan: impl FnMut(RuntimeEnvironmentRecord),
    ) {
        self.environments.scan_card(
            card_index,
            std::mem::size_of::<RuntimeEnvironmentRecord>(),
            scan,
        );
    }

    pub(crate) fn scan_function_payload_card(
        &self,
        card_index: usize,
        scan: impl FnMut(RuntimeFunctionRecord),
    ) {
        self.function_payloads.scan_card(
            card_index,
            std::mem::size_of::<RuntimeFunctionRecord>(),
            scan,
        );
    }

    pub(crate) fn scan_value_cell_card(
        &self,
        card_index: usize,
        scan: impl FnMut(PrimitiveValueCellRecord),
    ) {
        self.value_cells.scan_card(
            card_index,
            std::mem::size_of::<PrimitiveValueCellRecord>(),
            scan,
        );
    }

    pub(crate) fn scan_suspended_execution_card(
        &self,
        card_index: usize,
        scan: impl FnMut(RuntimeSuspendedExecutionRecord),
    ) {
        self.suspended_executions.scan_card(
            card_index,
            std::mem::size_of::<RuntimeSuspendedExecutionRecord>(),
            scan,
        );
    }

    pub(crate) fn scan_realm_card(&self, card_index: usize, scan: impl FnMut(RuntimeRealmRecord)) {
        self.realms
            .scan_card(card_index, std::mem::size_of::<RuntimeRealmRecord>(), scan);
    }

    pub(crate) fn scan_shape_card(&self, card_index: usize, scan: impl FnMut(RuntimeShapeRecord)) {
        self.shapes
            .scan_card(card_index, std::mem::size_of::<RuntimeShapeRecord>(), scan);
    }

    pub(crate) fn take_dirty_cards(&mut self) -> Vec<CardKey> {
        self.card_table.take_dirty()
    }

    pub(crate) const fn card_table_dirtied_since_minor(&self) -> usize {
        self.card_table.dirtied_since_minor()
    }

    pub(crate) fn mark_card(&mut self, domain: CardDomain, index: usize) {
        self.card_table.mark(CardKey::new(domain, index));
    }

    pub(crate) fn clear_all_marks(&mut self) {
        self.clear_string_marks();
        self.clear_symbol_marks();
        self.clear_bigint_marks();
        self.clear_value_cell_marks();
        self.clear_object_marks();
        self.function_payloads.clear_marks();
        self.suspended_registers.clear_marks();
        self.clear_suspended_execution_marks();
        self.clear_environment_marks();
        self.environment_slots.clear_marks();
        self.object_slots.clear_marks();
        self.clear_code_marks();
        self.code_slots.clear_marks();
        self.clear_realm_marks();
        self.clear_shape_marks();
    }

    pub(crate) fn sweep_young_generation(
        &mut self,
        cards_dirtied: usize,
        cards_scanned: usize,
        pause: std::time::Duration,
    ) -> crate::PrimitiveMinorCollectionStats {
        let tenuring_threshold = self.nursery_tenuring_threshold();
        let mut young = YoungSweepStats::default();

        macro_rules! merge {
            ($stats:expr) => {{
                let stats = $stats;
                young.survivors += stats.survivors;
                young.tenured += stats.tenured;
                young.reclaimed += stats.reclaimed;
            }};
        }

        merge!(self.strings.sweep_young(tenuring_threshold, |record| {
            if let Some(payload) = record.payload() {
                self.string_payloads.free(payload);
            }
        }));
        merge!(self.symbols.sweep_young(tenuring_threshold, |_| {}));
        merge!(self.bigints.sweep_young(tenuring_threshold, |record| {
            if let Some(storage) = record.limb_storage() {
                self.bigint_payloads.free(storage);
            }
        }));
        merge!(self.value_cells.sweep_young(tenuring_threshold, |_| {}));
        merge!(self.objects.sweep_young(tenuring_threshold, |record| {
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
        }));
        merge!(self
            .function_payloads
            .sweep_young(tenuring_threshold, |_| {}));
        merge!(self.object_slots.sweep_young(tenuring_threshold));
        merge!(self
            .suspended_executions
            .sweep_young(tenuring_threshold, |record| {
                if let Some(registers) = record.registers() {
                    self.suspended_registers.free(registers);
                }
            }));
        merge!(self.suspended_registers.sweep_young(tenuring_threshold));
        merge!(self.environments.sweep_young(tenuring_threshold, |record| {
            if let Some(slots) = record.slots() {
                self.environment_slots.free(slots);
            }
        }));
        merge!(self.environment_slots.sweep_young(tenuring_threshold));

        let pause_ns = pause.as_nanos().max(1);
        let accounting = self.accounting();
        let minor = crate::PrimitiveMinorCollectionStats {
            survivors: young.survivors,
            tenured: young.tenured,
            reclaimed: young.reclaimed,
            cards_dirtied,
            cards_scanned,
            pause_ns,
        };
        self.nursery
            .finish_minor_collection(accounting.young_live_bytes, minor);
        minor
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
            major_mark_slice_budget: DEFAULT_MAJOR_MARK_SLICE_BUDGET,
            last_major_mark_slices: 0,
            last_major_mark_work_items: 0,
            last_major_max_mark_slice_work_items: 0,
            last_major_total_mark_pause_ns: 0,
            last_major_max_mark_pause_ns: 0,
            last_major_mark_finish_work_items: 0,
            last_major_mark_finish_pause_ns: 0,
            last_major_gray_work_items_after_finish: 0,
            active_major_mark: None,
            nursery: Nursery::default(),
            card_table: CardTable::default(),
        }
    }
}

fn normalized_bigint_limb_count(limbs: &[u64]) -> usize {
    limbs
        .iter()
        .rposition(|limb| *limb != 0)
        .map_or(0, |index| index + 1)
}

const fn expected_string_payload_len(encoding: StringEncoding, code_unit_len: u32) -> usize {
    match encoding {
        StringEncoding::Latin1 => code_unit_len as usize,
        StringEncoding::Utf16 => (code_unit_len as usize)
            .checked_mul(2)
            .expect("UTF-16 strings must fit in addressable side storage"),
    }
}

fn append_flat_string_payload(
    source_encoding: StringEncoding,
    target_encoding: StringEncoding,
    payload: &[u8],
    output: &mut Vec<u8>,
) {
    match (source_encoding, target_encoding) {
        (StringEncoding::Latin1, StringEncoding::Latin1)
        | (StringEncoding::Utf16, StringEncoding::Utf16) => output.extend_from_slice(payload),
        (StringEncoding::Latin1, StringEncoding::Utf16) => {
            for byte in payload {
                output.extend_from_slice(&u16::from(*byte).to_le_bytes());
            }
        }
        (StringEncoding::Utf16, StringEncoding::Latin1) => {
            debug_assert!(
                false,
                "UTF-16 payloads cannot be flattened into Latin-1 strings"
            );
        }
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
