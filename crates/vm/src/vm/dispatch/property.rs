#![allow(
    clippy::inline_always,
    reason = "Property dispatch helpers sit on named/keyed access hot paths and are intentionally forced through small inline probes"
)]

use crate::error::VmResult;
use crate::frame::FrameView;
use crate::vm::property_access::VmProxyBridge;
use crate::vm::registers::absolute_register;
use crate::{Vm, VmError};
use lyng_bytecode::Opcode;
use lyng_common::AtomId;
use lyng_env::Agent;
use lyng_gc::{AllocationLifetime, ValueStoreTarget};
use lyng_host::HostHooks;
use lyng_objects::{
    AdaptiveProtoLoadDispatch, InternalMethodError, NamedPropertyCacheEntry,
    NamedPropertyCachePurpose, NamedPropertyDirectGet, NativeFunctionRegistry, SlotLocation,
};
use lyng_ops::{errors, object};
use lyng_types::{CodeRef, FeedbackSlotId, ObjectRef, PropertyDescriptor, PropertyKey, Value};

impl Vm {
    pub(in crate::vm) fn execute_in_opcode(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameView,
        key_register: u16,
        receiver_register: u16,
    ) -> VmResult<Value> {
        let caller_realm = self.realm_of(agent, frame.cfr());
        let caller_lexical_env = self.frame_header(frame.cfr()).lexical_env();
        let caller_code = frame.code();
        let caller_pc = frame.instruction_offset();
        let key_value = self.read_register(frame.registers(), key_register);
        let receiver = self.read_register(frame.registers(), receiver_register);
        let object = receiver
            .as_object_ref()
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let key = self.property_key_from_value(
            agent,
            host,
            registry,
            caller_realm,
            caller_lexical_env,
            caller_code,
            caller_pc,
            key_value,
        )?;
        let has_property = {
            let mut bridge = VmProxyBridge {
                vm: self,
                agent,
                host,
                registry,
                caller_realm,
                caller_lexical_env,
                caller_code,
                caller_pc,
            };
            object::has_property_in_context(&mut bridge, object, key)
        }?;
        Ok(Value::from_bool(has_property))
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper keeps dispatch state explicit while isolating the property opcode family"
    )]
    #[inline]
    pub(in crate::vm) fn execute_get_named_property_opcode(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameView,
        feedback_slot: Option<FeedbackSlotId>,
        receiver_register: u16,
        atom_operand: u16,
    ) -> VmResult<Value> {
        let caller_realm = self.realm_of(agent, frame.cfr());
        let caller_lexical_env = self.frame_header(frame.cfr()).lexical_env();
        let caller_code = frame.code();
        let caller_pc = frame.instruction_offset();
        let registers = frame.registers();
        let receiver_index = absolute_register(registers, receiver_register);
        let receiver = self.arena.slots()[receiver_index];
        let atom = self.read_atom_constant(frame.code(), u32::from(atom_operand))?;
        let key = PropertyKey::from_atom(atom);
        // Asm slow-path entry: repaint PropertyMetadata if stale so the next asm
        // execution can take the inline IC route. Common case (mode non-zero) is a
        // single byte load + predicted-not-taken branch.
        if let Some(slot) = feedback_slot {
            self.refresh_named_property_metadata_if_stale(frame.code(), slot);
        }
        let value = if let Some(object) = receiver.as_object_ref() {
            // Monomorphic OwnData inline IC hit path: single packed-handler load,
            // shape compare, slot read. Polymorphic / PrototypeData / megamorphic
            // fall through below. AdaptiveProtoLoad watchpoints clear the IC slot
            // on any proto-chain mutation, so no epoch check is needed.
            if let Some(handler) = self.named_property_own_data_handler(frame.code(), feedback_slot)
            {
                let heap_view = agent.heap().view();
                if let Some(record) = heap_view.object_ref(object)
                    && record.shape() == handler.receiver_shape()
                {
                    let cached_value = match handler.slot_location() {
                        SlotLocation::Inline(index) => record.inline_named_slot(index as usize),
                        SlotLocation::OutOfLine(offset) => record
                            .named_slots()
                            .and_then(|slots| heap_view.object_slots(slots))
                            .and_then(|slots| slots.get(offset as usize).copied()),
                    };
                    if let Some(value) = cached_value {
                        if let Some(slot) = feedback_slot {
                            self.record_named_property_cache_hit(frame.code(), slot);
                        }
                        return Ok(value);
                    }
                }
            }
            // Polymorphic OwnData hit path: walks the [NamedPropertyHandler; POLY_LIMIT]
            // sidecar for a shape match before falling to the proto-data or slow chain.
            if let Some(value) = self.try_named_property_polymorphic_own_data_load(
                agent,
                frame.code(),
                feedback_slot,
                object,
            ) {
                return Ok(value);
            }
            // One-hop PrototypeData hit path: validates receiver shape + prototype shape
            // before reading the cached slot off the prototype. Multi-hop and other
            // shapes fall through to the slow chain.
            if let Some(value) =
                self.try_named_property_proto_data_load(agent, frame.code(), feedback_slot, object)
            {
                return Ok(value);
            }
            // Slow path: entries[POLY_LIMIT..entry_count] / multi-hop
            // PrototypeData / megamorphic / miss.
            if let Some(value) = self.try_named_property_load_inline_cache_hit(
                agent,
                frame.code(),
                feedback_slot,
                object,
            ) {
                return Ok(value);
            }
            if let Some(direct_get) =
                agent
                    .objects()
                    .try_direct_get_named_data_property(agent.heap().view(), object, key)
            {
                let value = match direct_get {
                    NamedPropertyDirectGet::Data(value) => {
                        self.observe_named_property_slow_path(
                            agent,
                            frame.code(),
                            feedback_slot,
                            object,
                            atom,
                            NamedPropertyCachePurpose::Load,
                        );
                        value
                    }
                    NamedPropertyDirectGet::Absent => {
                        // Absent loads only need a warmup ping; must not promote to Megamorphic.
                        self.record_feedback_slot(frame.code(), feedback_slot);
                        Value::undefined()
                    }
                };
                return Ok(value);
            }
            let property_result = self.get_property_from_value(
                agent,
                host,
                registry,
                caller_realm,
                caller_lexical_env,
                caller_code,
                caller_pc,
                receiver,
                key,
            );
            let value = property_result?;
            self.observe_named_property_slow_path(
                agent,
                frame.code(),
                feedback_slot,
                object,
                atom,
                NamedPropertyCachePurpose::Load,
            );
            value
        } else {
            let property_result = self.get_property_from_value(
                agent,
                host,
                registry,
                caller_realm,
                caller_lexical_env,
                caller_code,
                caller_pc,
                receiver,
                key,
            );
            property_result?
        };
        Ok(value)
    }

    pub(crate) fn try_assign_named_property_rust_probe_for_dsl(
        &mut self,
        agent: &mut Agent,
        frame: FrameView,
        feedback_slot: Option<FeedbackSlotId>,
        receiver_register: u16,
        value_register: u16,
    ) -> bool {
        let registers = frame.registers();
        let receiver = self.read_register(registers, receiver_register);
        let value = self.read_register(registers, value_register);
        let Some(object) = receiver.as_object_ref() else {
            return false;
        };

        let cached_target = self
            .named_property_own_data_handler(frame.code(), feedback_slot)
            .and_then(|handler| {
                let view = agent.heap().view();
                let record = view.object_ref(object)?;
                if record.shape() != handler.receiver_shape() || !handler.writable() {
                    return None;
                }
                match handler.slot_location() {
                    SlotLocation::Inline(index) => {
                        Some(ValueStoreTarget::InlineNamedSlot(object, index))
                    }
                    SlotLocation::OutOfLine(offset) => {
                        let slots = record.named_slots()?;
                        Some(ValueStoreTarget::ObjectSlot(slots, offset))
                    }
                }
            });
        if let Some(target) = cached_target {
            let stored = agent.with_heap_and_objects(|heap, _objects| {
                let mut mutator = heap.mutator();
                mutator.mut_store_value(target, value)
            });
            if stored {
                self.record_feedback_slot(frame.code(), feedback_slot);
                return true;
            }
            return false;
        }

        if self.try_named_property_polymorphic_own_data_store(
            agent,
            frame.code(),
            feedback_slot,
            object,
            value,
        ) == Some(Some(true))
        {
            self.record_feedback_slot(frame.code(), feedback_slot);
            return true;
        }

        false
    }

    #[expect(
        clippy::too_many_arguments,
        clippy::too_many_lines,
        reason = "VM helper keeps dispatch state explicit while isolating the property opcode family"
    )]
    pub(in crate::vm) fn execute_set_named_property_opcode(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameView,
        feedback_slot: Option<FeedbackSlotId>,
        opcode: Opcode,
        receiver_register: u16,
        value_register: u16,
        atom_operand: u16,
    ) -> VmResult<()> {
        let caller_realm = self.realm_of(agent, frame.cfr());
        let caller_lexical_env = self.frame_header(frame.cfr()).lexical_env();
        let caller_code = frame.code();
        let caller_pc = frame.instruction_offset();
        let assignment = matches!(
            opcode,
            Opcode::AssignNamedProperty | Opcode::StrictAssignNamedProperty
        );
        let strict_assignment = matches!(opcode, Opcode::StrictAssignNamedProperty);
        let registers = frame.registers();
        let receiver = self.arena.slots()[absolute_register(registers, receiver_register)];
        let value = self.arena.slots()[absolute_register(registers, value_register)];
        let atom = self.read_atom_constant(frame.code(), u32::from(atom_operand))?;
        let key = PropertyKey::from_atom(atom);
        if let Some(object) = receiver.as_object_ref() {
            // Monomorphic OwnData inline IC hit path (store side). Encodes the hit
            // decision into Option<Option<ValueStoreTarget>>:
            //   - Outer Some => take the cache hit path
            //   - Outer None => fall through to slow chain
            //   - Inner Some(target) => writable, do the store
            //   - Inner None        => read-only (writable bit clear), no store
            let cached_target = self
                .named_property_own_data_handler(frame.code(), feedback_slot)
                .and_then(|handler| {
                    let view = agent.heap().view();
                    let record = view.object_ref(object)?;
                    if record.shape() != handler.receiver_shape() {
                        return None;
                    }
                    if !handler.writable() {
                        return Some(None);
                    }
                    let target = match handler.slot_location() {
                        SlotLocation::Inline(index) => {
                            ValueStoreTarget::InlineNamedSlot(object, index)
                        }
                        SlotLocation::OutOfLine(offset) => {
                            // Cache invariant: named_slots is Some for any out-of-line
                            // OwnData hit. If it isn't (corrupt state), bail to slow.
                            let slots = record.named_slots()?;
                            ValueStoreTarget::ObjectSlot(slots, offset)
                        }
                    };
                    Some(Some(target))
                });
            if let Some(target_opt) = cached_target {
                let stored = target_opt.is_some_and(|target| {
                    agent.with_heap_and_objects(|heap, _objects| {
                        let mut mutator = heap.mutator();
                        mutator.mut_store_value(target, value)
                    })
                });
                if assignment {
                    let assignment_result = self.check_property_assignment_result(
                        agent,
                        frame,
                        stored,
                        strict_assignment,
                    );
                    assignment_result?;
                }
                self.record_feedback_slot(frame.code(), feedback_slot);
                return Ok(());
            }
            // Polymorphic OwnData store hit path. Same Option<Option<bool>> encoding:
            // Some(Some(_))=writable hit, Some(None)=non-writable, None=fall through.
            if let Some(target_opt) = self.try_named_property_polymorphic_own_data_store(
                agent,
                frame.code(),
                feedback_slot,
                object,
                value,
            ) {
                let stored = target_opt.unwrap_or(false);
                if assignment {
                    let assignment_result = self.check_property_assignment_result(
                        agent,
                        frame,
                        stored,
                        strict_assignment,
                    );
                    assignment_result?;
                }
                self.record_feedback_slot(frame.code(), feedback_slot);
                return Ok(());
            }
            // Slow path: entries[POLY_LIMIT..entry_count] / PrototypeData /
            // megamorphic / miss.
            if let Some(stored) = self.try_named_property_store_inline_cache(
                agent,
                frame.code(),
                feedback_slot,
                object,
                atom,
                value,
            ) {
                if assignment {
                    let assignment_result = self.check_property_assignment_result(
                        agent,
                        frame,
                        stored,
                        strict_assignment,
                    );
                    assignment_result?;
                }
                self.record_feedback_slot(frame.code(), feedback_slot);
                return Ok(());
            }
            match Self::try_named_property_transition_store(
                agent,
                self,
                object,
                key,
                value,
                AllocationLifetime::Default,
            ) {
                Ok(Some((stored, plan))) => {
                    if assignment {
                        let assignment_result = self.check_property_assignment_result(
                            agent,
                            frame,
                            stored,
                            strict_assignment,
                        );
                        assignment_result?;
                    }
                    self.observe_named_property_cache_entry(
                        agent,
                        frame.code(),
                        feedback_slot,
                        Some(plan),
                        NamedPropertyCachePurpose::Store,
                    );
                    return Ok(());
                }
                Ok(None) => {}
                Err(_error) => {
                    return Err(VmError::Abrupt(errors::throw_type_error(agent)));
                }
            }
            let set_result = if Self::prototype_chain_has_proxy(agent, object) {
                // Proxy-chain assignment funnels existing-own-data writes through
                // `Agent::define_own_property` (with this Vm as `vm_dispatch`), so
                // the construct `.prototype` watchpoint is fired there.
                self.set_property_on_value(
                    agent,
                    host,
                    registry,
                    caller_realm,
                    caller_lexical_env,
                    caller_code,
                    caller_pc,
                    receiver,
                    key,
                    value,
                )
            } else {
                // Non-proxy assignment stores in place via the objects-layer
                // define, which carries no `vm_dispatch` and fires no watchpoints.
                // Reassigning a function's `prototype` own slot is a same-shape
                // value write the shape-keyed watchpoints cannot observe, so fire
                // the per-constructor construct `.prototype` watchpoint here, at
                // the VM dispatch site where this Vm is available as the
                // dispatcher. The gate inside the helper keeps non-`prototype`
                // writes free of any added cost.
                let set_result =
                    object::ordinary_set(agent, object, key, value, AllocationLifetime::Default)
                        .map_err(VmError::Abrupt);
                match set_result {
                    Ok(result) => {
                        if result {
                            agent.fire_construct_prototype_watchpoint_if_function_prototype(
                                object, key, self,
                            );
                        }
                        Ok(result)
                    }
                    Err(VmError::Abrupt(_)) => self.set_property_on_value(
                        agent,
                        host,
                        registry,
                        caller_realm,
                        caller_lexical_env,
                        caller_code,
                        caller_pc,
                        receiver,
                        key,
                        value,
                    ),
                    Err(error) => Err(error),
                }
            };
            let stored = set_result?;
            if assignment {
                let assignment_result =
                    self.check_property_assignment_result(agent, frame, stored, strict_assignment);
                assignment_result?;
            }
            self.observe_named_property_slow_path(
                agent,
                frame.code(),
                feedback_slot,
                object,
                atom,
                NamedPropertyCachePurpose::Store,
            );
        } else {
            let store_result = self.set_property_on_value(
                agent,
                host,
                registry,
                caller_realm,
                caller_lexical_env,
                caller_code,
                caller_pc,
                receiver,
                key,
                value,
            );
            let stored = store_result?;
            if assignment {
                let assignment_result =
                    self.check_property_assignment_result(agent, frame, stored, strict_assignment);
                assignment_result?;
            }
        }
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper keeps dispatch state explicit while isolating the property opcode family"
    )]
    pub(in crate::vm) fn execute_define_named_property_opcode(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameView,
        object_register: u16,
        value_register: u16,
        atom_operand: u16,
    ) -> VmResult<()> {
        let object = self.object_register(frame, object_register)?;
        let value = self.read_register(frame.registers(), value_register);
        let key =
            PropertyKey::from_atom(self.read_atom_constant(frame.code(), u32::from(atom_operand))?);
        self.define_data_property(agent, host, registry, frame, object, key, value)?;
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        clippy::too_many_lines,
        reason = "VM helper keeps dispatch state explicit while isolating the property opcode family"
    )]
    #[inline]
    pub(in crate::vm) fn execute_get_keyed_property_opcode(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameView,
        feedback_slot: Option<FeedbackSlotId>,
        receiver_register: u16,
        key_register: u16,
    ) -> VmResult<Value> {
        let caller_realm = self.realm_of(agent, frame.cfr());
        let caller_lexical_env = self.frame_header(frame.cfr()).lexical_env();
        let caller_code = frame.code();
        let caller_pc = frame.instruction_offset();
        let receiver = self.read_register(frame.registers(), receiver_register);
        let key_value = self.read_register(frame.registers(), key_register);
        let coercible_result = Self::check_object_coercible(agent, receiver);
        coercible_result?;
        if let Some(object) = receiver.as_object_ref()
            && let Some(index) = key_value
                .as_smi()
                .and_then(|index| u32::try_from(index).ok())
        {
            // Dense-index monomorphic IC hit (SMI key).
            if let Some(value) = self.try_keyed_dense_index_cache_load(
                agent,
                frame.code(),
                feedback_slot,
                object,
                index,
            ) {
                return Ok(value);
            }
            // Dense-index polymorphic IC hit (SMI key).
            if let Some(value) = self.try_keyed_dense_polymorphic_cache_load(
                agent,
                frame.code(),
                feedback_slot,
                object,
                index,
            ) {
                return Ok(value);
            }
            // Slow path: dense_entries[POLY_LIMIT..] / Generic / megamorphic / miss.
            if let Some(value) = self.try_keyed_dense_index_load_inline_cache_hit(
                agent,
                frame.code(),
                feedback_slot,
                object,
                index,
            ) {
                return Ok(value);
            }
            let value = if let Some(result) = self.mapped_arguments_get(agent, object, index) {
                let value = result?;
                Some(value)
            } else if let Some(value) =
                Self::try_direct_typed_array_index_value(agent, object, index)
            {
                Some(value)
            } else {
                Self::try_direct_own_index_value(agent, object, index)?
            };
            if let Some(value) = value {
                self.observe_keyed_index_access(agent, frame.code(), feedback_slot, object, index);
                return Ok(value);
            }
        }
        let key_result = self.property_key_from_value(
            agent,
            host,
            registry,
            caller_realm,
            caller_lexical_env,
            caller_code,
            caller_pc,
            key_value,
        );
        let key = key_result?;
        let value = if let Some(object) = receiver.as_object_ref() {
            if let Some(index) = key.as_index() {
                // Dense-index monomorphic IC hit (post-coercion index).
                if let Some(value) = self.try_keyed_dense_index_cache_load(
                    agent,
                    frame.code(),
                    feedback_slot,
                    object,
                    index,
                ) {
                    return Ok(value);
                }
                // Dense-index polymorphic IC hit (post-coercion index).
                if let Some(value) = self.try_keyed_dense_polymorphic_cache_load(
                    agent,
                    frame.code(),
                    feedback_slot,
                    object,
                    index,
                ) {
                    return Ok(value);
                }
                // Slow path: dense_entries[POLY_LIMIT..] / Generic / megamorphic / miss.
                if let Some(value) = self.try_keyed_dense_index_load_inline_cache_hit(
                    agent,
                    frame.code(),
                    feedback_slot,
                    object,
                    index,
                ) {
                    return Ok(value);
                }
                let value = if let Some(result) = self.mapped_arguments_get(agent, object, index) {
                    result?
                } else if let Some(value) =
                    Self::try_direct_typed_array_index_value(agent, object, index)
                {
                    value
                } else if let Some(value) = Self::try_direct_own_index_value(agent, object, index)?
                {
                    value
                } else {
                    let property_result = self.get_property_from_value(
                        agent,
                        host,
                        registry,
                        caller_realm,
                        caller_lexical_env,
                        caller_code,
                        caller_pc,
                        receiver,
                        key,
                    );
                    property_result?
                };
                self.observe_keyed_index_access(agent, frame.code(), feedback_slot, object, index);
                value
            } else if let Some(atom) = key.as_atom() {
                // Named-keyed monomorphic OwnData IC hit.
                if let Some(value) = self.try_keyed_named_own_data_load(
                    agent,
                    frame.code(),
                    feedback_slot,
                    object,
                    atom,
                ) {
                    return Ok(value);
                }
                // Named-keyed polymorphic OwnData IC hit (extra atom-equality check).
                if let Some(value) = self.try_keyed_named_polymorphic_own_data_load(
                    agent,
                    frame.code(),
                    feedback_slot,
                    object,
                    atom,
                ) {
                    return Ok(value);
                }
                // Named-keyed one-hop PrototypeData IC hit.
                if let Some(value) = self.try_keyed_named_proto_data_load(
                    agent,
                    frame.code(),
                    feedback_slot,
                    object,
                    atom,
                ) {
                    return Ok(value);
                }
                // Slow path: named_entries[POLY_LIMIT..] / multi-hop
                // PrototypeData / megamorphic / Generic / miss.
                if let Some(value) = self.try_keyed_property_load_inline_cache(
                    agent,
                    frame.code(),
                    feedback_slot,
                    object,
                    atom,
                ) {
                    self.record_feedback_slot(frame.code(), feedback_slot);
                    return Ok(value);
                }
                let property_result = self.get_property_from_value(
                    agent,
                    host,
                    registry,
                    caller_realm,
                    caller_lexical_env,
                    caller_code,
                    caller_pc,
                    receiver,
                    key,
                );
                let value = property_result?;
                self.observe_keyed_atom_slow_path(
                    agent,
                    frame.code(),
                    feedback_slot,
                    object,
                    atom,
                    NamedPropertyCachePurpose::Load,
                );
                value
            } else {
                let property_result = self.get_property_from_value(
                    agent,
                    host,
                    registry,
                    caller_realm,
                    caller_lexical_env,
                    caller_code,
                    caller_pc,
                    receiver,
                    key,
                );
                let value = property_result?;
                self.observe_keyed_generic_slow_path(frame.code(), feedback_slot);
                value
            }
        } else {
            let property_result = self.get_property_from_value(
                agent,
                host,
                registry,
                caller_realm,
                caller_lexical_env,
                caller_code,
                caller_pc,
                receiver,
                key,
            );
            property_result?
        };
        Ok(value)
    }

    #[expect(
        clippy::too_many_arguments,
        clippy::too_many_lines,
        reason = "VM helper keeps dispatch state explicit while isolating the property opcode family"
    )]
    pub(in crate::vm) fn execute_set_keyed_property_opcode(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameView,
        feedback_slot: Option<FeedbackSlotId>,
        opcode: Opcode,
        receiver_register: u16,
        value_register: u16,
        key_register: u16,
    ) -> VmResult<()> {
        let caller_realm = self.realm_of(agent, frame.cfr());
        let caller_lexical_env = self.frame_header(frame.cfr()).lexical_env();
        let caller_code = frame.code();
        let caller_pc = frame.instruction_offset();
        let assignment = matches!(
            opcode,
            Opcode::AssignKeyedProperty | Opcode::StrictAssignKeyedProperty
        );
        let strict_assignment = matches!(opcode, Opcode::StrictAssignKeyedProperty);
        let receiver = self.read_register(frame.registers(), receiver_register);
        let value = self.read_register(frame.registers(), value_register);
        let key_value = self.read_register(frame.registers(), key_register);
        let coercible_result = Self::check_object_coercible(agent, receiver);
        coercible_result?;
        if let Some(object) = receiver.as_object_ref()
            && let Some(index) = key_value
                .as_smi()
                .and_then(|index| u32::try_from(index).ok())
        {
            // Dense-index monomorphic IC hit (SMI key, store).
            if let Some(stored) = self.try_keyed_dense_index_cache_store(
                agent,
                frame.code(),
                feedback_slot,
                object,
                index,
                value,
            ) {
                if assignment {
                    let assignment_result = self.check_property_assignment_result(
                        agent,
                        frame,
                        stored,
                        strict_assignment,
                    );
                    assignment_result?;
                }
                return Ok(());
            }
            // Dense-index polymorphic IC hit (SMI key, store).
            if let Some(stored) = self.try_keyed_dense_polymorphic_cache_store(
                agent,
                frame.code(),
                feedback_slot,
                object,
                index,
                value,
            ) {
                if assignment {
                    let assignment_result = self.check_property_assignment_result(
                        agent,
                        frame,
                        stored,
                        strict_assignment,
                    );
                    assignment_result?;
                }
                return Ok(());
            }
            // Slow path: dense_entries[POLY_LIMIT..] / Generic / megamorphic / miss.
            if let Some(stored) = self.try_keyed_dense_index_store_inline_cache_hit(
                agent,
                frame.code(),
                feedback_slot,
                object,
                index,
                value,
            ) {
                if assignment {
                    let assignment_result = self.check_property_assignment_result(
                        agent,
                        frame,
                        stored,
                        strict_assignment,
                    );
                    assignment_result?;
                }
                return Ok(());
            }
            let mut used_index_direct_path = false;
            let stored =
                if let Some(result) = self.mapped_arguments_set(agent, object, index, value) {
                    result?;
                    Some(true)
                } else {
                    let direct_result = self.try_direct_set_typed_array_index(
                        agent,
                        host,
                        registry,
                        caller_realm,
                        caller_lexical_env,
                        object,
                        index,
                        value,
                    );
                    let direct_result = direct_result?;
                    if let Some(stored) = direct_result {
                        used_index_direct_path = true;
                        Some(stored)
                    } else {
                        let direct_result =
                            Self::try_direct_set_engine_array_index(agent, object, index, value);
                        let direct_result = direct_result?;
                        if let Some(stored) = direct_result {
                            used_index_direct_path = true;
                            Some(stored)
                        } else {
                            let direct_result = Self::try_direct_set_ordinary_index_data_property(
                                agent, object, index, value,
                            );
                            let direct_result = direct_result?;
                            direct_result.inspect(|_| {
                                used_index_direct_path = true;
                            })
                        }
                    }
                };
            if let Some(stored) = stored {
                if assignment {
                    let assignment_result = self.check_property_assignment_result(
                        agent,
                        frame,
                        stored,
                        strict_assignment,
                    );
                    assignment_result?;
                }
                if !used_index_direct_path {
                    Self::sync_engine_array_length(agent, object)?;
                }
                self.observe_keyed_index_access(agent, frame.code(), feedback_slot, object, index);
                return Ok(());
            }
        }
        let key_result = self.property_key_from_value(
            agent,
            host,
            registry,
            caller_realm,
            caller_lexical_env,
            caller_code,
            caller_pc,
            key_value,
        );
        let key = key_result?;
        if let Some(object) = receiver.as_object_ref() {
            if let Some(index) = key.as_index() {
                // Dense-index monomorphic IC hit (post-coercion index, store).
                if let Some(stored) = self.try_keyed_dense_index_cache_store(
                    agent,
                    frame.code(),
                    feedback_slot,
                    object,
                    index,
                    value,
                ) {
                    if assignment {
                        let assignment_result = self.check_property_assignment_result(
                            agent,
                            frame,
                            stored,
                            strict_assignment,
                        );
                        assignment_result?;
                    }
                    return Ok(());
                }
                // Dense-index polymorphic IC hit (post-coercion index, store).
                if let Some(stored) = self.try_keyed_dense_polymorphic_cache_store(
                    agent,
                    frame.code(),
                    feedback_slot,
                    object,
                    index,
                    value,
                ) {
                    if assignment {
                        let assignment_result = self.check_property_assignment_result(
                            agent,
                            frame,
                            stored,
                            strict_assignment,
                        );
                        assignment_result?;
                    }
                    return Ok(());
                }
                // Slow path: dense_entries[POLY_LIMIT..] / Generic / megamorphic / miss.
                if let Some(stored) = self.try_keyed_dense_index_store_inline_cache_hit(
                    agent,
                    frame.code(),
                    feedback_slot,
                    object,
                    index,
                    value,
                ) {
                    if assignment {
                        let assignment_result = self.check_property_assignment_result(
                            agent,
                            frame,
                            stored,
                            strict_assignment,
                        );
                        assignment_result?;
                    }
                    return Ok(());
                }
                let mut used_index_direct_path = false;
                let stored = if let Some(result) =
                    self.mapped_arguments_set(agent, object, index, value)
                {
                    result?;
                    true
                } else {
                    let direct_result = self.try_direct_set_typed_array_index(
                        agent,
                        host,
                        registry,
                        caller_realm,
                        caller_lexical_env,
                        object,
                        index,
                        value,
                    );
                    let direct_result = direct_result?;
                    if let Some(stored) = direct_result {
                        used_index_direct_path = true;
                        stored
                    } else {
                        let direct_result =
                            Self::try_direct_set_engine_array_index(agent, object, index, value);
                        let direct_result = direct_result?;
                        if let Some(stored) = direct_result {
                            used_index_direct_path = true;
                            stored
                        } else {
                            let direct_result = Self::try_direct_set_ordinary_index_data_property(
                                agent, object, index, value,
                            );
                            let direct_result = direct_result?;
                            if let Some(stored) = direct_result {
                                used_index_direct_path = true;
                                stored
                            } else {
                                let set_result = self.set_property_on_value(
                                    agent,
                                    host,
                                    registry,
                                    caller_realm,
                                    caller_lexical_env,
                                    caller_code,
                                    caller_pc,
                                    receiver,
                                    key,
                                    value,
                                );
                                set_result?
                            }
                        }
                    }
                };
                if assignment {
                    let assignment_result = self.check_property_assignment_result(
                        agent,
                        frame,
                        stored,
                        strict_assignment,
                    );
                    assignment_result?;
                }
                if !used_index_direct_path {
                    Self::sync_engine_array_length(agent, object)?;
                }
                self.observe_keyed_index_access(agent, frame.code(), feedback_slot, object, index);
            } else if let Some(atom) = key.as_atom() {
                // Named-keyed monomorphic OwnData IC hit (store).
                if let Some(stored) = self.try_keyed_named_own_data_store(
                    agent,
                    frame.code(),
                    feedback_slot,
                    object,
                    atom,
                    value,
                ) {
                    if assignment {
                        let assignment_result = self.check_property_assignment_result(
                            agent,
                            frame,
                            stored,
                            strict_assignment,
                        );
                        assignment_result?;
                    }
                    return Ok(());
                }
                // Named-keyed polymorphic OwnData IC hit (store).
                if let Some(stored) = self.try_keyed_named_polymorphic_own_data_store(
                    agent,
                    frame.code(),
                    feedback_slot,
                    object,
                    atom,
                    value,
                ) {
                    if assignment {
                        let assignment_result = self.check_property_assignment_result(
                            agent,
                            frame,
                            stored,
                            strict_assignment,
                        );
                        assignment_result?;
                    }
                    return Ok(());
                }
                // Slow path: named_entries[POLY_LIMIT..] / megamorphic /
                // Generic / miss.
                if let Some(stored) = self.try_keyed_property_store_inline_cache(
                    agent,
                    frame.code(),
                    feedback_slot,
                    object,
                    atom,
                    value,
                ) {
                    if assignment {
                        let assignment_result = self.check_property_assignment_result(
                            agent,
                            frame,
                            stored,
                            strict_assignment,
                        );
                        assignment_result?;
                    }
                    self.record_feedback_slot(frame.code(), feedback_slot);
                    return Ok(());
                }
                let set_result = self.set_property_on_value(
                    agent,
                    host,
                    registry,
                    caller_realm,
                    caller_lexical_env,
                    caller_code,
                    caller_pc,
                    receiver,
                    key,
                    value,
                );
                let stored = set_result?;
                if assignment {
                    let assignment_result = self.check_property_assignment_result(
                        agent,
                        frame,
                        stored,
                        strict_assignment,
                    );
                    assignment_result?;
                }
                self.observe_keyed_atom_slow_path(
                    agent,
                    frame.code(),
                    feedback_slot,
                    object,
                    atom,
                    NamedPropertyCachePurpose::Store,
                );
            } else {
                let set_result = self.set_property_on_value(
                    agent,
                    host,
                    registry,
                    caller_realm,
                    caller_lexical_env,
                    caller_code,
                    caller_pc,
                    receiver,
                    key,
                    value,
                );
                let stored = set_result?;
                if assignment {
                    let assignment_result = self.check_property_assignment_result(
                        agent,
                        frame,
                        stored,
                        strict_assignment,
                    );
                    assignment_result?;
                }
                self.observe_keyed_generic_slow_path(frame.code(), feedback_slot);
            }
        } else {
            let store_result = self.set_property_on_value(
                agent,
                host,
                registry,
                caller_realm,
                caller_lexical_env,
                caller_code,
                caller_pc,
                receiver,
                key,
                value,
            );
            let stored = store_result?;
            if assignment {
                let assignment_result =
                    self.check_property_assignment_result(agent, frame, stored, strict_assignment);
                assignment_result?;
            }
        }
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper keeps dispatch state explicit while isolating the property opcode family"
    )]
    pub(in crate::vm) fn execute_define_keyed_property_opcode(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameView,
        object_register: u16,
        value_register: u16,
        key_register: u16,
    ) -> VmResult<()> {
        let caller_realm = self.realm_of(agent, frame.cfr());
        let caller_lexical_env = self.frame_header(frame.cfr()).lexical_env();
        let caller_code = frame.code();
        let caller_pc = frame.instruction_offset();
        let object = self.object_register(frame, object_register)?;
        let value = self.read_register(frame.registers(), value_register);
        let key_value = self.read_register(frame.registers(), key_register);
        let key_result = self.property_key_from_value(
            agent,
            host,
            registry,
            caller_realm,
            caller_lexical_env,
            caller_code,
            caller_pc,
            key_value,
        );
        let key = key_result?;
        self.define_data_property(agent, host, registry, frame, object, key, value)?;
        if key.as_index().is_some() {
            Self::sync_engine_array_length(agent, object)?;
        }
        Ok(())
    }

    pub(in crate::vm) fn execute_to_property_key_opcode(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameView,
        key_register: u16,
    ) -> VmResult<Value> {
        let caller_realm = self.realm_of(agent, frame.cfr());
        let caller_lexical_env = self.frame_header(frame.cfr()).lexical_env();
        let caller_code = frame.code();
        let caller_pc = frame.instruction_offset();
        let key_value = self.read_register(frame.registers(), key_register);
        let key_result = self.property_key_from_value(
            agent,
            host,
            registry,
            caller_realm,
            caller_lexical_env,
            caller_code,
            caller_pc,
            key_value,
        );
        let key = key_result?;
        let value = self.property_key_to_enumeration_value(agent, key);
        Ok(value)
    }

    pub(in crate::vm) fn execute_delete_property_opcode(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameView,
        receiver_register: u16,
        key_register: u16,
    ) -> VmResult<Value> {
        let caller_realm = self.realm_of(agent, frame.cfr());
        let caller_lexical_env = self.frame_header(frame.cfr()).lexical_env();
        let caller_code = frame.code();
        let caller_pc = frame.instruction_offset();
        let receiver = self.read_register(frame.registers(), receiver_register);
        let key_value = self.read_register(frame.registers(), key_register);
        let coercible_result = Self::check_object_coercible(agent, receiver);
        coercible_result?;
        let key_result = self.property_key_from_value(
            agent,
            host,
            registry,
            caller_realm,
            caller_lexical_env,
            caller_code,
            caller_pc,
            key_value,
        );
        let key = key_result?;
        let delete_result = self.delete_property_from_value(
            agent,
            host,
            registry,
            caller_realm,
            caller_lexical_env,
            caller_code,
            caller_pc,
            receiver,
            key,
        );
        let deleted = delete_result?;
        if !deleted && self.frame_is_strict(frame) {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        Ok(Value::from_bool(deleted))
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper keeps dispatch state explicit while isolating the property opcode family"
    )]
    pub(in crate::vm) fn execute_copy_data_properties_opcode(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameView,
        target_register: u16,
        source_register: u16,
        excluded_register: u16,
    ) -> VmResult<()> {
        let caller_realm = self.realm_of(agent, frame.cfr());
        let caller_lexical_env = self.frame_header(frame.cfr()).lexical_env();
        let caller_code = frame.code();
        let caller_pc = frame.instruction_offset();
        let target = self.object_register(frame, target_register)?;
        let source = self.read_register(frame.registers(), source_register);
        let excluded_keys = self.read_register(frame.registers(), excluded_register);
        let copy_result = self.copy_data_properties(
            agent,
            host,
            registry,
            caller_realm,
            caller_lexical_env,
            caller_code,
            caller_pc,
            target,
            source,
            excluded_keys,
        );
        copy_result?;
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper keeps dispatch state explicit while isolating the property opcode family"
    )]
    pub(in crate::vm) fn execute_store_dense_element_opcode(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameView,
        receiver_register: u16,
        value_register: u16,
        index_operand: u16,
    ) -> VmResult<()> {
        let caller_realm = self.realm_of(agent, frame.cfr());
        let caller_lexical_env = self.frame_header(frame.cfr()).lexical_env();
        let caller_code = frame.code();
        let caller_pc = frame.instruction_offset();
        let receiver = self.read_register(frame.registers(), receiver_register);
        let value = self.read_register(frame.registers(), value_register);
        if let Some(object) = receiver.as_object_ref() {
            if let Some(result) =
                self.mapped_arguments_set(agent, object, u32::from(index_operand), value)
            {
                result?;
            }
            let _ = agent.with_heap_and_objects(|heap, objects| {
                let mut mutator = heap.mutator();
                objects.set_element(
                    &mut mutator,
                    object,
                    u32::from(index_operand),
                    value,
                    AllocationLifetime::Default,
                )
            });
        } else {
            let store_result = self.set_property_on_value(
                agent,
                host,
                registry,
                caller_realm,
                caller_lexical_env,
                caller_code,
                caller_pc,
                receiver,
                PropertyKey::Index(u32::from(index_operand)),
                value,
            );
            store_result?;
        }
        Ok(())
    }

    pub(in crate::vm) fn execute_load_dense_element_opcode(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameView,
        receiver_register: u16,
        index_operand: u16,
    ) -> VmResult<Value> {
        let caller_realm = self.realm_of(agent, frame.cfr());
        let caller_lexical_env = self.frame_header(frame.cfr()).lexical_env();
        let caller_code = frame.code();
        let caller_pc = frame.instruction_offset();
        let receiver = self.read_register(frame.registers(), receiver_register);
        let value = if let Some(object) = receiver.as_object_ref() {
            if let Some(result) = self.mapped_arguments_get(agent, object, u32::from(index_operand))
            {
                result?
            } else if let Some(value) =
                Self::try_direct_own_index_value(agent, object, u32::from(index_operand))?
            {
                value
            } else if Self::prototype_chain_has_proxy(agent, object) {
                let property_result = self.get_property_from_value(
                    agent,
                    host,
                    registry,
                    caller_realm,
                    caller_lexical_env,
                    caller_code,
                    caller_pc,
                    receiver,
                    PropertyKey::Index(u32::from(index_operand)),
                );
                property_result?
            } else {
                let element = object::ordinary_get(
                    agent,
                    object,
                    PropertyKey::Index(u32::from(index_operand)),
                )
                .map_err(VmError::Abrupt);
                element?
            }
        } else {
            let property_result = self.get_property_from_value(
                agent,
                host,
                registry,
                caller_realm,
                caller_lexical_env,
                caller_code,
                caller_pc,
                receiver,
                PropertyKey::Index(u32::from(index_operand)),
            );
            property_result?
        };
        Ok(value)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly"
    )]
    fn define_data_property(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameView,
        object: lyng_types::ObjectRef,
        key: PropertyKey,
        value: Value,
    ) -> VmResult<()> {
        let caller_realm = self.realm_of(agent, frame.cfr());
        let caller_lexical_env = self.frame_header(frame.cfr()).lexical_env();
        let caller_code = frame.code();
        let caller_pc = frame.instruction_offset();
        let mut descriptor = PropertyDescriptor::new();
        descriptor.set_value(value);
        descriptor.set_writable(true);
        descriptor.set_enumerable(true);
        descriptor.set_configurable(true);
        let define_result = object::define_property_in_context(
            &mut VmProxyBridge {
                vm: self,
                agent,
                host,
                registry,
                caller_realm,
                caller_lexical_env,
                caller_code,
                caller_pc,
            },
            object,
            key,
            descriptor,
            AllocationLifetime::Default,
        );
        let created = define_result?;
        if !created {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        Ok(())
    }

    fn check_property_assignment_result(
        &self,
        agent: &mut Agent,
        frame: FrameView,
        stored: bool,
        strict_override: bool,
    ) -> VmResult<()> {
        if !stored && (strict_override || self.frame_is_strict(frame)) {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        Ok(())
    }

    fn try_named_property_transition_store(
        agent: &mut Agent,
        vm_dispatch: &mut dyn AdaptiveProtoLoadDispatch,
        object: ObjectRef,
        key: PropertyKey,
        value: Value,
        lifetime: AllocationLifetime,
    ) -> Result<Option<(bool, NamedPropertyCacheEntry)>, InternalMethodError> {
        let result = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            let Some(plan) = objects.plan_named_property_transition_store_entry(
                &mut mutator,
                object,
                key,
                lifetime,
            )?
            else {
                return Ok(None);
            };
            let stored =
                objects.store_to_named_property_cache(&mut mutator, object, key, plan, value)?;
            Ok(stored.map(|stored| (stored, plan)))
        });
        // Fire watchpoints on the pre-transition shape for any successful plan.
        if let Ok(Some((_, plan))) = result {
            agent.fire_watchpoints_for_shape(plan.receiver_shape(), vm_dispatch);
        }
        result
    }

    /// Monomorphic dense-index load IC hit. Returns the cached element value,
    /// or `None` on shape/flags miss, hole, or non-monomorphic cache state.
    #[inline(always)]
    fn try_keyed_dense_index_cache_load(
        &mut self,
        agent: &Agent,
        code: CodeRef,
        feedback_slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
        index: u32,
    ) -> Option<Value> {
        let handler = self.keyed_property_dense_index_handler(code, feedback_slot)?;
        let view = agent.heap().view();
        let header = agent.objects().object_header(view, receiver)?;
        if handler.receiver_shape() != Some(header.shape())
            || handler.receiver_flags() != header.flags()
        {
            return None;
        }
        let elements = header.elements()?;
        let value = view
            .object_slots(elements.raw())?
            .get(index as usize)
            .copied()?;
        if value == Value::array_hole() {
            return None;
        }
        if let Some(slot) = feedback_slot {
            self.record_named_property_cache_hit(code, slot);
        }
        Some(value)
    }

    /// Monomorphic dense-index store IC hit. Returns `Some(true)` on a successful
    /// barrier-aware write, `None` on miss / hole / shape mismatch / out-of-bounds.
    #[inline(always)]
    fn try_keyed_dense_index_cache_store(
        &mut self,
        agent: &mut Agent,
        code: CodeRef,
        feedback_slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
        index: u32,
        value: Value,
    ) -> Option<bool> {
        if value == Value::array_hole() {
            return None;
        }
        let handler = self.keyed_property_dense_index_handler(code, feedback_slot)?;
        let (elements, current) = {
            let view = agent.heap().view();
            let header = agent.objects().object_header(view, receiver)?;
            if handler.receiver_shape() != Some(header.shape())
                || handler.receiver_flags() != header.flags()
            {
                return None;
            }
            let elements = header.elements()?;
            let current = view
                .object_slots(elements.raw())?
                .get(index as usize)
                .copied()
                .unwrap_or(Value::array_hole());
            (elements, current)
        };
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
        if let Some(slot) = feedback_slot {
            self.record_named_property_cache_hit(code, slot);
        }
        Some(true)
    }

    /// Monomorphic named-keyed (atom) `OwnData` load IC hit. Returns the cached slot
    /// value, or `None` on miss.
    #[inline(always)]
    fn try_keyed_named_own_data_load(
        &mut self,
        agent: &Agent,
        code: CodeRef,
        feedback_slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
        atom: AtomId,
    ) -> Option<Value> {
        let handler = self.keyed_property_named_own_data_handler(code, feedback_slot, atom)?;
        let view = agent.heap().view();
        let record = view.object_ref(receiver)?;
        if record.shape() != handler.receiver_shape() {
            return None;
        }
        let value = match handler.slot_location() {
            SlotLocation::Inline(i) => record.inline_named_slot(i as usize)?,
            SlotLocation::OutOfLine(off) => view
                .object_slots(record.named_slots()?)?
                .get(off as usize)
                .copied()?,
        };
        self.record_feedback_slot(code, feedback_slot);
        Some(value)
    }

    /// Monomorphic one-hop `PrototypeData` load IC hit (non-keyed). Validates
    /// receiver shape + prototype shape, then reads the cached slot from the
    /// prototype. Returns `None` on any miss. `AdaptiveProtoLoad` watchpoints
    /// clear the IC slot on proto-chain mutation, so no epoch check is needed.
    #[inline(always)]
    pub(in crate::vm) fn try_named_property_proto_data_load(
        &mut self,
        agent: &Agent,
        code: CodeRef,
        feedback_slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
    ) -> Option<Value> {
        let handler = self.named_property_proto_data_handler(code, feedback_slot)?;
        let view = agent.heap().view();
        let record = view.object_ref(receiver)?;
        if record.shape() != handler.receiver_shape() {
            return None;
        }
        let prototype_id = record.prototype()?;
        let prototype_record = view.object_ref(prototype_id)?;
        if prototype_record.shape() != handler.prototype_shape() {
            return None;
        }
        let value = match handler.slot_location() {
            SlotLocation::Inline(i) => prototype_record.inline_named_slot(i as usize)?,
            SlotLocation::OutOfLine(off) => view
                .object_slots(prototype_record.named_slots()?)?
                .get(off as usize)
                .copied()?,
        };
        if let Some(slot) = feedback_slot {
            self.record_named_property_cache_hit(code, slot);
        }
        Some(value)
    }

    /// Monomorphic named-keyed (atom) one-hop `PrototypeData` load IC hit.
    /// Returns the cached slot value from the prototype, or `None` on miss.
    #[inline(always)]
    fn try_keyed_named_proto_data_load(
        &mut self,
        agent: &Agent,
        code: CodeRef,
        feedback_slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
        atom: AtomId,
    ) -> Option<Value> {
        let handler = self.keyed_property_named_proto_data_handler(code, feedback_slot, atom)?;
        let view = agent.heap().view();
        let record = view.object_ref(receiver)?;
        if record.shape() != handler.receiver_shape() {
            return None;
        }
        let prototype_id = record.prototype()?;
        let prototype_record = view.object_ref(prototype_id)?;
        if prototype_record.shape() != handler.prototype_shape() {
            return None;
        }
        let value = match handler.slot_location() {
            SlotLocation::Inline(i) => prototype_record.inline_named_slot(i as usize)?,
            SlotLocation::OutOfLine(off) => view
                .object_slots(prototype_record.named_slots()?)?
                .get(off as usize)
                .copied()?,
        };
        self.record_feedback_slot(code, feedback_slot);
        Some(value)
    }

    /// Monomorphic named-keyed (atom) `OwnData` store IC hit. Returns `Some(stored)`
    /// on hit (`Some(false)` for non-writable), `None` on miss.
    #[inline(always)]
    fn try_keyed_named_own_data_store(
        &mut self,
        agent: &mut Agent,
        code: CodeRef,
        feedback_slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
        atom: AtomId,
        value: Value,
    ) -> Option<bool> {
        let handler = self.keyed_property_named_own_data_handler(code, feedback_slot, atom)?;
        let (named_slots, shape_match) = {
            let view = agent.heap().view();
            let record = view.object_ref(receiver)?;
            let shape_match = record.shape() == handler.receiver_shape();
            (record.named_slots(), shape_match)
        };
        if !shape_match {
            return None;
        }
        let stored = if handler.writable() {
            let target = match handler.slot_location() {
                SlotLocation::Inline(i) => ValueStoreTarget::InlineNamedSlot(receiver, i),
                SlotLocation::OutOfLine(off) => ValueStoreTarget::ObjectSlot(named_slots?, off),
            };
            agent.with_heap_and_objects(|heap, _objects| {
                let mut mutator = heap.mutator();
                mutator.mut_store_value(target, value)
            })
        } else {
            false
        };
        self.record_feedback_slot(code, feedback_slot);
        Some(stored)
    }

    /// Polymorphic `OwnData` load IC hit (non-keyed). Walks the inline sidecar
    /// (up to `POLY_LIMIT` shapes) for a shape match. Returns `None` on miss.
    /// `AdaptiveProtoLoad` watchpoints clear the IC on proto-chain mutation.
    #[inline(always)]
    pub(in crate::vm) fn try_named_property_polymorphic_own_data_load(
        &mut self,
        agent: &Agent,
        code: CodeRef,
        feedback_slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
    ) -> Option<Value> {
        let view = agent.heap().view();
        let record = view.object_ref(receiver)?;
        let shape = record.shape()?;
        let handler =
            self.named_property_polymorphic_own_data_handler(code, feedback_slot, shape)?;
        let value = match handler.slot_location() {
            SlotLocation::Inline(i) => record.inline_named_slot(i as usize)?,
            SlotLocation::OutOfLine(off) => view
                .object_slots(record.named_slots()?)?
                .get(off as usize)
                .copied()?,
        };
        if let Some(slot) = feedback_slot {
            self.record_named_property_cache_hit(code, slot);
        }
        Some(value)
    }

    /// Polymorphic `OwnData` store IC hit (non-keyed). Mirrors
    /// [`Self::try_named_property_polymorphic_own_data_load`] for Set /
    /// Assign / `StrictAssign` / global-store opcodes.
    ///
    /// `Option<Option<bool>>` encoding: `None`=miss, `Some(None)`=non-writable
    /// hit, `Some(Some(stored))`=writable hit with barrier result.
    #[inline(always)]
    #[allow(
        clippy::option_option,
        reason = "store cache probes need three states: miss, non-writable hit, and writable hit with barrier result"
    )]
    pub(in crate::vm) fn try_named_property_polymorphic_own_data_store(
        &self,
        agent: &mut Agent,
        code: CodeRef,
        feedback_slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
        value: Value,
    ) -> Option<Option<bool>> {
        let (handler, named_slots) = {
            let view = agent.heap().view();
            let record = view.object_ref(receiver)?;
            let shape = record.shape()?;
            let handler =
                self.named_property_polymorphic_own_data_handler(code, feedback_slot, shape)?;
            (handler, record.named_slots())
        };
        if !handler.writable() {
            return Some(None);
        }
        let target = match handler.slot_location() {
            SlotLocation::Inline(i) => ValueStoreTarget::InlineNamedSlot(receiver, i),
            SlotLocation::OutOfLine(off) => ValueStoreTarget::ObjectSlot(named_slots?, off),
        };
        let stored = agent.with_heap_and_objects(|heap, _objects| {
            let mut mutator = heap.mutator();
            mutator.mut_store_value(target, value)
        });
        Some(Some(stored))
    }

    /// Polymorphic named-keyed (atom) `OwnData` load IC hit. Matches atom + shape
    /// in the keyed polymorphic sidecar. Returns `None` on miss.
    #[inline(always)]
    fn try_keyed_named_polymorphic_own_data_load(
        &mut self,
        agent: &Agent,
        code: CodeRef,
        feedback_slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
        atom: AtomId,
    ) -> Option<Value> {
        let view = agent.heap().view();
        let record = view.object_ref(receiver)?;
        let shape = record.shape()?;
        let handler = self.keyed_property_named_polymorphic_own_data_handler(
            code,
            feedback_slot,
            atom,
            shape,
        )?;
        let value = match handler.slot_location() {
            SlotLocation::Inline(i) => record.inline_named_slot(i as usize)?,
            SlotLocation::OutOfLine(off) => view
                .object_slots(record.named_slots()?)?
                .get(off as usize)
                .copied()?,
        };
        self.record_feedback_slot(code, feedback_slot);
        Some(value)
    }

    /// Polymorphic dense-index load IC hit. Walks the `[KeyedDenseIndexHandler; POLY_LIMIT]`
    /// sidecar for a shape+flags match. Mirrors
    /// [`Self::try_keyed_dense_index_cache_load`] for shapes `2..POLY_LIMIT`.
    #[inline(always)]
    fn try_keyed_dense_polymorphic_cache_load(
        &mut self,
        agent: &Agent,
        code: CodeRef,
        feedback_slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
        index: u32,
    ) -> Option<Value> {
        let handlers = self.keyed_property_dense_polymorphic_handlers(code, feedback_slot)?;
        let view = agent.heap().view();
        let header = agent.objects().object_header(view, receiver)?;
        let target_shape = Some(header.shape());
        let target_flags = header.flags();
        let matched = handlers.iter().any(|handler| {
            handler.is_valid()
                && handler.receiver_shape() == target_shape
                && handler.receiver_flags() == target_flags
        });
        if !matched {
            return None;
        }
        let elements = header.elements()?;
        let value = view
            .object_slots(elements.raw())?
            .get(index as usize)
            .copied()?;
        if value == Value::array_hole() {
            return None;
        }
        if let Some(slot) = feedback_slot {
            self.record_named_property_cache_hit(code, slot);
        }
        Some(value)
    }

    /// Polymorphic dense-index store IC hit. Mirrors
    /// [`Self::try_keyed_dense_index_cache_store`] for shapes `2..POLY_LIMIT`.
    #[inline(always)]
    fn try_keyed_dense_polymorphic_cache_store(
        &mut self,
        agent: &mut Agent,
        code: CodeRef,
        feedback_slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
        index: u32,
        value: Value,
    ) -> Option<bool> {
        if value == Value::array_hole() {
            return None;
        }
        let (elements, current) = {
            let handlers = self.keyed_property_dense_polymorphic_handlers(code, feedback_slot)?;
            let view = agent.heap().view();
            let header = agent.objects().object_header(view, receiver)?;
            let target_shape = Some(header.shape());
            let target_flags = header.flags();
            let matched = handlers.iter().any(|handler| {
                handler.is_valid()
                    && handler.receiver_shape() == target_shape
                    && handler.receiver_flags() == target_flags
            });
            if !matched {
                return None;
            }
            let elements = header.elements()?;
            let current = view
                .object_slots(elements.raw())?
                .get(index as usize)
                .copied()
                .unwrap_or(Value::array_hole());
            (elements, current)
        };
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
        if let Some(slot) = feedback_slot {
            self.record_named_property_cache_hit(code, slot);
        }
        Some(true)
    }

    /// Polymorphic named-keyed (atom) `OwnData` store IC hit. Mirrors
    /// [`Self::try_keyed_named_own_data_store`] for shapes `2..POLY_LIMIT`.
    #[inline(always)]
    fn try_keyed_named_polymorphic_own_data_store(
        &mut self,
        agent: &mut Agent,
        code: CodeRef,
        feedback_slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
        atom: AtomId,
        value: Value,
    ) -> Option<bool> {
        let (handler, named_slots) = {
            let view = agent.heap().view();
            let record = view.object_ref(receiver)?;
            let shape = record.shape()?;
            let handler = self.keyed_property_named_polymorphic_own_data_handler(
                code,
                feedback_slot,
                atom,
                shape,
            )?;
            (handler, record.named_slots())
        };
        let stored = if handler.writable() {
            let target = match handler.slot_location() {
                SlotLocation::Inline(i) => ValueStoreTarget::InlineNamedSlot(receiver, i),
                SlotLocation::OutOfLine(off) => ValueStoreTarget::ObjectSlot(named_slots?, off),
            };
            agent.with_heap_and_objects(|heap, _objects| {
                let mut mutator = heap.mutator();
                mutator.mut_store_value(target, value)
            })
        } else {
            false
        };
        self.record_feedback_slot(code, feedback_slot);
        Some(stored)
    }
}
