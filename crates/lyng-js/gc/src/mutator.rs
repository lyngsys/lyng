use crate::{
    nursery::NurseryDomain, AllocationLifetime, BigIntSign, CodeSlotsRef, EnvironmentSlotsRef,
    FunctionPayloadRef, ObjectSlotsRef, PrimitiveBigIntRecord, PrimitiveBigIntView,
    PrimitiveCollectionReport, PrimitiveCollectionTrigger, PrimitiveDomainStats, PrimitiveHeap,
    PrimitiveHeapAccounting, PrimitiveRoots, PrimitiveStringRecord, PrimitiveStringView,
    PrimitiveSymbolRecord, PrimitiveSymbolView, PrimitiveValueCellRecord, PrimitiveValueCellRef,
    RuntimeCodeRecord, RuntimeEnvironmentRecord, RuntimeFunctionRecord, RuntimeObjectRecord,
    RuntimeRealmRecord, RuntimeShapeRecord, RuntimeSuspendedExecutionRecord, StringEncoding,
    SuspendedRegistersRef, WeakHeapRef,
};
use lyng_js_common::AtomId;
use lyng_js_types::{
    BigIntRef, CodeRef, EnvironmentRef, ObjectRef, RealmRef, ShapeId, StringRef,
    SuspendedExecutionRef, SymbolRef, Value,
};

/// Read-only borrowed view into the primitive heap.
#[derive(Clone, Copy)]
pub struct PrimitiveHeapView<'a> {
    heap: &'a PrimitiveHeap,
}

/// Allocation-capable mutation entrypoint for the primitive heap.
pub struct PrimitiveMutator<'a> {
    heap: &'a mut PrimitiveHeap,
    roots: Option<&'a PrimitiveRoots>,
    last_collection_report: Option<PrimitiveCollectionReport>,
}

/// Store target for traced `Value` fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueStoreTarget {
    ValueCell(PrimitiveValueCellRef),
    EnvironmentThisValue(EnvironmentRef),
    ObjectSlot(ObjectSlotsRef, u32),
    EnvironmentSlot(EnvironmentSlotsRef, u32),
    CodeSlot(CodeSlotsRef, u32),
}

/// Store target for traced `StringRef` handle fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StringHandleStoreTarget {
    SymbolDescription(SymbolRef),
    ValueCellLinkedString(PrimitiveValueCellRef),
}

/// Store target for traced `ObjectRef` handle fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectHandleStoreTarget {
    ObjectPrototype(ObjectRef),
    EnvironmentFunctionObject(EnvironmentRef),
    EnvironmentNewTarget(EnvironmentRef),
    EnvironmentHomeObject(EnvironmentRef),
    RealmGlobalObject(RealmRef),
    ShapePrototypeGuard(ShapeId),
}

/// Store target for traced `ObjectSlotsRef` handle fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectSlotsHandleStoreTarget {
    ObjectNamedSlots(ObjectRef),
    ObjectElements(ObjectRef),
    ObjectPrivateSlots(ObjectRef),
}

/// Store target for traced `EnvironmentRef` handle fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnvironmentHandleStoreTarget {
    EnvironmentOuter(EnvironmentRef),
    RealmGlobalEnv(RealmRef),
}

/// Store target for traced `CodeRef` handle fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodeHandleStoreTarget {
    CodeParent(CodeRef),
    RealmBootstrapCode(RealmRef),
}

/// Store target for traced `RealmRef` handle fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RealmHandleStoreTarget {
    CodeRealm(CodeRef),
}

/// Store target for traced `ShapeId` handle fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShapeHandleStoreTarget {
    ObjectShape(ObjectRef),
    RealmRootShape(RealmRef),
    ShapeParent(ShapeId),
}

impl PrimitiveHeap {
    #[inline]
    pub const fn view(&self) -> PrimitiveHeapView<'_> {
        PrimitiveHeapView { heap: self }
    }

    #[inline]
    pub const fn mutator(&mut self) -> PrimitiveMutator<'_> {
        PrimitiveMutator {
            heap: self,
            roots: None,
            last_collection_report: None,
        }
    }

    #[inline]
    pub const fn mutator_with_roots<'a>(
        &'a mut self,
        roots: &'a PrimitiveRoots,
    ) -> PrimitiveMutator<'a> {
        PrimitiveMutator {
            heap: self,
            roots: Some(roots),
            last_collection_report: None,
        }
    }
}

impl<'a> PrimitiveHeapView<'a> {
    #[inline]
    pub fn accounting(self) -> PrimitiveHeapAccounting {
        self.heap.accounting()
    }

    #[inline]
    pub const fn collection_budget_bytes(self) -> usize {
        self.heap.collection_budget_bytes()
    }

    #[inline]
    pub fn string(self, id: StringRef) -> Option<PrimitiveStringRecord> {
        self.heap.string(id)
    }

    #[inline]
    pub fn string_view(self, id: StringRef) -> Option<PrimitiveStringView<'a>> {
        self.heap.string_view(id)
    }

    #[inline]
    pub fn string_payload(self, id: StringRef) -> Option<&'a [u8]> {
        self.heap.string_payload(id)
    }

    pub fn strings_equal(self, left: StringRef, right: StringRef) -> Option<bool> {
        if left == right {
            return Some(true);
        }

        let left_view = self.string_view(left)?;
        let right_view = self.string_view(right)?;
        Some(left_view.equals(&right_view))
    }

    #[inline]
    pub fn symbol(self, id: SymbolRef) -> Option<PrimitiveSymbolRecord> {
        self.heap.symbol(id)
    }

    #[inline]
    pub fn symbol_view(self, id: SymbolRef) -> Option<PrimitiveSymbolView<'a>> {
        self.heap.symbol_view(id)
    }

    #[inline]
    pub fn bigint(self, id: BigIntRef) -> Option<PrimitiveBigIntRecord> {
        self.heap.bigint(id)
    }

    #[inline]
    pub fn bigint_view(self, id: BigIntRef) -> Option<PrimitiveBigIntView<'a>> {
        self.heap.bigint_view(id)
    }

    #[inline]
    pub fn bigint_limbs(self, id: BigIntRef) -> Option<Vec<u64>> {
        self.heap.bigint_limbs(id)
    }

    #[inline]
    pub fn value_cell(self, id: PrimitiveValueCellRef) -> Option<PrimitiveValueCellRecord> {
        self.heap.value_cell(id)
    }

    #[inline]
    pub fn object(self, id: ObjectRef) -> Option<RuntimeObjectRecord> {
        self.heap.object(id)
    }

    #[inline]
    pub fn function_payload(self, id: FunctionPayloadRef) -> Option<RuntimeFunctionRecord> {
        self.heap.function_payload(id)
    }

    #[inline]
    pub fn suspended_execution(
        self,
        id: SuspendedExecutionRef,
    ) -> Option<RuntimeSuspendedExecutionRecord> {
        self.heap.suspended_execution(id)
    }

    #[inline]
    pub fn object_slots(self, id: ObjectSlotsRef) -> Option<&'a [Value]> {
        self.heap.object_slots(id)
    }

    #[inline]
    pub fn suspended_registers(self, id: SuspendedRegistersRef) -> Option<&'a [Value]> {
        self.heap.suspended_registers(id)
    }

    #[inline]
    pub fn environment(self, id: EnvironmentRef) -> Option<RuntimeEnvironmentRecord> {
        self.heap.environment(id)
    }

    #[inline]
    pub fn environment_slots(self, id: EnvironmentSlotsRef) -> Option<&'a [Value]> {
        self.heap.environment_slots(id)
    }

    #[inline]
    pub fn code(self, id: CodeRef) -> Option<RuntimeCodeRecord> {
        self.heap.code(id)
    }

    #[inline]
    pub fn code_slots(self, id: CodeSlotsRef) -> Option<&'a [Value]> {
        self.heap.code_slots(id)
    }

    #[inline]
    pub fn realm(self, id: RealmRef) -> Option<RuntimeRealmRecord> {
        self.heap.realm(id)
    }

    #[inline]
    pub fn shape(self, id: ShapeId) -> Option<RuntimeShapeRecord> {
        self.heap.shape(id)
    }

    #[inline]
    pub fn string_stats(self) -> PrimitiveDomainStats {
        self.heap.string_stats()
    }

    #[inline]
    pub fn symbol_stats(self) -> PrimitiveDomainStats {
        self.heap.symbol_stats()
    }

    #[inline]
    pub fn bigint_stats(self) -> PrimitiveDomainStats {
        self.heap.bigint_stats()
    }

    #[inline]
    pub fn value_cell_stats(self) -> PrimitiveDomainStats {
        self.heap.value_cell_stats()
    }

    #[inline]
    pub fn object_stats(self) -> PrimitiveDomainStats {
        self.heap.object_stats()
    }

    #[inline]
    pub fn environment_stats(self) -> PrimitiveDomainStats {
        self.heap.environment_stats()
    }

    #[inline]
    pub fn code_stats(self) -> PrimitiveDomainStats {
        self.heap.code_stats()
    }

    #[inline]
    pub fn realm_stats(self) -> PrimitiveDomainStats {
        self.heap.realm_stats()
    }

    #[inline]
    pub fn shape_stats(self) -> PrimitiveDomainStats {
        self.heap.shape_stats()
    }

    #[inline]
    pub fn weak_map_get(self, owner: ObjectRef, key: WeakHeapRef) -> Option<Option<Value>> {
        self.heap.weak_map_get(owner, key)
    }

    #[inline]
    pub fn weak_set_contains(self, owner: ObjectRef, value: WeakHeapRef) -> Option<bool> {
        self.heap.weak_set_contains(owner, value)
    }

    #[inline]
    pub fn weak_ref_target(self, owner: ObjectRef) -> Option<Option<WeakHeapRef>> {
        self.heap.weak_ref_target(owner)
    }

    #[inline]
    pub fn finalization_cleanup_pending(self, owner: ObjectRef) -> Option<bool> {
        self.heap.finalization_cleanup_pending(owner)
    }
}

impl PrimitiveMutator<'_> {
    #[inline]
    pub fn accounting(&self) -> PrimitiveHeapAccounting {
        self.heap.accounting()
    }

    #[inline]
    pub const fn collection_budget_bytes(&self) -> usize {
        self.heap.collection_budget_bytes()
    }

    #[inline]
    pub const fn last_collection_report(&self) -> Option<PrimitiveCollectionReport> {
        self.last_collection_report
    }

    #[inline]
    pub fn force_collect(&mut self) -> Option<PrimitiveCollectionReport> {
        let report = self.roots.map(|roots| self.heap.force_collect(roots))?;
        self.last_collection_report = Some(report);
        Some(report)
    }

    #[inline]
    pub fn init_weak_map(&mut self, owner: ObjectRef) -> bool {
        self.heap.init_weak_map(owner)
    }

    #[inline]
    pub fn weak_map_set(&mut self, owner: ObjectRef, key: WeakHeapRef, value: Value) -> bool {
        self.heap.weak_map_set(owner, key, value)
    }

    #[inline]
    pub fn weak_map_delete(&mut self, owner: ObjectRef, key: WeakHeapRef) -> Option<bool> {
        self.heap.weak_map_delete(owner, key)
    }

    #[inline]
    pub fn init_weak_set(&mut self, owner: ObjectRef) -> bool {
        self.heap.init_weak_set(owner)
    }

    #[inline]
    pub fn weak_set_insert(&mut self, owner: ObjectRef, value: WeakHeapRef) -> bool {
        self.heap.weak_set_insert(owner, value)
    }

    #[inline]
    pub fn weak_set_delete(&mut self, owner: ObjectRef, value: WeakHeapRef) -> Option<bool> {
        self.heap.weak_set_delete(owner, value)
    }

    #[inline]
    pub fn init_weak_ref(&mut self, owner: ObjectRef, target: WeakHeapRef) -> bool {
        self.heap.init_weak_ref(owner, target)
    }

    #[inline]
    pub fn init_finalization_registry(&mut self, owner: ObjectRef) -> bool {
        self.heap.init_finalization_registry(owner)
    }

    #[inline]
    pub fn finalization_registry_register(
        &mut self,
        owner: ObjectRef,
        target: WeakHeapRef,
        holdings: Value,
        unregister_token: Option<WeakHeapRef>,
    ) -> bool {
        self.heap
            .finalization_registry_register(owner, target, holdings, unregister_token)
    }

    #[inline]
    pub fn finalization_registry_unregister(
        &mut self,
        owner: ObjectRef,
        unregister_token: WeakHeapRef,
    ) -> Option<bool> {
        self.heap
            .finalization_registry_unregister(owner, unregister_token)
    }

    #[inline]
    pub fn pending_finalization_registries(&self) -> Vec<ObjectRef> {
        self.heap.pending_finalization_registries().to_vec()
    }

    #[inline]
    pub fn take_finalization_cleanup_holdings(&mut self, owner: ObjectRef) -> Vec<Value> {
        self.heap.take_finalization_cleanup_holdings(owner)
    }

    #[inline]
    pub fn set_finalization_cleanup_active(&mut self, owner: ObjectRef, active: bool) -> bool {
        self.heap.set_finalization_cleanup_active(owner, active)
    }

    #[inline]
    pub fn finalization_cleanup_pending(&self, owner: ObjectRef) -> Option<bool> {
        self.heap.finalization_cleanup_pending(owner)
    }

    #[inline]
    pub const fn view(&self) -> PrimitiveHeapView<'_> {
        PrimitiveHeapView { heap: self.heap }
    }

    #[inline]
    pub fn symbol_view(&self, id: SymbolRef) -> Option<PrimitiveSymbolView<'_>> {
        self.heap.symbol_view(id)
    }

    #[inline]
    pub fn bigint_view(&self, id: BigIntRef) -> Option<PrimitiveBigIntView<'_>> {
        self.heap.bigint_view(id)
    }

    #[inline]
    pub fn string_view(&self, id: StringRef) -> Option<PrimitiveStringView<'_>> {
        self.heap.string_view(id)
    }

    #[inline]
    pub fn alloc_string(
        &mut self,
        encoding: StringEncoding,
        code_unit_len: u32,
        payload_bytes: &[u8],
        cached_atom: Option<AtomId>,
        lifetime: AllocationLifetime,
    ) -> StringRef {
        self.maybe_collect_for_nursery(
            NurseryDomain::String,
            std::mem::size_of::<PrimitiveStringRecord>(),
            lifetime,
        );
        self.maybe_collect_for_growth(
            PrimitiveCollectionTrigger::StringAllocationSlowPath,
            self.heap
                .string_allocation_requires_growth(payload_bytes.len()),
        );
        self.heap.alloc_string(
            encoding,
            code_unit_len,
            payload_bytes,
            cached_atom,
            lifetime,
        )
    }

    #[inline]
    pub fn alloc_latin1_concat_string(
        &mut self,
        left: StringRef,
        right: StringRef,
        lifetime: AllocationLifetime,
    ) -> Option<StringRef> {
        self.heap.latin1_concat_string_payload_len(left, right)?;
        self.maybe_collect_for_growth(
            PrimitiveCollectionTrigger::StringAllocationSlowPath,
            self.heap.string_record_allocation_requires_growth(),
        );
        self.heap.alloc_latin1_concat_string(left, right, lifetime)
    }

    #[inline]
    pub fn alloc_utf16_concat_string(
        &mut self,
        left: StringRef,
        right: StringRef,
        lifetime: AllocationLifetime,
    ) -> Option<StringRef> {
        self.heap.utf16_concat_string_payload_len(left, right)?;
        self.maybe_collect_for_growth(
            PrimitiveCollectionTrigger::StringAllocationSlowPath,
            self.heap.string_record_allocation_requires_growth(),
        );
        self.heap.alloc_utf16_concat_string(left, right, lifetime)
    }

    #[inline]
    pub fn alloc_symbol(
        &mut self,
        description: Option<StringRef>,
        flags: crate::SymbolFlags,
        lifetime: AllocationLifetime,
    ) -> SymbolRef {
        self.maybe_collect_for_nursery(
            NurseryDomain::Symbol,
            std::mem::size_of::<PrimitiveSymbolRecord>(),
            lifetime,
        );
        self.maybe_collect_for_growth(
            PrimitiveCollectionTrigger::SymbolAllocationSlowPath,
            self.heap.symbol_allocation_requires_growth(),
        );
        self.heap.alloc_symbol(description, flags, lifetime)
    }

    #[inline]
    pub fn alloc_bigint(
        &mut self,
        sign: BigIntSign,
        limbs: &[u64],
        lifetime: AllocationLifetime,
    ) -> BigIntRef {
        self.maybe_collect_for_nursery(
            NurseryDomain::BigInt,
            std::mem::size_of::<PrimitiveBigIntRecord>(),
            lifetime,
        );
        self.maybe_collect_for_growth(
            PrimitiveCollectionTrigger::BigIntAllocationSlowPath,
            self.heap.bigint_allocation_requires_growth(limbs),
        );
        self.heap.alloc_bigint(sign, limbs, lifetime)
    }

    #[inline]
    pub fn alloc_value_cell(&mut self, lifetime: AllocationLifetime) -> PrimitiveValueCellRef {
        self.maybe_collect_for_nursery(
            NurseryDomain::ValueCell,
            std::mem::size_of::<PrimitiveValueCellRecord>(),
            lifetime,
        );
        self.maybe_collect_for_growth(
            PrimitiveCollectionTrigger::ValueCellAllocationSlowPath,
            self.heap.value_cell_allocation_requires_growth(),
        );
        self.heap.alloc_value_cell(lifetime)
    }

    #[inline]
    pub fn alloc_object(
        &mut self,
        record: RuntimeObjectRecord,
        lifetime: AllocationLifetime,
    ) -> ObjectRef {
        self.maybe_collect_for_nursery(
            NurseryDomain::Object,
            std::mem::size_of::<RuntimeObjectRecord>(),
            lifetime,
        );
        self.maybe_collect_for_growth(
            PrimitiveCollectionTrigger::ObjectAllocationSlowPath,
            self.heap.object_allocation_requires_growth(),
        );
        self.heap.alloc_object(record, lifetime)
    }

    #[inline]
    pub fn alloc_function_payload(
        &mut self,
        record: RuntimeFunctionRecord,
        lifetime: AllocationLifetime,
    ) -> FunctionPayloadRef {
        self.maybe_collect_for_nursery(
            NurseryDomain::FunctionPayload,
            std::mem::size_of::<RuntimeFunctionRecord>(),
            lifetime,
        );
        self.maybe_collect_for_growth(
            PrimitiveCollectionTrigger::ObjectAllocationSlowPath,
            self.heap.function_payload_allocation_requires_growth(),
        );
        self.heap.alloc_function_payload(record, lifetime)
    }

    #[inline]
    pub fn alloc_suspended_execution(
        &mut self,
        record: RuntimeSuspendedExecutionRecord,
        lifetime: AllocationLifetime,
    ) -> SuspendedExecutionRef {
        self.maybe_collect_for_nursery(
            NurseryDomain::SuspendedExecution,
            std::mem::size_of::<RuntimeSuspendedExecutionRecord>(),
            lifetime,
        );
        self.maybe_collect_for_growth(
            PrimitiveCollectionTrigger::ObjectAllocationSlowPath,
            self.heap.suspended_execution_allocation_requires_growth(),
        );
        self.heap.alloc_suspended_execution(record, lifetime)
    }

    #[inline]
    pub fn alloc_object_slots(
        &mut self,
        slot_count: usize,
        fill: Value,
        lifetime: AllocationLifetime,
    ) -> ObjectSlotsRef {
        self.maybe_collect_for_nursery(
            NurseryDomain::ObjectSlots,
            slot_count.saturating_mul(std::mem::size_of::<Value>()),
            lifetime,
        );
        self.maybe_collect_for_growth(
            PrimitiveCollectionTrigger::ObjectAllocationSlowPath,
            self.heap
                .object_slots_allocation_requires_growth(slot_count),
        );
        self.heap.alloc_object_slots(slot_count, fill, lifetime)
    }

    #[inline]
    pub fn alloc_suspended_registers(
        &mut self,
        slot_count: usize,
        fill: Value,
        lifetime: AllocationLifetime,
    ) -> SuspendedRegistersRef {
        self.maybe_collect_for_nursery(
            NurseryDomain::SuspendedRegisters,
            slot_count.saturating_mul(std::mem::size_of::<Value>()),
            lifetime,
        );
        self.maybe_collect_for_growth(
            PrimitiveCollectionTrigger::ObjectAllocationSlowPath,
            self.heap
                .suspended_registers_allocation_requires_growth(slot_count),
        );
        self.heap
            .alloc_suspended_registers(slot_count, fill, lifetime)
    }

    #[inline]
    pub fn alloc_environment(
        &mut self,
        record: RuntimeEnvironmentRecord,
        lifetime: AllocationLifetime,
    ) -> EnvironmentRef {
        self.maybe_collect_for_nursery(
            NurseryDomain::Environment,
            std::mem::size_of::<RuntimeEnvironmentRecord>(),
            lifetime,
        );
        self.maybe_collect_for_growth(
            PrimitiveCollectionTrigger::EnvironmentAllocationSlowPath,
            self.heap.environment_allocation_requires_growth(),
        );
        self.heap.alloc_environment(record, lifetime)
    }

    #[inline]
    pub fn alloc_environment_slots(
        &mut self,
        slot_count: usize,
        fill: Value,
        lifetime: AllocationLifetime,
    ) -> EnvironmentSlotsRef {
        self.maybe_collect_for_nursery(
            NurseryDomain::EnvironmentSlots,
            slot_count.saturating_mul(std::mem::size_of::<Value>()),
            lifetime,
        );
        self.maybe_collect_for_growth(
            PrimitiveCollectionTrigger::EnvironmentAllocationSlowPath,
            self.heap
                .environment_slots_allocation_requires_growth(slot_count),
        );
        self.heap
            .alloc_environment_slots(slot_count, fill, lifetime)
    }

    #[inline]
    pub fn alloc_code(
        &mut self,
        record: RuntimeCodeRecord,
        lifetime: AllocationLifetime,
    ) -> CodeRef {
        self.maybe_collect_for_nursery(
            NurseryDomain::Code,
            std::mem::size_of::<RuntimeCodeRecord>(),
            lifetime,
        );
        self.maybe_collect_for_growth(
            PrimitiveCollectionTrigger::CodeAllocationSlowPath,
            self.heap.code_allocation_requires_growth(),
        );
        self.heap.alloc_code(record, lifetime)
    }

    #[inline]
    pub fn alloc_code_slots(
        &mut self,
        slot_count: usize,
        fill: Value,
        lifetime: AllocationLifetime,
    ) -> CodeSlotsRef {
        self.maybe_collect_for_nursery(
            NurseryDomain::CodeSlots,
            slot_count.saturating_mul(std::mem::size_of::<Value>()),
            lifetime,
        );
        self.maybe_collect_for_growth(
            PrimitiveCollectionTrigger::CodeAllocationSlowPath,
            self.heap.code_slots_allocation_requires_growth(slot_count),
        );
        self.heap.alloc_code_slots(slot_count, fill, lifetime)
    }

    #[inline]
    pub fn alloc_realm(
        &mut self,
        record: RuntimeRealmRecord,
        lifetime: AllocationLifetime,
    ) -> RealmRef {
        self.maybe_collect_for_nursery(
            NurseryDomain::Realm,
            std::mem::size_of::<RuntimeRealmRecord>(),
            lifetime,
        );
        self.maybe_collect_for_growth(
            PrimitiveCollectionTrigger::RealmAllocationSlowPath,
            self.heap.realm_allocation_requires_growth(),
        );
        self.heap.alloc_realm(record, lifetime)
    }

    #[inline]
    pub fn alloc_shape(
        &mut self,
        record: RuntimeShapeRecord,
        lifetime: AllocationLifetime,
    ) -> ShapeId {
        self.maybe_collect_for_nursery(
            NurseryDomain::Shape,
            std::mem::size_of::<RuntimeShapeRecord>(),
            lifetime,
        );
        self.maybe_collect_for_growth(
            PrimitiveCollectionTrigger::ShapeAllocationSlowPath,
            self.heap.shape_allocation_requires_growth(),
        );
        self.heap.alloc_shape(record, lifetime)
    }

    #[inline]
    pub fn free_string(&mut self, id: StringRef) -> Option<PrimitiveStringRecord> {
        self.heap.free_string(id)
    }

    #[inline]
    pub fn cache_string_hash(&mut self, id: StringRef) -> Option<u32> {
        self.heap.cache_string_hash(id)
    }

    #[inline]
    pub fn memoize_string_atom(&mut self, id: StringRef, atom: AtomId) -> bool {
        self.heap.memoize_string_atom(id, atom)
    }

    #[inline]
    pub fn free_symbol(&mut self, id: SymbolRef) -> Option<PrimitiveSymbolRecord> {
        self.heap.free_symbol(id)
    }

    #[inline]
    pub fn free_bigint(&mut self, id: BigIntRef) -> Option<PrimitiveBigIntRecord> {
        self.heap.free_bigint(id)
    }

    #[inline]
    pub fn free_value_cell(
        &mut self,
        id: PrimitiveValueCellRef,
    ) -> Option<PrimitiveValueCellRecord> {
        self.heap.free_value_cell(id)
    }

    #[inline]
    pub fn free_object(&mut self, id: ObjectRef) -> Option<RuntimeObjectRecord> {
        self.heap.free_object(id)
    }

    #[inline]
    pub fn free_function_payload(
        &mut self,
        id: FunctionPayloadRef,
    ) -> Option<RuntimeFunctionRecord> {
        self.heap.free_function_payload(id)
    }

    #[inline]
    pub fn free_suspended_execution(
        &mut self,
        id: SuspendedExecutionRef,
    ) -> Option<RuntimeSuspendedExecutionRecord> {
        self.heap.free_suspended_execution(id)
    }

    #[inline]
    pub fn write_suspended_register(
        &mut self,
        id: SuspendedRegistersRef,
        index: u32,
        value: Value,
    ) -> bool {
        self.heap.write_suspended_register(id, index, value)
    }

    #[inline]
    pub fn set_function_payload_home_object(
        &mut self,
        id: FunctionPayloadRef,
        home_object: Option<ObjectRef>,
    ) -> bool {
        self.heap.set_function_payload_home_object(id, home_object)
    }

    #[inline]
    pub fn set_function_payload_environment(
        &mut self,
        id: FunctionPayloadRef,
        environment: Option<EnvironmentRef>,
    ) -> bool {
        self.heap.set_function_payload_environment(id, environment)
    }

    #[inline]
    pub fn set_function_payload_private_env(
        &mut self,
        id: FunctionPayloadRef,
        private_env: Option<EnvironmentRef>,
    ) -> bool {
        self.heap.set_function_payload_private_env(id, private_env)
    }

    #[inline]
    pub fn free_environment(&mut self, id: EnvironmentRef) -> Option<RuntimeEnvironmentRecord> {
        self.heap.free_environment(id)
    }

    #[inline]
    pub fn free_code(&mut self, id: CodeRef) -> Option<RuntimeCodeRecord> {
        self.heap.free_code(id)
    }

    #[inline]
    pub fn free_realm(&mut self, id: RealmRef) -> Option<RuntimeRealmRecord> {
        self.heap.free_realm(id)
    }

    #[inline]
    pub fn free_shape(&mut self, id: ShapeId) -> Option<RuntimeShapeRecord> {
        self.heap.free_shape(id)
    }

    /// Writes into a freshly allocated traced `Value` field before publication.
    #[inline]
    pub fn init_store_value(&mut self, target: ValueStoreTarget, value: Value) -> bool {
        self.store_value(target, value)
    }

    /// Overwrites an existing traced `Value` field through the barrier-ready boundary.
    #[inline]
    pub fn mut_store_value(&mut self, target: ValueStoreTarget, value: Value) -> bool {
        self.store_value(target, value)
    }

    /// Writes into a freshly allocated traced `StringRef` field before publication.
    #[inline]
    pub fn init_store_string_handle(
        &mut self,
        target: StringHandleStoreTarget,
        value: Option<StringRef>,
    ) -> bool {
        self.store_string_handle(target, value)
    }

    /// Overwrites an existing traced `StringRef` field through the barrier-ready boundary.
    #[inline]
    pub fn mut_store_string_handle(
        &mut self,
        target: StringHandleStoreTarget,
        value: Option<StringRef>,
    ) -> bool {
        self.store_string_handle(target, value)
    }

    /// Writes into a freshly allocated traced `ObjectRef` field before publication.
    #[inline]
    pub fn init_store_object_handle(
        &mut self,
        target: ObjectHandleStoreTarget,
        value: Option<ObjectRef>,
    ) -> bool {
        self.store_object_handle(target, value)
    }

    /// Overwrites an existing traced `ObjectRef` field through the barrier-ready boundary.
    #[inline]
    pub fn mut_store_object_handle(
        &mut self,
        target: ObjectHandleStoreTarget,
        value: Option<ObjectRef>,
    ) -> bool {
        self.store_object_handle(target, value)
    }

    /// Writes into a freshly allocated traced `ObjectSlotsRef` field before publication.
    #[inline]
    pub fn init_store_object_slots_handle(
        &mut self,
        target: ObjectSlotsHandleStoreTarget,
        value: Option<ObjectSlotsRef>,
    ) -> bool {
        self.store_object_slots_handle(target, value)
    }

    /// Overwrites an existing traced `ObjectSlotsRef` field through the barrier-ready boundary.
    #[inline]
    pub fn mut_store_object_slots_handle(
        &mut self,
        target: ObjectSlotsHandleStoreTarget,
        value: Option<ObjectSlotsRef>,
    ) -> bool {
        self.store_object_slots_handle(target, value)
    }

    /// Writes into a freshly allocated traced `EnvironmentRef` field before publication.
    #[inline]
    pub fn init_store_environment_handle(
        &mut self,
        target: EnvironmentHandleStoreTarget,
        value: Option<EnvironmentRef>,
    ) -> bool {
        self.store_environment_handle(target, value)
    }

    /// Overwrites an existing traced `EnvironmentRef` field through the barrier-ready boundary.
    #[inline]
    pub fn mut_store_environment_handle(
        &mut self,
        target: EnvironmentHandleStoreTarget,
        value: Option<EnvironmentRef>,
    ) -> bool {
        self.store_environment_handle(target, value)
    }

    /// Writes into a freshly allocated traced `CodeRef` field before publication.
    #[inline]
    pub fn init_store_code_handle(
        &mut self,
        target: CodeHandleStoreTarget,
        value: Option<CodeRef>,
    ) -> bool {
        self.store_code_handle(target, value)
    }

    /// Overwrites an existing traced `CodeRef` field through the barrier-ready boundary.
    #[inline]
    pub fn mut_store_code_handle(
        &mut self,
        target: CodeHandleStoreTarget,
        value: Option<CodeRef>,
    ) -> bool {
        self.store_code_handle(target, value)
    }

    /// Writes into a freshly allocated traced `RealmRef` field before publication.
    #[inline]
    pub fn init_store_realm_handle(
        &mut self,
        target: RealmHandleStoreTarget,
        value: Option<RealmRef>,
    ) -> bool {
        self.store_realm_handle(target, value)
    }

    /// Overwrites an existing traced `RealmRef` field through the barrier-ready boundary.
    #[inline]
    pub fn mut_store_realm_handle(
        &mut self,
        target: RealmHandleStoreTarget,
        value: Option<RealmRef>,
    ) -> bool {
        self.store_realm_handle(target, value)
    }

    /// Writes into a freshly allocated traced `ShapeId` field before publication.
    #[inline]
    pub fn init_store_shape_handle(
        &mut self,
        target: ShapeHandleStoreTarget,
        value: Option<ShapeId>,
    ) -> bool {
        self.store_shape_handle(target, value)
    }

    /// Overwrites an existing traced `ShapeId` field through the barrier-ready boundary.
    #[inline]
    pub fn mut_store_shape_handle(
        &mut self,
        target: ShapeHandleStoreTarget,
        value: Option<ShapeId>,
    ) -> bool {
        self.store_shape_handle(target, value)
    }

    #[inline]
    fn store_value(&mut self, target: ValueStoreTarget, value: Value) -> bool {
        match target {
            ValueStoreTarget::ValueCell(id) => self.heap.set_value_cell_value(id, value),
            ValueStoreTarget::EnvironmentThisValue(id) => {
                self.heap.set_environment_this_value(id, value)
            }
            ValueStoreTarget::ObjectSlot(id, index) => {
                self.heap.write_object_slot(id, index, value)
            }
            ValueStoreTarget::EnvironmentSlot(id, index) => {
                self.heap.write_environment_slot(id, index, value)
            }
            ValueStoreTarget::CodeSlot(id, index) => self.heap.write_code_slot(id, index, value),
        }
    }

    #[inline]
    fn store_string_handle(
        &mut self,
        target: StringHandleStoreTarget,
        value: Option<StringRef>,
    ) -> bool {
        match target {
            StringHandleStoreTarget::SymbolDescription(id) => {
                self.heap.set_symbol_description(id, value)
            }
            StringHandleStoreTarget::ValueCellLinkedString(id) => {
                self.heap.set_value_cell_linked_string(id, value)
            }
        }
    }

    #[inline]
    fn store_object_handle(
        &mut self,
        target: ObjectHandleStoreTarget,
        value: Option<ObjectRef>,
    ) -> bool {
        match target {
            ObjectHandleStoreTarget::ObjectPrototype(id) => {
                self.heap.set_object_prototype(id, value)
            }
            ObjectHandleStoreTarget::EnvironmentFunctionObject(id) => {
                self.heap.set_environment_function_object(id, value)
            }
            ObjectHandleStoreTarget::EnvironmentNewTarget(id) => {
                self.heap.set_environment_new_target(id, value)
            }
            ObjectHandleStoreTarget::EnvironmentHomeObject(id) => {
                self.heap.set_environment_home_object(id, value)
            }
            ObjectHandleStoreTarget::RealmGlobalObject(id) => {
                self.heap.set_realm_global_object(id, value)
            }
            ObjectHandleStoreTarget::ShapePrototypeGuard(id) => {
                self.heap.set_shape_prototype_guard(id, value)
            }
        }
    }

    #[inline]
    fn store_object_slots_handle(
        &mut self,
        target: ObjectSlotsHandleStoreTarget,
        value: Option<ObjectSlotsRef>,
    ) -> bool {
        match target {
            ObjectSlotsHandleStoreTarget::ObjectNamedSlots(id) => {
                self.heap.set_object_named_slots(id, value)
            }
            ObjectSlotsHandleStoreTarget::ObjectElements(id) => {
                self.heap.set_object_elements(id, value)
            }
            ObjectSlotsHandleStoreTarget::ObjectPrivateSlots(id) => {
                self.heap.set_object_private_slots(id, value)
            }
        }
    }

    #[inline]
    fn store_environment_handle(
        &mut self,
        target: EnvironmentHandleStoreTarget,
        value: Option<EnvironmentRef>,
    ) -> bool {
        match target {
            EnvironmentHandleStoreTarget::EnvironmentOuter(id) => {
                self.heap.set_environment_outer(id, value)
            }
            EnvironmentHandleStoreTarget::RealmGlobalEnv(id) => {
                self.heap.set_realm_global_env(id, value)
            }
        }
    }

    #[inline]
    fn store_code_handle(&mut self, target: CodeHandleStoreTarget, value: Option<CodeRef>) -> bool {
        match target {
            CodeHandleStoreTarget::CodeParent(id) => self.heap.set_code_parent(id, value),
            CodeHandleStoreTarget::RealmBootstrapCode(id) => {
                self.heap.set_realm_bootstrap_code(id, value)
            }
        }
    }

    #[inline]
    fn store_realm_handle(
        &mut self,
        target: RealmHandleStoreTarget,
        value: Option<RealmRef>,
    ) -> bool {
        match target {
            RealmHandleStoreTarget::CodeRealm(id) => self.heap.set_code_realm(id, value),
        }
    }

    #[inline]
    fn store_shape_handle(
        &mut self,
        target: ShapeHandleStoreTarget,
        value: Option<ShapeId>,
    ) -> bool {
        match target {
            ShapeHandleStoreTarget::ObjectShape(id) => self.heap.set_object_shape(id, value),
            ShapeHandleStoreTarget::RealmRootShape(id) => self.heap.set_realm_root_shape(id, value),
            ShapeHandleStoreTarget::ShapeParent(id) => self.heap.set_shape_parent(id, value),
        }
    }

    fn maybe_collect_for_growth(
        &mut self,
        trigger: PrimitiveCollectionTrigger,
        requires_growth: bool,
    ) {
        let _ = self.heap.poll_incremental_mark_step();
        let Some(roots) = self.roots else {
            return;
        };
        if !requires_growth {
            return;
        }
        if let Some(report) = self.heap.maybe_collect_before_growth(roots, trigger) {
            self.last_collection_report = Some(report);
        }
    }

    fn maybe_collect_for_nursery(
        &mut self,
        domain: NurseryDomain,
        bytes: usize,
        lifetime: AllocationLifetime,
    ) {
        if !PrimitiveHeap::should_allocate_in_nursery(domain, lifetime)
            || self.heap.nursery_can_fit(bytes)
        {
            return;
        }
        let Some(roots) = self.roots else {
            return;
        };
        let report = self
            .heap
            .minor_collect_with_trigger(roots, PrimitiveCollectionTrigger::NurseryAllocationLimit);
        self.last_collection_report = Some(report);
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::too_many_lines)]

    use super::*;
    use crate::{
        CodeHandleStoreTarget, EnvironmentHandleStoreTarget, ObjectHandleStoreTarget,
        ObjectSlotsHandleStoreTarget, PrimitiveRoots, RealmHandleStoreTarget, RuntimeCodeRecord,
        RuntimeEnvironmentRecord, RuntimeObjectRecord, RuntimeRealmRecord, RuntimeShapeRecord,
        ShapeHandleStoreTarget, StringHandleStoreTarget, SymbolFlags, ValueStoreTarget,
        PRIMITIVE_SLOTS_PER_PAGE,
    };
    use lyng_js_common::AtomId;

    #[test]
    fn read_only_views_and_mutators_split_access_paths() {
        let mut heap = PrimitiveHeap::new();
        let mut mutator = heap.mutator();
        let description = mutator.alloc_string(
            StringEncoding::Latin1,
            4,
            b"desc",
            None,
            AllocationLifetime::Default,
        );
        let symbol =
            mutator.alloc_symbol(None, SymbolFlags::ordinary(), AllocationLifetime::Default);

        assert!(mutator.init_store_string_handle(
            StringHandleStoreTarget::SymbolDescription(symbol),
            Some(description),
        ));

        let view = mutator.view();
        assert_eq!(
            view.symbol(symbol).unwrap().description(),
            Some(description)
        );
        assert_eq!(view.string_payload(description), Some(&b"desc"[..]));
    }

    #[test]
    fn latin1_concat_allocates_from_existing_string_payloads() {
        let mut heap = PrimitiveHeap::new();
        let mut mutator = heap.mutator();
        let left = mutator.alloc_string(
            StringEncoding::Latin1,
            16,
            b"left-hand-string",
            None,
            AllocationLifetime::Default,
        );
        let right = mutator.alloc_string(
            StringEncoding::Latin1,
            17,
            b"right-hand-string",
            None,
            AllocationLifetime::Default,
        );
        let wide = mutator.alloc_string(
            StringEncoding::Utf16,
            1,
            &[0x00, 0x01],
            None,
            AllocationLifetime::Default,
        );

        let combined = mutator
            .alloc_latin1_concat_string(left, right, AllocationLifetime::Default)
            .expect("latin1 payloads should concatenate directly");
        assert!(
            mutator
                .alloc_latin1_concat_string(left, wide, AllocationLifetime::Default)
                .is_none(),
            "non-latin1 strings should fall back to the generic concat path"
        );

        let view = mutator.view();
        assert_eq!(view.string(combined).unwrap().code_unit_len(), 33);
        assert_eq!(
            view.string_payload(combined),
            None,
            "concat results should defer flat payload allocation"
        );
        let combined_view = view
            .string_view(combined)
            .expect("concat result should remain viewable");
        assert_eq!(combined_view.code_unit_at(0), Some(u16::from(b'l')));
        assert_eq!(combined_view.code_unit_at(16), Some(u16::from(b'r')));
        assert_eq!(combined_view.code_unit_at(32), Some(u16::from(b'g')));
        assert_eq!(
            combined_view.latin1_bytes(),
            Some(&b"left-hand-stringright-hand-string"[..])
        );
    }

    #[test]
    fn utf16_concat_allocates_from_existing_string_payloads() {
        let mut heap = PrimitiveHeap::new();
        let mut mutator = heap.mutator();
        let latin1 = mutator.alloc_string(
            StringEncoding::Latin1,
            2,
            b"Az",
            None,
            AllocationLifetime::Default,
        );
        let wide = mutator.alloc_string(
            StringEncoding::Utf16,
            2,
            &[0x00, 0x01, 0x34, 0x12],
            None,
            AllocationLifetime::Default,
        );

        let combined = mutator
            .alloc_utf16_concat_string(latin1, wide, AllocationLifetime::Default)
            .expect("mixed payloads should concatenate directly into UTF-16 storage");
        assert!(
            mutator
                .alloc_utf16_concat_string(latin1, latin1, AllocationLifetime::Default)
                .is_none(),
            "latin1-only strings should keep using the narrower concat path"
        );

        let view = mutator.view();
        assert_eq!(
            view.string(combined).unwrap().encoding(),
            StringEncoding::Utf16
        );
        assert_eq!(view.string(combined).unwrap().code_unit_len(), 4);
        assert_eq!(
            view.string_payload(combined),
            None,
            "concat results should defer flat payload allocation"
        );
        let combined_view = view
            .string_view(combined)
            .expect("concat result should remain viewable");
        assert_eq!(combined_view.code_unit_at(0), Some(0x0041));
        assert_eq!(combined_view.code_unit_at(1), Some(0x007A));
        assert_eq!(combined_view.code_unit_at(2), Some(0x0100));
        assert_eq!(combined_view.code_unit_at(3), Some(0x1234));
        assert_eq!(
            combined_view.utf16_bytes(),
            Some(&[b'A', 0x00, b'z', 0x00, 0x00, 0x01, 0x34, 0x12][..])
        );
    }

    #[test]
    fn rooted_concat_string_traces_child_strings() {
        let mut heap = PrimitiveHeap::new();
        let roots = PrimitiveRoots::new();
        let (left, right, combined, dead) = {
            let mut mutator = heap.mutator();
            let left = mutator.alloc_string(
                StringEncoding::Latin1,
                4,
                b"left",
                None,
                AllocationLifetime::Default,
            );
            let right = mutator.alloc_string(
                StringEncoding::Latin1,
                5,
                b"right",
                None,
                AllocationLifetime::Default,
            );
            let combined = mutator
                .alloc_latin1_concat_string(left, right, AllocationLifetime::Default)
                .expect("latin1 strings should concatenate");
            let dead = mutator.alloc_string(
                StringEncoding::Latin1,
                4,
                b"dead",
                None,
                AllocationLifetime::Default,
            );
            (left, right, combined, dead)
        };
        let _rooted = roots.root_string(combined);

        let stats = heap.collect(&roots);
        let view = heap.view();

        assert_eq!(stats.trace.strings_marked, 3);
        assert_eq!(stats.strings_reclaimed, 1);
        assert!(view.string(left).is_some());
        assert!(view.string(right).is_some());
        assert!(view.string(combined).is_some());
        assert_eq!(view.string(dead), None);
    }

    #[test]
    fn init_store_and_mut_store_update_traced_value_and_handle_fields() {
        let mut heap = PrimitiveHeap::new();
        let mut mutator = heap.mutator();
        let string_a = mutator.alloc_string(
            StringEncoding::Latin1,
            1,
            b"a",
            None,
            AllocationLifetime::Default,
        );
        let string_b = mutator.alloc_string(
            StringEncoding::Latin1,
            1,
            b"b",
            None,
            AllocationLifetime::Default,
        );
        let cell = mutator.alloc_value_cell(AllocationLifetime::Default);

        assert!(mutator.init_store_value(ValueStoreTarget::ValueCell(cell), Value::from_smi(7)));
        assert!(mutator.init_store_string_handle(
            StringHandleStoreTarget::ValueCellLinkedString(cell),
            Some(string_a),
        ));
        assert!(mutator.mut_store_value(
            ValueStoreTarget::ValueCell(cell),
            Value::from_string_ref(string_b),
        ));
        assert!(mutator.mut_store_string_handle(
            StringHandleStoreTarget::ValueCellLinkedString(cell),
            Some(string_b),
        ));

        let view = mutator.view();
        let record = view.value_cell(cell).unwrap();
        assert_eq!(record.stored_value(), Value::from_string_ref(string_b));
        assert_eq!(record.linked_string(), Some(string_b));
    }

    #[test]
    fn helper_mediated_mutation_keeps_rooted_traced_edges_alive() {
        let mut heap = PrimitiveHeap::new();
        let roots = PrimitiveRoots::new();
        let (string, cell) = {
            let mut mutator = heap.mutator();
            let string = mutator.alloc_string(
                StringEncoding::Latin1,
                4,
                b"root",
                None,
                AllocationLifetime::Default,
            );
            let cell = mutator.alloc_value_cell(AllocationLifetime::Default);

            assert!(mutator.init_store_value(
                ValueStoreTarget::ValueCell(cell),
                Value::from_string_ref(string),
            ));
            assert!(mutator.init_store_string_handle(
                StringHandleStoreTarget::ValueCellLinkedString(cell),
                Some(string),
            ));
            (string, cell)
        };
        let _rooted = roots.root_value_cell(cell);

        let stats = heap.collect(&roots);
        let view = heap.view();

        assert_eq!(stats.trace.value_cells_marked, 1);
        assert_eq!(stats.trace.values_traced, 1);
        assert_eq!(stats.trace.strings_marked, 1);
        assert_eq!(stats.value_cells_reclaimed, 0);
        assert_eq!(stats.strings_reclaimed, 0);
        assert_eq!(
            view.value_cell(cell).unwrap().stored_value(),
            Value::from_string_ref(string)
        );
        assert_eq!(view.string_payload(string), Some(&b"root"[..]));
    }

    #[test]
    fn runtime_store_helpers_update_slot_buffers_and_typed_handle_fields() {
        let mut heap = PrimitiveHeap::new();
        let mut mutator = heap.mutator();
        let string = mutator.alloc_string(
            StringEncoding::Latin1,
            4,
            b"slot",
            None,
            AllocationLifetime::Default,
        );
        let bigint = mutator.alloc_bigint(BigIntSign::Negative, &[5], AllocationLifetime::Default);
        let proto = mutator.alloc_object(
            RuntimeObjectRecord::new(None, None, None, None, None),
            AllocationLifetime::Default,
        );
        let object_slots = mutator.alloc_object_slots(
            2,
            Value::empty_internal_slot(),
            AllocationLifetime::Default,
        );
        let object_elements = mutator.alloc_object_slots(
            1,
            Value::empty_internal_slot(),
            AllocationLifetime::Default,
        );
        let object = mutator.alloc_object(
            RuntimeObjectRecord::new(None, None, Some(object_slots), Some(object_elements), None),
            AllocationLifetime::Default,
        );
        let shape_parent = mutator.alloc_shape(
            RuntimeShapeRecord::new(None, None, 1),
            AllocationLifetime::Default,
        );
        let shape = mutator.alloc_shape(
            RuntimeShapeRecord::new(None, None, 2),
            AllocationLifetime::Default,
        );
        let env_outer = mutator.alloc_environment(
            RuntimeEnvironmentRecord::new(
                None,
                None,
                None,
                Value::empty_internal_slot(),
                None,
                None,
            ),
            AllocationLifetime::Default,
        );
        let env_slots = mutator.alloc_environment_slots(
            1,
            Value::empty_internal_slot(),
            AllocationLifetime::Default,
        );
        let env = mutator.alloc_environment(
            RuntimeEnvironmentRecord::new(
                None,
                Some(env_slots),
                None,
                Value::empty_internal_slot(),
                None,
                None,
            ),
            AllocationLifetime::Default,
        );
        let code_parent = mutator.alloc_code(
            RuntimeCodeRecord::new(None, None, None),
            AllocationLifetime::Default,
        );
        let code_slots =
            mutator.alloc_code_slots(1, Value::empty_internal_slot(), AllocationLifetime::Default);
        let code = mutator.alloc_code(
            RuntimeCodeRecord::new(None, None, Some(code_slots)),
            AllocationLifetime::Default,
        );
        let realm = mutator.alloc_realm(
            RuntimeRealmRecord::new(None, None, None, None),
            AllocationLifetime::Default,
        );

        assert!(mutator.init_store_object_handle(
            ObjectHandleStoreTarget::ObjectPrototype(object),
            Some(proto),
        ));
        assert!(mutator
            .init_store_shape_handle(ShapeHandleStoreTarget::ObjectShape(object), Some(shape),));
        assert!(mutator.init_store_value(
            ValueStoreTarget::ObjectSlot(object_slots, 0),
            Value::from_string_ref(string),
        ));
        assert!(mutator.init_store_value(
            ValueStoreTarget::ObjectSlot(object_slots, 1),
            Value::from_object_ref(proto),
        ));
        assert!(mutator.init_store_value(
            ValueStoreTarget::ObjectSlot(object_elements, 0),
            Value::from_smi(17),
        ));
        assert!(mutator.init_store_environment_handle(
            EnvironmentHandleStoreTarget::EnvironmentOuter(env),
            Some(env_outer),
        ));
        assert!(mutator.init_store_object_handle(
            ObjectHandleStoreTarget::EnvironmentFunctionObject(env),
            Some(object),
        ));
        assert!(mutator.init_store_value(
            ValueStoreTarget::EnvironmentThisValue(env),
            Value::from_object_ref(object),
        ));
        assert!(mutator.init_store_object_handle(
            ObjectHandleStoreTarget::EnvironmentNewTarget(env),
            Some(proto),
        ));
        assert!(mutator.init_store_object_handle(
            ObjectHandleStoreTarget::EnvironmentHomeObject(env),
            Some(object),
        ));
        assert!(mutator.init_store_value(
            ValueStoreTarget::EnvironmentSlot(env_slots, 0),
            Value::from_string_ref(string),
        ));
        assert!(mutator
            .init_store_code_handle(CodeHandleStoreTarget::CodeParent(code), Some(code_parent),));
        assert!(
            mutator.init_store_realm_handle(RealmHandleStoreTarget::CodeRealm(code), Some(realm),)
        );
        assert!(mutator.init_store_value(
            ValueStoreTarget::CodeSlot(code_slots, 0),
            Value::from_bigint_ref(bigint),
        ));
        assert!(mutator.init_store_object_handle(
            ObjectHandleStoreTarget::RealmGlobalObject(realm),
            Some(object),
        ));
        assert!(mutator.init_store_environment_handle(
            EnvironmentHandleStoreTarget::RealmGlobalEnv(realm),
            Some(env),
        ));
        assert!(mutator
            .init_store_code_handle(CodeHandleStoreTarget::RealmBootstrapCode(realm), Some(code),));
        assert!(mutator
            .init_store_shape_handle(ShapeHandleStoreTarget::RealmRootShape(realm), Some(shape),));
        assert!(mutator.init_store_shape_handle(
            ShapeHandleStoreTarget::ShapeParent(shape),
            Some(shape_parent),
        ));
        assert!(mutator.init_store_object_handle(
            ObjectHandleStoreTarget::ShapePrototypeGuard(shape),
            Some(proto),
        ));

        let replacement = mutator.alloc_string(
            StringEncoding::Latin1,
            3,
            b"new",
            None,
            AllocationLifetime::Default,
        );
        let replacement_elements =
            mutator.alloc_object_slots(3, Value::array_hole(), AllocationLifetime::Default);
        assert!(mutator.mut_store_value(
            ValueStoreTarget::ObjectSlot(object_slots, 0),
            Value::from_string_ref(replacement),
        ));
        assert!(mutator.init_store_value(
            ValueStoreTarget::ObjectSlot(replacement_elements, 2),
            Value::from_smi(99),
        ));
        assert!(mutator.mut_store_object_slots_handle(
            ObjectSlotsHandleStoreTarget::ObjectElements(object),
            Some(replacement_elements),
        ));

        let view = mutator.view();
        let object_record = view.object(object).unwrap();
        let environment_record = view.environment(env).unwrap();
        let code_record = view.code(code).unwrap();
        let realm_record = view.realm(realm).unwrap();
        let shape_record = view.shape(shape).unwrap();

        assert_eq!(object_record.prototype(), Some(proto));
        assert_eq!(object_record.shape(), Some(shape));
        assert_eq!(
            view.object_slots(object_slots).unwrap(),
            &[
                Value::from_string_ref(replacement),
                Value::from_object_ref(proto)
            ]
        );
        assert_eq!(
            view.object_slots(object_record.elements().unwrap())
                .unwrap(),
            &[
                Value::array_hole(),
                Value::array_hole(),
                Value::from_smi(99)
            ]
        );
        assert_eq!(environment_record.outer(), Some(env_outer));
        assert_eq!(environment_record.function_object(), Some(object));
        assert_eq!(
            environment_record.this_value(),
            Value::from_object_ref(object)
        );
        assert_eq!(environment_record.new_target(), Some(proto));
        assert_eq!(environment_record.home_object(), Some(object));
        assert_eq!(
            view.environment_slots(env_slots).unwrap(),
            &[Value::from_string_ref(string)]
        );
        assert_eq!(code_record.parent(), Some(code_parent));
        assert_eq!(code_record.realm(), Some(realm));
        assert_eq!(
            view.code_slots(code_slots).unwrap(),
            &[Value::from_bigint_ref(bigint)]
        );
        assert_eq!(realm_record.global_object(), Some(object));
        assert_eq!(realm_record.global_env(), Some(env));
        assert_eq!(realm_record.bootstrap_code(), Some(code));
        assert_eq!(realm_record.root_shape(), Some(shape));
        assert_eq!(shape_record.parent(), Some(shape_parent));
        assert_eq!(shape_record.prototype_guard(), Some(proto));
    }

    #[test]
    fn rooted_runtime_domains_trace_slot_buffers_and_reclaim_dead_records() {
        let mut heap = PrimitiveHeap::new();
        let roots = PrimitiveRoots::new();
        let (
            live_string,
            live_bigint,
            live_object,
            live_code,
            live_realm,
            dead_string,
            dead_bigint,
            dead_object,
            dead_code,
            dead_realm,
        ) = {
            let mut mutator = heap.mutator();

            let live_string = mutator.alloc_string(
                StringEncoding::Latin1,
                4,
                b"live",
                None,
                AllocationLifetime::Default,
            );
            let dead_string = mutator.alloc_string(
                StringEncoding::Latin1,
                4,
                b"dead",
                None,
                AllocationLifetime::Default,
            );
            let live_bigint =
                mutator.alloc_bigint(BigIntSign::Negative, &[11], AllocationLifetime::Default);
            let dead_bigint =
                mutator.alloc_bigint(BigIntSign::Negative, &[13], AllocationLifetime::Default);

            let live_shape = mutator.alloc_shape(
                RuntimeShapeRecord::new(None, None, 1),
                AllocationLifetime::Default,
            );
            let live_object_slots = mutator.alloc_object_slots(
                1,
                Value::empty_internal_slot(),
                AllocationLifetime::Default,
            );
            let live_object = mutator.alloc_object(
                RuntimeObjectRecord::new(
                    None,
                    Some(live_shape),
                    Some(live_object_slots),
                    None,
                    None,
                ),
                AllocationLifetime::Default,
            );
            let live_env_slots = mutator.alloc_environment_slots(
                1,
                Value::empty_internal_slot(),
                AllocationLifetime::Default,
            );
            let live_env = mutator.alloc_environment(
                RuntimeEnvironmentRecord::new(
                    None,
                    Some(live_env_slots),
                    Some(live_object),
                    Value::empty_internal_slot(),
                    None,
                    None,
                ),
                AllocationLifetime::Default,
            );
            let live_code_slots = mutator.alloc_code_slots(
                1,
                Value::empty_internal_slot(),
                AllocationLifetime::Default,
            );
            let live_realm = mutator.alloc_realm(
                RuntimeRealmRecord::new(Some(live_object), Some(live_env), None, Some(live_shape)),
                AllocationLifetime::Default,
            );
            let live_code = mutator.alloc_code(
                RuntimeCodeRecord::new(None, Some(live_realm), Some(live_code_slots)),
                AllocationLifetime::Default,
            );
            assert!(mutator.init_store_code_handle(
                CodeHandleStoreTarget::RealmBootstrapCode(live_realm),
                Some(live_code),
            ));
            assert!(mutator.init_store_value(
                ValueStoreTarget::ObjectSlot(live_object_slots, 0),
                Value::from_string_ref(live_string),
            ));
            assert!(mutator.init_store_value(
                ValueStoreTarget::CodeSlot(live_code_slots, 0),
                Value::from_bigint_ref(live_bigint),
            ));

            let dead_shape = mutator.alloc_shape(
                RuntimeShapeRecord::new(None, None, 1),
                AllocationLifetime::Default,
            );
            let dead_object_slots = mutator.alloc_object_slots(
                1,
                Value::empty_internal_slot(),
                AllocationLifetime::Default,
            );
            let dead_object = mutator.alloc_object(
                RuntimeObjectRecord::new(
                    None,
                    Some(dead_shape),
                    Some(dead_object_slots),
                    None,
                    None,
                ),
                AllocationLifetime::Default,
            );
            let dead_env_slots = mutator.alloc_environment_slots(
                1,
                Value::empty_internal_slot(),
                AllocationLifetime::Default,
            );
            let dead_env = mutator.alloc_environment(
                RuntimeEnvironmentRecord::new(
                    None,
                    Some(dead_env_slots),
                    Some(dead_object),
                    Value::empty_internal_slot(),
                    None,
                    None,
                ),
                AllocationLifetime::Default,
            );
            let dead_code_slots = mutator.alloc_code_slots(
                1,
                Value::empty_internal_slot(),
                AllocationLifetime::Default,
            );
            let dead_realm = mutator.alloc_realm(
                RuntimeRealmRecord::new(Some(dead_object), Some(dead_env), None, Some(dead_shape)),
                AllocationLifetime::Default,
            );
            let dead_code = mutator.alloc_code(
                RuntimeCodeRecord::new(None, Some(dead_realm), Some(dead_code_slots)),
                AllocationLifetime::Default,
            );
            assert!(mutator.init_store_code_handle(
                CodeHandleStoreTarget::RealmBootstrapCode(dead_realm),
                Some(dead_code),
            ));
            assert!(mutator.init_store_value(
                ValueStoreTarget::ObjectSlot(dead_object_slots, 0),
                Value::from_string_ref(dead_string),
            ));
            assert!(mutator.init_store_value(
                ValueStoreTarget::CodeSlot(dead_code_slots, 0),
                Value::from_bigint_ref(dead_bigint),
            ));

            (
                live_string,
                live_bigint,
                live_object,
                live_code,
                live_realm,
                dead_string,
                dead_bigint,
                dead_object,
                dead_code,
                dead_realm,
            )
        };
        let _rooted = roots.root_realm(live_realm);

        let stats = heap.collect(&roots);
        let view = heap.view();

        assert_eq!(stats.trace.realms_marked, 1);
        assert_eq!(stats.trace.objects_marked, 1);
        assert_eq!(stats.trace.environments_marked, 1);
        assert_eq!(stats.trace.codes_marked, 1);
        assert_eq!(stats.trace.shapes_marked, 1);
        assert_eq!(stats.trace.strings_marked, 1);
        assert_eq!(stats.trace.bigints_marked, 1);
        assert_eq!(stats.objects_reclaimed, 1);
        assert_eq!(stats.environments_reclaimed, 1);
        assert_eq!(stats.codes_reclaimed, 1);
        assert_eq!(stats.realms_reclaimed, 1);
        assert_eq!(stats.shapes_reclaimed, 1);
        assert_eq!(stats.strings_reclaimed, 1);
        assert_eq!(stats.bigints_reclaimed, 1);
        assert_eq!(view.string_payload(live_string), Some(&b"live"[..]));
        assert!(view.bigint(live_bigint).is_some());
        assert!(view.object(live_object).is_some());
        assert!(view.code(live_code).is_some());
        assert!(view.realm(live_realm).is_some());
        assert_eq!(view.string(dead_string), None);
        assert_eq!(view.bigint(dead_bigint), None);
        assert_eq!(view.object(dead_object), None);
        assert_eq!(view.code(dead_code), None);
        assert_eq!(view.realm(dead_realm), None);
    }

    #[test]
    fn rooted_mutator_collects_on_string_slow_path_before_growing_pages() {
        let mut heap = PrimitiveHeap::new();
        heap.set_collection_budget_bytes(1);
        let roots = PrimitiveRoots::new();
        let mut mutator = heap.mutator_with_roots(&roots);
        let live = mutator.alloc_string(
            StringEncoding::Latin1,
            0,
            b"",
            None,
            AllocationLifetime::Default,
        );
        let _rooted = roots.root_string(live);

        let mut reusable = None;
        for _ in 1..PRIMITIVE_SLOTS_PER_PAGE {
            reusable = Some(mutator.alloc_string(
                StringEncoding::Latin1,
                0,
                b"",
                None,
                AllocationLifetime::Default,
            ));
        }

        let replacement = mutator.alloc_string(
            StringEncoding::Latin1,
            0,
            b"",
            None,
            AllocationLifetime::Default,
        );
        let report = mutator.last_collection_report().unwrap();
        let view = mutator.view();

        assert_eq!(
            report.trigger,
            PrimitiveCollectionTrigger::StringAllocationSlowPath
        );
        assert_eq!(report.stats.trace.strings_marked, 1);
        assert_eq!(report.stats.strings_reclaimed, PRIMITIVE_SLOTS_PER_PAGE - 1);
        assert_eq!(
            report.before.strings.reserved_bytes,
            report.after.strings.reserved_bytes
        );
        assert_eq!(view.string_stats().pages, 1);
        assert_eq!(replacement, reusable.unwrap());
        assert_eq!(view.string_payload(replacement), None);
        assert_eq!(view.string(replacement).unwrap().code_unit_len(), 0);
        assert!(report.after.reclaimable_bytes > 0);
        assert_eq!(view.collection_budget_bytes(), report.next_budget_bytes);
    }

    #[test]
    fn allocation_safepoint_polls_active_incremental_major_mark() {
        let roots = PrimitiveRoots::new();
        let mut heap = PrimitiveHeap::new();
        heap.set_major_mark_slice_budget(1);

        let live = heap.mutator().alloc_string(
            StringEncoding::Latin1,
            0,
            b"",
            None,
            AllocationLifetime::Default,
        );
        let _rooted = roots.root_string(live);

        assert!(heap.begin_incremental_mark(&roots));
        assert_eq!(heap.active_incremental_mark_pending_work_items(), Some(1));

        let mutator = &mut heap.mutator_with_roots(&roots);
        let _unrooted = mutator.alloc_string(
            StringEncoding::Latin1,
            7,
            b"trigger",
            None,
            AllocationLifetime::Default,
        );

        assert_eq!(heap.active_incremental_mark_pending_work_items(), Some(0));
        let stats = heap.finish_active_incremental_mark().unwrap();
        assert_eq!(stats.trace.strings_marked, 1);
    }

    #[test]
    fn repeated_allocation_safepoints_drain_active_incremental_major_mark() {
        let roots = PrimitiveRoots::new();
        let mut heap = PrimitiveHeap::new();
        heap.set_major_mark_slice_budget(1);

        let live = heap.mutator().alloc_string(
            StringEncoding::Latin1,
            4,
            b"live",
            None,
            AllocationLifetime::Default,
        );
        let _rooted = roots.root_string(live);

        assert!(heap.begin_incremental_mark(&roots));
        for _ in 0..(PRIMITIVE_SLOTS_PER_PAGE / 2) {
            let mutator = &mut heap.mutator_with_roots(&roots);
            let _allocation = mutator.alloc_string(
                StringEncoding::Latin1,
                0,
                b"",
                None,
                AllocationLifetime::Default,
            );
        }

        assert_eq!(heap.active_incremental_mark_pending_work_items(), Some(0));
        let stats = heap.finish_active_incremental_mark().unwrap();
        assert_eq!(stats.trace.strings_marked, 1);
    }

    #[test]
    fn borrowed_string_views_preserve_encoding_and_lifetimes() {
        let mut heap = PrimitiveHeap::new();
        let mut mutator = heap.mutator();
        let latin1 = mutator.alloc_string(
            StringEncoding::Latin1,
            4,
            b"cafe",
            None,
            AllocationLifetime::Default,
        );
        let utf16 = mutator.alloc_string(
            StringEncoding::Utf16,
            2,
            &[0x41, 0x00, 0xA9, 0x03],
            None,
            AllocationLifetime::Default,
        );

        let latin1_view = {
            let view = mutator.view();
            view.string_view(latin1).unwrap()
        };
        let utf16_view = {
            let view = mutator.view();
            view.string_view(utf16).unwrap()
        };

        assert_eq!(latin1_view.encoding(), StringEncoding::Latin1);
        assert_eq!(latin1_view.latin1_bytes(), Some(&b"cafe"[..]));
        assert_eq!(latin1_view.code_unit_at(3), Some(u16::from(b'e')));
        assert_eq!(latin1_view.code_unit_at(4), None);

        assert_eq!(utf16_view.encoding(), StringEncoding::Utf16);
        assert_eq!(
            utf16_view.utf16_bytes(),
            Some(&[0x41, 0x00, 0xA9, 0x03][..])
        );
        assert_eq!(utf16_view.code_unit_at(0), Some(0x0041));
        assert_eq!(utf16_view.code_unit_at(1), Some(0x03A9));
    }

    #[test]
    fn mutator_caches_string_hashes_and_atom_memoization_through_explicit_api() {
        let mut heap = PrimitiveHeap::new();
        let mut mutator = heap.mutator();
        let latin1 = mutator.alloc_string(
            StringEncoding::Latin1,
            3,
            b"abc",
            None,
            AllocationLifetime::Default,
        );
        let utf16 = mutator.alloc_string(
            StringEncoding::Utf16,
            3,
            &[0x61, 0x00, 0x62, 0x00, 0x63, 0x00],
            None,
            AllocationLifetime::Default,
        );
        let different = mutator.alloc_string(
            StringEncoding::Latin1,
            3,
            b"abd",
            None,
            AllocationLifetime::Default,
        );

        assert_eq!(mutator.string_view(latin1).unwrap().cached_hash(), None);
        let latin1_hash = mutator.cache_string_hash(latin1).unwrap();
        let utf16_hash = mutator.cache_string_hash(utf16).unwrap();
        let different_hash = mutator.cache_string_hash(different).unwrap();
        let atom = AtomId::from_raw(77);

        assert_eq!(latin1_hash, utf16_hash);
        assert_ne!(latin1_hash, different_hash);
        assert_eq!(
            mutator.string_view(latin1).unwrap().cached_hash(),
            Some(latin1_hash)
        );
        assert!(mutator.memoize_string_atom(latin1, atom));
        assert_eq!(
            mutator.string_view(latin1).unwrap().cached_atom(),
            Some(atom)
        );

        let view = mutator.view();
        assert_eq!(view.strings_equal(latin1, utf16), Some(true));
        assert_eq!(view.strings_equal(latin1, different), Some(false));
    }

    #[test]
    fn symbol_views_keep_identity_separate_from_description_text() {
        let mut heap = PrimitiveHeap::new();
        let mut mutator = heap.mutator();
        let description_a = mutator.alloc_string(
            StringEncoding::Latin1,
            4,
            b"same",
            None,
            AllocationLifetime::Default,
        );
        let description_b = mutator.alloc_string(
            StringEncoding::Utf16,
            4,
            &[0x73, 0x00, 0x61, 0x00, 0x6D, 0x00, 0x65, 0x00],
            None,
            AllocationLifetime::Default,
        );
        let ordinary = mutator.alloc_symbol(
            Some(description_a),
            SymbolFlags::for_class(crate::PrimitiveSymbolClass::Ordinary),
            AllocationLifetime::Default,
        );
        let other = mutator.alloc_symbol(
            Some(description_b),
            SymbolFlags::for_class(crate::PrimitiveSymbolClass::Ordinary),
            AllocationLifetime::Default,
        );
        let well_known = mutator.alloc_symbol(
            Some(description_a),
            SymbolFlags::for_class(crate::PrimitiveSymbolClass::WellKnown),
            AllocationLifetime::LongLived,
        );

        let view = mutator.view();
        let ordinary_view = view.symbol_view(ordinary).unwrap();
        let other_view = view.symbol_view(other).unwrap();
        let well_known_view = view.symbol_view(well_known).unwrap();

        assert_ne!(ordinary_view.identity(), other_view.identity());
        assert!(ordinary_view.is_ordinary());
        assert!(well_known_view.is_well_known());
        assert!(ordinary_view
            .description_view()
            .unwrap()
            .equals(other_view.description_view().unwrap()));
        assert_eq!(ordinary_view.class(), crate::PrimitiveSymbolClass::Ordinary);
        assert_eq!(
            well_known_view.class(),
            crate::PrimitiveSymbolClass::WellKnown
        );
    }

    #[test]
    fn bigint_views_expose_canonical_zero_and_little_endian_magnitude() {
        let mut heap = PrimitiveHeap::new();
        let mut mutator = heap.mutator();
        let zero = mutator.alloc_bigint(
            BigIntSign::Negative,
            &[0, 0, 0],
            AllocationLifetime::Default,
        );
        let value = mutator.alloc_bigint(
            BigIntSign::Negative,
            &[0x0102_0304_0506_0708, u64::MAX, 0, 0],
            AllocationLifetime::LongLived,
        );

        let zero_view = mutator.bigint_view(zero).unwrap();
        let value_view = mutator.bigint_view(value).unwrap();

        assert_eq!(zero_view.sign(), BigIntSign::NonNegative);
        assert_eq!(zero_view.limb_count(), 0);
        assert!(zero_view.is_zero());
        assert_eq!(zero_view.limb_bytes_le(), &[]);
        assert_eq!(zero_view.limb_at(0), None);

        assert_eq!(value_view.sign(), BigIntSign::Negative);
        assert_eq!(value_view.limb_count(), 2);
        assert!(!value_view.is_zero());
        assert_eq!(value_view.limb_at(0), Some(0x0102_0304_0506_0708));
        assert_eq!(value_view.limb_at(1), Some(u64::MAX));
        assert_eq!(value_view.limb_at(2), None);
        assert_eq!(
            value_view.limb_bytes_le(),
            &[
                0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0xFF, 0xFF,
            ][..]
        );
    }

    #[test]
    fn store_helpers_report_dead_targets_without_panicking() {
        let mut heap = PrimitiveHeap::new();
        let mut mutator = heap.mutator();
        let cell = mutator.alloc_value_cell(AllocationLifetime::Default);
        let symbol =
            mutator.alloc_symbol(None, SymbolFlags::ordinary(), AllocationLifetime::Default);

        assert!(mutator.free_value_cell(cell).is_some());
        assert!(mutator.free_symbol(symbol).is_some());
        assert!(!mutator.init_store_value(ValueStoreTarget::ValueCell(cell), Value::from_smi(1)));
        assert!(!mutator
            .mut_store_string_handle(StringHandleStoreTarget::SymbolDescription(symbol), None,));
    }
}
