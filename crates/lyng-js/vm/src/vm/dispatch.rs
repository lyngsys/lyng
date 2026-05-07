use super::values::{
    bigint_bitwise_and_values, bigint_bitwise_or_values, bigint_bitwise_xor_values,
    bigint_shift_left_values, bigint_shift_right_values, compare_numeric_values,
    decode_env_operand, encode_number,
};
use super::*;
use crate::vm::property_access::ToPrimitiveHint;
use crate::vm::property_access::VmProxyBridge;
use lyng_js_ops::{errors, object, read};
use lyng_js_types::{AbruptCompletion, PropertyKey};

impl Vm {
    pub(super) fn handle_vm_result<T>(
        &mut self,
        agent: &mut Agent,
        result: VmResult<T>,
    ) -> VmResult<Option<T>> {
        match result {
            Ok(value) => Ok(Some(value)),
            Err(VmError::Abrupt(AbruptCompletion::Throw(value))) => {
                if self.transfer_to_exception_handler(agent, value)? {
                    Ok(None)
                } else {
                    Err(VmError::Abrupt(AbruptCompletion::Throw(value)))
                }
            }
            Err(error) => Err(error),
        }
    }

    pub(super) fn run(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
    ) -> VmResult<Value> {
        loop {
            let frame = self
                .frame()
                .expect("evaluation should install one active frame");
            let installed = self
                .installed
                .get(code_index(frame.code()))
                .and_then(Option::as_ref)
                .ok_or(VmError::MissingInstalledCode(frame.code()))?;
            let instruction = installed
                .function
                .instructions()
                .get(
                    usize::try_from(frame.instruction_offset())
                        .expect("instruction offset should fit into usize"),
                )
                .copied()
                .ok_or(VmError::InstructionOutOfBounds {
                    code: frame.code(),
                    instruction_offset: frame.instruction_offset(),
                })?;

            match instruction {
                Instruction::Abc { opcode, a, b, c } => {
                    let (a, b, c) = self.decode_abc_operands(installed, frame, opcode, a, b, c);
                    match opcode {
                        Opcode::Move => {
                            let value = self.read_register(frame, b)?;
                            self.write_register(frame, a, value)?;
                            self.advance_instruction()?;
                        }
                        Opcode::Add
                        | Opcode::Sub
                        | Opcode::Mul
                        | Opcode::Div
                        | Opcode::Mod
                        | Opcode::Exp
                        | Opcode::BitOr
                        | Opcode::BitAnd
                        | Opcode::BitXor
                        | Opcode::ShiftLeft
                        | Opcode::ShiftRight
                        | Opcode::UnsignedShiftRight
                        | Opcode::Equal
                        | Opcode::StrictEqual
                        | Opcode::LessThan
                        | Opcode::LessEqual
                        | Opcode::GreaterThan
                        | Opcode::GreaterEqual => {
                            let opcode_result = self.execute_abc_value_opcode(
                                agent, host, registry, frame, opcode, b, c,
                            );
                            let Some(value) = self.handle_vm_result(agent, opcode_result)? else {
                                continue;
                            };
                            self.record_feedback_site(frame.code(), frame.instruction_offset());
                            self.write_register(frame, a, value)?;
                            self.advance_instruction()?;
                        }
                        Opcode::In => {
                            let key_value = self.read_register(frame, b)?;
                            let receiver = self.read_register(frame, c)?;
                            let object_result = receiver
                                .as_object_ref()
                                .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)));
                            let Some(object) = self.handle_vm_result(agent, object_result)? else {
                                continue;
                            };
                            let key_result = self.to_property_key_from_value(
                                agent, host, registry, frame, key_value,
                            );
                            let Some(key) = self.handle_vm_result(agent, key_result)? else {
                                continue;
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
                            let Some(has_property) = self.handle_vm_result(agent, has_property)?
                            else {
                                continue;
                            };
                            self.write_register(frame, a, Value::from_bool(has_property))?;
                            self.advance_instruction()?;
                        }
                        Opcode::Negate => {
                            let negate_result = self.negate_value(agent, host, registry, frame, b);
                            let Some(value) = self.handle_vm_result(agent, negate_result)? else {
                                continue;
                            };
                            self.record_feedback_site(frame.code(), frame.instruction_offset());
                            self.write_register(frame, a, value)?;
                            self.advance_instruction()?;
                        }
                        Opcode::BitNot => {
                            let bit_not_result =
                                self.bitwise_not_value(agent, host, registry, frame, b);
                            let Some(value) = self.handle_vm_result(agent, bit_not_result)? else {
                                continue;
                            };
                            self.record_feedback_site(frame.code(), frame.instruction_offset());
                            self.write_register(frame, a, value)?;
                            self.advance_instruction()?;
                        }
                        Opcode::Increment | Opcode::Decrement => {
                            let update_result = self.update_register_value(
                                agent,
                                host,
                                registry,
                                frame,
                                b,
                                opcode == Opcode::Increment,
                            );
                            let Some((numeric, value)) =
                                self.handle_vm_result(agent, update_result)?
                            else {
                                continue;
                            };
                            self.write_register(frame, b, numeric)?;
                            self.record_feedback_site(frame.code(), frame.instruction_offset());
                            self.write_register(frame, a, value)?;
                            self.advance_instruction()?;
                        }
                        Opcode::GetNamedProperty => {
                            let receiver = self.read_register(frame, b)?;
                            let atom = self.read_atom_constant(frame.code(), u32::from(c))?;
                            let key = PropertyKey::from_atom(atom);
                            let value = if let Some(object) = receiver.as_object_ref() {
                                if let Some(value) = self.try_named_property_load_inline_cache_hit(
                                    agent,
                                    frame.code(),
                                    frame.instruction_offset(),
                                    object,
                                ) {
                                    self.write_register(frame, a, value)?;
                                    self.advance_instruction()?;
                                    continue;
                                }
                                let property_result = self.get_property_from_value(
                                    agent, host, registry, frame, receiver, key,
                                );
                                let Some(value) = self.handle_vm_result(agent, property_result)?
                                else {
                                    continue;
                                };
                                self.observe_named_property_slow_path(
                                    agent,
                                    frame.code(),
                                    frame.instruction_offset(),
                                    object,
                                    atom,
                                    lyng_js_objects::NamedPropertyCachePurpose::Load,
                                );
                                value
                            } else {
                                let property_result = self.get_property_from_value(
                                    agent, host, registry, frame, receiver, key,
                                );
                                let Some(value) = self.handle_vm_result(agent, property_result)?
                                else {
                                    continue;
                                };
                                value
                            };
                            self.write_register(frame, a, value)?;
                            self.advance_instruction()?;
                        }
                        Opcode::SetNamedProperty
                        | Opcode::AssignNamedProperty
                        | Opcode::StrictAssignNamedProperty => {
                            let assignment = matches!(
                                opcode,
                                Opcode::AssignNamedProperty | Opcode::StrictAssignNamedProperty
                            );
                            let strict_assignment =
                                matches!(opcode, Opcode::StrictAssignNamedProperty);
                            let receiver = self.read_register(frame, a)?;
                            let value = self.read_register(frame, b)?;
                            let atom = self.read_atom_constant(frame.code(), u32::from(c))?;
                            let key = PropertyKey::from_atom(atom);
                            if let Some(object) = receiver.as_object_ref() {
                                if let Some(stored) = self.try_named_property_store_inline_cache(
                                    agent,
                                    frame.code(),
                                    frame.instruction_offset(),
                                    object,
                                    value,
                                ) {
                                    if assignment {
                                        let assignment_result = self
                                            .check_property_assignment_result(
                                                agent,
                                                frame,
                                                stored,
                                                strict_assignment,
                                            );
                                        let Some(()) =
                                            self.handle_vm_result(agent, assignment_result)?
                                        else {
                                            continue;
                                        };
                                    }
                                    self.record_feedback_site(
                                        frame.code(),
                                        frame.instruction_offset(),
                                    );
                                    self.advance_instruction()?;
                                    continue;
                                }
                                let set_result = if Self::prototype_chain_has_proxy(agent, object) {
                                    self.set_property_on_value(
                                        agent, host, registry, frame, receiver, key, value,
                                    )
                                } else {
                                    let set_result = object::ordinary_set(
                                        agent,
                                        object,
                                        key,
                                        value,
                                        AllocationLifetime::Default,
                                    )
                                    .map_err(VmError::Abrupt);
                                    match set_result {
                                        Ok(result) => Ok(result),
                                        Err(VmError::Abrupt(_)) => self.set_property_on_value(
                                            agent, host, registry, frame, receiver, key, value,
                                        ),
                                        Err(error) => Err(error),
                                    }
                                };
                                let Some(stored) = self.handle_vm_result(agent, set_result)? else {
                                    continue;
                                };
                                if assignment {
                                    let assignment_result = self.check_property_assignment_result(
                                        agent,
                                        frame,
                                        stored,
                                        strict_assignment,
                                    );
                                    let Some(()) =
                                        self.handle_vm_result(agent, assignment_result)?
                                    else {
                                        continue;
                                    };
                                }
                                self.observe_named_property_slow_path(
                                    agent,
                                    frame.code(),
                                    frame.instruction_offset(),
                                    object,
                                    atom,
                                    lyng_js_objects::NamedPropertyCachePurpose::Store,
                                );
                            } else {
                                let store_result = self.set_property_on_value(
                                    agent, host, registry, frame, receiver, key, value,
                                );
                                let Some(stored) = self.handle_vm_result(agent, store_result)?
                                else {
                                    continue;
                                };
                                if assignment {
                                    let assignment_result = self.check_property_assignment_result(
                                        agent,
                                        frame,
                                        stored,
                                        strict_assignment,
                                    );
                                    let Some(()) =
                                        self.handle_vm_result(agent, assignment_result)?
                                    else {
                                        continue;
                                    };
                                }
                            }
                            self.advance_instruction()?;
                        }
                        Opcode::DefineNamedProperty => {
                            let object = self.object_register(frame, a)?;
                            let value = self.read_register(frame, b)?;
                            let key = PropertyKey::from_atom(
                                self.read_atom_constant(frame.code(), u32::from(c))?,
                            );
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
                            let Some(created) = self.handle_vm_result(agent, define_result)? else {
                                continue;
                            };
                            if !created {
                                let type_error =
                                    Err(VmError::Abrupt(errors::throw_type_error(agent)));
                                let Some(()) = self.handle_vm_result(agent, type_error)? else {
                                    continue;
                                };
                            }
                            self.advance_instruction()?;
                        }
                        Opcode::GetKeyedProperty => {
                            let receiver = self.read_register(frame, b)?;
                            let key_value = self.read_register(frame, c)?;
                            let coercible_result = self.check_object_coercible(agent, receiver);
                            let Some(()) = self.handle_vm_result(agent, coercible_result)? else {
                                continue;
                            };
                            let key_result = self.to_property_key_from_value(
                                agent, host, registry, frame, key_value,
                            );
                            let Some(key) = self.handle_vm_result(agent, key_result)? else {
                                continue;
                            };
                            let value = if let Some(object) = receiver.as_object_ref() {
                                if let Some(index) = key.as_index() {
                                    let value = if let Some(result) =
                                        self.mapped_arguments_get(agent, object, index)
                                    {
                                        let Some(value) = self.handle_vm_result(agent, result)?
                                        else {
                                            continue;
                                        };
                                        value
                                    } else if let Some(value) =
                                        self.try_fast_own_index_value(agent, object, index)?
                                    {
                                        value
                                    } else {
                                        let property_result = self.get_property_from_value(
                                            agent, host, registry, frame, receiver, key,
                                        );
                                        let Some(value) =
                                            self.handle_vm_result(agent, property_result)?
                                        else {
                                            continue;
                                        };
                                        value
                                    };
                                    self.observe_keyed_index_slow_path(
                                        frame.code(),
                                        frame.instruction_offset(),
                                    );
                                    value
                                } else if let Some(atom) = key.as_atom() {
                                    if let Some(value) = self.try_keyed_property_load_inline_cache(
                                        agent,
                                        frame.code(),
                                        frame.instruction_offset(),
                                        object,
                                        atom,
                                    ) {
                                        self.record_feedback_site(
                                            frame.code(),
                                            frame.instruction_offset(),
                                        );
                                        self.write_register(frame, a, value)?;
                                        self.advance_instruction()?;
                                        continue;
                                    }
                                    let property_result = self.get_property_from_value(
                                        agent, host, registry, frame, receiver, key,
                                    );
                                    let Some(value) =
                                        self.handle_vm_result(agent, property_result)?
                                    else {
                                        continue;
                                    };
                                    self.observe_keyed_atom_slow_path(
                                        agent,
                                        frame.code(),
                                        frame.instruction_offset(),
                                        object,
                                        atom,
                                        lyng_js_objects::NamedPropertyCachePurpose::Load,
                                    );
                                    value
                                } else {
                                    let property_result = self.get_property_from_value(
                                        agent, host, registry, frame, receiver, key,
                                    );
                                    let Some(value) =
                                        self.handle_vm_result(agent, property_result)?
                                    else {
                                        continue;
                                    };
                                    self.observe_keyed_generic_slow_path(
                                        frame.code(),
                                        frame.instruction_offset(),
                                    );
                                    value
                                }
                            } else {
                                let property_result = self.get_property_from_value(
                                    agent, host, registry, frame, receiver, key,
                                );
                                let Some(value) = self.handle_vm_result(agent, property_result)?
                                else {
                                    continue;
                                };
                                value
                            };
                            self.write_register(frame, a, value)?;
                            self.advance_instruction()?;
                        }
                        Opcode::SetKeyedProperty
                        | Opcode::AssignKeyedProperty
                        | Opcode::StrictAssignKeyedProperty => {
                            let assignment = matches!(
                                opcode,
                                Opcode::AssignKeyedProperty | Opcode::StrictAssignKeyedProperty
                            );
                            let strict_assignment =
                                matches!(opcode, Opcode::StrictAssignKeyedProperty);
                            let receiver = self.read_register(frame, a)?;
                            let value = self.read_register(frame, b)?;
                            let key_value = self.read_register(frame, c)?;
                            let coercible_result = self.check_object_coercible(agent, receiver);
                            let Some(()) = self.handle_vm_result(agent, coercible_result)? else {
                                continue;
                            };
                            let key_result = self.to_property_key_from_value(
                                agent, host, registry, frame, key_value,
                            );
                            let Some(key) = self.handle_vm_result(agent, key_result)? else {
                                continue;
                            };
                            if let Some(object) = receiver.as_object_ref() {
                                if let Some(index) = key.as_index() {
                                    let mut used_index_fast_path = false;
                                    let stored = if let Some(result) =
                                        self.mapped_arguments_set(agent, object, index, value)
                                    {
                                        let Some(()) = self.handle_vm_result(agent, result)? else {
                                            continue;
                                        };
                                        true
                                    } else {
                                        let fast_result = self.try_fast_set_engine_array_index(
                                            agent, object, index, value,
                                        );
                                        let Some(fast_result) =
                                            self.handle_vm_result(agent, fast_result)?
                                        else {
                                            continue;
                                        };
                                        if let Some(stored) = fast_result {
                                            used_index_fast_path = true;
                                            stored
                                        } else {
                                            let set_result = self.set_property_on_value(
                                                agent, host, registry, frame, receiver, key, value,
                                            );
                                            let Some(stored) =
                                                self.handle_vm_result(agent, set_result)?
                                            else {
                                                continue;
                                            };
                                            stored
                                        }
                                    };
                                    if assignment {
                                        let assignment_result = self
                                            .check_property_assignment_result(
                                                agent,
                                                frame,
                                                stored,
                                                strict_assignment,
                                            );
                                        let Some(()) =
                                            self.handle_vm_result(agent, assignment_result)?
                                        else {
                                            continue;
                                        };
                                    }
                                    if !used_index_fast_path {
                                        self.sync_engine_array_length(agent, object)?;
                                        self.observe_keyed_index_slow_path(
                                            frame.code(),
                                            frame.instruction_offset(),
                                        );
                                    }
                                } else if let Some(atom) = key.as_atom() {
                                    if let Some(stored) = self
                                        .try_keyed_property_store_inline_cache(
                                            agent,
                                            frame.code(),
                                            frame.instruction_offset(),
                                            object,
                                            atom,
                                            value,
                                        )
                                    {
                                        if assignment {
                                            let assignment_result = self
                                                .check_property_assignment_result(
                                                    agent,
                                                    frame,
                                                    stored,
                                                    strict_assignment,
                                                );
                                            let Some(()) =
                                                self.handle_vm_result(agent, assignment_result)?
                                            else {
                                                continue;
                                            };
                                        }
                                        self.record_feedback_site(
                                            frame.code(),
                                            frame.instruction_offset(),
                                        );
                                        self.advance_instruction()?;
                                        continue;
                                    }
                                    let set_result = self.set_property_on_value(
                                        agent, host, registry, frame, receiver, key, value,
                                    );
                                    let Some(stored) = self.handle_vm_result(agent, set_result)?
                                    else {
                                        continue;
                                    };
                                    if assignment {
                                        let assignment_result = self
                                            .check_property_assignment_result(
                                                agent,
                                                frame,
                                                stored,
                                                strict_assignment,
                                            );
                                        let Some(()) =
                                            self.handle_vm_result(agent, assignment_result)?
                                        else {
                                            continue;
                                        };
                                    }
                                    self.observe_keyed_atom_slow_path(
                                        agent,
                                        frame.code(),
                                        frame.instruction_offset(),
                                        object,
                                        atom,
                                        lyng_js_objects::NamedPropertyCachePurpose::Store,
                                    );
                                } else {
                                    let set_result = self.set_property_on_value(
                                        agent, host, registry, frame, receiver, key, value,
                                    );
                                    let Some(stored) = self.handle_vm_result(agent, set_result)?
                                    else {
                                        continue;
                                    };
                                    if assignment {
                                        let assignment_result = self
                                            .check_property_assignment_result(
                                                agent,
                                                frame,
                                                stored,
                                                strict_assignment,
                                            );
                                        let Some(()) =
                                            self.handle_vm_result(agent, assignment_result)?
                                        else {
                                            continue;
                                        };
                                    }
                                    self.observe_keyed_generic_slow_path(
                                        frame.code(),
                                        frame.instruction_offset(),
                                    );
                                }
                            } else {
                                let store_result = self.set_property_on_value(
                                    agent, host, registry, frame, receiver, key, value,
                                );
                                let Some(stored) = self.handle_vm_result(agent, store_result)?
                                else {
                                    continue;
                                };
                                if assignment {
                                    let assignment_result = self.check_property_assignment_result(
                                        agent,
                                        frame,
                                        stored,
                                        strict_assignment,
                                    );
                                    let Some(()) =
                                        self.handle_vm_result(agent, assignment_result)?
                                    else {
                                        continue;
                                    };
                                }
                            }
                            self.advance_instruction()?;
                        }
                        Opcode::DefineKeyedProperty => {
                            let object = self.object_register(frame, a)?;
                            let value = self.read_register(frame, b)?;
                            let key_value = self.read_register(frame, c)?;
                            let key_result = self.to_property_key_from_value(
                                agent, host, registry, frame, key_value,
                            );
                            let Some(key) = self.handle_vm_result(agent, key_result)? else {
                                continue;
                            };
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
                            let Some(created) = self.handle_vm_result(agent, define_result)? else {
                                continue;
                            };
                            if !created {
                                let type_error =
                                    Err(VmError::Abrupt(errors::throw_type_error(agent)));
                                let Some(()) = self.handle_vm_result(agent, type_error)? else {
                                    continue;
                                };
                            }
                            if key.as_index().is_some() {
                                self.sync_engine_array_length(agent, object)?;
                            }
                            self.advance_instruction()?;
                        }
                        Opcode::SetFunctionName => {
                            let function = self.object_register(frame, a)?;
                            let name_value = self.read_register(frame, b)?;
                            let set_result = self.set_function_name(agent, function, name_value);
                            let Some(()) = self.handle_vm_result(agent, set_result)? else {
                                continue;
                            };
                            self.advance_instruction()?;
                        }
                        Opcode::ToPropertyKey => {
                            let key_value = self.read_register(frame, b)?;
                            let key_result = self.to_property_key_from_value(
                                agent, host, registry, frame, key_value,
                            );
                            let Some(key) = self.handle_vm_result(agent, key_result)? else {
                                continue;
                            };
                            let value = self.property_key_to_enumeration_value(agent, key);
                            self.write_register(frame, a, value)?;
                            self.advance_instruction()?;
                        }
                        Opcode::DeleteProperty => {
                            let receiver = self.read_register(frame, b)?;
                            let key_value = self.read_register(frame, c)?;
                            let coercible_result = self.check_object_coercible(agent, receiver);
                            let Some(()) = self.handle_vm_result(agent, coercible_result)? else {
                                continue;
                            };
                            let key_result = self.to_property_key_from_value(
                                agent, host, registry, frame, key_value,
                            );
                            let Some(key) = self.handle_vm_result(agent, key_result)? else {
                                continue;
                            };
                            let delete_result = self.delete_property_from_value(
                                agent, host, registry, frame, receiver, key,
                            );
                            let Some(deleted) = self.handle_vm_result(agent, delete_result)? else {
                                continue;
                            };
                            if !deleted && self.frame_is_strict(frame) {
                                let type_error =
                                    Err(VmError::Abrupt(errors::throw_type_error(agent)));
                                let Some(()) = self.handle_vm_result(agent, type_error)? else {
                                    continue;
                                };
                            }
                            self.write_register(frame, a, Value::from_bool(deleted))?;
                            self.advance_instruction()?;
                        }
                        Opcode::CopyDataProperties => {
                            let target = self.object_register(frame, a)?;
                            let source = self.read_register(frame, b)?;
                            let excluded_keys = self.read_register(frame, c)?;
                            let copy_result = self.copy_data_properties(
                                agent,
                                host,
                                registry,
                                frame,
                                target,
                                source,
                                excluded_keys,
                            );
                            let Some(()) = self.handle_vm_result(agent, copy_result)? else {
                                continue;
                            };
                            self.advance_instruction()?;
                        }
                        Opcode::StoreDenseElement => {
                            let receiver = self.read_register(frame, a)?;
                            let value = self.read_register(frame, b)?;
                            if let Some(object) = receiver.as_object_ref() {
                                if let Some(result) =
                                    self.mapped_arguments_set(agent, object, u32::from(c), value)
                                {
                                    result?;
                                }
                                let _ = agent.with_heap_and_objects(|heap, objects| {
                                    let mut mutator = heap.mutator();
                                    objects.set_element(
                                        &mut mutator,
                                        object,
                                        u32::from(c),
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
                                    PropertyKey::Index(u32::from(c)),
                                    value,
                                );
                                let Some(_) = self.handle_vm_result(agent, store_result)? else {
                                    continue;
                                };
                            }
                            self.advance_instruction()?;
                        }
                        Opcode::LoadDenseElement => {
                            let receiver = self.read_register(frame, b)?;
                            let value = if let Some(object) = receiver.as_object_ref() {
                                if let Some(result) =
                                    self.mapped_arguments_get(agent, object, u32::from(c))
                                {
                                    let Some(value) = self.handle_vm_result(agent, result)? else {
                                        continue;
                                    };
                                    value
                                } else if let Some(value) =
                                    self.try_fast_own_index_value(agent, object, u32::from(c))?
                                {
                                    value
                                } else if Self::prototype_chain_has_proxy(agent, object) {
                                    let property_result = self.get_property_from_value(
                                        agent,
                                        host,
                                        registry,
                                        frame,
                                        receiver,
                                        PropertyKey::Index(u32::from(c)),
                                    );
                                    let Some(value) =
                                        self.handle_vm_result(agent, property_result)?
                                    else {
                                        continue;
                                    };
                                    value
                                } else {
                                    let element = object::ordinary_get(
                                        agent,
                                        object,
                                        PropertyKey::Index(u32::from(c)),
                                    )
                                    .map_err(VmError::Abrupt);
                                    let Some(value) = self.handle_vm_result(agent, element)? else {
                                        continue;
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
                                    PropertyKey::Index(u32::from(c)),
                                );
                                let Some(value) = self.handle_vm_result(agent, property_result)?
                                else {
                                    continue;
                                };
                                value
                            };
                            self.write_register(frame, a, value)?;
                            self.advance_instruction()?;
                        }
                        Opcode::Call => {
                            let payload = installed
                                .wide_payload(frame.instruction_offset())
                                .ok_or(VmError::MissingWideOperand {
                                    code: frame.code(),
                                    instruction_offset: frame.instruction_offset(),
                                    opcode,
                                })?;
                            let spread_mask = installed
                                .feedback_descriptor(frame.instruction_offset())
                                .and_then(|descriptor| descriptor.metadata().spread_mask());
                            let call_result = self.call_value(
                                agent,
                                host,
                                registry,
                                frame,
                                a,
                                b,
                                c,
                                CallRange::decode(payload),
                                spread_mask,
                            );
                            let Some(_) = self.handle_vm_result(agent, call_result)? else {
                                continue;
                            };
                            self.record_feedback_site(frame.code(), frame.instruction_offset());
                        }
                        Opcode::TailCall => {
                            let payload = installed
                                .wide_payload(frame.instruction_offset())
                                .ok_or(VmError::MissingWideOperand {
                                    code: frame.code(),
                                    instruction_offset: frame.instruction_offset(),
                                    opcode,
                                })?;
                            let spread_mask = installed
                                .feedback_descriptor(frame.instruction_offset())
                                .and_then(|descriptor| descriptor.metadata().spread_mask());
                            let tail_result = self.tail_call_value(
                                agent,
                                host,
                                registry,
                                frame,
                                a,
                                b,
                                CallRange::decode(payload),
                                spread_mask,
                            );
                            let Some(result) = self.handle_vm_result(agent, tail_result)? else {
                                continue;
                            };
                            self.record_feedback_site(frame.code(), frame.instruction_offset());
                            if let Some(result) = result {
                                return Ok(result);
                            }
                        }
                        Opcode::Construct => {
                            let payload = installed
                                .wide_payload(frame.instruction_offset())
                                .ok_or(VmError::MissingWideOperand {
                                    code: frame.code(),
                                    instruction_offset: frame.instruction_offset(),
                                    opcode,
                                })?;
                            let spread_mask = installed
                                .feedback_descriptor(frame.instruction_offset())
                                .and_then(|descriptor| descriptor.metadata().spread_mask());
                            let construct_result = self.construct_value(
                                agent,
                                host,
                                registry,
                                frame,
                                a,
                                b,
                                CallRange::decode(payload),
                                spread_mask,
                            );
                            let Some(_) = self.handle_vm_result(agent, construct_result)? else {
                                continue;
                            };
                            self.record_feedback_site(frame.code(), frame.instruction_offset());
                        }
                        Opcode::CreateForIn => {
                            let value = self.read_register(frame, b)?;
                            let enumerator_result = self.create_for_in_enumerator_for_value(
                                agent, host, registry, frame, value,
                            );
                            let Some(enumerator) =
                                self.handle_vm_result(agent, enumerator_result)?
                            else {
                                continue;
                            };
                            self.for_in_states
                                .insert(frame.registers().base(), a, enumerator);
                            self.advance_instruction()?;
                        }
                        Opcode::AdvanceForIn => {
                            let next =
                                self.for_in_states
                                    .advance(agent, frame.registers().base(), a);
                            let Some(next): Option<Option<PropertyKey>> =
                                self.handle_vm_result(agent, next)?
                            else {
                                continue;
                            };
                            let done = next.is_none();
                            if let Some(key) = next {
                                let value = self.property_key_to_enumeration_value(agent, key);
                                self.write_register(frame, b, value)?;
                            } else {
                                self.write_register(frame, b, Value::undefined())?;
                            }
                            self.write_register(frame, c, Value::from_bool(done))?;
                            self.advance_instruction()?;
                        }
                        Opcode::CreateIterator => {
                            let value = self.read_register(frame, b)?;
                            let iterator_result = self.create_iterator_for_value(
                                agent,
                                host,
                                registry,
                                frame,
                                value,
                                c != 0,
                            );
                            let Some(iterator) = self.handle_vm_result(agent, iterator_result)?
                            else {
                                continue;
                            };
                            self.iterator_states
                                .insert(frame.registers().base(), a, iterator);
                            self.advance_instruction()?;
                        }
                        Opcode::AdvanceIterator => {
                            let next = self.advance_iterator_state(agent, host, registry, frame, a);
                            let Some(next) = self.handle_vm_result(agent, next)? else {
                                continue;
                            };
                            let done = next.is_none();
                            self.write_register(frame, b, next.unwrap_or(Value::undefined()))?;
                            self.write_register(frame, c, Value::from_bool(done))?;
                            self.advance_instruction()?;
                        }
                        Opcode::DelegateYield => {
                            let delegate_result =
                                self.delegate_yield(agent, host, registry, frame, a, b, c);
                            let Some(()) = self.handle_vm_result(agent, delegate_result)? else {
                                continue;
                            };
                        }
                        _ => {
                            return Err(VmError::UnsupportedOpcode {
                                code: frame.code(),
                                instruction_offset: frame.instruction_offset(),
                                opcode,
                            });
                        }
                    }
                }
                Instruction::Abx { opcode, a, bx } => {
                    let (a, bx) = self.decode_abx_operands(installed, frame, a, bx);
                    match opcode {
                        Opcode::LoadUndefined => {
                            self.write_register(frame, a, Value::undefined())?;
                            self.advance_instruction()?;
                        }
                        Opcode::LoadUninitializedLexical => {
                            self.write_register(frame, a, Value::uninitialized_lexical())?;
                            self.advance_instruction()?;
                        }
                        Opcode::LoadNull => {
                            self.write_register(frame, a, Value::null())?;
                            self.advance_instruction()?;
                        }
                        Opcode::LoadTrue => {
                            self.write_register(frame, a, Value::from_bool(true))?;
                            self.advance_instruction()?;
                        }
                        Opcode::LoadFalse => {
                            self.write_register(frame, a, Value::from_bool(false))?;
                            self.advance_instruction()?;
                        }
                        Opcode::LoadSmi => {
                            let value = i16::from_le_bytes((bx as u16).to_le_bytes());
                            self.write_register(frame, a, Value::from_smi(i32::from(value)))?;
                            self.advance_instruction()?;
                        }
                        Opcode::LoadConst => {
                            let value = self.read_constant(agent, frame.code(), bx)?;
                            self.write_register(frame, a, value)?;
                            self.advance_instruction()?;
                        }
                        Opcode::LoadEnvSlot => {
                            let (depth, slot) = decode_env_operand(bx);
                            let environment = self.environment_for_slot_access(
                                agent,
                                frame.lexical_env(),
                                depth,
                                slot,
                            )?;
                            let slot_value = self.read_environment_slot(agent, environment, slot);
                            let Some(value) = self.handle_vm_result(agent, slot_value)? else {
                                continue;
                            };
                            self.write_register(frame, a, value)?;
                            self.advance_instruction()?;
                        }
                        Opcode::StoreEnvSlot => {
                            let (depth, slot) = decode_env_operand(bx);
                            let environment = self.environment_for_slot_access(
                                agent,
                                frame.lexical_env(),
                                depth,
                                slot,
                            )?;
                            let value = self.read_register(frame, a)?;
                            let store_result =
                                self.write_environment_slot(agent, environment, slot, value);
                            let Some(()) = self.handle_vm_result(agent, store_result)? else {
                                continue;
                            };
                            self.advance_instruction()?;
                        }
                        Opcode::AssignEnvSlot => {
                            let (depth, slot) = decode_env_operand(bx);
                            let environment = self.environment_for_slot_access(
                                agent,
                                frame.lexical_env(),
                                depth,
                                slot,
                            )?;
                            let value = self.read_register(frame, a)?;
                            let assign_result = self.assign_environment_slot(
                                agent,
                                environment,
                                slot,
                                value,
                                self.frame_is_strict(frame),
                            );
                            let Some(()) = self.handle_vm_result(agent, assign_result)? else {
                                continue;
                            };
                            self.advance_instruction()?;
                        }
                        Opcode::EnterEnvScope => {
                            self.enter_env_scope(agent, frame, a, bx)?;
                            self.advance_instruction()?;
                        }
                        Opcode::LeaveEnvScope => {
                            self.leave_env_scope(frame, a, bx);
                            self.advance_instruction()?;
                        }
                        Opcode::LoadGlobal => {
                            let atom = self.read_atom_constant(frame.code(), bx)?;
                            let load_result = self.load_global_with_feedback(
                                agent,
                                host,
                                registry,
                                frame,
                                atom,
                                frame.code(),
                                frame.instruction_offset(),
                            );
                            let Some(value) = self.handle_vm_result(agent, load_result)? else {
                                continue;
                            };
                            self.write_register(frame, a, value)?;
                            self.advance_instruction()?;
                        }
                        Opcode::LoadName => {
                            let atom = self.read_atom_constant(frame.code(), bx)?;
                            let load_result =
                                self.load_name_with_context(agent, host, registry, frame, atom);
                            let Some(value) = self.handle_vm_result(agent, load_result)? else {
                                continue;
                            };
                            self.write_register(frame, a, value)?;
                            self.advance_instruction()?;
                        }
                        Opcode::ResolveName => {
                            let atom = self.read_atom_constant(frame.code(), bx)?;
                            let resolve_result =
                                self.resolve_name_with_context(agent, host, registry, frame, atom);
                            let Some(value) = self.handle_vm_result(agent, resolve_result)? else {
                                continue;
                            };
                            self.write_register(frame, a, value)?;
                            self.advance_instruction()?;
                        }
                        Opcode::ResolveGlobal => {
                            let atom = self.read_atom_constant(frame.code(), bx)?;
                            let resolve_result =
                                self.resolve_global(agent, host, registry, frame, atom);
                            let Some(value) = self.handle_vm_result(agent, resolve_result)? else {
                                continue;
                            };
                            self.write_register(frame, a, value)?;
                            self.advance_instruction()?;
                        }
                        Opcode::AssignName => {
                            let atom = self.read_atom_constant(frame.code(), bx)?;
                            let value = self.read_register(frame, a)?;
                            let assign_result = self.assign_name_with_context(
                                agent, host, registry, frame, atom, value,
                            );
                            let Some(_) = self.handle_vm_result(agent, assign_result)? else {
                                continue;
                            };
                            self.advance_instruction()?;
                        }
                        Opcode::AssignVariableName => {
                            let atom = self.read_atom_constant(frame.code(), bx)?;
                            let value = self.read_register(frame, a)?;
                            let assign_result = self.assign_variable_name_with_context(
                                agent, host, registry, frame, atom, value,
                            );
                            let Some(_) = self.handle_vm_result(agent, assign_result)? else {
                                continue;
                            };
                            self.advance_instruction()?;
                        }
                        Opcode::DeleteName => {
                            let atom = self.read_atom_constant(frame.code(), bx)?;
                            let delete_result =
                                self.delete_name_with_context(agent, host, registry, frame, atom);
                            let Some(deleted) = self.handle_vm_result(agent, delete_result)? else {
                                continue;
                            };
                            self.write_register(frame, a, Value::from_bool(deleted))?;
                            self.advance_instruction()?;
                        }
                        Opcode::CaptureName => {
                            let atom = self.read_atom_constant(frame.code(), bx)?;
                            let capture_result = self
                                .capture_name_with_context(agent, host, registry, frame, a, atom);
                            let Some(()) = self.handle_vm_result(agent, capture_result)? else {
                                continue;
                            };
                            self.advance_instruction()?;
                        }
                        Opcode::LoadCapturedName => {
                            let reference_register =
                                u16::try_from(bx).map_err(|_| VmError::RegisterOutOfBounds {
                                    code: frame.code(),
                                    register: u16::MAX,
                                })?;
                            let load_result = self.load_captured_name_with_context(
                                agent,
                                host,
                                registry,
                                frame,
                                reference_register,
                            );
                            let Some(value) = self.handle_vm_result(agent, load_result)? else {
                                continue;
                            };
                            self.write_register(frame, a, value)?;
                            self.advance_instruction()?;
                        }
                        Opcode::LoadCapturedNameThis => {
                            let reference_register =
                                u16::try_from(bx).map_err(|_| VmError::RegisterOutOfBounds {
                                    code: frame.code(),
                                    register: u16::MAX,
                                })?;
                            let load_result = self
                                .load_captured_name_this_with_context(frame, reference_register);
                            let Some(value) = self.handle_vm_result(agent, load_result)? else {
                                continue;
                            };
                            self.write_register(frame, a, value)?;
                            self.advance_instruction()?;
                        }
                        Opcode::AssignCapturedName => {
                            let reference_register =
                                u16::try_from(bx).map_err(|_| VmError::RegisterOutOfBounds {
                                    code: frame.code(),
                                    register: u16::MAX,
                                })?;
                            let value = self.read_register(frame, a)?;
                            let assign_result = self.assign_captured_name_with_context(
                                agent,
                                host,
                                registry,
                                frame,
                                reference_register,
                                value,
                            );
                            let Some(()) = self.handle_vm_result(agent, assign_result)? else {
                                continue;
                            };
                            self.advance_instruction()?;
                        }
                        Opcode::StoreGlobal => {
                            let atom = self.read_atom_constant(frame.code(), bx)?;
                            let value = self.read_register(frame, a)?;
                            let store_result = self.store_global_with_feedback(
                                agent,
                                host,
                                registry,
                                frame,
                                atom,
                                value,
                                frame.code(),
                                frame.instruction_offset(),
                            );
                            let Some(_) = self.handle_vm_result(agent, store_result)? else {
                                continue;
                            };
                            self.advance_instruction()?;
                        }
                        Opcode::AssignGlobal => {
                            let atom = self.read_atom_constant(frame.code(), bx)?;
                            let value = self.read_register(frame, a)?;
                            let assign_result = self.assign_global_with_feedback(
                                agent,
                                host,
                                registry,
                                frame,
                                atom,
                                value,
                                frame.code(),
                                frame.instruction_offset(),
                            );
                            let Some(_) = self.handle_vm_result(agent, assign_result)? else {
                                continue;
                            };
                            self.advance_instruction()?;
                        }
                        Opcode::DeleteGlobal => {
                            let atom = self.read_atom_constant(frame.code(), bx)?;
                            let delete_result = self.delete_global(agent, frame, atom);
                            let Some(deleted) = self.handle_vm_result(agent, delete_result)? else {
                                continue;
                            };
                            self.write_register(frame, a, Value::from_bool(deleted))?;
                            self.advance_instruction()?;
                        }
                        Opcode::LoadThis => {
                            let load_this = (|| -> VmResult<Value> {
                                match agent
                                    .current_execution_context()
                                    .map(|context| context.this_state())
                                    .unwrap_or(ThisState::Value(frame.this_value()))
                                {
                                    ThisState::Value(value) => Ok(value),
                                    ThisState::Uninitialized => {
                                        Err(VmError::Abrupt(errors::throw_reference_error(agent)))
                                    }
                                    ThisState::Lexical => Self::resolve_this_binding(
                                        agent,
                                        frame.lexical_env(),
                                        frame,
                                    ),
                                }
                            })();
                            let Some(value) = self.handle_vm_result(agent, load_this)? else {
                                continue;
                            };
                            self.write_register(frame, a, value)?;
                            self.advance_instruction()?;
                        }
                        Opcode::LoadCallee => {
                            let value = frame
                                .callee()
                                .map_or(Value::undefined(), Value::from_object_ref);
                            self.write_register(frame, a, value)?;
                            self.advance_instruction()?;
                        }
                        Opcode::LoadNewTarget => {
                            let value = frame
                                .new_target()
                                .map_or(Value::undefined(), Value::from_object_ref);
                            self.write_register(frame, a, value)?;
                            self.advance_instruction()?;
                        }
                        Opcode::JumpIfTrue | Opcode::JumpIfFalse => {
                            let condition = self.read_register(frame, a)?;
                            let Some(truthy) = self.handle_vm_result(
                                agent,
                                read::to_boolean_agent(agent, condition).map_err(VmError::Abrupt),
                            )?
                            else {
                                continue;
                            };
                            let delta = i32::from_le_bytes(bx.to_le_bytes());
                            let should_jump = match opcode {
                                Opcode::JumpIfTrue => truthy,
                                Opcode::JumpIfFalse => !truthy,
                                _ => unreachable!("guarded by opcode match"),
                            };
                            if should_jump {
                                self.jump_by(delta)?;
                            } else {
                                self.advance_instruction()?;
                            }
                        }
                        Opcode::CreateObject => {
                            let object = self.create_object(
                                agent,
                                frame.realm(),
                                usize::try_from(bx).unwrap_or(usize::MAX),
                            )?;
                            self.write_register(frame, a, Value::from_object_ref(object))?;
                            self.advance_instruction()?;
                        }
                        Opcode::CreateArray => {
                            let length = usize::try_from(bx).unwrap_or(usize::MAX);
                            let object = self.create_array(agent, frame.realm(), length)?;
                            let length = u32::try_from(length).unwrap_or(u32::MAX);
                            if length != 0 {
                                Self::define_length_property(agent, object, length, false)?;
                            }
                            self.write_register(frame, a, Value::from_object_ref(object))?;
                            self.advance_instruction()?;
                        }
                        Opcode::CheckObjectCoercible => {
                            let value = self.read_register(frame, a)?;
                            let coercible = self.check_object_coercible(agent, value);
                            let Some(()) = self.handle_vm_result(agent, coercible)? else {
                                continue;
                            };
                            self.advance_instruction()?;
                        }
                        Opcode::ThrowIfUninitialized => {
                            let value = self.read_register(frame, a)?;
                            if value == Value::uninitialized_lexical() {
                                let result =
                                    Err(VmError::Abrupt(errors::throw_reference_error(agent)));
                                let Some(()) = self.handle_vm_result(agent, result)? else {
                                    continue;
                                };
                            }
                            self.advance_instruction()?;
                        }
                        Opcode::CreateClosure => {
                            let closure_result = self.create_closure(agent, frame, bx);
                            let Some(closure) = self.handle_vm_result(agent, closure_result)?
                            else {
                                continue;
                            };
                            self.write_register(frame, a, Value::from_object_ref(closure))?;
                            self.advance_instruction()?;
                        }
                        Opcode::CloseForIn => {
                            let _ = self.for_in_states.remove(frame.registers().base(), a);
                            self.advance_instruction()?;
                        }
                        Opcode::CloseIterator => {
                            let close_result =
                                self.close_iterator_state(agent, host, registry, frame, a, bx != 0);
                            let Some(()) = self.handle_vm_result(agent, close_result)? else {
                                continue;
                            };
                            self.advance_instruction()?;
                        }
                        _ => {
                            return Err(VmError::UnsupportedOpcode {
                                code: frame.code(),
                                instruction_offset: frame.instruction_offset(),
                                opcode,
                            });
                        }
                    }
                }
                Instruction::Ax { opcode, ax } => match opcode {
                    Opcode::Nop => self.advance_instruction()?,
                    Opcode::LoopHeader => {
                        self.observe_tier_backedge_event(frame.code());
                        self.advance_instruction()?;
                    }
                    Opcode::Jump => {
                        if ax < 0 {
                            self.observe_tier_backedge_event(frame.code());
                        }
                        self.jump_by(ax)?;
                    }
                    Opcode::PushClosureEnv => {
                        let site = installed
                            .loop_iteration_environment_site(frame.instruction_offset())
                            .cloned();
                        let mirrored_slot = if ax > 0 {
                            Some(
                                u32::try_from(ax - 1).map_err(|_| VmError::UnsupportedOpcode {
                                    code: frame.code(),
                                    instruction_offset: frame.instruction_offset(),
                                    opcode,
                                })?,
                            )
                        } else {
                            None
                        };
                        self.push_loop_iteration_environment(agent, frame, site, mirrored_slot)?;
                        self.advance_instruction()?;
                    }
                    Opcode::PopClosureEnv => {
                        self.pop_loop_iteration_environment();
                        self.advance_instruction()?;
                    }
                    Opcode::PushWithEnv => {
                        let register =
                            u16::try_from(ax).map_err(|_| VmError::RegisterOutOfBounds {
                                code: frame.code(),
                                register: 0,
                            })?;
                        let value = self.read_register(frame, register)?;
                        let push_result = self.push_with_environment(agent, frame, value);
                        let Some(()) = self.handle_vm_result(agent, push_result)? else {
                            continue;
                        };
                        self.advance_instruction()?;
                    }
                    Opcode::PopWithEnv => {
                        self.pop_with_environment()?;
                        self.advance_instruction()?;
                    }
                    Opcode::TypeOf => {
                        let register =
                            u16::try_from(ax).map_err(|_| VmError::RegisterOutOfBounds {
                                code: frame.code(),
                                register: 0,
                            })?;
                        let value = self.read_register(frame, register)?;
                        let type_string = self.type_of_value(agent, value);
                        self.write_register(frame, register, type_string)?;
                        self.advance_instruction()?;
                    }
                    Opcode::Return => {
                        let register =
                            u16::try_from(ax).map_err(|_| VmError::RegisterOutOfBounds {
                                code: frame.code(),
                                register: 0,
                            })?;
                        let value = self.read_register(frame, register)?;
                        let _ = agent.pop_execution_context();
                        if let Some(result) = self.finish_frame(agent, value)? {
                            return Ok(result);
                        }
                    }
                    Opcode::ReturnUndefined => {
                        let _ = agent.pop_execution_context();
                        if let Some(result) = self.finish_frame(agent, Value::undefined())? {
                            return Ok(result);
                        }
                    }
                    Opcode::SuspendGeneratorStart => {
                        self.suspend_generator_start(
                            agent,
                            frame,
                            frame.instruction_offset().saturating_add(1),
                        )?;
                    }
                    Opcode::Yield => {
                        let register =
                            u16::try_from(ax).map_err(|_| VmError::RegisterOutOfBounds {
                                code: frame.code(),
                                register: 0,
                            })?;
                        let value = self.read_register(frame, register)?;
                        self.suspend_current_generator_frame(
                            agent,
                            frame,
                            value,
                            frame.instruction_offset().saturating_add(1),
                            false,
                        )?;
                    }
                    Opcode::Await => {
                        let register =
                            u16::try_from(ax).map_err(|_| VmError::RegisterOutOfBounds {
                                code: frame.code(),
                                register: 0,
                            })?;
                        self.await_value(agent, host, registry, frame, register)?;
                    }
                    Opcode::Throw => {
                        let register =
                            u16::try_from(ax).map_err(|_| VmError::RegisterOutOfBounds {
                                code: frame.code(),
                                register: 0,
                            })?;
                        let value = self.read_register(frame, register)?;
                        if self.transfer_to_exception_handler(agent, value)? {
                            continue;
                        }
                        return Err(VmError::Abrupt(AbruptCompletion::Throw(value)));
                    }
                    Opcode::EnterHandler | Opcode::LeaveHandler => self.advance_instruction()?,
                    Opcode::LoadException => {
                        let register =
                            u16::try_from(ax).map_err(|_| VmError::RegisterOutOfBounds {
                                code: frame.code(),
                                register: 0,
                            })?;
                        let value = self.current_exception.unwrap_or(Value::undefined());
                        self.write_register(frame, register, value)?;
                        self.advance_instruction()?;
                    }
                    Opcode::LoadResumeKind => {
                        let register =
                            u16::try_from(ax).map_err(|_| VmError::RegisterOutOfBounds {
                                code: frame.code(),
                                register: 0,
                            })?;
                        self.write_register(
                            frame,
                            register,
                            Value::from_smi(i32::from(frame.resume_kind().raw())),
                        )?;
                        self.advance_instruction()?;
                    }
                    Opcode::LoadResumeValue => {
                        let register =
                            u16::try_from(ax).map_err(|_| VmError::RegisterOutOfBounds {
                                code: frame.code(),
                                register: 0,
                            })?;
                        self.write_register(frame, register, frame.resume_value())?;
                        self.clear_active_resume();
                        self.advance_instruction()?;
                    }
                    _ => {
                        return Err(VmError::UnsupportedOpcode {
                            code: frame.code(),
                            instruction_offset: frame.instruction_offset(),
                            opcode,
                        });
                    }
                },
            }
        }
    }

    pub(super) fn execute_abc_value_opcode(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        opcode: Opcode,
        left: u16,
        right: u16,
    ) -> VmResult<Value> {
        if let Some(value) = self.try_primitive_number_binary_opcode(frame, opcode, left, right)? {
            return Ok(value);
        }
        match opcode {
            Opcode::Add => self.add_values(agent, host, registry, frame, left, right),
            Opcode::Sub => self.sub_values(agent, host, registry, frame, left, right),
            Opcode::Mul => self.mul_values(agent, host, registry, frame, left, right),
            Opcode::Div => self.div_values(agent, host, registry, frame, left, right),
            Opcode::Mod => self.rem_values(agent, host, registry, frame, left, right),
            Opcode::Exp => self.exp_values(agent, host, registry, frame, left, right),
            Opcode::BitOr => self.bitwise_or(agent, host, registry, frame, left, right),
            Opcode::BitAnd => self.bitwise_and(agent, host, registry, frame, left, right),
            Opcode::BitXor => self.bitwise_xor(agent, host, registry, frame, left, right),
            Opcode::ShiftLeft => self.shift_left(agent, host, registry, frame, left, right),
            Opcode::ShiftRight => self.shift_right(agent, host, registry, frame, left, right),
            Opcode::UnsignedShiftRight => {
                self.unsigned_shift_right(agent, host, registry, frame, left, right)
            }
            Opcode::Equal => {
                let left = self.read_register(frame, left)?;
                let right = self.read_register(frame, right)?;
                Ok(Value::from_bool(self.loosely_equal(
                    agent, host, registry, frame, left, right,
                )?))
            }
            Opcode::StrictEqual => {
                let left = self.read_register(frame, left)?;
                let right = self.read_register(frame, right)?;
                Ok(Value::from_bool(
                    read::is_strictly_equal(agent.heap().view(), left, right)
                        .map_err(VmError::Abrupt)?,
                ))
            }
            Opcode::LessThan => {
                self.relational_compare(agent, host, registry, frame, left, right, |ordering| {
                    ordering.is_lt()
                })
            }
            Opcode::LessEqual => {
                self.relational_compare(agent, host, registry, frame, left, right, |ordering| {
                    !ordering.is_gt()
                })
            }
            Opcode::GreaterThan => {
                self.relational_compare(agent, host, registry, frame, left, right, |ordering| {
                    ordering.is_gt()
                })
            }
            Opcode::GreaterEqual => {
                self.relational_compare(agent, host, registry, frame, left, right, |ordering| {
                    !ordering.is_lt()
                })
            }
            _ => unreachable!("caller filters supported ABC value opcodes"),
        }
    }

    fn try_primitive_number_binary_opcode(
        &self,
        frame: FrameRecord,
        opcode: Opcode,
        left: u16,
        right: u16,
    ) -> VmResult<Option<Value>> {
        let left = self.read_register(frame, left)?;
        let right = self.read_register(frame, right)?;
        if let (Some(left), Some(right)) = (left.as_smi(), right.as_smi()) {
            let value = match opcode {
                Opcode::Add => encode_number(f64::from(left) + f64::from(right)),
                Opcode::Sub => encode_number(f64::from(left) - f64::from(right)),
                Opcode::Mul => encode_number(f64::from(left) * f64::from(right)),
                Opcode::Div => encode_number(f64::from(left) / f64::from(right)),
                Opcode::Mod => encode_number(f64::from(left) % f64::from(right)),
                Opcode::Exp => encode_number(f64::from(left).powf(f64::from(right))),
                Opcode::BitOr => Value::from_smi(left | right),
                Opcode::BitAnd => Value::from_smi(left & right),
                Opcode::BitXor => Value::from_smi(left ^ right),
                Opcode::ShiftLeft => Value::from_smi(left.wrapping_shl((right & 0x1f) as u32)),
                Opcode::ShiftRight => Value::from_smi(left.wrapping_shr((right & 0x1f) as u32)),
                Opcode::UnsignedShiftRight => {
                    let shifted = (left as u32).wrapping_shr((right & 0x1f) as u32);
                    encode_number(f64::from(shifted))
                }
                Opcode::Equal | Opcode::StrictEqual => Value::from_bool(left == right),
                Opcode::LessThan => Value::from_bool(left < right),
                Opcode::LessEqual => Value::from_bool(left <= right),
                Opcode::GreaterThan => Value::from_bool(left > right),
                Opcode::GreaterEqual => Value::from_bool(left >= right),
                _ => return Ok(None),
            };
            return Ok(Some(value));
        }
        if !left.is_number() || !right.is_number() {
            return Ok(None);
        };
        let left = left
            .as_f64()
            .expect("Number value should expose an f64 payload");
        let right = right
            .as_f64()
            .expect("Number value should expose an f64 payload");
        let value = match opcode {
            Opcode::Add => encode_number(left + right),
            Opcode::Sub => encode_number(left - right),
            Opcode::Mul => encode_number(left * right),
            Opcode::Div => encode_number(left / right),
            Opcode::Mod => encode_number(left % right),
            Opcode::Exp => {
                if left.abs() == 1.0 && right.is_infinite() {
                    Value::from_f64(f64::NAN)
                } else {
                    encode_number(left.powf(right))
                }
            }
            Opcode::BitOr => Value::from_smi(number_to_int32(left) | number_to_int32(right)),
            Opcode::BitAnd => Value::from_smi(number_to_int32(left) & number_to_int32(right)),
            Opcode::BitXor => Value::from_smi(number_to_int32(left) ^ number_to_int32(right)),
            Opcode::ShiftLeft => {
                let left = number_to_int32(left);
                let right = number_to_uint32(right) & 0x1f;
                Value::from_smi(left.wrapping_shl(right))
            }
            Opcode::ShiftRight => {
                let left = number_to_int32(left);
                let right = number_to_uint32(right) & 0x1f;
                Value::from_smi(left >> right)
            }
            Opcode::UnsignedShiftRight => {
                let left = number_to_uint32(left);
                let right = number_to_uint32(right) & 0x1f;
                let result = left >> right;
                if let Ok(result) = i32::try_from(result) {
                    Value::from_smi(result)
                } else {
                    Value::from_f64(f64::from(result))
                }
            }
            Opcode::Equal | Opcode::StrictEqual => Value::from_bool(left == right),
            Opcode::LessThan => {
                Value::from_bool(left.partial_cmp(&right).is_some_and(|o| o.is_lt()))
            }
            Opcode::LessEqual => {
                Value::from_bool(left.partial_cmp(&right).is_some_and(|o| !o.is_gt()))
            }
            Opcode::GreaterThan => {
                Value::from_bool(left.partial_cmp(&right).is_some_and(|o| o.is_gt()))
            }
            Opcode::GreaterEqual => {
                Value::from_bool(left.partial_cmp(&right).is_some_and(|o| !o.is_lt()))
            }
            _ => return Ok(None),
        };
        Ok(Some(value))
    }

    // ECMAScript IsLooselyEqual only converts an Object operand via ToPrimitive
    // when the other side is String/Number/BigInt/Symbol/Boolean. Object vs
    // null/undefined returns false directly, and Object vs Object falls through
    // to strict (reference) equality.
    fn loosely_equal(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        left: Value,
        right: Value,
    ) -> VmResult<bool> {
        if Self::is_html_dda_equal_nullish(agent, left, right) {
            return Ok(true);
        }
        if (left.is_object() && (right.is_null() || right.is_undefined()))
            || (right.is_object() && (left.is_null() || left.is_undefined()))
        {
            return Ok(false);
        }
        if left.is_object() && !right.is_object() {
            let left =
                self.to_primitive(agent, host, registry, frame, left, ToPrimitiveHint::Default)?;
            return self.loosely_equal(agent, host, registry, frame, left, right);
        }
        if right.is_object() && !left.is_object() {
            let right = self.to_primitive(
                agent,
                host,
                registry,
                frame,
                right,
                ToPrimitiveHint::Default,
            )?;
            return self.loosely_equal(agent, host, registry, frame, left, right);
        }

        read::is_loosely_equal(agent.heap().view(), left, right).map_err(VmError::Abrupt)
    }

    fn is_html_dda_equal_nullish(agent: &Agent, left: Value, right: Value) -> bool {
        fn is_html_dda(agent: &Agent, value: Value) -> bool {
            value
                .as_object_ref()
                .is_some_and(|object| agent.objects().is_html_dda_object(object))
        }

        (is_html_dda(agent, left) && (right.is_null() || right.is_undefined()))
            || (is_html_dda(agent, right) && (left.is_null() || left.is_undefined()))
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

    fn to_numeric_register(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        register: u16,
    ) -> VmResult<Value> {
        let primitive = self.to_primitive(
            agent,
            host,
            registry,
            frame,
            self.read_register(frame, register)?,
            ToPrimitiveHint::Number,
        )?;
        self.to_numeric_primitive(agent, primitive)
    }

    fn to_numeric_primitive(&mut self, agent: &mut Agent, value: Value) -> VmResult<Value> {
        read::to_numeric(agent.heap().view(), value)
            .map_err(|abrupt| numeric_conversion_error(agent, abrupt))
    }

    fn update_register_value(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        register: u16,
        increment: bool,
    ) -> VmResult<(Value, Value)> {
        let numeric = self.to_numeric_register(agent, host, registry, frame, register)?;
        let updated = self.update_numeric_value(agent, numeric, increment)?;
        Ok((numeric, updated))
    }

    pub(super) fn relational_compare(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        left_register: u16,
        right_register: u16,
        compare_op: impl FnOnce(std::cmp::Ordering) -> bool,
    ) -> VmResult<Value> {
        let left = self.to_primitive(
            agent,
            host,
            registry,
            frame,
            self.read_register(frame, left_register)?,
            ToPrimitiveHint::Number,
        )?;
        let right = self.to_primitive(
            agent,
            host,
            registry,
            frame,
            self.read_register(frame, right_register)?,
            ToPrimitiveHint::Number,
        )?;
        if left.is_string() && right.is_string() {
            let left_units = self.value_to_string_code_units(agent, left)?;
            let right_units = self.value_to_string_code_units(agent, right)?;
            return Ok(Value::from_bool(compare_op(left_units.cmp(&right_units))));
        }
        if left.is_bigint() && right.is_string() {
            let Some(right) =
                object::string_to_bigint_value(agent, right).map_err(VmError::Abrupt)?
            else {
                return Ok(Value::from_bool(false));
            };
            let ordering = compare_numeric_values(agent, left, right)?
                .expect("BigInt/StringToBigInt comparison must be ordered");
            return Ok(Value::from_bool(compare_op(ordering)));
        }
        if left.is_string() && right.is_bigint() {
            let Some(left) =
                object::string_to_bigint_value(agent, left).map_err(VmError::Abrupt)?
            else {
                return Ok(Value::from_bool(false));
            };
            let ordering = compare_numeric_values(agent, left, right)?
                .expect("StringToBigInt/BigInt comparison must be ordered");
            return Ok(Value::from_bool(compare_op(ordering)));
        }
        let left = self.to_numeric_primitive(agent, left)?;
        let right = self.to_numeric_primitive(agent, right)?;
        let Some(ordering) = compare_numeric_values(agent, left, right)? else {
            return Ok(Value::from_bool(false));
        };
        Ok(Value::from_bool(compare_op(ordering)))
    }

    pub(super) fn bitwise_and(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        left_register: u16,
        right_register: u16,
    ) -> VmResult<Value> {
        let left = self.to_numeric_register(agent, host, registry, frame, left_register)?;
        let right = self.to_numeric_register(agent, host, registry, frame, right_register)?;
        if left.is_bigint() || right.is_bigint() {
            if !left.is_bigint() || !right.is_bigint() {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
            return bigint_bitwise_and_values(agent, left, right);
        }
        let left = number_to_int32(numeric_value_to_f64(left));
        let right = number_to_int32(numeric_value_to_f64(right));
        Ok(Value::from_smi(left & right))
    }

    pub(super) fn bitwise_or(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        left_register: u16,
        right_register: u16,
    ) -> VmResult<Value> {
        let left = self.to_numeric_register(agent, host, registry, frame, left_register)?;
        let right = self.to_numeric_register(agent, host, registry, frame, right_register)?;
        if left.is_bigint() || right.is_bigint() {
            if !left.is_bigint() || !right.is_bigint() {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
            return bigint_bitwise_or_values(agent, left, right);
        }
        let left = number_to_int32(numeric_value_to_f64(left));
        let right = number_to_int32(numeric_value_to_f64(right));
        Ok(Value::from_smi(left | right))
    }

    pub(super) fn bitwise_xor(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        left_register: u16,
        right_register: u16,
    ) -> VmResult<Value> {
        let left = self.to_numeric_register(agent, host, registry, frame, left_register)?;
        let right = self.to_numeric_register(agent, host, registry, frame, right_register)?;
        if left.is_bigint() || right.is_bigint() {
            if !left.is_bigint() || !right.is_bigint() {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
            return bigint_bitwise_xor_values(agent, left, right);
        }
        let left = number_to_int32(numeric_value_to_f64(left));
        let right = number_to_int32(numeric_value_to_f64(right));
        Ok(Value::from_smi(left ^ right))
    }

    pub(super) fn shift_left(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        left_register: u16,
        right_register: u16,
    ) -> VmResult<Value> {
        let left = self.to_numeric_register(agent, host, registry, frame, left_register)?;
        let right = self.to_numeric_register(agent, host, registry, frame, right_register)?;
        if left.is_bigint() || right.is_bigint() {
            if !left.is_bigint() || !right.is_bigint() {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
            return bigint_shift_left_values(agent, left, right);
        }
        let left = number_to_int32(numeric_value_to_f64(left));
        let right = number_to_uint32(numeric_value_to_f64(right)) & 0x1f;
        Ok(Value::from_smi(left.wrapping_shl(right)))
    }

    pub(super) fn shift_right(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        left_register: u16,
        right_register: u16,
    ) -> VmResult<Value> {
        let left = self.to_numeric_register(agent, host, registry, frame, left_register)?;
        let right = self.to_numeric_register(agent, host, registry, frame, right_register)?;
        if left.is_bigint() || right.is_bigint() {
            if !left.is_bigint() || !right.is_bigint() {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
            return bigint_shift_right_values(agent, left, right);
        }
        let left = number_to_int32(numeric_value_to_f64(left));
        let right = number_to_uint32(numeric_value_to_f64(right)) & 0x1f;
        Ok(Value::from_smi(left >> right))
    }

    pub(super) fn unsigned_shift_right(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        left_register: u16,
        right_register: u16,
    ) -> VmResult<Value> {
        let left = self.to_numeric_register(agent, host, registry, frame, left_register)?;
        let right = self.to_numeric_register(agent, host, registry, frame, right_register)?;
        if left.is_bigint() || right.is_bigint() {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        let left = number_to_uint32(numeric_value_to_f64(left));
        let right = number_to_uint32(numeric_value_to_f64(right)) & 0x1f;
        let result = left >> right;
        if let Ok(result) = i32::try_from(result) {
            Ok(Value::from_smi(result))
        } else {
            Ok(Value::from_f64(f64::from(result)))
        }
    }
}

fn numeric_conversion_error(agent: &mut Agent, abrupt: AbruptCompletion) -> VmError {
    match abrupt {
        AbruptCompletion::Throw(value) if value.is_undefined() => {
            VmError::Abrupt(errors::throw_type_error(agent))
        }
        abrupt => VmError::Abrupt(abrupt),
    }
}

fn numeric_value_to_f64(value: Value) -> f64 {
    value
        .as_f64()
        .expect("numeric non-BigInt Value should expose an f64 payload")
}

fn number_to_int32(number: f64) -> i32 {
    if !number.is_finite() || number == 0.0 {
        return 0;
    }
    let truncated = number.trunc();
    let modulo = truncated.rem_euclid(4_294_967_296.0);
    if modulo >= 2_147_483_648.0 {
        (modulo - 4_294_967_296.0) as i32
    } else {
        modulo as i32
    }
}

fn number_to_uint32(number: f64) -> u32 {
    if !number.is_finite() || number == 0.0 {
        return 0;
    }
    let truncated = number.trunc();
    truncated.rem_euclid(4_294_967_296.0) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FrameRecord, RegisterWindow};
    use lyng_js_env::ExecutionContextKind;
    use lyng_js_types::{CodeRef, EnvironmentRef, RealmRef};

    fn test_frame() -> FrameRecord {
        FrameRecord::new(
            CodeRef::from_raw(1).expect("test code ref should be non-zero"),
            0,
            RegisterWindow::new(0, 2),
            None,
            RealmRef::from_raw(1).expect("test realm ref should be non-zero"),
            EnvironmentRef::from_raw(1).expect("test lexical env should be non-zero"),
            EnvironmentRef::from_raw(1).expect("test variable env should be non-zero"),
            ExecutionContextKind::Script,
        )
    }

    #[test]
    fn number_binary_opcode_fast_path_handles_double_operands() {
        let mut vm = Vm::new();
        vm.register_stack = vec![Value::from_f64(1.5), Value::from_f64(2.25)];

        let value = vm
            .try_primitive_number_binary_opcode(test_frame(), Opcode::Add, 0, 1)
            .expect("register reads should succeed");

        assert_eq!(
            value.and_then(Value::as_f64),
            Some(3.75),
            "primitive number addition should avoid the generic conversion path"
        );
    }
}
