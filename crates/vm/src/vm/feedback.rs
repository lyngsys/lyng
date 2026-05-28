#![allow(
    clippy::inline_always,
    reason = "Feedback helpers are dispatch hot-path probes where the call boundary shows up in tight opcode loops"
)]

use super::{
    code_index,
    status::{
        ArithStatus, CallStatus, CalleeSummary, ComparisonStatus, ConstructStatus,
        KeyedPropertyDenseStatusEntry, KeyedPropertyNamedStatusEntry, KeyedPropertyStatus,
        MetadataTableFootprint, NamedPropertyStatus, NamedPropertyStatusEntry,
    },
    Agent, AtomId, CodeRef, ObjectRef, RealmRef, Value, Vm,
};
use crate::vm::ic_state::{
    keyed_property::{KeyedIcDenseEntry, KeyedIcFamily, KeyedIcNamedEntry},
    CallIcState, KeyedPropertyIcState, PropertyIcState,
};
use crate::vm::metadata_table::{MetadataKind, PropertyMetadata, METADATA_KIND_COUNT};
pub(crate) use crate::vm::metadata_table::LLINT_IC_MODE_NAMED_OWN_INLINE_WRITE;
use lyng_bytecode::FeedbackSiteKind;
use lyng_gc::ValueStoreTarget;
use lyng_objects::{
    FunctionEntryIdentity, KeyedDenseIndexHandler, NamedPropertyCacheEntry, NamedPropertyCachePath,
    NamedPropertyCachePurpose, NamedPropertyHandler, NamedPropertyInlineWriteHandler,
    NamedPropertyProtoHandler, ObjectFlags, ObjectHeader, ObjectKind, PrimitiveWrapperKind,
    ShapeInvalidationObserver, SlotLocation, Watchpoint, PROPERTY_CACHE_MAX_DEPENDENCIES,
};
use lyng_types::{BuiltinFunctionId, FeedbackSlotId, PropertyKey, ShapeId};
use std::cmp::Ordering;

mod polymorphic;

pub(crate) use polymorphic::PolymorphicChain;

const FEEDBACK_ALLOCATION_THRESHOLD: u16 = 2;
const POLYMORPHIC_PROPERTY_CACHE_LIMIT: usize = 8;
const POLYMORPHIC_CALL_CACHE_LIMIT: usize = 8;

/// Phase 3f polymorphic IC cache-hit sidecar capacity. The first `POLY_LIMIT`
/// `entries` are mirrored into a flat `[NamedPropertyHandler; POLY_LIMIT]`
/// array on the feedback site so the inline check can walk shapes 2..N
/// without entering the binary-search slow chain. Entries beyond
/// `POLY_LIMIT` (up to `POLYMORPHIC_PROPERTY_CACHE_LIMIT`) still live in
/// `entries` and are reached via the slow path; mega-poly transition is
/// unchanged. Value chosen by bench evidence on V8 v7 — see
/// `reports/lyng/phase-3f-bench.md`.
pub(in crate::vm) const POLY_LIMIT: usize = 2;

#[inline]
pub(super) fn call_feedback_builtin_is_frame_safe(entry: BuiltinFunctionId) -> bool {
    // Keep this whitelist narrow: these direct-call targets do not inspect caller
    // strictness, dynamically compile source, or re-enter through Function.prototype
    // call helpers, so dispatching from a monomorphic feedback entry preserves the
    // general call path's caller frame and callee realm behavior.
    entry == lyng_types::regexp_exec_builtin()
        || entry == lyng_types::regexp_symbol_replace_builtin()
        || entry == lyng_types::regexp_test_builtin()
        || entry == lyng_types::string_char_code_at_builtin()
        || entry == lyng_types::string_from_char_code_builtin()
        || entry == lyng_types::string_replace_builtin()
        || entry == lyng_types::string_to_upper_case_builtin()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FeedbackInlineCacheState {
    Uninitialized,
    Monomorphic,
    Polymorphic,
    Megamorphic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FeedbackKeyedPropertyFamily {
    DenseIndex,
    NamedAtom,
    Generic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum InlineCacheState {
    Uninitialized,
    Monomorphic,
    Polymorphic,
    Megamorphic,
}

impl From<InlineCacheState> for FeedbackInlineCacheState {
    #[inline]
    fn from(value: InlineCacheState) -> Self {
        match value {
            InlineCacheState::Uninitialized => Self::Uninitialized,
            InlineCacheState::Monomorphic => Self::Monomorphic,
            InlineCacheState::Polymorphic => Self::Polymorphic,
            InlineCacheState::Megamorphic => Self::Megamorphic,
        }
    }
}

impl From<KeyedIcFamily> for FeedbackKeyedPropertyFamily {
    #[inline]
    fn from(value: KeyedIcFamily) -> Self {
        match value {
            KeyedIcFamily::DenseIndex => Self::DenseIndex,
            KeyedIcFamily::NamedAtom => Self::NamedAtom,
            KeyedIcFamily::Generic => Self::Generic,
        }
    }
}

// ── LlInt named-property header variants ─────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LlIntNamedPropertyHeader {
    OwnInline {
        handler_bits: u64,
    },
    OwnOutline {
        handler_bits: u64,
    },
    ProtoInline {
        receiver_word: u64,
        proto_word: u64,
    },
    OwnPolymorphic {
        slot0_handler_bits: u64,
        slot1_handler_bits: u64,
    },
}

// ── Dense-index cache entry ───────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DenseIndexCacheEntry {
    receiver_shape: ShapeId,
    receiver_flags: ObjectFlags,
}

impl DenseIndexCacheEntry {
    #[inline]
    const fn new(receiver_shape: ShapeId, receiver_flags: ObjectFlags) -> Self {
        Self {
            receiver_shape,
            receiver_flags,
        }
    }

    #[inline]
    const fn from_header(header: ObjectHeader) -> Self {
        Self::new(header.shape(), header.flags())
    }

    #[inline]
    fn matches_header(self, header: ObjectHeader) -> bool {
        self.receiver_shape == header.shape() && self.receiver_flags == header.flags()
    }
}

// ── Call/Construct cache entries ──────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CallCacheEntry {
    pub(super) callee: ObjectRef,
    pub(super) callee_shape: ShapeId,
    pub(super) realm: Option<RealmRef>,
    pub(super) builtin: Option<BuiltinFunctionId>,
}

impl CallCacheEntry {
    #[inline]
    fn from_callee(agent: &Agent, callee: ObjectRef) -> Option<Self> {
        let callee_shape = agent
            .objects()
            .object_header(agent.heap().view(), callee)?
            .shape();
        let function = agent.objects().function_data(callee);
        let realm = function.and_then(lyng_objects::FunctionObjectData::realm);
        let builtin = function.and_then(|function| {
            let FunctionEntryIdentity::Native(entry) = function.entry()? else {
                return None;
            };
            entry.builtin_entry()
        });
        Some(Self {
            callee,
            callee_shape,
            realm,
            builtin,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ConstructCacheEntry {
    pub(super) constructor: ObjectRef,
    pub(super) constructor_shape: ShapeId,
    pub(super) realm: Option<RealmRef>,
    pub(super) created_shape: Option<ShapeId>,
}

impl ConstructCacheEntry {
    #[inline]
    fn from_constructor(
        agent: &Agent,
        constructor: ObjectRef,
        created: Option<ObjectRef>,
    ) -> Option<Self> {
        let constructor_shape = agent
            .objects()
            .object_header(agent.heap().view(), constructor)?
            .shape();
        let realm = agent
            .objects()
            .function_data(constructor)
            .and_then(lyng_objects::FunctionObjectData::realm);
        let created_shape = Self::created_shape(agent, created);
        Some(Self {
            constructor,
            constructor_shape,
            realm,
            created_shape,
        })
    }

    #[inline]
    fn created_shape(agent: &Agent, created: Option<ObjectRef>) -> Option<ShapeId> {
        created.and_then(|object| {
            agent
                .objects()
                .object_header(agent.heap().view(), object)
                .map(lyng_objects::ObjectHeader::shape)
        })
    }
}

// ── Keyed property named-atom cache entry ────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct KeyedNamedPropertyCacheEntry {
    pub(super) atom: AtomId,
    pub(super) entry: NamedPropertyCacheEntry,
}

// ── Per-code-object call/construct cache storage ──────────────────────────────

/// Per-code-object Call/Construct cache entries. Stored alongside the
/// `CallIcState` side-table in `Vm::call_cache_entries` /
/// `Vm::construct_cache_entries`. Phase D.2.4 makes these the sole source of
/// truth for IC cache-hit resolution.
#[derive(Clone, Debug, Default)]
pub(super) struct CallCacheStorage {
    pub(super) entries: [Option<CallCacheEntry>; POLYMORPHIC_CALL_CACHE_LIMIT],
}

/// Per-code-object Construct cache entries (parallel to `CallCacheStorage`).
#[derive(Clone, Debug, Default)]
pub(super) struct ConstructCacheStorage {
    pub(super) entries: [Option<ConstructCacheEntry>; POLYMORPHIC_CALL_CACHE_LIMIT],
}

/// Per-code-object KeyedProperty named-atom cache entries. These are
/// the actual cache entry data; `KeyedPropertyIcState` holds the
/// Rust-only structural state (entry count, family, sidecars).
/// Stored in `Vm::keyed_property_named_entries`.
#[derive(Clone, Debug)]
pub(super) struct KeyedPropertyNamedEntries {
    pub(super) entries: [Option<KeyedNamedPropertyCacheEntry>; POLYMORPHIC_PROPERTY_CACHE_LIMIT],
}

impl Default for KeyedPropertyNamedEntries {
    fn default() -> Self {
        Self {
            entries: [None; POLYMORPHIC_PROPERTY_CACHE_LIMIT],
        }
    }
}

// ── Vm impl ───────────────────────────────────────────────────────────────────

impl Vm {
    // ── Warmup / allocation bookkeeping ──────────────────────────────────────

    /// Bumps the warmup counter and (once the threshold is hit) marks the code
    /// as having its IC side-tables activated. Returns `true` if the slot is
    /// valid AND the code has reached the allocation threshold (either before
    /// this call or crossing it on this call). Returns `false` in two cases:
    ///   - the descriptor for `slot` is not found in `installed` (invalid slot)
    ///   - the slot exists but the warmup counter is still below the threshold
    ///     (still in the warm-up phase; callers should skip IC state updates)
    ///
    /// This mirrors the original `FeedbackVector::site_mut` gate: IC state
    /// recording only starts after the allocation threshold is crossed. Before
    /// that point only the warmup counter is updated.
    fn ensure_feedback_slot_execution(&mut self, code: CodeRef, slot: FeedbackSlotId) -> bool {
        let index = code_index(code);
        // Ensure installed capacity.
        if self.installed.get(index).and_then(Option::as_ref).is_none() {
            return false;
        }
        let installed = self.installed[index].as_ref().unwrap();
        if installed.feedback_descriptor_for_slot(slot).is_none() {
            return false;
        }
        // Bump warmup counter and maybe allocate.
        if !self.tiering.is_allocated(code) {
            self.tiering.bump_warmup(code);
            if self.tiering.warmup_counter(code) >= FEEDBACK_ALLOCATION_THRESHOLD {
                self.tiering.mark_allocated(code);
            }
        }
        self.tiering.observe_feedback_event(code);
        // Return `true` only once the code is fully allocated (threshold reached).
        // Callers use this gate to decide whether to update IC side-table state.
        self.tiering.is_allocated(code)
    }

    pub(crate) fn record_feedback_slot(&mut self, code: CodeRef, slot: Option<FeedbackSlotId>) {
        let Some(slot) = slot else {
            return;
        };
        let _ = self.ensure_feedback_slot_execution(code, slot);
    }

    pub(in crate::vm) fn drain_llint_scalar_feedback(&mut self) {
        // Phase C.4: x21 now holds the MetadataTable base. Arith IC sites write
        // directly to ArithMetadata.{observed_bits, execution_count}. Drain those
        // from metadata_tables instead of feedback_flat_storage.
        //
        // Step 1: collect (code, slot) pairs for all Arith-kind slots, so that the
        // immutable `installed` borrow can be dropped before we mutate the tables.
        let mut arith_slots: Vec<(CodeRef, FeedbackSlotId)> = Vec::new();
        for (tbl_index, installed_opt) in self.installed.iter().enumerate() {
            let Some(installed) = installed_opt.as_ref() else {
                continue;
            };
            let Some(code_raw) = u32::try_from(tbl_index)
                .ok()
                .and_then(|index| index.checked_add(1))
            else {
                continue;
            };
            let Some(code) = CodeRef::from_raw(code_raw) else {
                continue;
            };
            for descriptor in installed.feedback_slot_descriptors().iter().flatten() {
                if descriptor.kind() == FeedbackSiteKind::Arithmetic {
                    arith_slots.push((code, descriptor.slot()));
                }
            }
        }

        // Step 2: for each Arith slot, drain ArithMetadata and collect pending updates.
        let mut pending = Vec::new();
        for (code, slot) in arith_slots {
            let Some(table) = self.metadata_table_mut(code) else {
                continue;
            };
            let arith = table.arith_mut(slot.get());
            let observed_bits = arith.observed_bits;
            let execution_count = arith.execution_count;
            // LLINT_FEEDBACK_OBSERVED_SMI = 0x1 (SMI observed bit in ArithMetadata)
            if execution_count == 0 || observed_bits & 0x1 == 0 {
                continue;
            }
            // Zero out both fields (drain-and-clear).
            arith.observed_bits = 0;
            arith.execution_count = 0;
            pending.push((code, slot, execution_count));
        }

        for (code, slot, count) in pending {
            self.record_feedback_slot_batch(code, slot, count);
        }
    }

    fn record_feedback_slot_batch(&mut self, code: CodeRef, slot: FeedbackSlotId, count: u32) {
        if count == 0 {
            return;
        }
        let index = code_index(code);
        let Some(installed) = self.installed.get(index).and_then(Option::as_ref) else {
            return;
        };
        if installed.feedback_descriptor_for_slot(slot).is_none() {
            return;
        }
        // Bump warmup by up to `count` until allocation threshold, then account remainder.
        if !self.tiering.is_allocated(code) {
            let current_warmup = self.tiering.warmup_counter(code);
            let events_until_allocation = u32::from(
                FEEDBACK_ALLOCATION_THRESHOLD
                    .saturating_sub(current_warmup)
                    .max(1),
            );
            let warmup_events = count.min(events_until_allocation);
            let warmup_increment =
                u16::try_from(warmup_events).expect("feedback warmup threshold fits in u16");
            self.tiering.bump_warmup_by(code, warmup_increment);
            if count >= events_until_allocation {
                self.tiering.mark_allocated(code);
            }
        }
        self.tiering.observe_feedback_events(code, count);
    }

    // ── Call / Construct observation ─────────────────────────────────────────

    #[inline]
    pub(super) fn observe_call_target(
        &mut self,
        agent: &Agent,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        callee: ObjectRef,
    ) {
        let Some(slot) = slot else {
            return;
        };
        if !self.ensure_feedback_slot_execution(code, slot) {
            return;
        }
        // Split-borrow: &mut CallIcState (via Vec slab) + &mut call_cache_entries.
        let cache_entry = CallCacheEntry::from_callee(agent, callee);
        let Self {
            call_ic_states,
            call_cache_entries,
            ..
        } = self;
        let index = code_index(code);
        let slot_zero = (slot.get() - 1) as usize;
        let slab = call_ic_states[index]
            .as_deref_mut()
            .expect("call_ic_states slab must be allocated at install");
        let call_state = slab[slot_zero].get_or_insert_with(CallIcState::default);
        call_state.execution_count = call_state.execution_count.saturating_add(1);
        let cache_storage = call_cache_entries
            .entry((code, slot))
            .or_insert_with(|| Box::new(CallCacheStorage::default()));
        observe_call_target_on_state(call_state, cache_storage, cache_entry);
        // Write asm-readable bits to CallMetadata.
        let ic_state = *call_state;
        if let Some(table) = self.metadata_table_mut(code) {
            let meta = table.call_mut(slot.get());
            meta.mode = ic_mode_from_cache_state(ic_state.cache_state);
            meta.execution_count = ic_state.execution_count;
        }
    }

    #[inline]
    pub(super) fn cached_frame_safe_builtin_call_target(
        &self,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        callee: ObjectRef,
    ) -> Option<BuiltinFunctionId> {
        let slot = slot?;
        let ic_state = self.call_ic_state(code, slot)?;
        if ic_state.cache_state != InlineCacheState::Monomorphic {
            return None;
        }
        let storage = self.call_cache_entries.get(&(code, slot))?;
        let entry = storage.entries[0]?;
        if entry.callee != callee {
            return None;
        }
        entry
            .builtin
            .filter(|builtin| call_feedback_builtin_is_frame_safe(*builtin))
    }

    #[inline]
    pub(super) fn observe_construct_target(
        &mut self,
        agent: &Agent,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        constructor: ObjectRef,
        created: Option<ObjectRef>,
    ) {
        let Some(slot) = slot else {
            return;
        };
        if !self.ensure_feedback_slot_execution(code, slot) {
            return;
        }
        // Split-borrow: &mut CallIcState (via construct slab) + &mut construct_cache_entries.
        let cache_entry = ConstructCacheEntry::from_constructor(agent, constructor, created);
        let Self {
            construct_ic_states,
            construct_cache_entries,
            ..
        } = self;
        let index = code_index(code);
        let slot_zero = (slot.get() - 1) as usize;
        let slab = construct_ic_states[index]
            .as_deref_mut()
            .expect("construct_ic_states slab must be allocated at install");
        let construct_state = slab[slot_zero].get_or_insert_with(CallIcState::default);
        construct_state.execution_count = construct_state.execution_count.saturating_add(1);
        let cache_storage = construct_cache_entries
            .entry((code, slot))
            .or_insert_with(|| Box::new(ConstructCacheStorage::default()));
        observe_construct_target_on_state(
            agent,
            construct_state,
            cache_storage,
            constructor,
            cache_entry,
            created,
        );
        // Write asm-readable bits to CallMetadata (construct and call share the metadata kind).
        let ic_state = *construct_state;
        if let Some(table) = self.metadata_table_mut(code) {
            let meta = table.call_mut(slot.get());
            meta.mode = ic_mode_from_cache_state(ic_state.cache_state);
            meta.execution_count = ic_state.execution_count;
        }
    }

    // ── Named property observation ────────────────────────────────────────────

    /// Read the bit-packed monomorphic `OwnData` IC handler for one feedback
    /// slot. Returns `None` when the slot is absent, the site isn't a
    /// named-property site, or the cache is in any state other than
    /// monomorphic-OwnData. Phase 3 IC cache hit path entry point.
    #[inline(always)]
    pub(super) fn named_property_own_data_handler(
        &self,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
    ) -> Option<NamedPropertyHandler> {
        let state = self.property_ic_state(code, slot?)?;
        if state.monomorphic_own_data_handler.is_valid() {
            Some(state.monomorphic_own_data_handler)
        } else {
            None
        }
    }

    /// Read the bit-packed one-hop `PrototypeData` IC handler for one
    /// feedback slot. Phase 3e IC cache path entry point.
    #[inline(always)]
    pub(super) fn named_property_proto_data_handler(
        &self,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
    ) -> Option<NamedPropertyProtoHandler> {
        let state = self.property_ic_state(code, slot?)?;
        if state.monomorphic_proto_data_handler.is_valid() {
            Some(state.monomorphic_proto_data_handler)
        } else {
            None
        }
    }

    /// Side-effect helper for the inlined IC cache hit path: increment the
    /// per-site execution counter and emit a tier feedback event.
    ///
    /// Only updates the execution count when the PropertyIcState already
    /// exists (i.e. the slow path has previously initialized this slot and
    /// the code has crossed the allocation threshold). A `None` result from
    /// `get_mut` means the slot was never through the slow path; we skip
    /// the update to match the old `FeedbackVector::site_mut` gate.
    ///
    /// Phase D.4.2: the PropertyMetadata mirror-write was removed from the
    /// hot path. `meta.execution_count` is not read by anything in production
    /// (it was test-only), and `meta.mode` is restored exclusively at
    /// install time (`named_property_install_slow_path`) and on demand from
    /// `refresh_named_property_metadata_if_stale` — see the doc-comment
    /// there for the C4 invariant.
    #[inline(always)]
    pub(super) fn record_named_property_cache_hit(&mut self, code: CodeRef, slot: FeedbackSlotId) {
        let Some(state) = self.property_ic_state_mut(code, slot) else {
            self.tiering.observe_feedback_event(code);
            return;
        };
        state.execution_count = state.execution_count.saturating_add(1);
        self.tiering.observe_feedback_event(code);
    }

    /// Restore `PropertyMetadata.mode` from `PropertyIcState` if the asm-visible
    /// mode byte has been zeroed while the Rust-side state still describes a
    /// live IC entry.
    ///
    /// Called from `execute_get_named_property_opcode` (the slow-path entry
    /// reachable from the asm `op_get_named_property_dsl` `.slow:` branch).
    /// Asm reads `meta.mode` to pick an inline IC handler; when asm reads
    /// `mode == 0` it falls into Rust, and we need to repaint `meta` so the
    /// next asm execution can take the inline IC route again. This is the C4
    /// invariant (verified by the `c4_asm_ic_*_reads_from_metadata_table`
    /// test): `mode = 0` on entry → mode must be non-zero on exit.
    ///
    /// The fast common case (mode already non-zero) is a single byte load +
    /// taken-not-taken branch.
    #[inline]
    pub(super) fn refresh_named_property_metadata_if_stale(
        &mut self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) {
        let Some(table) = self
            .metadata_tables
            .get(code_index(code))
            .and_then(Option::as_ref)
        else {
            return;
        };
        if table.property(slot.get()).mode != 0 {
            return;
        }
        let Some(state) = self.property_ic_state(code, slot) else {
            return;
        };
        let llint_header = Self::named_llint_load_header_from_state(state);
        let generation = state.generation;
        let execution_count = state.execution_count;
        // `state` borrow ends here; reacquire `table` mutably for the write.
        if let Some(table) = self.metadata_table_mut(code) {
            let meta = table.property_mut(slot.get());
            Self::project_property_into_meta(llint_header, generation, execution_count, meta);
        }
    }

    /// Phase 3d named-keyed cache handler lookup.
    #[inline(always)]
    pub(super) fn keyed_property_named_own_data_handler(
        &self,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        atom: AtomId,
    ) -> Option<NamedPropertyHandler> {
        let state = self.keyed_property_ic_state(code, slot?)?;
        if state.monomorphic_named_atom == atom.raw()
            && state.monomorphic_named_own_data_handler.is_valid()
        {
            Some(state.monomorphic_named_own_data_handler)
        } else {
            None
        }
    }

    /// Phase 3e named-keyed proto cache handler lookup.
    #[inline(always)]
    pub(super) fn keyed_property_named_proto_data_handler(
        &self,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        atom: AtomId,
    ) -> Option<NamedPropertyProtoHandler> {
        let state = self.keyed_property_ic_state(code, slot?)?;
        if state.monomorphic_named_atom == atom.raw()
            && state.monomorphic_named_proto_data_handler.is_valid()
        {
            Some(state.monomorphic_named_proto_data_handler)
        } else {
            None
        }
    }

    /// Phase 3d dense-keyed cache handler lookup.
    #[inline(always)]
    pub(super) fn keyed_property_dense_index_handler(
        &self,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
    ) -> Option<KeyedDenseIndexHandler> {
        let state = self.keyed_property_ic_state(code, slot?)?;
        if state.monomorphic_dense_index_handler.is_valid() {
            Some(state.monomorphic_dense_index_handler)
        } else {
            None
        }
    }

    /// Phase 3f polymorphic-OwnData IC lookup.
    #[inline(always)]
    pub(super) fn named_property_polymorphic_own_data_handler(
        &self,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        receiver_shape: ShapeId,
    ) -> Option<NamedPropertyHandler> {
        let state = self.property_ic_state(code, slot?)?;
        let target = Some(receiver_shape);
        for slot_index in 0..POLY_LIMIT {
            let handler = state.polymorphic_own_data_handlers[slot_index];
            if handler.is_valid() && handler.receiver_shape() == target {
                return Some(handler);
            }
        }
        None
    }

    /// Phase 3f polymorphic-OwnData keyed-named IC cache-hit lookup.
    #[inline(always)]
    pub(super) fn keyed_property_named_polymorphic_own_data_handler(
        &self,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        atom: AtomId,
        receiver_shape: ShapeId,
    ) -> Option<NamedPropertyHandler> {
        let state = self.keyed_property_ic_state(code, slot?)?;
        let target = Some(receiver_shape);
        let atom_raw = atom.raw();
        for slot_index in 0..POLY_LIMIT {
            let handler = state.polymorphic_named_own_data_handlers[slot_index];
            if handler.is_valid()
                && state.polymorphic_named_atoms[slot_index] == atom_raw
                && handler.receiver_shape() == target
            {
                return Some(handler);
            }
        }
        None
    }

    /// Phase 3f polymorphic dense-index keyed IC cache-hit lookup.
    #[inline(always)]
    pub(super) fn keyed_property_dense_polymorphic_handlers(
        &self,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
    ) -> Option<&[KeyedDenseIndexHandler; POLY_LIMIT]> {
        let state = self.keyed_property_ic_state(code, slot?)?;
        if matches!(state.cache_state, InlineCacheState::Polymorphic)
            && state.family == Some(KeyedIcFamily::DenseIndex)
        {
            Some(&state.polymorphic_dense_index_handlers)
        } else {
            None
        }
    }

    // ── Named property inline-cache load/store ────────────────────────────────

    pub(super) fn try_named_property_load_inline_cache_hit(
        &mut self,
        agent: &Agent,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
    ) -> Option<Value> {
        let slot = slot?;
        let state = self.property_ic_state(code, slot)?;
        let chain = self.polymorphic_chain(code, slot);
        let value = state.try_load(agent, chain, receiver)?;
        // Phase D.4.2: the PropertyMetadata mirror-write was removed from the
        // hot path. `meta.execution_count` is not read by anything in
        // production, and `meta.mode` is restored at install time or via
        // `refresh_named_property_metadata_if_stale` on slow-path entry.
        let state = self
            .property_ic_state_mut(code, slot)
            .expect("state exists — checked by the immutable borrow above");
        state.execution_count = state.execution_count.saturating_add(1);
        self.tiering.observe_feedback_event(code);
        Some(value)
    }

    pub(super) fn try_named_property_store_inline_cache(
        &self,
        agent: &mut Agent,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
        atom: AtomId,
        value: Value,
    ) -> Option<bool> {
        let slot = slot?;
        let state = self.property_ic_state(code, slot)?;
        let chain = self.polymorphic_chain(code, slot);
        state.try_store(agent, chain, receiver, atom, value)
    }

    pub(super) fn observe_named_property_slow_path(
        &mut self,
        agent: &mut Agent,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
        atom: AtomId,
        purpose: NamedPropertyCachePurpose,
    ) {
        let Some(slot) = slot else {
            return;
        };
        // Gate IC state updates on allocation (mirrors the original
        // `FeedbackVector::site_mut` gate: only record after threshold).
        if !self.ensure_feedback_slot_execution(code, slot) {
            return;
        }
        let plan = agent
            .objects()
            .plan_named_property_cache_entry(
                agent.heap().view(),
                receiver,
                PropertyKey::from_atom(atom),
                purpose,
            )
            .ok()
            .flatten();
        self.record_named_property_cache_entry(agent, code, slot, plan, purpose);
    }

    pub(super) fn observe_named_property_cache_entry(
        &mut self,
        agent: &mut Agent,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        plan: Option<NamedPropertyCacheEntry>,
        purpose: NamedPropertyCachePurpose,
    ) {
        let Some(slot) = slot else {
            return;
        };
        // Gate IC state updates on allocation.
        if !self.ensure_feedback_slot_execution(code, slot) {
            return;
        }
        self.record_named_property_cache_entry(agent, code, slot, plan, purpose);
    }

    fn record_named_property_cache_entry(
        &mut self,
        agent: &mut Agent,
        code: CodeRef,
        slot: FeedbackSlotId,
        plan: Option<NamedPropertyCacheEntry>,
        purpose: NamedPropertyCachePurpose,
    ) {
        // Phase D.2.4: cleared slots are removed from the map entirely by
        // `clear_ic_slot_if_generation_matches`. No reinit check needed here;
        // `named_property_install_slow_path` will lazily insert a fresh entry
        // via `entry(...).or_default()` if the map entry is absent.

        // Spec 2 Phase A: register AdaptiveProtoLoad watchpoints for PrototypeData plans.
        if let Some(plan_entry) = plan
            && plan_entry.path() == NamedPropertyCachePath::PrototypeData
            && !Self::register_proto_chain_watchpoints(self, agent, code, slot, plan_entry)
        {
            return;
        }
        self.named_property_install_slow_path(code, slot, plan, purpose);

        // Spec 2 Task 6: register AdaptiveOwnWrite on the source shape for Store
        // installs of OwnData / OwnDataTransition entries that produced a valid
        // asm handler. Registering AFTER install so the generation captured here
        // matches what clear_ic_slot_if_generation_matches will see when the
        // watchpoint fires.
        if purpose == NamedPropertyCachePurpose::Store
            && let Some(plan_entry) = plan
            && matches!(
                plan_entry.path(),
                NamedPropertyCachePath::OwnData | NamedPropertyCachePath::OwnDataTransition
            )
            && let Some(state) = self.property_ic_state(code, slot)
            && state.monomorphic_own_inline_write_handler.is_valid()
        {
            let source_shape = plan_entry.receiver_shape();
            let generation = state.generation;
            let observer = ShapeInvalidationObserver::AdaptiveOwnWrite {
                code,
                slot,
                generation,
            };
            // Registration may fail (Err(Invalidated)) if the shape is
            // already invalidated — in that case the next slow-path miss
            // re-attempts on the post-invalidation shape. Drop the result.
            let _ = agent
                .objects_mut()
                .watchpoint_set_mut(source_shape)
                .register(Watchpoint::ShapeInvalidation { observer });
        }
    }

    /// Slow-path install for a named-property IC entry.
    fn named_property_install_slow_path(
        &mut self,
        code: CodeRef,
        slot: FeedbackSlotId,
        plan: Option<NamedPropertyCacheEntry>,
        purpose: NamedPropertyCachePurpose,
    ) {
        // Split-borrow: we need &mut PropertyIcState and the per-(code,slot)
        // polymorphic-chain entry at the same time. Project into both Vec
        // slabs without going through accessors.
        let Self {
            property_ic_states,
            polymorphic_chains,
            ..
        } = self;
        let index = code_index(code);
        let slot_zero = (slot.get() - 1) as usize;
        let slab = property_ic_states[index]
            .as_deref_mut()
            .expect("property_ic_states slab must be allocated at install");
        let state = slab[slot_zero].get_or_insert_with(PropertyIcState::default);
        let chain_slot = &mut polymorphic_chains[index]
            .as_deref_mut()
            .expect("polymorphic_chains slab must be allocated at install")[slot_zero];
        Self::named_property_observe_slow_path_on_state(state, chain_slot, plan);

        // Write asm-readable bits to PropertyMetadata, routing on purpose.
        // Store slots project the write-side handler (mode = 5); Load slots use
        // the existing load-side projection (mode = 1/2/3/4).
        // Extract all Copy fields from `state` before releasing the split-borrow
        // so that `self.metadata_table_mut(code)` can take `&mut self`.
        let generation = state.generation;
        let execution_count = state.execution_count;
        let write_handler = state.monomorphic_own_inline_write_handler;
        let llint_header = Self::named_llint_load_header_from_state(state);
        // Split-borrow on `state` / `chain_slot` ends here.
        if let Some(table) = self.metadata_table_mut(code) {
            let meta = table.property_mut(slot.get());
            match purpose {
                NamedPropertyCachePurpose::Store => {
                    Self::project_property_write_into_meta(
                        write_handler,
                        generation,
                        execution_count,
                        meta,
                    );
                }
                NamedPropertyCachePurpose::Load => {
                    Self::project_property_into_meta(
                        llint_header,
                        generation,
                        execution_count,
                        meta,
                    );
                }
            }
        }
    }

    fn named_property_observe_slow_path_on_state(
        state: &mut PropertyIcState,
        chain_slot: &mut Option<PolymorphicChain>,
        plan: Option<NamedPropertyCacheEntry>,
    ) {
        let Some(plan) = plan else {
            state.promote_to_megamorphic();
            *chain_slot = None;
            return;
        };
        match state.cache_state {
            InlineCacheState::Megamorphic => {}
            InlineCacheState::Uninitialized => {
                // Note: generation is managed exclusively via
                // `bump_generation_for_install` (called from
                // `register_adaptive_proto_load_for_chain` before
                // watchpoint registration). Do NOT bump it here — OwnData
                // entries never register watchpoints and would get a
                // spurious bump; PrototypeData entries bump via the
                // watchpoint path and would double-bump.
                state.install_first_entry(plan);
            }
            InlineCacheState::Monomorphic | InlineCacheState::Polymorphic => {
                Self::named_property_install_or_update_on_state(state, chain_slot, plan);
            }
        }
    }

    fn named_property_install_or_update_on_state(
        state: &mut PropertyIcState,
        chain_slot: &mut Option<PolymorphicChain>,
        plan: NamedPropertyCacheEntry,
    ) {
        let receiver_shape = plan.receiver_shape();
        match state.search_entry_index(receiver_shape) {
            Ok(index) => {
                state.entries[index] = Some(plan);
                state.refresh_sidecars();
            }
            Err(inline_insert) => {
                if inline_insert < POLY_LIMIT {
                    Self::named_property_insert_into_inline_on_state(
                        state,
                        chain_slot,
                        inline_insert,
                        plan,
                    );
                } else {
                    Self::named_property_insert_into_chain_on_state(state, chain_slot, plan);
                }
            }
        }
    }

    fn named_property_insert_into_inline_on_state(
        state: &mut PropertyIcState,
        chain_slot: &mut Option<PolymorphicChain>,
        inline_insert: usize,
        plan: NamedPropertyCacheEntry,
    ) {
        debug_assert!(inline_insert < POLY_LIMIT);
        let inline_count = state.inline_count();
        let chain_len = chain_slot.as_ref().map_or(0, PolymorphicChain::len);
        let total = inline_count + chain_len;
        if total >= POLYMORPHIC_PROPERTY_CACHE_LIMIT {
            state.promote_to_megamorphic();
            *chain_slot = None;
            return;
        }

        if inline_count >= POLY_LIMIT {
            let displaced = state.entries[POLY_LIMIT - 1]
                .take()
                .expect("inline slot must be populated when inline_count >= POLY_LIMIT");
            let chain = chain_slot.get_or_insert_with(PolymorphicChain::new);
            chain.insert_at(0, displaced);
            state
                .entries
                .copy_within(inline_insert..POLY_LIMIT - 1, inline_insert + 1);
        } else {
            state
                .entries
                .copy_within(inline_insert..inline_count, inline_insert + 1);
        }
        state.entries[inline_insert] = Some(plan);
        state.entry_count = state.entry_count.saturating_add(1);
        state.cache_state = if state.entry_count == 1 {
            InlineCacheState::Monomorphic
        } else {
            InlineCacheState::Polymorphic
        };
        state.refresh_sidecars();
    }

    fn named_property_insert_into_chain_on_state(
        state: &mut PropertyIcState,
        chain_slot: &mut Option<PolymorphicChain>,
        plan: NamedPropertyCacheEntry,
    ) {
        let receiver_shape = plan.receiver_shape();
        if let Some(chain) = chain_slot.as_mut()
            && let Ok(index) = chain.search_sorted(receiver_shape)
        {
            chain.replace_at(index, plan);
            return;
        }

        let inline_count = state.inline_count();
        let chain_len = chain_slot.as_ref().map_or(0, PolymorphicChain::len);
        if inline_count + chain_len >= POLYMORPHIC_PROPERTY_CACHE_LIMIT {
            state.promote_to_megamorphic();
            *chain_slot = None;
            return;
        }
        let chain = chain_slot.get_or_insert_with(PolymorphicChain::new);
        let insert_at = match chain.search_sorted(receiver_shape) {
            Err(index) => index,
            Ok(_) => unreachable!(
                "chain replace branch handled above; chain.search_sorted must miss here"
            ),
        };
        chain.insert_at(insert_at, plan);
        state.entry_count = state.entry_count.saturating_add(1);
        state.cache_state = InlineCacheState::Polymorphic;
    }

    /// Returns `true` when the install may proceed.
    fn register_proto_chain_watchpoints(
        vm: &mut Self,
        agent: &mut Agent,
        code: CodeRef,
        slot: FeedbackSlotId,
        entry: NamedPropertyCacheEntry,
    ) -> bool {
        let mut chain: Vec<ShapeId> = Vec::with_capacity(PROPERTY_CACHE_MAX_DEPENDENCIES);
        for index in 1..usize::from(entry.dependency_count()) {
            let Some(dep) = entry.dependency(index) else {
                return false;
            };
            chain.push(dep.shape());
        }
        agent
            .register_adaptive_proto_load_for_chain(code, slot, &chain, vm)
            .is_ok()
    }

    // ── Keyed property observation ────────────────────────────────────────────

    pub(super) fn try_keyed_property_load_inline_cache(
        &self,
        agent: &Agent,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
        atom: AtomId,
    ) -> Option<Value> {
        let slot = slot?;
        let state = self.keyed_property_ic_state(code, slot)?;
        let named_entries = self.keyed_property_named_entries.get(&(code, slot));
        try_keyed_named_load(state, named_entries, agent, receiver, atom)
    }

    pub(super) fn try_keyed_property_store_inline_cache(
        &self,
        agent: &mut Agent,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
        atom: AtomId,
        value: Value,
    ) -> Option<bool> {
        let slot = slot?;
        let state = self.keyed_property_ic_state(code, slot)?;
        let named_entries = self.keyed_property_named_entries.get(&(code, slot));
        try_keyed_named_store(state, named_entries, agent, receiver, atom, value)
    }

    pub(super) fn try_keyed_dense_index_load_inline_cache_hit(
        &mut self,
        agent: &Agent,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
        index: u32,
    ) -> Option<Value> {
        let slot = slot?;
        let state = self.keyed_property_ic_state(code, slot)?;
        let dense_entries = &state.dense_entries;
        let dense_entry_count = state.dense_entry_count;
        let family = state.family;
        let cache_state = state.cache_state;
        // Require DenseIndex family and Mono/Poly.
        if family != Some(KeyedIcFamily::DenseIndex) {
            return None;
        }
        match cache_state {
            InlineCacheState::Monomorphic | InlineCacheState::Polymorphic => {}
            _ => return None,
        }
        let header = agent
            .objects()
            .object_header(agent.heap().view(), receiver)?;
        let matched = (0..usize::from(dense_entry_count)).any(|i| {
            if let Some(entry) = dense_entries[i] {
                DenseIndexCacheEntry::new(entry.receiver_shape, entry.receiver_flags)
                    .matches_header(header)
            } else {
                false
            }
        });
        if !matched {
            return None;
        }
        let value = Self::dense_value_from_header(agent, header, index)?;

        // Hit: bump keyed state execution count.
        let state = self.keyed_property_ic_state_or_default_mut(code, slot);
        state.execution_count = state.execution_count.wrapping_add(1);
        let ec = state.execution_count;
        if let Some(table) = self.metadata_table_mut(code) {
            table.keyed_property_mut(slot.get()).execution_count = ec;
        }
        self.tiering.observe_feedback_event(code);
        Some(value)
    }

    pub(super) fn try_keyed_dense_index_store_inline_cache_hit(
        &mut self,
        agent: &mut Agent,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
        index: u32,
        value: Value,
    ) -> Option<bool> {
        let slot = slot?;
        if value == Value::array_hole() {
            return None;
        }
        let state = self.keyed_property_ic_state(code, slot)?;
        if state.family != Some(KeyedIcFamily::DenseIndex) {
            return None;
        }
        match state.cache_state {
            InlineCacheState::Monomorphic | InlineCacheState::Polymorphic => {}
            _ => return None,
        }
        let header = agent
            .objects()
            .object_header(agent.heap().view(), receiver)?;
        let dense_entries = &state.dense_entries;
        let dense_entry_count = state.dense_entry_count;
        let matched = (0..usize::from(dense_entry_count)).any(|i| {
            if let Some(entry) = dense_entries[i] {
                DenseIndexCacheEntry::new(entry.receiver_shape, entry.receiver_flags)
                    .matches_header(header)
            } else {
                false
            }
        });
        if !matched {
            return None;
        }
        let elements = header.elements()?;
        let index_usize = usize::try_from(index).expect("u32 index should fit into usize");
        let current = agent
            .heap()
            .view()
            .object_slots(elements.raw())?
            .get(index_usize)
            .copied()
            .unwrap_or(Value::array_hole());
        if current == Value::array_hole() {
            return None;
        }
        let stored = agent.with_heap_and_objects(|heap, _objects| {
            let mut mutator = heap.mutator();
            mutator.mut_store_value(ValueStoreTarget::ObjectSlot(elements.raw(), index), value)
        });
        if !stored {
            return None;
        }
        // Hit: bump execution count.
        let state = self.keyed_property_ic_state_or_default_mut(code, slot);
        state.execution_count = state.execution_count.wrapping_add(1);
        let ec = state.execution_count;
        if let Some(table) = self.metadata_table_mut(code) {
            table.keyed_property_mut(slot.get()).execution_count = ec;
        }
        self.tiering.observe_feedback_event(code);
        Some(true)
    }

    pub(super) fn observe_keyed_atom_slow_path(
        &mut self,
        agent: &mut Agent,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
        atom: AtomId,
        purpose: NamedPropertyCachePurpose,
    ) {
        let Some(slot) = slot else {
            return;
        };
        // Gate on allocation threshold.
        if !self.ensure_feedback_slot_execution(code, slot) {
            return;
        }
        let plan = agent
            .objects()
            .plan_named_property_cache_entry(
                agent.heap().view(),
                receiver,
                PropertyKey::from_atom(atom),
                purpose,
            )
            .ok()
            .flatten();
        // Spec 2 Phase A: register proto-chain watchpoints.
        if let Some(plan_entry) = plan
            && plan_entry.path() == NamedPropertyCachePath::PrototypeData
            && !Self::register_proto_chain_watchpoints(self, agent, code, slot, plan_entry)
        {
            return;
        }
        self.observe_keyed_named_atom_on_state(code, slot, atom, plan);
    }

    fn observe_keyed_named_atom_on_state(
        &mut self,
        code: CodeRef,
        slot: FeedbackSlotId,
        atom: AtomId,
        plan: Option<NamedPropertyCacheEntry>,
    ) {
        // Split-borrow: we need &mut KeyedPropertyIcState and &mut named_entries
        // map simultaneously.
        let Self {
            keyed_property_ic_states,
            keyed_property_named_entries,
            ..
        } = self;
        let index = code_index(code);
        let slot_zero = (slot.get() - 1) as usize;
        let slab = keyed_property_ic_states[index]
            .as_deref_mut()
            .expect("keyed_property_ic_states slab must be allocated at install");
        let state = slab[slot_zero].get_or_insert_with(KeyedPropertyIcState::default);
        let named_entries = keyed_property_named_entries
            .entry((code, slot))
            .or_default();
        keyed_observe_named_atom_slow_path(state, named_entries, atom, plan);
        // Write asm-readable bits.
        let cache_state = state.cache_state;
        let mono_handler = state.monomorphic_named_own_data_handler;
        let ec = state.execution_count;
        if let Some(table) = self.metadata_table_mut(code) {
            let meta = table.keyed_property_mut(slot.get());
            meta.mode = ic_mode_from_cache_state(cache_state);
            meta.handler_bits = if matches!(cache_state, InlineCacheState::Monomorphic) {
                mono_handler.bits()
            } else {
                0
            };
            meta.execution_count = ec;
        }
    }

    fn observe_keyed_index_slow_path(
        &mut self,
        code: CodeRef,
        slot: FeedbackSlotId,
        plan: Option<DenseIndexCacheEntry>,
    ) {
        if !self.ensure_feedback_slot_execution(code, slot) {
            return;
        }
        let state = self.keyed_property_ic_state_or_default_mut(code, slot);
        let changed = keyed_observe_dense_index_on_state(state, plan);
        let cache_state = state.cache_state;
        let ec = state.execution_count;
        if changed && let Some(table) = self.metadata_table_mut(code) {
            let meta = table.keyed_property_mut(slot.get());
            meta.mode = ic_mode_from_cache_state(cache_state);
            meta.handler_bits = 0;
            meta.execution_count = ec;
        }
    }

    pub(super) fn observe_keyed_index_access(
        &mut self,
        agent: &Agent,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
        index: u32,
    ) {
        let Some(slot) = slot else {
            return;
        };
        let plan = dense_index_plan(agent, receiver, index);
        self.observe_keyed_index_slow_path(code, slot, plan);
    }

    pub(super) fn observe_keyed_generic_slow_path(
        &mut self,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
    ) {
        let Some(slot) = slot else {
            return;
        };
        // Gate on allocation threshold.
        if !self.ensure_feedback_slot_execution(code, slot) {
            return;
        }
        let state = self.keyed_property_ic_state_or_default_mut(code, slot);
        keyed_observe_generic_on_state(state);
        let cache_state = state.cache_state;
        let ec = state.execution_count;
        if let Some(table) = self.metadata_table_mut(code) {
            let meta = table.keyed_property_mut(slot.get());
            meta.mode = ic_mode_from_cache_state(cache_state);
            meta.handler_bits = 0;
            meta.execution_count = ec;
        }
    }

    // ── Public API ────────────────────────────────────────────────────────────

    #[cfg(test)]
    pub(crate) fn has_feedback_vector(&self, code: CodeRef) -> bool {
        self.tiering.is_allocated(code)
    }

    /// Project the IC state for a `NamedProperty` (Load or Store) slot into
    /// a plain-value `NamedPropertyStatus`. Returns `None` if `code` isn't
    /// installed or the slot has no `PropertyIcState` entry yet.
    ///
    /// The entries list is composed by walking inline `PropertyIcState.entries`
    /// followed by the out-of-line `polymorphic_chain(code, slot)`; entries
    /// come out in ascending receiver-shape order (the install logic keeps
    /// inline shapes strictly less than chain shapes, and the chain itself
    /// is sorted).
    pub fn named_property_status(
        &self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> Option<NamedPropertyStatus> {
        let state = self.property_ic_state(code, slot)?;
        let mut entries: Vec<NamedPropertyStatusEntry> =
            Vec::with_capacity(usize::from(state.entry_count));
        for entry in state
            .entries
            .iter()
            .take(state.inline_count())
            .copied()
            .flatten()
        {
            entries.push(NamedPropertyStatusEntry::from_entry(entry));
        }
        if let Some(chain) = self.polymorphic_chain(code, slot) {
            for entry in chain.entries() {
                entries.push(NamedPropertyStatusEntry::from_entry(*entry));
            }
        }
        Some(NamedPropertyStatus {
            state: state.cache_state.into(),
            generation: state.generation,
            execution_count: state.execution_count,
            entries,
        })
    }

    /// Project the IC state for a `Call` slot into a plain-value `CallStatus`.
    /// Returns `None` if `code` isn't installed.
    ///
    /// `expected_arity` is read from the compile-time feedback descriptor
    /// when present, regardless of whether the IC state has been populated;
    /// this matches the legacy snapshot's behavior.
    pub fn call_status(&self, code: CodeRef, slot: FeedbackSlotId) -> Option<CallStatus> {
        let installed = self
            .installed
            .get(code_index(code))
            .and_then(Option::as_ref)?;
        let descriptor = installed.feedback_descriptor_for_slot(slot)?;
        let expected_arity = descriptor.metadata().expected_arity();
        let Some(state) = self.call_ic_state(code, slot) else {
            return Some(CallStatus {
                state: FeedbackInlineCacheState::Uninitialized,
                generation: 0,
                execution_count: 0,
                callee: None,
                entries: Vec::new(),
                expected_arity,
            });
        };
        let entries = self
            .call_cache_entries
            .get(&(code, slot))
            .map(|storage| {
                let mut out = Vec::with_capacity(usize::from(state.entry_count));
                for entry in storage
                    .entries
                    .iter()
                    .take(usize::from(state.entry_count))
                    .copied()
                    .flatten()
                {
                    out.push(CalleeSummary {
                        function: entry.callee,
                        function_shape: entry.callee_shape,
                        realm: entry.realm,
                        builtin: entry.builtin,
                        created_shape: None,
                    });
                }
                out
            })
            .unwrap_or_default();
        let callee = if matches!(state.cache_state, InlineCacheState::Monomorphic) {
            entries.first().copied()
        } else {
            None
        };
        let generation = self
            .metadata_table(code)
            .map(|table| table.call(slot.get()).generation)
            .unwrap_or(0);
        Some(CallStatus {
            state: state.cache_state.into(),
            generation,
            execution_count: state.execution_count,
            callee,
            entries,
            expected_arity,
        })
    }

    /// Project the IC state for a `Construct` slot into a plain-value
    /// `ConstructStatus`. Returns `None` if `code` isn't installed.
    pub fn construct_status(&self, code: CodeRef, slot: FeedbackSlotId) -> Option<ConstructStatus> {
        let installed = self
            .installed
            .get(code_index(code))
            .and_then(Option::as_ref)?;
        let descriptor = installed.feedback_descriptor_for_slot(slot)?;
        let expected_arity = descriptor.metadata().expected_arity();
        let Some(state) = self.construct_ic_state(code, slot) else {
            return Some(ConstructStatus {
                state: FeedbackInlineCacheState::Uninitialized,
                generation: 0,
                execution_count: 0,
                callee: None,
                entries: Vec::new(),
                expected_arity,
            });
        };
        let entries = self
            .construct_cache_entries
            .get(&(code, slot))
            .map(|storage| {
                let mut out = Vec::with_capacity(usize::from(state.entry_count));
                for entry in storage
                    .entries
                    .iter()
                    .take(usize::from(state.entry_count))
                    .copied()
                    .flatten()
                {
                    out.push(CalleeSummary {
                        function: entry.constructor,
                        function_shape: entry.constructor_shape,
                        realm: entry.realm,
                        builtin: None,
                        created_shape: entry.created_shape,
                    });
                }
                out
            })
            .unwrap_or_default();
        let callee = if matches!(state.cache_state, InlineCacheState::Monomorphic) {
            entries.first().copied()
        } else {
            None
        };
        let generation = self
            .metadata_table(code)
            .map(|table| table.call(slot.get()).generation)
            .unwrap_or(0);
        Some(ConstructStatus {
            state: state.cache_state.into(),
            generation,
            execution_count: state.execution_count,
            callee,
            entries,
            expected_arity,
        })
    }

    /// Project the per-kind metadata for an `Arithmetic` slot into a
    /// plain-value `ArithStatus`. Returns `None` if `code` isn't installed.
    ///
    /// Arithmetic feedback is MetadataTable-owned (no Rust-side IC state):
    /// reads come straight off the asm-visible `ArithMetadata.observed_bits`
    /// and `ArithMetadata.execution_count`.
    pub fn arith_status(&self, code: CodeRef, slot: FeedbackSlotId) -> Option<ArithStatus> {
        let table = self.metadata_table(code)?;
        let meta = table.arith(slot.get());
        Some(ArithStatus::from_metadata(
            meta.observed_bits,
            meta.execution_count,
        ))
    }

    /// Project the per-kind metadata for a `Comparison` slot into a
    /// plain-value `ComparisonStatus`. Returns `None` if `code` isn't
    /// installed.
    pub fn comparison_status(
        &self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> Option<ComparisonStatus> {
        let table = self.metadata_table(code)?;
        let meta = table.comparison(slot.get());
        Some(ComparisonStatus::from_metadata(
            meta.observed_bits,
            meta.execution_count,
        ))
    }

    /// Project the IC state for a `KeyedPropertyAccess` slot into a
    /// plain-value `KeyedPropertyStatus`. Returns `None` if `code` isn't
    /// installed or the slot has no `KeyedPropertyIcState` entry yet.
    pub fn keyed_property_status(
        &self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> Option<KeyedPropertyStatus> {
        let state = self.keyed_property_ic_state(code, slot)?;
        let family = state.family.map(FeedbackKeyedPropertyFamily::from);
        let mut named_entries: Vec<KeyedPropertyNamedStatusEntry> =
            Vec::with_capacity(usize::from(state.named_entry_count));
        let storage = self.keyed_property_named_entries.get(&(code, slot));
        for index in 0..usize::from(state.named_entry_count) {
            let Some(slot_entry) = state.named_entries[index] else {
                continue;
            };
            let handler_summary = storage
                .and_then(|s| s.entries.get(index))
                .and_then(|cell| cell.as_ref())
                .map(|named| super::status::NamedPropertyHandlerSummary::from_entry(named.entry));
            named_entries.push(KeyedPropertyNamedStatusEntry {
                atom: AtomId::from_raw(slot_entry.atom_raw),
                receiver_shape: slot_entry.receiver_shape,
                handler_summary,
            });
        }
        let mut dense_entries: Vec<KeyedPropertyDenseStatusEntry> =
            Vec::with_capacity(usize::from(state.dense_entry_count));
        for index in 0..usize::from(state.dense_entry_count) {
            let Some(entry) = state.dense_entries[index] else {
                continue;
            };
            dense_entries.push(KeyedPropertyDenseStatusEntry {
                receiver_shape: entry.receiver_shape,
                receiver_flags: entry.receiver_flags,
            });
        }
        let generation = self
            .metadata_table(code)
            .map(|table| table.keyed_property(slot.get()).generation)
            .unwrap_or(0);
        Some(KeyedPropertyStatus {
            state: state.cache_state.into(),
            generation,
            execution_count: state.execution_count,
            family,
            named_entries,
            dense_entries,
        })
    }

    /// Project the per-code IC memory + per-kind site-count footprint.
    ///
    /// `allocated_bytes` counts both the `MetadataTable` buffer and the
    /// per-code Vec-indexed side-table allocations
    /// (`PropertyIcState`/`CallIcState`/`KeyedPropertyIcState`/polymorphic
    /// chains) so consumers see the full IC memory cost.
    pub fn metadata_table_footprint(&self, code: CodeRef) -> Option<MetadataTableFootprint> {
        let index = code_index(code);
        let installed = self.installed.get(index).and_then(Option::as_ref)?;
        let slot_count = installed.feedback_slot_descriptors().len();
        let live_site_count = installed
            .feedback_slot_descriptors()
            .iter()
            .flatten()
            .count();
        let allocated = self.tiering.is_allocated(code);
        let warmup_counter = self.tiering.warmup_counter(code);

        let table = self.metadata_table(code);
        let mut live_site_count_by_kind = [0usize; METADATA_KIND_COUNT];
        let mut allocated_bytes = 0usize;
        let mut total_execution_count: u64 = 0;
        if let Some(table) = table {
            for (kind_idx, count) in live_site_count_by_kind.iter_mut().enumerate() {
                *count = usize::try_from(table.run_len_for_kind_index(kind_idx))
                    .expect("per-kind run length fits in usize");
            }
            allocated_bytes = allocated_bytes.saturating_add(table.buffer().len());
            // Sum execution counts across populated metadata entries.
            for descriptor in installed.feedback_slot_descriptors().iter().flatten() {
                let slot_value = descriptor.slot().get();
                match MetadataKind::from_site_kind(descriptor.kind()) {
                    MetadataKind::Property => {
                        total_execution_count = total_execution_count
                            .saturating_add(u64::from(table.property(slot_value).execution_count));
                    }
                    MetadataKind::Call => {
                        total_execution_count = total_execution_count
                            .saturating_add(u64::from(table.call(slot_value).execution_count));
                    }
                    MetadataKind::Arith => {
                        total_execution_count = total_execution_count
                            .saturating_add(u64::from(table.arith(slot_value).execution_count));
                    }
                    MetadataKind::Comparison => {
                        total_execution_count = total_execution_count.saturating_add(u64::from(
                            table.comparison(slot_value).execution_count,
                        ));
                    }
                    MetadataKind::KeyedProperty => {
                        total_execution_count = total_execution_count.saturating_add(u64::from(
                            table.keyed_property(slot_value).execution_count,
                        ));
                    }
                }
            }
        }
        allocated_bytes = allocated_bytes.saturating_add(self.ic_side_table_bytes_for_code(index));

        Some(MetadataTableFootprint {
            allocated,
            allocated_bytes,
            live_site_count_by_kind,
            total_execution_count,
            warmup_counter,
            slot_count,
            live_site_count,
        })
    }

    /// Sum of side-table allocations (PropertyIcState/CallIcState/
    /// ConstructIcState/KeyedPropertyIcState/PolymorphicChain) for the
    /// code object at the given `code_index`. Returns 0 when no slab has
    /// been allocated yet.
    fn ic_side_table_bytes_for_code(&self, index: usize) -> usize {
        let mut total = 0usize;
        if let Some(Some(slab)) = self.property_ic_states.get(index) {
            total = total.saturating_add(
                slab.len()
                    .saturating_mul(std::mem::size_of::<Option<PropertyIcState>>()),
            );
        }
        if let Some(Some(slab)) = self.call_ic_states.get(index) {
            total = total.saturating_add(
                slab.len()
                    .saturating_mul(std::mem::size_of::<Option<CallIcState>>()),
            );
        }
        if let Some(Some(slab)) = self.construct_ic_states.get(index) {
            total = total.saturating_add(
                slab.len()
                    .saturating_mul(std::mem::size_of::<Option<CallIcState>>()),
            );
        }
        if let Some(Some(slab)) = self.keyed_property_ic_states.get(index) {
            total = total.saturating_add(
                slab.len()
                    .saturating_mul(std::mem::size_of::<Option<KeyedPropertyIcState>>()),
            );
        }
        if let Some(Some(slab)) = self.polymorphic_chains.get(index) {
            total = total.saturating_add(
                slab.len()
                    .saturating_mul(std::mem::size_of::<Option<PolymorphicChain>>()),
            );
        }
        total
    }

    #[cfg(test)]
    pub(crate) fn feedback_warmup_counter(&self, code: CodeRef) -> Option<u16> {
        self.installed
            .get(code_index(code))
            .and_then(Option::as_ref)
            .map(|_| self.tiering.warmup_counter(code))
    }

    #[cfg(test)]
    pub(crate) fn feedback_execution_count(
        &self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> Option<u32> {
        // For NamedProperty slots, read from PropertyIcState.
        if let Some(state) = self.property_ic_state(code, slot) {
            return Some(state.execution_count);
        }
        // For Call slots, read from CallIcState.
        if let Some(state) = self.call_ic_state(code, slot) {
            return Some(state.execution_count);
        }
        // For Construct slots, read from ConstructIcState.
        if let Some(state) = self.construct_ic_state(code, slot) {
            return Some(state.execution_count);
        }
        // Arithmetic and Comparison slots have no Rust-side state (Phase D.1.0).
        None
    }

    #[cfg(test)]
    pub(crate) fn named_property_cache_snapshot(
        &self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> Option<(
        &'static str,
        u8,
        Option<lyng_objects::NamedPropertyCachePath>,
    )> {
        let state = self.property_ic_state(code, slot)?;
        Some((
            match state.cache_state {
                InlineCacheState::Uninitialized => "Uninitialized",
                InlineCacheState::Monomorphic => "Monomorphic",
                InlineCacheState::Polymorphic => "Polymorphic",
                InlineCacheState::Megamorphic => "Megamorphic",
            },
            state.entry_count,
            state.entries[0].map(NamedPropertyCacheEntry::path),
        ))
    }

    /// Returns the IC slot's current `(cache_state, generation)` tuple
    /// for a NamedProperty site. `None` if the slot is empty.
    #[cfg(test)]
    pub(crate) fn named_property_generation_snapshot(
        &self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> Option<(&'static str, u32)> {
        let state = self.property_ic_state(code, slot)?;
        Some((
            match state.cache_state {
                InlineCacheState::Uninitialized => "Uninitialized",
                InlineCacheState::Monomorphic => "Monomorphic",
                InlineCacheState::Polymorphic => "Polymorphic",
                InlineCacheState::Megamorphic => "Megamorphic",
            },
            state.generation,
        ))
    }

    /// Returns `true` iff the slot has a populated `PropertyIcState` entry.
    ///
    /// Phase D.2.4: cleared slots are removed from the map by
    /// `clear_ic_slot_if_generation_matches`, so `contains_key` is sufficient.
    #[cfg(test)]
    pub(crate) fn named_property_slot_is_present(
        &self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> bool {
        self.property_ic_state(code, slot).is_some()
    }

    #[cfg(test)]
    pub(crate) fn keyed_property_cache_snapshot(
        &self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> Option<(&'static str, Option<&'static str>, u8)> {
        let state = self.keyed_property_ic_state(code, slot)?;
        Some((
            match state.cache_state {
                InlineCacheState::Uninitialized => "Uninitialized",
                InlineCacheState::Monomorphic => "Monomorphic",
                InlineCacheState::Polymorphic => "Polymorphic",
                InlineCacheState::Megamorphic => "Megamorphic",
            },
            state.family.map(|family| match family {
                KeyedIcFamily::DenseIndex => "DenseIndex",
                KeyedIcFamily::NamedAtom => "NamedAtom",
                KeyedIcFamily::Generic => "Generic",
            }),
            match state.family {
                Some(KeyedIcFamily::DenseIndex) => state.dense_entry_count,
                Some(KeyedIcFamily::NamedAtom) => state.named_entry_count,
                Some(KeyedIcFamily::Generic) | None => 0,
            },
        ))
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    fn named_llint_load_header_from_state(
        state: &PropertyIcState,
    ) -> Option<LlIntNamedPropertyHeader> {
        // OwnInline
        let handler = state.monomorphic_own_data_handler;
        if handler.is_valid() {
            if matches!(handler.slot_location(), SlotLocation::Inline(_)) {
                return Some(LlIntNamedPropertyHeader::OwnInline {
                    handler_bits: handler.bits(),
                });
            }
            if matches!(handler.slot_location(), SlotLocation::OutOfLine(_)) {
                return Some(LlIntNamedPropertyHeader::OwnOutline {
                    handler_bits: handler.bits(),
                });
            }
        }
        // Polymorphic OwnData
        let h0 = state.polymorphic_own_data_handlers[0];
        let h1 = state.polymorphic_own_data_handlers[1];
        if h0.is_valid()
            && h1.is_valid()
            && matches!(h0.slot_location(), SlotLocation::Inline(_))
            && matches!(h1.slot_location(), SlotLocation::Inline(_))
        {
            return Some(LlIntNamedPropertyHeader::OwnPolymorphic {
                slot0_handler_bits: h0.bits(),
                slot1_handler_bits: h1.bits(),
            });
        }
        // ProtoInline
        let proto_handler = state.monomorphic_proto_data_handler;
        if proto_handler.is_valid()
            && matches!(proto_handler.slot_location(), SlotLocation::Inline(_))
        {
            return Some(LlIntNamedPropertyHeader::ProtoInline {
                receiver_word: proto_handler.receiver_word(),
                proto_word: proto_handler.proto_word(),
            });
        }
        None
    }

    fn project_property_into_meta(
        header: Option<LlIntNamedPropertyHeader>,
        generation: u32,
        execution_count: u32,
        meta: &mut PropertyMetadata,
    ) {
        const MODE_EMPTY: u8 = 0;
        const MODE_NAMED_OWN_INLINE_LOAD: u8 = 1;
        const MODE_NAMED_PROTO_INLINE_LOAD: u8 = 2;
        const MODE_NAMED_OWN_OUTLINE_LOAD: u8 = 3;
        const MODE_NAMED_OWN_POLYMORPHIC: u8 = 4;
        let (mode, handler_bits, aux_bits) = match header {
            Some(LlIntNamedPropertyHeader::OwnInline { handler_bits }) => {
                (MODE_NAMED_OWN_INLINE_LOAD, handler_bits, 0u64)
            }
            Some(LlIntNamedPropertyHeader::OwnOutline { handler_bits }) => {
                (MODE_NAMED_OWN_OUTLINE_LOAD, handler_bits, 0u64)
            }
            Some(LlIntNamedPropertyHeader::ProtoInline {
                receiver_word,
                proto_word,
            }) => (MODE_NAMED_PROTO_INLINE_LOAD, proto_word, receiver_word),
            Some(LlIntNamedPropertyHeader::OwnPolymorphic {
                slot0_handler_bits,
                slot1_handler_bits,
            }) => (
                MODE_NAMED_OWN_POLYMORPHIC,
                slot0_handler_bits,
                slot1_handler_bits,
            ),
            None => (MODE_EMPTY, 0u64, 0u64),
        };
        meta.mode = mode;
        meta.generation = generation;
        meta.handler_bits = handler_bits;
        meta.aux_bits = aux_bits;
        meta.execution_count = execution_count;
    }

    /// Project the write-side cache state into `PropertyMetadata`. Called
    /// from the assign opcode's slow-path install (Task 5) when the slot
    /// is a Store-purpose IC site. Writes mode = `LLINT_IC_MODE_NAMED_OWN_INLINE_WRITE`
    /// (= 5), `handler_bits` packed source_shape + slot + writable_flag,
    /// `aux_bits` = target_shape. When the handler is NONE, zeroes the
    /// metadata so the asm mode-byte check bails to the Rust probe.
    pub(super) fn project_property_write_into_meta(
        write_handler: NamedPropertyInlineWriteHandler,
        generation: u32,
        execution_count: u32,
        meta: &mut PropertyMetadata,
    ) {
        if write_handler.is_valid() {
            meta.mode = LLINT_IC_MODE_NAMED_OWN_INLINE_WRITE;
            meta.handler_bits = write_handler.handler_bits();
            meta.aux_bits = write_handler.target_bits();
        } else {
            meta.mode = 0;
            meta.handler_bits = 0;
            meta.aux_bits = 0;
        }
        meta.generation = generation;
        meta.execution_count = execution_count;
    }

    fn dense_value_from_header(agent: &Agent, header: ObjectHeader, index: u32) -> Option<Value> {
        let elements = header.elements()?;
        let index = usize::try_from(index).expect("u32 index should fit into usize");
        let value = agent
            .heap()
            .view()
            .object_slots(elements.raw())?
            .get(index)
            .copied()
            .unwrap_or(Value::array_hole());
        (value != Value::array_hole()).then_some(value)
    }
}

// ── PropertyIcState slow-path methods ────────────────────────────────────────

impl PropertyIcState {
    #[inline]
    fn try_load(
        &self,
        agent: &Agent,
        chain: Option<&PolymorphicChain>,
        receiver: ObjectRef,
    ) -> Option<Value> {
        match self.cache_state {
            InlineCacheState::Monomorphic | InlineCacheState::Polymorphic => {}
            InlineCacheState::Uninitialized | InlineCacheState::Megamorphic => return None,
        }
        let receiver_shape = agent
            .objects()
            .object_header(agent.heap().view(), receiver)?
            .shape();
        let entry = self.lookup_entry_for_shape(chain, receiver_shape)?;
        if let Ok(Some(value)) =
            agent
                .objects()
                .load_from_named_property_cache(agent.heap().view(), receiver, entry)
        {
            return Some(value);
        }
        None
    }

    #[inline]
    fn try_store(
        &self,
        agent: &mut Agent,
        chain: Option<&PolymorphicChain>,
        receiver: ObjectRef,
        atom: AtomId,
        value: Value,
    ) -> Option<bool> {
        match self.cache_state {
            InlineCacheState::Monomorphic | InlineCacheState::Polymorphic => {}
            InlineCacheState::Uninitialized | InlineCacheState::Megamorphic => return None,
        }
        let receiver_shape = agent
            .objects()
            .object_header(agent.heap().view(), receiver)?
            .shape();
        let entry = self.lookup_entry_for_shape(chain, receiver_shape)?;
        let result = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.store_to_named_property_cache(
                &mut mutator,
                receiver,
                PropertyKey::from_atom(atom),
                entry,
                value,
            )
        });
        if let Ok(Some(stored)) = result {
            return Some(stored);
        }
        None
    }

    #[inline]
    fn lookup_entry_for_shape(
        &self,
        chain: Option<&PolymorphicChain>,
        receiver_shape: ShapeId,
    ) -> Option<NamedPropertyCacheEntry> {
        if let Ok(index) = self.search_entry_index(receiver_shape) {
            return self.entries[index];
        }
        chain.and_then(|chain| chain.find_by_shape(receiver_shape).copied())
    }

    /// Iterates the inline `entries` slice (at most `POLY_LIMIT`).
    #[inline]
    #[allow(
        dead_code,
        reason = "TODO(Phase E): snapshot restoration will use this iterator"
    )]
    pub(crate) fn inline_active_entries(
        &self,
    ) -> impl Iterator<Item = NamedPropertyCacheEntry> + '_ {
        self.entries
            .iter()
            .take(self.inline_count())
            .filter_map(|entry| *entry)
    }
}

// ── Keyed property slow-path helpers (free functions) ────────────────────────

fn keyed_observe_named_atom_slow_path(
    state: &mut KeyedPropertyIcState,
    named_entries: &mut KeyedPropertyNamedEntries,
    atom: AtomId,
    plan: Option<NamedPropertyCacheEntry>,
) {
    let Some(plan) = plan else {
        // No plan → megamorphic NamedAtom (generic key fallback).
        keyed_promote_to_megamorphic(state, named_entries, Some(KeyedIcFamily::NamedAtom));
        return;
    };
    match state.family {
        None => {
            state.family = Some(KeyedIcFamily::NamedAtom);
            named_entries.entries[0] = Some(KeyedNamedPropertyCacheEntry { atom, entry: plan });
            state.named_entries[0] = Some(KeyedIcNamedEntry {
                atom_raw: atom.raw(),
                receiver_shape: plan.receiver_shape(),
            });
            state.named_entry_count = 1;
            state.cache_state = InlineCacheState::Monomorphic;
        }
        Some(KeyedIcFamily::NamedAtom) => match state.cache_state {
            InlineCacheState::Megamorphic => {}
            InlineCacheState::Uninitialized => {
                named_entries.entries[0] = Some(KeyedNamedPropertyCacheEntry { atom, entry: plan });
                state.named_entries[0] = Some(KeyedIcNamedEntry {
                    atom_raw: atom.raw(),
                    receiver_shape: plan.receiver_shape(),
                });
                state.named_entry_count = 1;
                state.cache_state = InlineCacheState::Monomorphic;
            }
            InlineCacheState::Monomorphic | InlineCacheState::Polymorphic => {
                let new_entry = KeyedNamedPropertyCacheEntry { atom, entry: plan };
                let new_ic_entry = KeyedIcNamedEntry {
                    atom_raw: atom.raw(),
                    receiver_shape: plan.receiver_shape(),
                };
                match search_keyed_named_entry_index(state, atom, plan.receiver_shape()) {
                    Ok(index) => {
                        named_entries.entries[index] = Some(new_entry);
                        state.named_entries[index] = Some(new_ic_entry);
                    }
                    Err(index) => {
                        insert_keyed_named_entry_at(
                            state,
                            named_entries,
                            index,
                            new_entry,
                            new_ic_entry,
                        );
                    }
                }
            }
        },
        Some(KeyedIcFamily::DenseIndex | KeyedIcFamily::Generic) => {
            keyed_promote_to_megamorphic(state, named_entries, Some(KeyedIcFamily::Generic));
        }
    }
    keyed_refresh_sidecars(state, named_entries);
}

fn keyed_observe_dense_index_on_state(
    state: &mut KeyedPropertyIcState,
    plan: Option<DenseIndexCacheEntry>,
) -> bool {
    let Some(plan) = plan else {
        return keyed_observe_uncacheable_dense_index(state);
    };
    let plan_ic = KeyedIcDenseEntry {
        receiver_shape: plan.receiver_shape,
        receiver_flags: plan.receiver_flags,
    };
    let changed = match state.family {
        None | Some(KeyedIcFamily::DenseIndex) => {
            if state.family.is_none() {
                keyed_install_first_dense_entry(state, plan_ic);
                true
            } else {
                match state.cache_state {
                    InlineCacheState::Megamorphic => false,
                    InlineCacheState::Uninitialized => {
                        keyed_install_first_dense_entry(state, plan_ic);
                        true
                    }
                    InlineCacheState::Monomorphic | InlineCacheState::Polymorphic => {
                        if let Some(index) = find_dense_entry_index(state, plan_ic) {
                            let changed = state.dense_entries[index] != Some(plan_ic);
                            state.dense_entries[index] = Some(plan_ic);
                            changed
                        } else if usize::from(state.dense_entry_count)
                            >= POLYMORPHIC_PROPERTY_CACHE_LIMIT
                        {
                            keyed_promote_to_megamorphic_dense(state);
                            true
                        } else {
                            state.dense_entries[usize::from(state.dense_entry_count)] =
                                Some(plan_ic);
                            state.dense_entry_count = state.dense_entry_count.saturating_add(1);
                            state.cache_state = if state.dense_entry_count <= 1 {
                                InlineCacheState::Monomorphic
                            } else {
                                InlineCacheState::Polymorphic
                            };
                            true
                        }
                    }
                }
            }
        }
        Some(KeyedIcFamily::NamedAtom | KeyedIcFamily::Generic) => {
            keyed_promote_mixed_to_generic(state)
        }
    };
    // Recompute dense sidecars.
    keyed_refresh_dense_sidecars(state);
    changed
}

fn keyed_observe_uncacheable_dense_index(state: &mut KeyedPropertyIcState) -> bool {
    match state.family {
        None | Some(KeyedIcFamily::DenseIndex) => {
            if state.family == Some(KeyedIcFamily::DenseIndex)
                && state.cache_state == InlineCacheState::Megamorphic
                && state.dense_entry_count == 0
            {
                return false;
            }
            keyed_promote_to_megamorphic_dense(state);
            true
        }
        Some(KeyedIcFamily::NamedAtom | KeyedIcFamily::Generic) => {
            keyed_promote_mixed_to_generic(state)
        }
    }
}

fn keyed_observe_generic_on_state(state: &mut KeyedPropertyIcState) {
    keyed_promote_to_megamorphic_named(state, Some(KeyedIcFamily::Generic));
}

// ── Keyed IC helper free functions ────────────────────────────────────────────

fn keyed_install_first_dense_entry(state: &mut KeyedPropertyIcState, entry: KeyedIcDenseEntry) {
    state.family = Some(KeyedIcFamily::DenseIndex);
    state.dense_entries[0] = Some(entry);
    state.dense_entry_count = 1;
    state.cache_state = InlineCacheState::Monomorphic;
}

fn find_dense_entry_index(state: &KeyedPropertyIcState, plan: KeyedIcDenseEntry) -> Option<usize> {
    (0..usize::from(state.dense_entry_count)).find(|&i| state.dense_entries[i] == Some(plan))
}

fn keyed_promote_to_megamorphic(
    state: &mut KeyedPropertyIcState,
    named_entries: &mut KeyedPropertyNamedEntries,
    family: Option<KeyedIcFamily>,
) {
    state.family = family;
    state.cache_state = InlineCacheState::Megamorphic;
    state.named_entry_count = 0;
    state.named_entries = [None; 8];
    state.dense_entry_count = 0;
    state.dense_entries = [None; 8];
    named_entries.entries = [None; POLYMORPHIC_PROPERTY_CACHE_LIMIT];
    clear_keyed_sidecars(state);
}

fn keyed_promote_to_megamorphic_named(
    state: &mut KeyedPropertyIcState,
    family: Option<KeyedIcFamily>,
) {
    state.family = family;
    state.cache_state = InlineCacheState::Megamorphic;
    state.named_entry_count = 0;
    state.named_entries = [None; 8];
    state.dense_entry_count = 0;
    state.dense_entries = [None; 8];
    clear_keyed_sidecars(state);
}

fn keyed_promote_to_megamorphic_dense(state: &mut KeyedPropertyIcState) {
    state.family = Some(KeyedIcFamily::DenseIndex);
    state.cache_state = InlineCacheState::Megamorphic;
    state.dense_entry_count = 0;
    state.dense_entries = [None; 8];
    clear_keyed_sidecars(state);
}

fn keyed_promote_mixed_to_generic(state: &mut KeyedPropertyIcState) -> bool {
    if state.family == Some(KeyedIcFamily::Generic)
        && state.cache_state == InlineCacheState::Megamorphic
        && state.named_entry_count == 0
        && state.dense_entry_count == 0
    {
        return false;
    }
    keyed_promote_to_megamorphic_named(state, Some(KeyedIcFamily::Generic));
    true
}

fn clear_keyed_sidecars(state: &mut KeyedPropertyIcState) {
    state.monomorphic_named_own_data_handler = NamedPropertyHandler::NONE;
    state.monomorphic_named_atom = 0;
    state.monomorphic_named_proto_data_handler = NamedPropertyProtoHandler::NONE;
    state.monomorphic_dense_index_handler = KeyedDenseIndexHandler::NONE;
    for slot in 0..POLY_LIMIT {
        state.polymorphic_named_own_data_handlers[slot] = NamedPropertyHandler::NONE;
        state.polymorphic_named_atoms[slot] = 0;
        state.polymorphic_dense_index_handlers[slot] = KeyedDenseIndexHandler::NONE;
    }
}

fn keyed_refresh_sidecars(
    state: &mut KeyedPropertyIcState,
    named_entries: &KeyedPropertyNamedEntries,
) {
    clear_keyed_sidecars(state);
    match (state.cache_state, state.family) {
        (InlineCacheState::Monomorphic, Some(KeyedIcFamily::NamedAtom)) => {
            if let Some(keyed_entry) = named_entries.entries[0] {
                match keyed_entry.entry.path() {
                    NamedPropertyCachePath::OwnData => {
                        let handler = NamedPropertyHandler::from_entry(keyed_entry.entry);
                        if handler.is_valid() {
                            state.monomorphic_named_own_data_handler = handler;
                            state.monomorphic_named_atom = keyed_entry.atom.raw();
                        }
                    }
                    NamedPropertyCachePath::OwnDataTransition => {}
                    NamedPropertyCachePath::PrototypeData => {
                        let handler = NamedPropertyProtoHandler::from_entry(keyed_entry.entry);
                        if handler.is_valid() {
                            state.monomorphic_named_proto_data_handler = handler;
                            state.monomorphic_named_atom = keyed_entry.atom.raw();
                        }
                    }
                }
            }
        }
        (InlineCacheState::Monomorphic, Some(KeyedIcFamily::DenseIndex)) => {
            if let Some(dense) = state.dense_entries[0] {
                state.monomorphic_dense_index_handler =
                    KeyedDenseIndexHandler::new(dense.receiver_shape, dense.receiver_flags);
            }
        }
        (InlineCacheState::Polymorphic, Some(KeyedIcFamily::NamedAtom)) => {
            let active = usize::from(state.named_entry_count).min(POLY_LIMIT);
            for slot in 0..active {
                let Some(keyed_entry) = named_entries.entries[slot] else {
                    continue;
                };
                if !matches!(keyed_entry.entry.path(), NamedPropertyCachePath::OwnData) {
                    continue;
                }
                let handler = NamedPropertyHandler::from_entry(keyed_entry.entry);
                if handler.is_valid() {
                    state.polymorphic_named_own_data_handlers[slot] = handler;
                    state.polymorphic_named_atoms[slot] = keyed_entry.atom.raw();
                }
            }
        }
        (InlineCacheState::Polymorphic, Some(KeyedIcFamily::DenseIndex)) => {
            let active = usize::from(state.dense_entry_count).min(POLY_LIMIT);
            for slot in 0..active {
                if let Some(dense) = state.dense_entries[slot] {
                    state.polymorphic_dense_index_handlers[slot] =
                        KeyedDenseIndexHandler::new(dense.receiver_shape, dense.receiver_flags);
                }
            }
        }
        _ => {}
    }
}

fn keyed_refresh_dense_sidecars(state: &mut KeyedPropertyIcState) {
    state.monomorphic_dense_index_handler = KeyedDenseIndexHandler::NONE;
    for slot in 0..POLY_LIMIT {
        state.polymorphic_dense_index_handlers[slot] = KeyedDenseIndexHandler::NONE;
    }
    match (state.cache_state, state.family) {
        (InlineCacheState::Monomorphic, Some(KeyedIcFamily::DenseIndex)) => {
            if let Some(dense) = state.dense_entries[0] {
                state.monomorphic_dense_index_handler =
                    KeyedDenseIndexHandler::new(dense.receiver_shape, dense.receiver_flags);
            }
        }
        (InlineCacheState::Polymorphic, Some(KeyedIcFamily::DenseIndex)) => {
            let active = usize::from(state.dense_entry_count).min(POLY_LIMIT);
            for slot in 0..active {
                if let Some(dense) = state.dense_entries[slot] {
                    state.polymorphic_dense_index_handlers[slot] =
                        KeyedDenseIndexHandler::new(dense.receiver_shape, dense.receiver_flags);
                }
            }
        }
        _ => {}
    }
}

fn search_keyed_named_entry_index(
    state: &KeyedPropertyIcState,
    atom: AtomId,
    receiver_shape: ShapeId,
) -> Result<usize, usize> {
    state.named_entries[..usize::from(state.named_entry_count)].binary_search_by(|entry| {
        let Some(entry) = *entry else {
            return Ordering::Greater;
        };
        (entry.atom_raw, entry.receiver_shape).cmp(&(atom.raw(), receiver_shape))
    })
}

fn insert_keyed_named_entry_at(
    state: &mut KeyedPropertyIcState,
    named_entries: &mut KeyedPropertyNamedEntries,
    index: usize,
    entry: KeyedNamedPropertyCacheEntry,
    ic_entry: KeyedIcNamedEntry,
) {
    let count = usize::from(state.named_entry_count);
    if count >= POLYMORPHIC_PROPERTY_CACHE_LIMIT {
        keyed_promote_to_megamorphic_named(state, Some(KeyedIcFamily::NamedAtom));
        named_entries.entries = [None; POLYMORPHIC_PROPERTY_CACHE_LIMIT];
        return;
    }
    if index < count {
        named_entries.entries.copy_within(index..count, index + 1);
        state.named_entries.copy_within(index..count, index + 1);
    }
    named_entries.entries[index] = Some(entry);
    state.named_entries[index] = Some(ic_entry);
    state.named_entry_count = state.named_entry_count.saturating_add(1);
    state.cache_state = InlineCacheState::Polymorphic;
}

fn try_keyed_named_load(
    state: &KeyedPropertyIcState,
    named_entries: Option<&KeyedPropertyNamedEntries>,
    agent: &Agent,
    receiver: ObjectRef,
    atom: AtomId,
) -> Option<Value> {
    if state.family != Some(KeyedIcFamily::NamedAtom) {
        return None;
    }
    match state.cache_state {
        InlineCacheState::Monomorphic | InlineCacheState::Polymorphic => {}
        InlineCacheState::Uninitialized | InlineCacheState::Megamorphic => return None,
    }
    let receiver_shape = agent
        .objects()
        .object_header(agent.heap().view(), receiver)?
        .shape();
    let index = search_keyed_named_entry_index(state, atom, receiver_shape).ok()?;
    let entries = named_entries?;
    let keyed_entry = entries.entries[index]?;
    if let Ok(Some(value)) = agent.objects().load_from_named_property_cache(
        agent.heap().view(),
        receiver,
        keyed_entry.entry,
    ) {
        return Some(value);
    }
    None
}

fn try_keyed_named_store(
    state: &KeyedPropertyIcState,
    named_entries: Option<&KeyedPropertyNamedEntries>,
    agent: &mut Agent,
    receiver: ObjectRef,
    atom: AtomId,
    value: Value,
) -> Option<bool> {
    if state.family != Some(KeyedIcFamily::NamedAtom) {
        return None;
    }
    match state.cache_state {
        InlineCacheState::Monomorphic | InlineCacheState::Polymorphic => {}
        InlineCacheState::Uninitialized | InlineCacheState::Megamorphic => return None,
    }
    let receiver_shape = agent
        .objects()
        .object_header(agent.heap().view(), receiver)?
        .shape();
    let index = search_keyed_named_entry_index(state, atom, receiver_shape).ok()?;
    let entries = named_entries?;
    let keyed_entry = entries.entries[index]?;
    let result = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.store_to_named_property_cache(
            &mut mutator,
            receiver,
            PropertyKey::from_atom(atom),
            keyed_entry.entry,
            value,
        )
    });
    if let Ok(Some(stored)) = result {
        return Some(stored);
    }
    None
}

// ── Call/Construct IC helpers (free functions) ────────────────────────────────

fn ic_mode_from_cache_state(state: InlineCacheState) -> u8 {
    match state {
        InlineCacheState::Uninitialized => 0,
        InlineCacheState::Monomorphic => 1,
        InlineCacheState::Polymorphic => 2,
        InlineCacheState::Megamorphic => 3,
    }
}

fn observe_call_target_on_state(
    state: &mut CallIcState,
    storage: &mut CallCacheStorage,
    cache_entry: Option<CallCacheEntry>,
) {
    match state.cache_state {
        InlineCacheState::Megamorphic => {}
        InlineCacheState::Uninitialized => {
            let Some(entry) = cache_entry else {
                state.cache_state = InlineCacheState::Megamorphic;
                state.entry_count = 0;
                storage.entries = [None; POLYMORPHIC_CALL_CACHE_LIMIT];
                return;
            };
            storage.entries[0] = Some(entry);
            state.entry_count = 1;
            state.cache_state = InlineCacheState::Monomorphic;
        }
        InlineCacheState::Monomorphic => {
            if storage.entries[0]
                .is_some_and(|e| cache_entry.is_some_and(|ce| e.callee == ce.callee))
            {
                return;
            }
            let Some(entry) = cache_entry else {
                state.cache_state = InlineCacheState::Megamorphic;
                state.entry_count = 0;
                storage.entries = [None; POLYMORPHIC_CALL_CACHE_LIMIT];
                return;
            };
            storage.entries[usize::from(state.entry_count)] = Some(entry);
            state.entry_count = state.entry_count.saturating_add(1);
            state.cache_state = InlineCacheState::Polymorphic;
        }
        InlineCacheState::Polymorphic => {
            let ce = match cache_entry {
                Some(ce) => ce,
                None => {
                    state.cache_state = InlineCacheState::Megamorphic;
                    state.entry_count = 0;
                    storage.entries = [None; POLYMORPHIC_CALL_CACHE_LIMIT];
                    return;
                }
            };
            for index in 0..usize::from(state.entry_count) {
                if storage.entries[index].is_some_and(|e| e.callee == ce.callee) {
                    return;
                }
            }
            if usize::from(state.entry_count) >= POLYMORPHIC_CALL_CACHE_LIMIT {
                state.cache_state = InlineCacheState::Megamorphic;
                state.entry_count = 0;
                storage.entries = [None; POLYMORPHIC_CALL_CACHE_LIMIT];
                return;
            }
            storage.entries[usize::from(state.entry_count)] = Some(ce);
            state.entry_count = state.entry_count.saturating_add(1);
        }
    }
}

fn observe_construct_target_on_state(
    agent: &Agent,
    state: &mut CallIcState,
    storage: &mut ConstructCacheStorage,
    constructor: ObjectRef,
    cache_entry: Option<ConstructCacheEntry>,
    created: Option<ObjectRef>,
) {
    match state.cache_state {
        InlineCacheState::Megamorphic => {}
        InlineCacheState::Uninitialized => {
            let Some(entry) = cache_entry else {
                state.cache_state = InlineCacheState::Megamorphic;
                state.entry_count = 0;
                storage.entries = [None; POLYMORPHIC_CALL_CACHE_LIMIT];
                return;
            };
            storage.entries[0] = Some(entry);
            state.entry_count = 1;
            state.cache_state = InlineCacheState::Monomorphic;
        }
        InlineCacheState::Monomorphic => {
            if refresh_matching_construct_entry_created_shape(
                agent,
                storage,
                0,
                constructor,
                created,
            ) {
                return;
            }
            let Some(entry) = cache_entry else {
                state.cache_state = InlineCacheState::Megamorphic;
                state.entry_count = 0;
                storage.entries = [None; POLYMORPHIC_CALL_CACHE_LIMIT];
                return;
            };
            storage.entries[usize::from(state.entry_count)] = Some(entry);
            state.entry_count = state.entry_count.saturating_add(1);
            state.cache_state = InlineCacheState::Polymorphic;
        }
        InlineCacheState::Polymorphic => {
            for index in 0..usize::from(state.entry_count) {
                if refresh_matching_construct_entry_created_shape(
                    agent,
                    storage,
                    index,
                    constructor,
                    created,
                ) {
                    return;
                }
            }
            if usize::from(state.entry_count) >= POLYMORPHIC_CALL_CACHE_LIMIT {
                state.cache_state = InlineCacheState::Megamorphic;
                state.entry_count = 0;
                storage.entries = [None; POLYMORPHIC_CALL_CACHE_LIMIT];
                return;
            }
            let Some(entry) = cache_entry else {
                state.cache_state = InlineCacheState::Megamorphic;
                state.entry_count = 0;
                storage.entries = [None; POLYMORPHIC_CALL_CACHE_LIMIT];
                return;
            };
            storage.entries[usize::from(state.entry_count)] = Some(entry);
            state.entry_count = state.entry_count.saturating_add(1);
        }
    }
}

fn refresh_matching_construct_entry_created_shape(
    agent: &Agent,
    storage: &mut ConstructCacheStorage,
    index: usize,
    constructor: ObjectRef,
    created: Option<ObjectRef>,
) -> bool {
    let Some(mut entry) = storage.entries[index] else {
        return false;
    };
    if entry.constructor != constructor {
        return false;
    }
    if entry.created_shape.is_none() {
        entry.created_shape = ConstructCacheEntry::created_shape(agent, created);
        storage.entries[index] = Some(entry);
    }
    true
}

fn dense_index_plan(
    agent: &Agent,
    receiver: ObjectRef,
    index: u32,
) -> Option<DenseIndexCacheEntry> {
    let header = agent
        .objects()
        .object_header(agent.heap().view(), receiver)?;
    if !dense_index_receiver_is_cacheable(agent, receiver, header) {
        return None;
    }
    Vm::dense_value_from_header(agent, header, index)?;
    Some(DenseIndexCacheEntry::from_header(header))
}

fn dense_index_receiver_is_cacheable(
    agent: &Agent,
    receiver: ObjectRef,
    header: ObjectHeader,
) -> bool {
    matches!(header.kind(), ObjectKind::Ordinary | ObjectKind::Function)
        && !header.flags().is_arguments_object()
        && !agent.objects().is_module_namespace_object(receiver)
        && !agent.objects().is_typed_array_object(receiver)
        && agent.objects().primitive_wrapper_kind(receiver) != Some(PrimitiveWrapperKind::String)
}

// ── vm.rs AdaptiveProtoLoadDispatch impl helpers ──────────────────────────────

impl Vm {
    /// Spec 2 Phase A: dispatched from `Agent::fire_watchpoints_for_shape` when
    /// an `AdaptiveProtoLoad` observer fires. Clears the IC slot identified by
    /// `(code, slot)` if its current generation matches `expected_generation`.
    pub(crate) fn clear_ic_slot_if_generation_matches(
        &mut self,
        code: CodeRef,
        slot: FeedbackSlotId,
        expected_generation: u32,
    ) {
        let matches = self
            .property_ic_state(code, slot)
            .is_some_and(|s| s.generation == expected_generation);
        if !matches {
            return;
        }
        // Clear the slot entry — next slow-path visit will re-insert a
        // fresh default. This satisfies the D4 invariant: after a watchpoint
        // fire, `property_ic_state(code, slot)` must return `None`.
        self.clear_property_ic_state(code, slot);
        // Drop the polymorphic chain.
        self.drop_polymorphic_chain(code, slot);
        // Drop the KeyedPropertyIcState for this slot if present.
        self.clear_keyed_property_ic_state(code, slot);
        // Zero out the PropertyMetadata entry.
        if let Some(table) = self.metadata_table_mut(code) {
            *table.property_mut(slot.get()) = PropertyMetadata::default();
        }
    }

    /// Classify a property-IC slow-path entry by inspecting the live
    /// [`PropertyIcState`] against the receiver's current shape. Used by
    /// [`super::IcSlowPathCounters`] to bucket slow-path entries by cause.
    /// Reads only — does not mutate the IC state.
    #[cfg(feature = "diagnostic-counters")]
    pub(crate) fn classify_named_slow_cause_for_object(
        &self,
        agent: &Agent,
        code: CodeRef,
        slot: FeedbackSlotId,
        object: ObjectRef,
    ) -> super::IcSlowPathCause {
        use super::IcSlowPathCause;
        let Some(state) = self.property_ic_state(code, slot) else {
            return IcSlowPathCause::Uninitialized;
        };
        match state.cache_state {
            InlineCacheState::Uninitialized => IcSlowPathCause::Uninitialized,
            InlineCacheState::Megamorphic => IcSlowPathCause::Megamorphic,
            InlineCacheState::Polymorphic => IcSlowPathCause::Polymorphic,
            InlineCacheState::Monomorphic => {
                // Phase A.2 removed the epoch check from the asm; watchpoints now
                // clear the slot directly. `is_cleared = true` after a slow-path
                // visit indicates a watchpoint fire that hasn't been re-installed yet.
                if state.is_cleared {
                    return IcSlowPathCause::EpochMismatch;
                }
                let view = agent.heap().view();
                let Some(record) = view.object_ref(object) else {
                    return IcSlowPathCause::Other;
                };
                let receiver_shape = record.shape();
                let own_handler = state.monomorphic_own_data_handler;
                let proto_handler = state.monomorphic_proto_data_handler;
                if own_handler.is_valid() {
                    if own_handler.receiver_shape() != receiver_shape {
                        IcSlowPathCause::ShapeMismatch
                    } else {
                        // Same shape — the asm inline cache hit path should have
                        // matched. Remaining reasons: out-of-line slot,
                        // non-writable on the assign side, or other asm
                        // bail conditions not visible here.
                        IcSlowPathCause::ModeMismatch
                    }
                } else if proto_handler.is_valid() {
                    if proto_handler.receiver_shape() != receiver_shape {
                        IcSlowPathCause::ShapeMismatch
                    } else {
                        // Receiver matched; the asm `try_proto` branch failed
                        // on the prototype guard. Bucket as ModeMismatch (the
                        // proto side has no separate cause in the simplified
                        // post–Phase A taxonomy).
                        IcSlowPathCause::ModeMismatch
                    }
                } else {
                    // Monomorphic but no inline cache hit handler — entry is
                    // OwnDataTransition or another asm-incompatible shape.
                    IcSlowPathCause::ModeMismatch
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        call_feedback_builtin_is_frame_safe, DenseIndexCacheEntry, InlineCacheState,
        KeyedPropertyNamedEntries,
    };
    use crate::vm::ic_state::keyed_property::KeyedPropertyIcState;
    use lyng_objects::ObjectFlags;
    use lyng_types::{
        eval_builtin, function_builtin, function_call_builtin, string_char_code_at_builtin, ShapeId,
    };

    #[test]
    fn dense_index_observation_reports_whether_classification_changed() {
        let mut state = KeyedPropertyIcState::new();
        let mut _named = KeyedPropertyNamedEntries::default();
        let plan = DenseIndexCacheEntry::new(
            ShapeId::from_raw(1).expect("test shape id should be non-zero"),
            ObjectFlags::extensible(),
        );

        assert!(super::keyed_observe_dense_index_on_state(
            &mut state,
            Some(plan)
        ));
        assert!(!super::keyed_observe_dense_index_on_state(
            &mut state,
            Some(plan)
        ));
        assert_eq!(
            state.family,
            Some(crate::vm::ic_state::keyed_property::KeyedIcFamily::DenseIndex)
        );
        assert_eq!(state.cache_state, InlineCacheState::Monomorphic);
        assert_eq!(state.dense_entry_count, 1);

        assert!(super::keyed_observe_dense_index_on_state(&mut state, None));
        assert!(!super::keyed_observe_dense_index_on_state(&mut state, None));
        assert_eq!(state.cache_state, InlineCacheState::Megamorphic);
        assert_eq!(state.dense_entry_count, 0);
    }

    #[test]
    fn frame_safe_builtin_classification_excludes_frame_observers() {
        assert!(call_feedback_builtin_is_frame_safe(
            string_char_code_at_builtin()
        ));
        assert!(!call_feedback_builtin_is_frame_safe(eval_builtin()));
        assert!(!call_feedback_builtin_is_frame_safe(function_builtin()));
        assert!(!call_feedback_builtin_is_frame_safe(function_call_builtin()));
    }

    #[test]
    fn project_property_write_into_meta_writes_mode_5_for_inline_write() {
        use crate::vm::ic_state::PropertyIcState;
        use crate::vm::feedback::LLINT_IC_MODE_NAMED_OWN_INLINE_WRITE;
        use lyng_objects::{
            INLINE_SLOT_OFFSET_FLAG, NamedPropertyCacheEntry, NamedPropertyCachePath,
            NamedPropertyInlineWriteHandler, PROPERTY_CACHE_MAX_DEPENDENCIES,
        };
        use lyng_types::{DescriptorAttributes, ObjectRef, ShapeId};

        let source = ShapeId::from_raw(7).expect("non-zero");
        let target = ShapeId::from_raw(11).expect("non-zero");
        let mut attrs = DescriptorAttributes::empty();
        attrs.set_writable(true);
        let entry = NamedPropertyCacheEntry::new_for_test(
            source,
            ObjectRef::from_raw(1).expect("non-zero"),
            target,
            INLINE_SLOT_OFFSET_FLAG | 3,
            attrs,
            NamedPropertyCachePath::OwnDataTransition,
            1,
            [None; PROPERTY_CACHE_MAX_DEPENDENCIES],
        );

        let mut state = PropertyIcState::new();
        state.cache_state = InlineCacheState::Monomorphic;
        state.entry_count = 1;
        state.entries[0] = Some(entry);
        state.refresh_sidecars();

        let mut meta = crate::vm::metadata_table::PropertyMetadata::default();
        crate::vm::Vm::project_property_write_into_meta(
            state.monomorphic_own_inline_write_handler,
            state.generation,
            state.execution_count,
            &mut meta,
        );

        assert_eq!(meta.mode, LLINT_IC_MODE_NAMED_OWN_INLINE_WRITE);
        let expected = NamedPropertyInlineWriteHandler::from_entry(entry);
        assert_eq!(meta.handler_bits, expected.handler_bits());
        assert_eq!(meta.aux_bits, expected.target_bits());
        assert_eq!(meta.generation, 0);
        assert_eq!(meta.execution_count, 0);
    }

    #[test]
    fn project_property_write_into_meta_writes_mode_zero_for_invalid_handler() {
        use lyng_objects::NamedPropertyInlineWriteHandler;

        let mut meta = crate::vm::metadata_table::PropertyMetadata {
            mode: 99, // pre-existing garbage we expect to be zeroed
            handler_bits: 0xdead_beef,
            aux_bits: 0xcafe,
            generation: 10,
            execution_count: 20,
            ..crate::vm::metadata_table::PropertyMetadata::default()
        };
        crate::vm::Vm::project_property_write_into_meta(
            NamedPropertyInlineWriteHandler::NONE,
            5,
            7,
            &mut meta,
        );
        assert_eq!(meta.mode, 0);
        assert_eq!(meta.handler_bits, 0);
        assert_eq!(meta.aux_bits, 0);
        assert_eq!(meta.generation, 5);
        assert_eq!(meta.execution_count, 7);
    }
}
