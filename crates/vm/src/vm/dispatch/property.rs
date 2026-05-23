use super::advance_dispatch_frame;
use crate::error::VmResult;
use crate::vm::property_access::VmProxyBridge;
use crate::vm::registers::absolute_register;
use crate::{FrameRecord, Vm, VmError};
use lyng_bytecode::Opcode;
use lyng_common::AtomId;
use lyng_env::Agent;
use lyng_gc::{AllocationLifetime, ValueStoreTarget};
use lyng_host::HostHooks;
use lyng_objects::{NamedPropertyCachePurpose, NativeFunctionRegistry, SlotLocation};
use lyng_ops::{errors, object};
use lyng_types::{CodeRef, FeedbackSlotId, ObjectRef, PropertyDescriptor, PropertyKey, Value};

impl Vm {
    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper keeps dispatch state explicit while isolating the property opcode family"
    )]
    pub(in crate::vm) fn execute_in_opcode(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame_depth: usize,
        frame: &mut FrameRecord,
        instruction_len: u32,
        target: u16,
        key_register: u16,
        receiver_register: u16,
    ) -> VmResult<()> {
        let key_value = self.read_register(frame.registers(), key_register);
        let receiver = self.read_register(frame.registers(), receiver_register);
        let object_result = receiver
            .as_object_ref()
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)));
        let Some(object) = self.handle_dispatch_result(agent, frame_depth, frame, object_result)?
        else {
            return Ok(());
        };
        let key_result = self.property_key_from_value(agent, host, registry, frame, key_value);
        let Some(key) = self.handle_dispatch_result(agent, frame_depth, frame, key_result)? else {
            return Ok(());
        };
        let has_property = {
            let mut bridge = VmProxyBridge {
                vm: self,
                agent,
                host,
                registry,
                frame,
            };
            object::has_property_in_context(&mut bridge, object, key)
        };
        let Some(has_property) =
            self.handle_dispatch_result(agent, frame_depth, frame, has_property)?
        else {
            return Ok(());
        };
        self.write_register(frame.registers(), target, Value::from_bool(has_property));
        advance_dispatch_frame(frame, instruction_len);
        Ok(())
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
        frame_depth: usize,
        frame: &mut FrameRecord,
        instruction_len: u32,
        feedback_slot: Option<FeedbackSlotId>,
        target: u16,
        receiver_register: u16,
        atom_operand: u16,
    ) -> VmResult<()> {
        let registers = frame.registers();
        let receiver_index = absolute_register(registers, receiver_register);
        let target_index = absolute_register(registers, target);
        let receiver = self.register_stack[receiver_index];
        let atom = self.read_atom_constant(frame.code(), u32::from(atom_operand))?;
        let key = PropertyKey::from_atom(atom);
        let value = if let Some(object) = receiver.as_object_ref() {
            // Phase 3 inline IC fast path: a single packed-handler load,
            // one shape compare, one epoch compare, one slot read. Bypasses
            // the 4-deep try_named_property_load_inline_cache_hit ->
            // try_load -> load_from_named_property_cache -> validated_holder
            // chain on the monomorphic OwnData hit. Polymorphic /
            // PrototypeData / megamorphic still fall through to the existing
            // chain below. The epoch compare mirrors
            // `record_matches_cache_dependency` and is what catches
            // non-shape invalidations like prototype mutation.
            if let Some((handler, cached_epoch)) =
                self.named_property_fast_handler(frame.code(), feedback_slot)
            {
                let heap_view = agent.heap().view();
                if let Some(record) = heap_view.object_ref(object) {
                    if record.shape() == handler.receiver_shape()
                        && record.last_invalidation_epoch().unwrap_or(0) == cached_epoch
                    {
                        let fast_value = match handler.slot_location() {
                            SlotLocation::Inline(index) => record.inline_named_slot(index as usize),
                            SlotLocation::OutOfLine(offset) => record
                                .named_slots()
                                .and_then(|slots| heap_view.object_slots(slots))
                                .and_then(|slots| slots.get(offset as usize).copied()),
                        };
                        if let Some(value) = fast_value {
                            if let Some(slot) = feedback_slot {
                                self.record_named_property_fast_hit(frame.code(), slot);
                            }
                            self.register_stack[target_index] = value;
                            advance_dispatch_frame(frame, instruction_len);
                            return Ok(());
                        }
                    }
                }
            }
            // Phase 3f polymorphic OwnData fast path. Walks the inline
            // [NamedPropertyHandler; POLY_LIMIT] sidecar for a shape match
            // before falling to the proto-fast / slow chain below. This is
            // the 2..POLY_LIMIT cached-shape equivalent of the Phase 3a
            // monomorphic check above — same packed-handler decode, but
            // chosen from a small fixed-size lookup rather than a single word.
            if let Some(value) = self.try_named_property_polymorphic_fast_load(
                agent,
                frame.code(),
                feedback_slot,
                object,
            ) {
                self.register_stack[target_index] = value;
                advance_dispatch_frame(frame, instruction_len);
                return Ok(());
            }
            // Phase 3e one-hop PrototypeData inline fast path. Class method
            // dispatch and Object.prototype lookups are PrototypeData with
            // dependency_count==2 — the OwnData handler above rejected them,
            // but this branch validates receiver shape+epoch and prototype
            // shape+epoch in straight-line code before reading the cached
            // slot off the prototype. Multi-hop PrototypeData and any other
            // shape still fall through to the slow chain below.
            if let Some(value) =
                self.try_named_property_proto_fast_load(agent, frame.code(), feedback_slot, object)
            {
                self.register_stack[target_index] = value;
                advance_dispatch_frame(frame, instruction_len);
                return Ok(());
            }
            // Slow path: entries[POLY_LIMIT..entry_count] / multi-hop
            // PrototypeData / megamorphic / miss.
            if let Some(value) = self.try_named_property_load_inline_cache_hit(
                agent,
                frame.code(),
                feedback_slot,
                object,
            ) {
                self.register_stack[target_index] = value;
                advance_dispatch_frame(frame, instruction_len);
                return Ok(());
            }
            let property_result =
                self.get_property_from_value(agent, host, registry, frame, receiver, key);
            let Some(value) =
                self.handle_dispatch_result(agent, frame_depth, frame, property_result)?
            else {
                return Ok(());
            };
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
            let property_result =
                self.get_property_from_value(agent, host, registry, frame, receiver, key);
            let Some(value) =
                self.handle_dispatch_result(agent, frame_depth, frame, property_result)?
            else {
                return Ok(());
            };
            value
        };
        self.register_stack[target_index] = value;
        advance_dispatch_frame(frame, instruction_len);
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper keeps dispatch state explicit while isolating the property opcode family"
    )]
    pub(in crate::vm) fn execute_set_named_property_opcode(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame_depth: usize,
        frame: &mut FrameRecord,
        instruction_len: u32,
        feedback_slot: Option<FeedbackSlotId>,
        opcode: Opcode,
        receiver_register: u16,
        value_register: u16,
        atom_operand: u16,
    ) -> VmResult<()> {
        let assignment = matches!(
            opcode,
            Opcode::AssignNamedProperty | Opcode::StrictAssignNamedProperty
        );
        let strict_assignment = matches!(opcode, Opcode::StrictAssignNamedProperty);
        let registers = frame.registers();
        let receiver = self.register_stack[absolute_register(registers, receiver_register)];
        let value = self.register_stack[absolute_register(registers, value_register)];
        let atom = self.read_atom_constant(frame.code(), u32::from(atom_operand))?;
        let key = PropertyKey::from_atom(atom);
        if let Some(object) = receiver.as_object_ref() {
            // Phase 3b inline IC fast path (store side). Mirrors the Phase 3a
            // load-side inlining: packed-handler load, shape compare, epoch
            // compare, writable check, then a barrier-aware store via
            // mut_store_value. Polymorphic / PrototypeData / megamorphic /
            // proxy / miss continue through the existing chain below.
            //
            // Encodes the hit decision into Option<Option<ValueStoreTarget>>:
            //   - Outer Some => take the fast path; stored = inner branch
            //   - Outer None => fall through to slow chain
            //   - Inner Some(target) => writable, do the store
            //   - Inner None        => read-only (writable bit clear), no store
            let fast_target = self
                .named_property_fast_handler(frame.code(), feedback_slot)
                .and_then(|(handler, cached_epoch)| {
                    let view = agent.heap().view();
                    let record = view.object_ref(object)?;
                    if record.shape() != handler.receiver_shape()
                        || record.last_invalidation_epoch().unwrap_or(0) != cached_epoch
                    {
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
            if let Some(target_opt) = fast_target {
                let stored = if let Some(target) = target_opt {
                    agent.with_heap_and_objects(|heap, _objects| {
                        let mut mutator = heap.mutator();
                        mutator.mut_store_value(target, value)
                    })
                } else {
                    // Non-writable own-data property: stored = false, no heap write.
                    // Matches store_to_named_property_cache → Ok(Some(false)).
                    false
                };
                if assignment {
                    let assignment_result = self.check_property_assignment_result(
                        agent,
                        frame,
                        stored,
                        strict_assignment,
                    );
                    let Some(()) =
                        self.handle_dispatch_result(agent, frame_depth, frame, assignment_result)?
                    else {
                        return Ok(());
                    };
                }
                self.record_feedback_slot(frame.code(), feedback_slot);
                advance_dispatch_frame(frame, instruction_len);
                return Ok(());
            }
            // Phase 3f polymorphic OwnData store fast path. Walks the inline
            // sidecar for a shape match across the same Option<Option<bool>>
            // encoding as the monomorphic branch above:
            //   Some(Some(true|false)) -> writable hit, store completed; the
            //     inner bool is the stored-ness reported back to the
            //     assignment-result check.
            //   Some(None)             -> non-writable hit, stored=false.
            //   None                   -> fall through to slow chain.
            if let Some(target_opt) = self.try_named_property_polymorphic_fast_store(
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
                    let Some(()) =
                        self.handle_dispatch_result(agent, frame_depth, frame, assignment_result)?
                    else {
                        return Ok(());
                    };
                }
                self.record_feedback_slot(frame.code(), feedback_slot);
                advance_dispatch_frame(frame, instruction_len);
                return Ok(());
            }
            // Slow path: entries[POLY_LIMIT..entry_count] / PrototypeData /
            // megamorphic / miss.
            if let Some(stored) = self.try_named_property_store_inline_cache(
                agent,
                frame.code(),
                feedback_slot,
                object,
                value,
            ) {
                if assignment {
                    let assignment_result = self.check_property_assignment_result(
                        agent,
                        frame,
                        stored,
                        strict_assignment,
                    );
                    let Some(()) =
                        self.handle_dispatch_result(agent, frame_depth, frame, assignment_result)?
                    else {
                        return Ok(());
                    };
                }
                self.record_feedback_slot(frame.code(), feedback_slot);
                advance_dispatch_frame(frame, instruction_len);
                return Ok(());
            }
            let set_result = if Self::prototype_chain_has_proxy(agent, object) {
                self.set_property_on_value(agent, host, registry, frame, receiver, key, value)
            } else {
                let set_result =
                    object::ordinary_set(agent, object, key, value, AllocationLifetime::Default)
                        .map_err(VmError::Abrupt);
                match set_result {
                    Ok(result) => Ok(result),
                    Err(VmError::Abrupt(_)) => self
                        .set_property_on_value(agent, host, registry, frame, receiver, key, value),
                    Err(error) => Err(error),
                }
            };
            let Some(stored) =
                self.handle_dispatch_result(agent, frame_depth, frame, set_result)?
            else {
                return Ok(());
            };
            if assignment {
                let assignment_result =
                    self.check_property_assignment_result(agent, frame, stored, strict_assignment);
                let Some(()) =
                    self.handle_dispatch_result(agent, frame_depth, frame, assignment_result)?
                else {
                    return Ok(());
                };
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
            let store_result =
                self.set_property_on_value(agent, host, registry, frame, receiver, key, value);
            let Some(stored) =
                self.handle_dispatch_result(agent, frame_depth, frame, store_result)?
            else {
                return Ok(());
            };
            if assignment {
                let assignment_result =
                    self.check_property_assignment_result(agent, frame, stored, strict_assignment);
                let Some(()) =
                    self.handle_dispatch_result(agent, frame_depth, frame, assignment_result)?
                else {
                    return Ok(());
                };
            }
        }
        advance_dispatch_frame(frame, instruction_len);
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
        frame_depth: usize,
        frame: &mut FrameRecord,
        instruction_len: u32,
        object_register: u16,
        value_register: u16,
        atom_operand: u16,
    ) -> VmResult<()> {
        let object = self.object_register(frame, object_register)?;
        let value = self.read_register(frame.registers(), value_register);
        let key =
            PropertyKey::from_atom(self.read_atom_constant(frame.code(), u32::from(atom_operand))?);
        self.define_data_property(
            agent,
            host,
            registry,
            frame_depth,
            frame,
            object,
            key,
            value,
        )?;
        advance_dispatch_frame(frame, instruction_len);
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
        frame_depth: usize,
        frame: &mut FrameRecord,
        instruction_len: u32,
        feedback_slot: Option<FeedbackSlotId>,
        target: u16,
        receiver_register: u16,
        key_register: u16,
    ) -> VmResult<()> {
        let receiver = self.read_register(frame.registers(), receiver_register);
        let key_value = self.read_register(frame.registers(), key_register);
        let coercible_result = Self::check_object_coercible(agent, receiver);
        let Some(()) = self.handle_dispatch_result(agent, frame_depth, frame, coercible_result)?
        else {
            return Ok(());
        };
        if let Some(object) = receiver.as_object_ref()
            && let Some(index) = key_value
                .as_smi()
                .and_then(|index| u32::try_from(index).ok())
        {
            // Phase 3d dense-index inline IC fast path (SMI key).
            if let Some(value) =
                self.try_keyed_dense_fast_load(agent, frame.code(), feedback_slot, object, index)
            {
                self.write_register(frame.registers(), target, value);
                advance_dispatch_frame(frame, instruction_len);
                return Ok(());
            }
            // Phase 3f polymorphic dense-index OwnData fast path.
            if let Some(value) = self.try_keyed_dense_polymorphic_fast_load(
                agent,
                frame.code(),
                feedback_slot,
                object,
                index,
            ) {
                self.write_register(frame.registers(), target, value);
                advance_dispatch_frame(frame, instruction_len);
                return Ok(());
            }
            // Slow path: dense_entries[POLY_LIMIT..] / Generic / megamorphic / miss.
            if let Some(value) = self.try_keyed_dense_index_load_inline_cache_hit(
                agent,
                frame.code(),
                feedback_slot,
                object,
                index,
            ) {
                self.write_register(frame.registers(), target, value);
                advance_dispatch_frame(frame, instruction_len);
                return Ok(());
            }
            let value = if let Some(result) = self.mapped_arguments_get(agent, object, index) {
                let Some(value) = self.handle_dispatch_result(agent, frame_depth, frame, result)?
                else {
                    return Ok(());
                };
                Some(value)
            } else if let Some(value) = Self::try_fast_typed_array_index_value(agent, object, index)
            {
                Some(value)
            } else {
                Self::try_fast_own_index_value(agent, object, index)?
            };
            if let Some(value) = value {
                self.observe_keyed_index_access(agent, frame.code(), feedback_slot, object, index);
                self.write_register(frame.registers(), target, value);
                advance_dispatch_frame(frame, instruction_len);
                return Ok(());
            }
        }
        let key_result = self.property_key_from_value(agent, host, registry, frame, key_value);
        let Some(key) = self.handle_dispatch_result(agent, frame_depth, frame, key_result)? else {
            return Ok(());
        };
        let value = if let Some(object) = receiver.as_object_ref() {
            if let Some(index) = key.as_index() {
                // Phase 3d dense-index inline IC fast path (post-coercion index).
                if let Some(value) = self.try_keyed_dense_fast_load(
                    agent,
                    frame.code(),
                    feedback_slot,
                    object,
                    index,
                ) {
                    self.write_register(frame.registers(), target, value);
                    advance_dispatch_frame(frame, instruction_len);
                    return Ok(());
                }
                // Phase 3f polymorphic dense-index OwnData fast path.
                if let Some(value) = self.try_keyed_dense_polymorphic_fast_load(
                    agent,
                    frame.code(),
                    feedback_slot,
                    object,
                    index,
                ) {
                    self.write_register(frame.registers(), target, value);
                    advance_dispatch_frame(frame, instruction_len);
                    return Ok(());
                }
                // Slow path: dense_entries[POLY_LIMIT..] / Generic / megamorphic / miss.
                if let Some(value) = self.try_keyed_dense_index_load_inline_cache_hit(
                    agent,
                    frame.code(),
                    feedback_slot,
                    object,
                    index,
                ) {
                    self.write_register(frame.registers(), target, value);
                    advance_dispatch_frame(frame, instruction_len);
                    return Ok(());
                }
                let value = if let Some(result) = self.mapped_arguments_get(agent, object, index) {
                    let Some(value) =
                        self.handle_dispatch_result(agent, frame_depth, frame, result)?
                    else {
                        return Ok(());
                    };
                    value
                } else if let Some(value) =
                    Self::try_fast_typed_array_index_value(agent, object, index)
                {
                    value
                } else if let Some(value) = Self::try_fast_own_index_value(agent, object, index)? {
                    value
                } else {
                    let property_result =
                        self.get_property_from_value(agent, host, registry, frame, receiver, key);
                    let Some(value) =
                        self.handle_dispatch_result(agent, frame_depth, frame, property_result)?
                    else {
                        return Ok(());
                    };
                    value
                };
                self.observe_keyed_index_access(agent, frame.code(), feedback_slot, object, index);
                value
            } else if let Some(atom) = key.as_atom() {
                // Phase 3d named-keyed (atom) inline IC fast path.
                if let Some(value) =
                    self.try_keyed_named_fast_load(agent, frame.code(), feedback_slot, object, atom)
                {
                    self.write_register(frame.registers(), target, value);
                    advance_dispatch_frame(frame, instruction_len);
                    return Ok(());
                }
                // Phase 3f polymorphic named-keyed (atom) OwnData fast path.
                // Same inline-sidecar walk as the non-keyed variant, but with
                // an extra atom-equality check inside the lookup.
                if let Some(value) = self.try_keyed_named_polymorphic_fast_load(
                    agent,
                    frame.code(),
                    feedback_slot,
                    object,
                    atom,
                ) {
                    self.write_register(frame.registers(), target, value);
                    advance_dispatch_frame(frame, instruction_len);
                    return Ok(());
                }
                // Phase 3e named-keyed (atom) one-hop PrototypeData fast path.
                if let Some(value) = self.try_keyed_named_proto_fast_load(
                    agent,
                    frame.code(),
                    feedback_slot,
                    object,
                    atom,
                ) {
                    self.write_register(frame.registers(), target, value);
                    advance_dispatch_frame(frame, instruction_len);
                    return Ok(());
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
                    self.write_register(frame.registers(), target, value);
                    advance_dispatch_frame(frame, instruction_len);
                    return Ok(());
                }
                let property_result =
                    self.get_property_from_value(agent, host, registry, frame, receiver, key);
                let Some(value) =
                    self.handle_dispatch_result(agent, frame_depth, frame, property_result)?
                else {
                    return Ok(());
                };
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
                let property_result =
                    self.get_property_from_value(agent, host, registry, frame, receiver, key);
                let Some(value) =
                    self.handle_dispatch_result(agent, frame_depth, frame, property_result)?
                else {
                    return Ok(());
                };
                self.observe_keyed_generic_slow_path(frame.code(), feedback_slot);
                value
            }
        } else {
            let property_result =
                self.get_property_from_value(agent, host, registry, frame, receiver, key);
            let Some(value) =
                self.handle_dispatch_result(agent, frame_depth, frame, property_result)?
            else {
                return Ok(());
            };
            value
        };
        self.write_register(frame.registers(), target, value);
        advance_dispatch_frame(frame, instruction_len);
        Ok(())
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
        frame_depth: usize,
        frame: &mut FrameRecord,
        instruction_len: u32,
        feedback_slot: Option<FeedbackSlotId>,
        opcode: Opcode,
        receiver_register: u16,
        value_register: u16,
        key_register: u16,
    ) -> VmResult<()> {
        let assignment = matches!(
            opcode,
            Opcode::AssignKeyedProperty | Opcode::StrictAssignKeyedProperty
        );
        let strict_assignment = matches!(opcode, Opcode::StrictAssignKeyedProperty);
        let receiver = self.read_register(frame.registers(), receiver_register);
        let value = self.read_register(frame.registers(), value_register);
        let key_value = self.read_register(frame.registers(), key_register);
        let coercible_result = Self::check_object_coercible(agent, receiver);
        let Some(()) = self.handle_dispatch_result(agent, frame_depth, frame, coercible_result)?
        else {
            return Ok(());
        };
        if let Some(object) = receiver.as_object_ref()
            && let Some(index) = key_value
                .as_smi()
                .and_then(|index| u32::try_from(index).ok())
        {
            // Phase 3d dense-index inline IC fast path (SMI key, store side).
            if let Some(stored) = self.try_keyed_dense_fast_store(
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
                    let Some(()) =
                        self.handle_dispatch_result(agent, frame_depth, frame, assignment_result)?
                    else {
                        return Ok(());
                    };
                }
                advance_dispatch_frame(frame, instruction_len);
                return Ok(());
            }
            // Phase 3f polymorphic dense-index OwnData store fast path.
            if let Some(stored) = self.try_keyed_dense_polymorphic_fast_store(
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
                    let Some(()) =
                        self.handle_dispatch_result(agent, frame_depth, frame, assignment_result)?
                    else {
                        return Ok(());
                    };
                }
                advance_dispatch_frame(frame, instruction_len);
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
                    let Some(()) =
                        self.handle_dispatch_result(agent, frame_depth, frame, assignment_result)?
                    else {
                        return Ok(());
                    };
                }
                advance_dispatch_frame(frame, instruction_len);
                return Ok(());
            }
            let mut used_index_fast_path = false;
            let stored = if let Some(result) =
                self.mapped_arguments_set(agent, object, index, value)
            {
                let Some(()) = self.handle_dispatch_result(agent, frame_depth, frame, result)?
                else {
                    return Ok(());
                };
                Some(true)
            } else {
                let fast_result = self.try_fast_set_typed_array_index(
                    agent, host, registry, frame, object, index, value,
                );
                let Some(fast_result) =
                    self.handle_dispatch_result(agent, frame_depth, frame, fast_result)?
                else {
                    return Ok(());
                };
                if let Some(stored) = fast_result {
                    used_index_fast_path = true;
                    Some(stored)
                } else {
                    let fast_result =
                        Self::try_fast_set_engine_array_index(agent, object, index, value);
                    let Some(fast_result) =
                        self.handle_dispatch_result(agent, frame_depth, frame, fast_result)?
                    else {
                        return Ok(());
                    };
                    if let Some(stored) = fast_result {
                        used_index_fast_path = true;
                        Some(stored)
                    } else {
                        let fast_result = Self::try_fast_set_ordinary_index_data_property(
                            agent, object, index, value,
                        );
                        let Some(fast_result) =
                            self.handle_dispatch_result(agent, frame_depth, frame, fast_result)?
                        else {
                            return Ok(());
                        };
                        fast_result.inspect(|_| {
                            used_index_fast_path = true;
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
                    let Some(()) =
                        self.handle_dispatch_result(agent, frame_depth, frame, assignment_result)?
                    else {
                        return Ok(());
                    };
                }
                if !used_index_fast_path {
                    Self::sync_engine_array_length(agent, object)?;
                }
                self.observe_keyed_index_access(agent, frame.code(), feedback_slot, object, index);
                advance_dispatch_frame(frame, instruction_len);
                return Ok(());
            }
        }
        let key_result = self.property_key_from_value(agent, host, registry, frame, key_value);
        let Some(key) = self.handle_dispatch_result(agent, frame_depth, frame, key_result)? else {
            return Ok(());
        };
        if let Some(object) = receiver.as_object_ref() {
            if let Some(index) = key.as_index() {
                // Phase 3d dense-index inline IC fast path (post-coercion index, store side).
                if let Some(stored) = self.try_keyed_dense_fast_store(
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
                        let Some(()) = self.handle_dispatch_result(
                            agent,
                            frame_depth,
                            frame,
                            assignment_result,
                        )?
                        else {
                            return Ok(());
                        };
                    }
                    advance_dispatch_frame(frame, instruction_len);
                    return Ok(());
                }
                // Phase 3f polymorphic dense-index OwnData store fast path.
                if let Some(stored) = self.try_keyed_dense_polymorphic_fast_store(
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
                        let Some(()) = self.handle_dispatch_result(
                            agent,
                            frame_depth,
                            frame,
                            assignment_result,
                        )?
                        else {
                            return Ok(());
                        };
                    }
                    advance_dispatch_frame(frame, instruction_len);
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
                        let Some(()) = self.handle_dispatch_result(
                            agent,
                            frame_depth,
                            frame,
                            assignment_result,
                        )?
                        else {
                            return Ok(());
                        };
                    }
                    advance_dispatch_frame(frame, instruction_len);
                    return Ok(());
                }
                let mut used_index_fast_path = false;
                let stored = if let Some(result) =
                    self.mapped_arguments_set(agent, object, index, value)
                {
                    let Some(()) =
                        self.handle_dispatch_result(agent, frame_depth, frame, result)?
                    else {
                        return Ok(());
                    };
                    true
                } else {
                    let fast_result = self.try_fast_set_typed_array_index(
                        agent, host, registry, frame, object, index, value,
                    );
                    let Some(fast_result) =
                        self.handle_dispatch_result(agent, frame_depth, frame, fast_result)?
                    else {
                        return Ok(());
                    };
                    if let Some(stored) = fast_result {
                        used_index_fast_path = true;
                        stored
                    } else {
                        let fast_result =
                            Self::try_fast_set_engine_array_index(agent, object, index, value);
                        let Some(fast_result) =
                            self.handle_dispatch_result(agent, frame_depth, frame, fast_result)?
                        else {
                            return Ok(());
                        };
                        if let Some(stored) = fast_result {
                            used_index_fast_path = true;
                            stored
                        } else {
                            let fast_result = Self::try_fast_set_ordinary_index_data_property(
                                agent, object, index, value,
                            );
                            let Some(fast_result) = self.handle_dispatch_result(
                                agent,
                                frame_depth,
                                frame,
                                fast_result,
                            )?
                            else {
                                return Ok(());
                            };
                            if let Some(stored) = fast_result {
                                used_index_fast_path = true;
                                stored
                            } else {
                                let set_result = self.set_property_on_value(
                                    agent, host, registry, frame, receiver, key, value,
                                );
                                let Some(stored) = self.handle_dispatch_result(
                                    agent,
                                    frame_depth,
                                    frame,
                                    set_result,
                                )?
                                else {
                                    return Ok(());
                                };
                                stored
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
                    let Some(()) =
                        self.handle_dispatch_result(agent, frame_depth, frame, assignment_result)?
                    else {
                        return Ok(());
                    };
                }
                if !used_index_fast_path {
                    Self::sync_engine_array_length(agent, object)?;
                }
                self.observe_keyed_index_access(agent, frame.code(), feedback_slot, object, index);
            } else if let Some(atom) = key.as_atom() {
                // Phase 3d named-keyed (atom) inline IC fast path (store side).
                if let Some(stored) = self.try_keyed_named_fast_store(
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
                        let Some(()) = self.handle_dispatch_result(
                            agent,
                            frame_depth,
                            frame,
                            assignment_result,
                        )?
                        else {
                            return Ok(());
                        };
                    }
                    advance_dispatch_frame(frame, instruction_len);
                    return Ok(());
                }
                // Phase 3f polymorphic named-keyed (atom) OwnData store fast
                // path. Walks the inline sidecar for a shape+atom match
                // before falling to the slow chain.
                if let Some(stored) = self.try_keyed_named_polymorphic_fast_store(
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
                        let Some(()) = self.handle_dispatch_result(
                            agent,
                            frame_depth,
                            frame,
                            assignment_result,
                        )?
                        else {
                            return Ok(());
                        };
                    }
                    advance_dispatch_frame(frame, instruction_len);
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
                        let Some(()) = self.handle_dispatch_result(
                            agent,
                            frame_depth,
                            frame,
                            assignment_result,
                        )?
                        else {
                            return Ok(());
                        };
                    }
                    self.record_feedback_slot(frame.code(), feedback_slot);
                    advance_dispatch_frame(frame, instruction_len);
                    return Ok(());
                }
                let set_result =
                    self.set_property_on_value(agent, host, registry, frame, receiver, key, value);
                let Some(stored) =
                    self.handle_dispatch_result(agent, frame_depth, frame, set_result)?
                else {
                    return Ok(());
                };
                if assignment {
                    let assignment_result = self.check_property_assignment_result(
                        agent,
                        frame,
                        stored,
                        strict_assignment,
                    );
                    let Some(()) =
                        self.handle_dispatch_result(agent, frame_depth, frame, assignment_result)?
                    else {
                        return Ok(());
                    };
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
                let set_result =
                    self.set_property_on_value(agent, host, registry, frame, receiver, key, value);
                let Some(stored) =
                    self.handle_dispatch_result(agent, frame_depth, frame, set_result)?
                else {
                    return Ok(());
                };
                if assignment {
                    let assignment_result = self.check_property_assignment_result(
                        agent,
                        frame,
                        stored,
                        strict_assignment,
                    );
                    let Some(()) =
                        self.handle_dispatch_result(agent, frame_depth, frame, assignment_result)?
                    else {
                        return Ok(());
                    };
                }
                self.observe_keyed_generic_slow_path(frame.code(), feedback_slot);
            }
        } else {
            let store_result =
                self.set_property_on_value(agent, host, registry, frame, receiver, key, value);
            let Some(stored) =
                self.handle_dispatch_result(agent, frame_depth, frame, store_result)?
            else {
                return Ok(());
            };
            if assignment {
                let assignment_result =
                    self.check_property_assignment_result(agent, frame, stored, strict_assignment);
                let Some(()) =
                    self.handle_dispatch_result(agent, frame_depth, frame, assignment_result)?
                else {
                    return Ok(());
                };
            }
        }
        advance_dispatch_frame(frame, instruction_len);
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
        frame_depth: usize,
        frame: &mut FrameRecord,
        instruction_len: u32,
        object_register: u16,
        value_register: u16,
        key_register: u16,
    ) -> VmResult<()> {
        let object = self.object_register(frame, object_register)?;
        let value = self.read_register(frame.registers(), value_register);
        let key_value = self.read_register(frame.registers(), key_register);
        let key_result = self.property_key_from_value(agent, host, registry, frame, key_value);
        let Some(key) = self.handle_dispatch_result(agent, frame_depth, frame, key_result)? else {
            return Ok(());
        };
        self.define_data_property(
            agent,
            host,
            registry,
            frame_depth,
            frame,
            object,
            key,
            value,
        )?;
        if key.as_index().is_some() {
            Self::sync_engine_array_length(agent, object)?;
        }
        advance_dispatch_frame(frame, instruction_len);
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and dispatch state explicitly at call sites"
    )]
    pub(in crate::vm) fn execute_to_property_key_opcode(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame_depth: usize,
        frame: &mut FrameRecord,
        instruction_len: u32,
        target: u16,
        key_register: u16,
    ) -> VmResult<()> {
        let key_value = self.read_register(frame.registers(), key_register);
        let key_result = self.property_key_from_value(agent, host, registry, frame, key_value);
        let Some(key) = self.handle_dispatch_result(agent, frame_depth, frame, key_result)? else {
            return Ok(());
        };
        let value = self.property_key_to_enumeration_value(agent, key);
        self.write_register(frame.registers(), target, value);
        advance_dispatch_frame(frame, instruction_len);
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper keeps dispatch state explicit while isolating the property opcode family"
    )]
    pub(in crate::vm) fn execute_delete_property_opcode(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame_depth: usize,
        frame: &mut FrameRecord,
        instruction_len: u32,
        target: u16,
        receiver_register: u16,
        key_register: u16,
    ) -> VmResult<()> {
        let receiver = self.read_register(frame.registers(), receiver_register);
        let key_value = self.read_register(frame.registers(), key_register);
        let coercible_result = Self::check_object_coercible(agent, receiver);
        let Some(()) = self.handle_dispatch_result(agent, frame_depth, frame, coercible_result)?
        else {
            return Ok(());
        };
        let key_result = self.property_key_from_value(agent, host, registry, frame, key_value);
        let Some(key) = self.handle_dispatch_result(agent, frame_depth, frame, key_result)? else {
            return Ok(());
        };
        let delete_result =
            self.delete_property_from_value(agent, host, registry, frame, receiver, key);
        let Some(deleted) =
            self.handle_dispatch_result(agent, frame_depth, frame, delete_result)?
        else {
            return Ok(());
        };
        if !deleted && self.frame_is_strict(frame) {
            let type_error = Err(VmError::Abrupt(errors::throw_type_error(agent)));
            let Some(()) = self.handle_dispatch_result(agent, frame_depth, frame, type_error)?
            else {
                return Ok(());
            };
        }
        self.write_register(frame.registers(), target, Value::from_bool(deleted));
        advance_dispatch_frame(frame, instruction_len);
        Ok(())
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
        frame_depth: usize,
        frame: &mut FrameRecord,
        instruction_len: u32,
        target_register: u16,
        source_register: u16,
        excluded_register: u16,
    ) -> VmResult<()> {
        let target = self.object_register(frame, target_register)?;
        let source = self.read_register(frame.registers(), source_register);
        let excluded_keys = self.read_register(frame.registers(), excluded_register);
        let copy_result =
            self.copy_data_properties(agent, host, registry, frame, target, source, excluded_keys);
        let Some(()) = self.handle_dispatch_result(agent, frame_depth, frame, copy_result)? else {
            return Ok(());
        };
        advance_dispatch_frame(frame, instruction_len);
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
        frame_depth: usize,
        frame: &mut FrameRecord,
        instruction_len: u32,
        receiver_register: u16,
        value_register: u16,
        index_operand: u16,
    ) -> VmResult<()> {
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
                frame,
                receiver,
                PropertyKey::Index(u32::from(index_operand)),
                value,
            );
            let Some(_) = self.handle_dispatch_result(agent, frame_depth, frame, store_result)?
            else {
                return Ok(());
            };
        }
        advance_dispatch_frame(frame, instruction_len);
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper keeps dispatch state explicit while isolating the property opcode family"
    )]
    pub(in crate::vm) fn execute_load_dense_element_opcode(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame_depth: usize,
        frame: &mut FrameRecord,
        instruction_len: u32,
        target: u16,
        receiver_register: u16,
        index_operand: u16,
    ) -> VmResult<()> {
        let receiver = self.read_register(frame.registers(), receiver_register);
        let value = if let Some(object) = receiver.as_object_ref() {
            if let Some(result) = self.mapped_arguments_get(agent, object, u32::from(index_operand))
            {
                let Some(value) = self.handle_dispatch_result(agent, frame_depth, frame, result)?
                else {
                    return Ok(());
                };
                value
            } else if let Some(value) =
                Self::try_fast_own_index_value(agent, object, u32::from(index_operand))?
            {
                value
            } else if Self::prototype_chain_has_proxy(agent, object) {
                let property_result = self.get_property_from_value(
                    agent,
                    host,
                    registry,
                    frame,
                    receiver,
                    PropertyKey::Index(u32::from(index_operand)),
                );
                let Some(value) =
                    self.handle_dispatch_result(agent, frame_depth, frame, property_result)?
                else {
                    return Ok(());
                };
                value
            } else {
                let element = object::ordinary_get(
                    agent,
                    object,
                    PropertyKey::Index(u32::from(index_operand)),
                )
                .map_err(VmError::Abrupt);
                let Some(value) =
                    self.handle_dispatch_result(agent, frame_depth, frame, element)?
                else {
                    return Ok(());
                };
                value
            }
        } else {
            let property_result = self.get_property_from_value(
                agent,
                host,
                registry,
                frame,
                receiver,
                PropertyKey::Index(u32::from(index_operand)),
            );
            let Some(value) =
                self.handle_dispatch_result(agent, frame_depth, frame, property_result)?
            else {
                return Ok(());
            };
            value
        };
        self.write_register(frame.registers(), target, value);
        advance_dispatch_frame(frame, instruction_len);
        Ok(())
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
        frame_depth: usize,
        frame: &mut FrameRecord,
        object: lyng_types::ObjectRef,
        key: PropertyKey,
        value: Value,
    ) -> VmResult<()> {
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
                frame,
            },
            object,
            key,
            descriptor,
            AllocationLifetime::Default,
        );
        let Some(created) =
            self.handle_dispatch_result(agent, frame_depth, frame, define_result)?
        else {
            return Ok(());
        };
        if !created {
            let type_error = Err(VmError::Abrupt(errors::throw_type_error(agent)));
            let Some(()) = self.handle_dispatch_result(agent, frame_depth, frame, type_error)?
            else {
                return Ok(());
            };
        }
        Ok(())
    }

    fn check_property_assignment_result(
        &self,
        agent: &mut Agent,
        frame: &FrameRecord,
        stored: bool,
        strict_override: bool,
    ) -> VmResult<()> {
        if !stored && (strict_override || self.frame_is_strict(frame)) {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        Ok(())
    }

    /// Phase 3d dense-keyed load fast path. Returns the cached element
    /// value on a monomorphic-DenseIndex hit, falling through to `None`
    /// on shape/flags miss, hole, or any cache state other than monomorphic.
    /// Records tier bookkeeping on hit. Mirrors the trailing two lines of
    /// `try_keyed_dense_index_load_inline_cache_hit`.
    #[inline(always)]
    fn try_keyed_dense_fast_load(
        &mut self,
        agent: &Agent,
        code: CodeRef,
        feedback_slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
        index: u32,
    ) -> Option<Value> {
        let handler = self.keyed_property_dense_fast_handler(code, feedback_slot)?;
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
            self.record_named_property_fast_hit(code, slot);
        }
        Some(value)
    }

    /// Phase 3d dense-keyed store fast path. Returns `Some(true)` on a
    /// successful barrier-aware write, `None` on miss / hole / shape
    /// mismatch / out-of-bounds. Mirrors the guards in
    /// `KeyedPropertyFeedback::try_dense_index_store`.
    #[inline(always)]
    fn try_keyed_dense_fast_store(
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
        let handler = self.keyed_property_dense_fast_handler(code, feedback_slot)?;
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
            self.record_named_property_fast_hit(code, slot);
        }
        Some(true)
    }

    /// Phase 3d named-keyed (atom) load fast path. Returns the cached
    /// slot value on a monomorphic-NamedAtom hit, `None` on miss.
    /// Records the slot via `record_feedback_slot` (matching the slow
    /// chain's bookkeeping on atom hit).
    #[inline(always)]
    fn try_keyed_named_fast_load(
        &mut self,
        agent: &Agent,
        code: CodeRef,
        feedback_slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
        atom: AtomId,
    ) -> Option<Value> {
        let (handler, cached_epoch) =
            self.keyed_property_named_fast_handler(code, feedback_slot, atom)?;
        let view = agent.heap().view();
        let record = view.object_ref(receiver)?;
        if record.shape() != handler.receiver_shape()
            || record.last_invalidation_epoch().unwrap_or(0) != cached_epoch
        {
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

    /// Phase 3e named-property (non-keyed) one-hop PrototypeData fast
    /// path. Returns the cached slot value from the prototype holder on a
    /// monomorphic + one-hop PrototypeData hit, `None` on any miss
    /// (shape/epoch mismatch on receiver or prototype, missing prototype,
    /// etc.). Bypasses the slow chain on the dominant class-method-
    /// dispatch / `Object.prototype` lookup pattern.
    #[inline(always)]
    pub(in crate::vm) fn try_named_property_proto_fast_load(
        &mut self,
        agent: &Agent,
        code: CodeRef,
        feedback_slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
    ) -> Option<Value> {
        let (handler, receiver_epoch, prototype_epoch) =
            self.named_property_proto_fast_handler(code, feedback_slot)?;
        let view = agent.heap().view();
        let record = view.object_ref(receiver)?;
        if record.shape() != handler.receiver_shape()
            || record.last_invalidation_epoch().unwrap_or(0) != receiver_epoch
        {
            return None;
        }
        let prototype_id = record.prototype()?;
        let prototype_record = view.object_ref(prototype_id)?;
        if prototype_record.shape() != handler.prototype_shape()
            || prototype_record.last_invalidation_epoch().unwrap_or(0) != prototype_epoch
        {
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
            self.record_named_property_fast_hit(code, slot);
        }
        Some(value)
    }

    /// Phase 3e named-keyed (atom) one-hop PrototypeData fast path.
    /// Returns the cached slot value from the prototype holder on a
    /// monomorphic + NamedAtom + one-hop PrototypeData hit, `None` on
    /// miss (shape/epoch mismatch on receiver or prototype, missing
    /// prototype, etc.). Records the slot via `record_feedback_slot`
    /// matching the OwnData sibling.
    #[inline(always)]
    fn try_keyed_named_proto_fast_load(
        &mut self,
        agent: &Agent,
        code: CodeRef,
        feedback_slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
        atom: AtomId,
    ) -> Option<Value> {
        let (handler, receiver_epoch, prototype_epoch) =
            self.keyed_property_named_proto_fast_handler(code, feedback_slot, atom)?;
        let view = agent.heap().view();
        let record = view.object_ref(receiver)?;
        if record.shape() != handler.receiver_shape()
            || record.last_invalidation_epoch().unwrap_or(0) != receiver_epoch
        {
            return None;
        }
        let prototype_id = record.prototype()?;
        let prototype_record = view.object_ref(prototype_id)?;
        if prototype_record.shape() != handler.prototype_shape()
            || prototype_record.last_invalidation_epoch().unwrap_or(0) != prototype_epoch
        {
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

    /// Phase 3d named-keyed (atom) store fast path. Returns `Some(stored)`
    /// on a monomorphic-NamedAtom hit, `None` on miss. Non-writable
    /// hits return `Some(false)` (matching slow-chain semantics — the
    /// caller's `assignment` logic converts this to a TypeError in
    /// strict mode).
    #[inline(always)]
    fn try_keyed_named_fast_store(
        &mut self,
        agent: &mut Agent,
        code: CodeRef,
        feedback_slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
        atom: AtomId,
        value: Value,
    ) -> Option<bool> {
        let (handler, cached_epoch) =
            self.keyed_property_named_fast_handler(code, feedback_slot, atom)?;
        let (named_slots, shape_match) = {
            let view = agent.heap().view();
            let record = view.object_ref(receiver)?;
            let shape_match = record.shape() == handler.receiver_shape()
                && record.last_invalidation_epoch().unwrap_or(0) == cached_epoch;
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

    /// Phase 3f named-property (non-keyed) polymorphic OwnData fast path.
    /// Walks the receiver shape through the inline polymorphic sidecar
    /// (up to `POLY_LIMIT` cached shapes), returning the slot value on
    /// hit. Returns `None` on any miss — shape not in the sidecar,
    /// epoch mismatch, or unloadable slot — so the caller can fall
    /// through to the proto-fast path or the slow chain. The receiver
    /// shape is loaded once and reused for the inline walk + slot read.
    #[inline(always)]
    pub(in crate::vm) fn try_named_property_polymorphic_fast_load(
        &mut self,
        agent: &Agent,
        code: CodeRef,
        feedback_slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
    ) -> Option<Value> {
        let view = agent.heap().view();
        let record = view.object_ref(receiver)?;
        let shape = record.shape()?;
        let (handler, cached_epoch) =
            self.named_property_polymorphic_fast_handler(code, feedback_slot, shape)?;
        if record.last_invalidation_epoch().unwrap_or(0) != cached_epoch {
            return None;
        }
        let value = match handler.slot_location() {
            SlotLocation::Inline(i) => record.inline_named_slot(i as usize)?,
            SlotLocation::OutOfLine(off) => view
                .object_slots(record.named_slots()?)?
                .get(off as usize)
                .copied()?,
        };
        if let Some(slot) = feedback_slot {
            self.record_named_property_fast_hit(code, slot);
        }
        Some(value)
    }

    /// Phase 3f named-property (non-keyed) polymorphic OwnData store fast
    /// path. Mirrors [`Self::try_named_property_polymorphic_fast_load`]
    /// for the Set / Assign / StrictAssign / StoreGlobal / AssignGlobal
    /// opcode family.
    ///
    /// Encodes the hit decision into `Option<Option<bool>>` to match the
    /// Phase 3a–3c store-side pattern:
    /// - `None` — fall through to the slow chain.
    /// - `Some(Some(true))` — writable hit, value stored.
    /// - `Some(Some(false))` — writable hit, but the heap write barrier
    ///   declined the store (e.g. immutable target).
    /// - `Some(None)` — non-writable hit; slow-chain analog returns
    ///   `Ok(Some(false))` and the caller handles the strict-mode error.
    #[inline(always)]
    pub(in crate::vm) fn try_named_property_polymorphic_fast_store(
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
            let (handler, cached_epoch) =
                self.named_property_polymorphic_fast_handler(code, feedback_slot, shape)?;
            if record.last_invalidation_epoch().unwrap_or(0) != cached_epoch {
                return None;
            }
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

    /// Phase 3f named-keyed (atom) polymorphic OwnData load fast path.
    /// Walks the keyed-atom polymorphic sidecar matching both the atom
    /// and the receiver shape. Mirrors
    /// [`Self::try_named_property_polymorphic_fast_load`] for the keyed
    /// IC family.
    #[inline(always)]
    fn try_keyed_named_polymorphic_fast_load(
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
        let (handler, cached_epoch) =
            self.keyed_property_named_polymorphic_fast_handler(code, feedback_slot, atom, shape)?;
        if record.last_invalidation_epoch().unwrap_or(0) != cached_epoch {
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

    /// Phase 3f dense-keyed polymorphic fast load. Walks the inline
    /// `[KeyedDenseIndexHandler; POLY_LIMIT]` sidecar for a shape+flags
    /// match before falling to the slow chain. Mirrors
    /// [`Self::try_keyed_dense_fast_load`] for shapes 2..POLY_LIMIT.
    #[inline(always)]
    fn try_keyed_dense_polymorphic_fast_load(
        &mut self,
        agent: &Agent,
        code: CodeRef,
        feedback_slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
        index: u32,
    ) -> Option<Value> {
        let handlers = self.keyed_property_dense_polymorphic_fast_handler(code, feedback_slot)?;
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
            self.record_named_property_fast_hit(code, slot);
        }
        Some(value)
    }

    /// Phase 3f dense-keyed polymorphic fast store. Mirrors
    /// [`Self::try_keyed_dense_fast_store`] for shapes 2..POLY_LIMIT.
    #[inline(always)]
    fn try_keyed_dense_polymorphic_fast_store(
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
            let handlers =
                self.keyed_property_dense_polymorphic_fast_handler(code, feedback_slot)?;
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
            self.record_named_property_fast_hit(code, slot);
        }
        Some(true)
    }

    /// Phase 3f named-keyed (atom) polymorphic OwnData store fast path.
    /// Mirrors [`Self::try_keyed_named_fast_store`] for shapes
    /// 2..POLY_LIMIT.
    #[inline(always)]
    fn try_keyed_named_polymorphic_fast_store(
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
            let (handler, cached_epoch) = self.keyed_property_named_polymorphic_fast_handler(
                code,
                feedback_slot,
                atom,
                shape,
            )?;
            if record.last_invalidation_epoch().unwrap_or(0) != cached_epoch {
                return None;
            }
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
