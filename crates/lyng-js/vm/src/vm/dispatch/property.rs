use crate::error::VmResult;
use crate::vm::property_access::VmProxyBridge;
use crate::{FrameRecord, Vm, VmError};
use lyng_js_bytecode::Opcode;
use lyng_js_env::Agent;
use lyng_js_gc::AllocationLifetime;
use lyng_js_host::HostHooks;
use lyng_js_objects::{NamedPropertyCachePurpose, NativeFunctionRegistry};
use lyng_js_ops::{errors, object};
use lyng_js_types::{FeedbackSlotId, PropertyDescriptor, PropertyKey, Value};

impl Vm {
    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper keeps dispatch state explicit while isolating the property opcode family"
    )]
    pub(super) fn execute_in_opcode(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        target: u16,
        key_register: u16,
        receiver_register: u16,
    ) -> VmResult<()> {
        let key_value = self.read_register(frame.registers(), key_register);
        let receiver = self.read_register(frame.registers(), receiver_register);
        let object_result = receiver
            .as_object_ref()
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)));
        let Some(object) = self.handle_vm_result(agent, object_result)? else {
            return Ok(());
        };
        let key_result = self.property_key_from_value(agent, host, registry, &frame, key_value);
        let Some(key) = self.handle_vm_result(agent, key_result)? else {
            return Ok(());
        };
        let has_property = {
            let mut bridge = VmProxyBridge {
                vm: self,
                agent,
                host,
                registry,
                frame: &frame,
            };
            object::has_property_in_context(&mut bridge, object, key)
        };
        let Some(has_property) = self.handle_vm_result(agent, has_property)? else {
            return Ok(());
        };
        self.write_register(frame.registers(), target, Value::from_bool(has_property));
        self.advance_instruction();
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper keeps dispatch state explicit while isolating the property opcode family"
    )]
    pub(super) fn execute_get_named_property_opcode(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        feedback_slot: Option<FeedbackSlotId>,
        target: u16,
        receiver_register: u16,
        atom_operand: u16,
    ) -> VmResult<()> {
        let receiver = self.read_register(frame.registers(), receiver_register);
        let atom = self.read_atom_constant(frame.code(), u32::from(atom_operand))?;
        let key = PropertyKey::from_atom(atom);
        let value = if let Some(object) = receiver.as_object_ref() {
            if let Some(value) = self.try_named_property_load_inline_cache_hit(
                agent,
                frame.code(),
                feedback_slot,
                object,
            ) {
                self.write_register(frame.registers(), target, value);
                self.advance_instruction();
                return Ok(());
            }
            let property_result =
                self.get_property_from_value(agent, host, registry, &frame, receiver, key);
            let Some(value) = self.handle_vm_result(agent, property_result)? else {
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
                self.get_property_from_value(agent, host, registry, &frame, receiver, key);
            let Some(value) = self.handle_vm_result(agent, property_result)? else {
                return Ok(());
            };
            value
        };
        self.write_register(frame.registers(), target, value);
        self.advance_instruction();
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper keeps dispatch state explicit while isolating the property opcode family"
    )]
    pub(super) fn execute_set_named_property_opcode(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
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
        let receiver = self.read_register(frame.registers(), receiver_register);
        let value = self.read_register(frame.registers(), value_register);
        let atom = self.read_atom_constant(frame.code(), u32::from(atom_operand))?;
        let key = PropertyKey::from_atom(atom);
        if let Some(object) = receiver.as_object_ref() {
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
                    let Some(()) = self.handle_vm_result(agent, assignment_result)? else {
                        return Ok(());
                    };
                }
                self.record_feedback_slot(frame.code(), feedback_slot);
                self.advance_instruction();
                return Ok(());
            }
            let set_result = if Self::prototype_chain_has_proxy(agent, object) {
                self.set_property_on_value(agent, host, registry, &frame, receiver, key, value)
            } else {
                let set_result =
                    object::ordinary_set(agent, object, key, value, AllocationLifetime::Default)
                        .map_err(VmError::Abrupt);
                match set_result {
                    Ok(result) => Ok(result),
                    Err(VmError::Abrupt(_)) => self
                        .set_property_on_value(agent, host, registry, &frame, receiver, key, value),
                    Err(error) => Err(error),
                }
            };
            let Some(stored) = self.handle_vm_result(agent, set_result)? else {
                return Ok(());
            };
            if assignment {
                let assignment_result =
                    self.check_property_assignment_result(agent, frame, stored, strict_assignment);
                let Some(()) = self.handle_vm_result(agent, assignment_result)? else {
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
                self.set_property_on_value(agent, host, registry, &frame, receiver, key, value);
            let Some(stored) = self.handle_vm_result(agent, store_result)? else {
                return Ok(());
            };
            if assignment {
                let assignment_result =
                    self.check_property_assignment_result(agent, frame, stored, strict_assignment);
                let Some(()) = self.handle_vm_result(agent, assignment_result)? else {
                    return Ok(());
                };
            }
        }
        self.advance_instruction();
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper keeps dispatch state explicit while isolating the property opcode family"
    )]
    pub(super) fn execute_define_named_property_opcode(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        object_register: u16,
        value_register: u16,
        atom_operand: u16,
    ) -> VmResult<()> {
        let object = self.object_register(&frame, object_register)?;
        let value = self.read_register(frame.registers(), value_register);
        let key =
            PropertyKey::from_atom(self.read_atom_constant(frame.code(), u32::from(atom_operand))?);
        self.define_data_property(agent, host, registry, frame, object, key, value)?;
        self.advance_instruction();
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        clippy::too_many_lines,
        reason = "VM helper keeps dispatch state explicit while isolating the property opcode family"
    )]
    pub(super) fn execute_get_keyed_property_opcode(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        feedback_slot: Option<FeedbackSlotId>,
        target: u16,
        receiver_register: u16,
        key_register: u16,
    ) -> VmResult<()> {
        let receiver = self.read_register(frame.registers(), receiver_register);
        let key_value = self.read_register(frame.registers(), key_register);
        let coercible_result = Self::check_object_coercible(agent, receiver);
        let Some(()) = self.handle_vm_result(agent, coercible_result)? else {
            return Ok(());
        };
        if let Some(object) = receiver.as_object_ref()
            && let Some(index) = key_value
                .as_smi()
                .and_then(|index| u32::try_from(index).ok())
        {
            if let Some(value) = self.try_keyed_dense_index_load_inline_cache_hit(
                agent,
                frame.code(),
                feedback_slot,
                object,
                index,
            ) {
                self.write_register(frame.registers(), target, value);
                self.advance_instruction();
                return Ok(());
            }
            let value = if let Some(result) = self.mapped_arguments_get(agent, object, index) {
                let Some(value) = self.handle_vm_result(agent, result)? else {
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
                self.advance_instruction();
                return Ok(());
            }
        }
        let key_result = self.property_key_from_value(agent, host, registry, &frame, key_value);
        let Some(key) = self.handle_vm_result(agent, key_result)? else {
            return Ok(());
        };
        let value = if let Some(object) = receiver.as_object_ref() {
            if let Some(index) = key.as_index() {
                if let Some(value) = self.try_keyed_dense_index_load_inline_cache_hit(
                    agent,
                    frame.code(),
                    feedback_slot,
                    object,
                    index,
                ) {
                    self.write_register(frame.registers(), target, value);
                    self.advance_instruction();
                    return Ok(());
                }
                let value = if let Some(result) = self.mapped_arguments_get(agent, object, index) {
                    let Some(value) = self.handle_vm_result(agent, result)? else {
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
                        self.get_property_from_value(agent, host, registry, &frame, receiver, key);
                    let Some(value) = self.handle_vm_result(agent, property_result)? else {
                        return Ok(());
                    };
                    value
                };
                self.observe_keyed_index_access(agent, frame.code(), feedback_slot, object, index);
                value
            } else if let Some(atom) = key.as_atom() {
                if let Some(value) = self.try_keyed_property_load_inline_cache(
                    agent,
                    frame.code(),
                    feedback_slot,
                    object,
                    atom,
                ) {
                    self.record_feedback_slot(frame.code(), feedback_slot);
                    self.write_register(frame.registers(), target, value);
                    self.advance_instruction();
                    return Ok(());
                }
                let property_result =
                    self.get_property_from_value(agent, host, registry, &frame, receiver, key);
                let Some(value) = self.handle_vm_result(agent, property_result)? else {
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
                    self.get_property_from_value(agent, host, registry, &frame, receiver, key);
                let Some(value) = self.handle_vm_result(agent, property_result)? else {
                    return Ok(());
                };
                self.observe_keyed_generic_slow_path(frame.code(), feedback_slot);
                value
            }
        } else {
            let property_result =
                self.get_property_from_value(agent, host, registry, &frame, receiver, key);
            let Some(value) = self.handle_vm_result(agent, property_result)? else {
                return Ok(());
            };
            value
        };
        self.write_register(frame.registers(), target, value);
        self.advance_instruction();
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        clippy::too_many_lines,
        reason = "VM helper keeps dispatch state explicit while isolating the property opcode family"
    )]
    pub(super) fn execute_set_keyed_property_opcode(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
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
        let Some(()) = self.handle_vm_result(agent, coercible_result)? else {
            return Ok(());
        };
        if let Some(object) = receiver.as_object_ref()
            && let Some(index) = key_value
                .as_smi()
                .and_then(|index| u32::try_from(index).ok())
        {
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
                    let Some(()) = self.handle_vm_result(agent, assignment_result)? else {
                        return Ok(());
                    };
                }
                self.advance_instruction();
                return Ok(());
            }
            let mut used_index_fast_path = false;
            let stored = if let Some(result) =
                self.mapped_arguments_set(agent, object, index, value)
            {
                let Some(()) = self.handle_vm_result(agent, result)? else {
                    return Ok(());
                };
                Some(true)
            } else {
                let fast_result = self.try_fast_set_typed_array_index(
                    agent, host, registry, &frame, object, index, value,
                );
                let Some(fast_result) = self.handle_vm_result(agent, fast_result)? else {
                    return Ok(());
                };
                if let Some(stored) = fast_result {
                    used_index_fast_path = true;
                    Some(stored)
                } else {
                    let fast_result =
                        Self::try_fast_set_engine_array_index(agent, object, index, value);
                    let Some(fast_result) = self.handle_vm_result(agent, fast_result)? else {
                        return Ok(());
                    };
                    if let Some(stored) = fast_result {
                        used_index_fast_path = true;
                        Some(stored)
                    } else {
                        let fast_result = Self::try_fast_set_ordinary_index_data_property(
                            agent, object, index, value,
                        );
                        let Some(fast_result) = self.handle_vm_result(agent, fast_result)? else {
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
                    let Some(()) = self.handle_vm_result(agent, assignment_result)? else {
                        return Ok(());
                    };
                }
                if !used_index_fast_path {
                    Self::sync_engine_array_length(agent, object)?;
                }
                self.observe_keyed_index_access(agent, frame.code(), feedback_slot, object, index);
                self.advance_instruction();
                return Ok(());
            }
        }
        let key_result = self.property_key_from_value(agent, host, registry, &frame, key_value);
        let Some(key) = self.handle_vm_result(agent, key_result)? else {
            return Ok(());
        };
        if let Some(object) = receiver.as_object_ref() {
            if let Some(index) = key.as_index() {
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
                        let Some(()) = self.handle_vm_result(agent, assignment_result)? else {
                            return Ok(());
                        };
                    }
                    self.advance_instruction();
                    return Ok(());
                }
                let mut used_index_fast_path = false;
                let stored = if let Some(result) =
                    self.mapped_arguments_set(agent, object, index, value)
                {
                    let Some(()) = self.handle_vm_result(agent, result)? else {
                        return Ok(());
                    };
                    true
                } else {
                    let fast_result = self.try_fast_set_typed_array_index(
                        agent, host, registry, &frame, object, index, value,
                    );
                    let Some(fast_result) = self.handle_vm_result(agent, fast_result)? else {
                        return Ok(());
                    };
                    if let Some(stored) = fast_result {
                        used_index_fast_path = true;
                        stored
                    } else {
                        let fast_result =
                            Self::try_fast_set_engine_array_index(agent, object, index, value);
                        let Some(fast_result) = self.handle_vm_result(agent, fast_result)? else {
                            return Ok(());
                        };
                        if let Some(stored) = fast_result {
                            used_index_fast_path = true;
                            stored
                        } else {
                            let fast_result = Self::try_fast_set_ordinary_index_data_property(
                                agent, object, index, value,
                            );
                            let Some(fast_result) = self.handle_vm_result(agent, fast_result)?
                            else {
                                return Ok(());
                            };
                            if let Some(stored) = fast_result {
                                used_index_fast_path = true;
                                stored
                            } else {
                                let set_result = self.set_property_on_value(
                                    agent, host, registry, &frame, receiver, key, value,
                                );
                                let Some(stored) = self.handle_vm_result(agent, set_result)? else {
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
                    let Some(()) = self.handle_vm_result(agent, assignment_result)? else {
                        return Ok(());
                    };
                }
                if !used_index_fast_path {
                    Self::sync_engine_array_length(agent, object)?;
                }
                self.observe_keyed_index_access(agent, frame.code(), feedback_slot, object, index);
            } else if let Some(atom) = key.as_atom() {
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
                        let Some(()) = self.handle_vm_result(agent, assignment_result)? else {
                            return Ok(());
                        };
                    }
                    self.record_feedback_slot(frame.code(), feedback_slot);
                    self.advance_instruction();
                    return Ok(());
                }
                let set_result =
                    self.set_property_on_value(agent, host, registry, &frame, receiver, key, value);
                let Some(stored) = self.handle_vm_result(agent, set_result)? else {
                    return Ok(());
                };
                if assignment {
                    let assignment_result = self.check_property_assignment_result(
                        agent,
                        frame,
                        stored,
                        strict_assignment,
                    );
                    let Some(()) = self.handle_vm_result(agent, assignment_result)? else {
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
                    self.set_property_on_value(agent, host, registry, &frame, receiver, key, value);
                let Some(stored) = self.handle_vm_result(agent, set_result)? else {
                    return Ok(());
                };
                if assignment {
                    let assignment_result = self.check_property_assignment_result(
                        agent,
                        frame,
                        stored,
                        strict_assignment,
                    );
                    let Some(()) = self.handle_vm_result(agent, assignment_result)? else {
                        return Ok(());
                    };
                }
                self.observe_keyed_generic_slow_path(frame.code(), feedback_slot);
            }
        } else {
            let store_result =
                self.set_property_on_value(agent, host, registry, &frame, receiver, key, value);
            let Some(stored) = self.handle_vm_result(agent, store_result)? else {
                return Ok(());
            };
            if assignment {
                let assignment_result =
                    self.check_property_assignment_result(agent, frame, stored, strict_assignment);
                let Some(()) = self.handle_vm_result(agent, assignment_result)? else {
                    return Ok(());
                };
            }
        }
        self.advance_instruction();
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper keeps dispatch state explicit while isolating the property opcode family"
    )]
    pub(super) fn execute_define_keyed_property_opcode(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        object_register: u16,
        value_register: u16,
        key_register: u16,
    ) -> VmResult<()> {
        let object = self.object_register(&frame, object_register)?;
        let value = self.read_register(frame.registers(), value_register);
        let key_value = self.read_register(frame.registers(), key_register);
        let key_result = self.property_key_from_value(agent, host, registry, &frame, key_value);
        let Some(key) = self.handle_vm_result(agent, key_result)? else {
            return Ok(());
        };
        self.define_data_property(agent, host, registry, frame, object, key, value)?;
        if key.as_index().is_some() {
            Self::sync_engine_array_length(agent, object)?;
        }
        self.advance_instruction();
        Ok(())
    }

    pub(super) fn execute_to_property_key_opcode(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        target: u16,
        key_register: u16,
    ) -> VmResult<()> {
        let key_value = self.read_register(frame.registers(), key_register);
        let key_result = self.property_key_from_value(agent, host, registry, &frame, key_value);
        let Some(key) = self.handle_vm_result(agent, key_result)? else {
            return Ok(());
        };
        let value = self.property_key_to_enumeration_value(agent, key);
        self.write_register(frame.registers(), target, value);
        self.advance_instruction();
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper keeps dispatch state explicit while isolating the property opcode family"
    )]
    pub(super) fn execute_delete_property_opcode(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        target: u16,
        receiver_register: u16,
        key_register: u16,
    ) -> VmResult<()> {
        let receiver = self.read_register(frame.registers(), receiver_register);
        let key_value = self.read_register(frame.registers(), key_register);
        let coercible_result = Self::check_object_coercible(agent, receiver);
        let Some(()) = self.handle_vm_result(agent, coercible_result)? else {
            return Ok(());
        };
        let key_result = self.property_key_from_value(agent, host, registry, &frame, key_value);
        let Some(key) = self.handle_vm_result(agent, key_result)? else {
            return Ok(());
        };
        let delete_result =
            self.delete_property_from_value(agent, host, registry, frame, receiver, key);
        let Some(deleted) = self.handle_vm_result(agent, delete_result)? else {
            return Ok(());
        };
        if !deleted && self.frame_is_strict(frame) {
            let type_error = Err(VmError::Abrupt(errors::throw_type_error(agent)));
            let Some(()) = self.handle_vm_result(agent, type_error)? else {
                return Ok(());
            };
        }
        self.write_register(frame.registers(), target, Value::from_bool(deleted));
        self.advance_instruction();
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper keeps dispatch state explicit while isolating the property opcode family"
    )]
    pub(super) fn execute_copy_data_properties_opcode(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        target_register: u16,
        source_register: u16,
        excluded_register: u16,
    ) -> VmResult<()> {
        let target = self.object_register(&frame, target_register)?;
        let source = self.read_register(frame.registers(), source_register);
        let excluded_keys = self.read_register(frame.registers(), excluded_register);
        let copy_result =
            self.copy_data_properties(agent, host, registry, &frame, target, source, excluded_keys);
        let Some(()) = self.handle_vm_result(agent, copy_result)? else {
            return Ok(());
        };
        self.advance_instruction();
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper keeps dispatch state explicit while isolating the property opcode family"
    )]
    pub(super) fn execute_store_dense_element_opcode(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
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
                &frame,
                receiver,
                PropertyKey::Index(u32::from(index_operand)),
                value,
            );
            let Some(_) = self.handle_vm_result(agent, store_result)? else {
                return Ok(());
            };
        }
        self.advance_instruction();
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper keeps dispatch state explicit while isolating the property opcode family"
    )]
    pub(super) fn execute_load_dense_element_opcode(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        target: u16,
        receiver_register: u16,
        index_operand: u16,
    ) -> VmResult<()> {
        let receiver = self.read_register(frame.registers(), receiver_register);
        let value = if let Some(object) = receiver.as_object_ref() {
            if let Some(result) = self.mapped_arguments_get(agent, object, u32::from(index_operand))
            {
                let Some(value) = self.handle_vm_result(agent, result)? else {
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
                    &frame,
                    receiver,
                    PropertyKey::Index(u32::from(index_operand)),
                );
                let Some(value) = self.handle_vm_result(agent, property_result)? else {
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
                let Some(value) = self.handle_vm_result(agent, element)? else {
                    return Ok(());
                };
                value
            }
        } else {
            let property_result = self.get_property_from_value(
                agent,
                host,
                registry,
                &frame,
                receiver,
                PropertyKey::Index(u32::from(index_operand)),
            );
            let Some(value) = self.handle_vm_result(agent, property_result)? else {
                return Ok(());
            };
            value
        };
        self.write_register(frame.registers(), target, value);
        self.advance_instruction();
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
        frame: FrameRecord,
        object: lyng_js_types::ObjectRef,
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
                frame: &frame,
            },
            object,
            key,
            descriptor,
            AllocationLifetime::Default,
        );
        let Some(created) = self.handle_vm_result(agent, define_result)? else {
            return Ok(());
        };
        if !created {
            let type_error = Err(VmError::Abrupt(errors::throw_type_error(agent)));
            let Some(()) = self.handle_vm_result(agent, type_error)? else {
                return Ok(());
            };
        }
        Ok(())
    }

    fn check_property_assignment_result(
        &self,
        agent: &mut Agent,
        frame: FrameRecord,
        stored: bool,
        strict_override: bool,
    ) -> VmResult<()> {
        if !stored && (strict_override || self.frame_is_strict(frame)) {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        Ok(())
    }
}
