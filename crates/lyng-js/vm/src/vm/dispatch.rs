use super::values::decode_env_operand;
use super::{
    code_index, Agent, CallRange, HostHooks, Instruction, NativeFunctionRegistry, Opcode,
    ThisState, Value, Vm, VmError, VmResult,
};
use lyng_js_ops::{errors, read};
use lyng_js_types::{AbruptCompletion, PropertyKey};

mod arithmetic;
mod property;

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

    #[expect(
        clippy::too_many_lines,
        reason = "spec-shaped VM algorithm stays contiguous until the VM module split issue extracts it"
    )]
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
                .ok_or_else(|| VmError::MissingInstalledCode(frame.code()))?;
            let instruction = installed
                .function
                .instructions()
                .get(
                    usize::try_from(frame.instruction_offset())
                        .expect("instruction offset should fit into usize"),
                )
                .copied()
                .ok_or_else(|| VmError::InstructionOutOfBounds {
                    code: frame.code(),
                    instruction_offset: frame.instruction_offset(),
                })?;

            match instruction {
                Instruction::Abc { opcode, a, b, c } => {
                    let (a, b, c) = Self::decode_abc_operands(installed, frame, opcode, a, b, c);
                    match opcode {
                        Opcode::Move => {
                            let value = self.read_register(frame, b)?;
                            self.write_register(frame, a, value)?;
                            self.advance_instruction();
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
                            self.advance_instruction();
                        }
                        Opcode::In => {
                            self.execute_in_opcode(agent, host, registry, frame, a, b, c)?;
                        }
                        Opcode::Negate => {
                            let negate_result = self.negate_value(agent, host, registry, frame, b);
                            let Some(value) = self.handle_vm_result(agent, negate_result)? else {
                                continue;
                            };
                            self.record_feedback_site(frame.code(), frame.instruction_offset());
                            self.write_register(frame, a, value)?;
                            self.advance_instruction();
                        }
                        Opcode::BitNot => {
                            let bit_not_result =
                                self.bitwise_not_value(agent, host, registry, frame, b);
                            let Some(value) = self.handle_vm_result(agent, bit_not_result)? else {
                                continue;
                            };
                            self.record_feedback_site(frame.code(), frame.instruction_offset());
                            self.write_register(frame, a, value)?;
                            self.advance_instruction();
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
                            self.advance_instruction();
                        }
                        Opcode::GetNamedProperty => {
                            self.execute_get_named_property_opcode(
                                agent, host, registry, frame, a, b, c,
                            )?;
                        }
                        Opcode::SetNamedProperty
                        | Opcode::AssignNamedProperty
                        | Opcode::StrictAssignNamedProperty => {
                            self.execute_set_named_property_opcode(
                                agent, host, registry, frame, opcode, a, b, c,
                            )?;
                        }
                        Opcode::DefineNamedProperty => {
                            self.execute_define_named_property_opcode(
                                agent, host, registry, frame, a, b, c,
                            )?;
                        }
                        Opcode::GetKeyedProperty => {
                            self.execute_get_keyed_property_opcode(
                                agent, host, registry, frame, a, b, c,
                            )?;
                        }
                        Opcode::SetKeyedProperty
                        | Opcode::AssignKeyedProperty
                        | Opcode::StrictAssignKeyedProperty => {
                            self.execute_set_keyed_property_opcode(
                                agent, host, registry, frame, opcode, a, b, c,
                            )?;
                        }
                        Opcode::DefineKeyedProperty => {
                            self.execute_define_keyed_property_opcode(
                                agent, host, registry, frame, a, b, c,
                            )?;
                        }
                        Opcode::SetFunctionName => {
                            let function = self.object_register(frame, a)?;
                            let name_value = self.read_register(frame, b)?;
                            let set_result = Self::set_function_name(agent, function, name_value);
                            let Some(()) = self.handle_vm_result(agent, set_result)? else {
                                continue;
                            };
                            self.advance_instruction();
                        }
                        Opcode::ToPropertyKey => {
                            self.execute_to_property_key_opcode(
                                agent, host, registry, frame, a, b,
                            )?;
                        }
                        Opcode::DeleteProperty => {
                            self.execute_delete_property_opcode(
                                agent, host, registry, frame, a, b, c,
                            )?;
                        }
                        Opcode::CopyDataProperties => {
                            self.execute_copy_data_properties_opcode(
                                agent, host, registry, frame, a, b, c,
                            )?;
                        }
                        Opcode::StoreDenseElement => {
                            self.execute_store_dense_element_opcode(
                                agent, host, registry, frame, a, b, c,
                            )?;
                        }
                        Opcode::LoadDenseElement => {
                            self.execute_load_dense_element_opcode(
                                agent, host, registry, frame, a, b, c,
                            )?;
                        }
                        Opcode::Call => {
                            let payload = installed
                                .wide_payload(frame.instruction_offset())
                                .ok_or_else(|| VmError::MissingWideOperand {
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
                            let Some(()) = self.handle_vm_result(agent, call_result)? else {
                                continue;
                            };
                            self.record_feedback_site(frame.code(), frame.instruction_offset());
                        }
                        Opcode::TailCall => {
                            let payload = installed
                                .wide_payload(frame.instruction_offset())
                                .ok_or_else(|| VmError::MissingWideOperand {
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
                                .ok_or_else(|| VmError::MissingWideOperand {
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
                            let Some(()) = self.handle_vm_result(agent, construct_result)? else {
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
                            self.advance_instruction();
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
                            self.advance_instruction();
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
                            self.advance_instruction();
                        }
                        Opcode::AdvanceIterator => {
                            let next = self.advance_iterator_state(agent, host, registry, frame, a);
                            let Some(next) = self.handle_vm_result(agent, next)? else {
                                continue;
                            };
                            let done = next.is_none();
                            self.write_register(frame, b, next.unwrap_or(Value::undefined()))?;
                            self.write_register(frame, c, Value::from_bool(done))?;
                            self.advance_instruction();
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
                    let (a, bx) = Self::decode_abx_operands(installed, frame, a, bx);
                    match opcode {
                        Opcode::LoadUndefined => {
                            self.write_register(frame, a, Value::undefined())?;
                            self.advance_instruction();
                        }
                        Opcode::LoadUninitializedLexical => {
                            self.write_register(frame, a, Value::uninitialized_lexical())?;
                            self.advance_instruction();
                        }
                        Opcode::LoadNull => {
                            self.write_register(frame, a, Value::null())?;
                            self.advance_instruction();
                        }
                        Opcode::LoadTrue => {
                            self.write_register(frame, a, Value::from_bool(true))?;
                            self.advance_instruction();
                        }
                        Opcode::LoadFalse => {
                            self.write_register(frame, a, Value::from_bool(false))?;
                            self.advance_instruction();
                        }
                        Opcode::LoadSmi => {
                            let bytes = bx.to_le_bytes();
                            let value = i16::from_le_bytes([bytes[0], bytes[1]]);
                            self.write_register(frame, a, Value::from_smi(i32::from(value)))?;
                            self.advance_instruction();
                        }
                        Opcode::LoadConst => {
                            let value = self.read_constant(agent, frame.code(), bx)?;
                            self.write_register(frame, a, value)?;
                            self.advance_instruction();
                        }
                        Opcode::LoadEnvSlot => {
                            let (depth, slot) = decode_env_operand(bx);
                            let environment = self.environment_for_slot_access(
                                agent,
                                frame.lexical_env(),
                                depth,
                                slot,
                            )?;
                            let slot_value = Self::read_environment_slot(agent, environment, slot);
                            let Some(value) = self.handle_vm_result(agent, slot_value)? else {
                                continue;
                            };
                            self.write_register(frame, a, value)?;
                            self.advance_instruction();
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
                            self.advance_instruction();
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
                            self.advance_instruction();
                        }
                        Opcode::EnterEnvScope => {
                            self.enter_env_scope(agent, frame, a, bx)?;
                            self.advance_instruction();
                        }
                        Opcode::LeaveEnvScope => {
                            self.leave_env_scope(frame, a, bx);
                            self.advance_instruction();
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
                            self.advance_instruction();
                        }
                        Opcode::LoadName => {
                            let atom = self.read_atom_constant(frame.code(), bx)?;
                            let load_result =
                                self.load_name_with_context(agent, host, registry, frame, atom);
                            let Some(value) = self.handle_vm_result(agent, load_result)? else {
                                continue;
                            };
                            self.write_register(frame, a, value)?;
                            self.advance_instruction();
                        }
                        Opcode::ResolveName => {
                            let atom = self.read_atom_constant(frame.code(), bx)?;
                            let resolve_result =
                                self.resolve_name_with_context(agent, host, registry, frame, atom);
                            let Some(value) = self.handle_vm_result(agent, resolve_result)? else {
                                continue;
                            };
                            self.write_register(frame, a, value)?;
                            self.advance_instruction();
                        }
                        Opcode::ResolveGlobal => {
                            let atom = self.read_atom_constant(frame.code(), bx)?;
                            let resolve_result =
                                self.resolve_global(agent, host, registry, frame, atom);
                            let Some(value) = self.handle_vm_result(agent, resolve_result)? else {
                                continue;
                            };
                            self.write_register(frame, a, value)?;
                            self.advance_instruction();
                        }
                        Opcode::AssignName => {
                            let atom = self.read_atom_constant(frame.code(), bx)?;
                            let value = self.read_register(frame, a)?;
                            let assign_result = self.assign_name_with_context(
                                agent, host, registry, frame, atom, value,
                            );
                            let Some(()) = self.handle_vm_result(agent, assign_result)? else {
                                continue;
                            };
                            self.advance_instruction();
                        }
                        Opcode::AssignVariableName => {
                            let atom = self.read_atom_constant(frame.code(), bx)?;
                            let value = self.read_register(frame, a)?;
                            let assign_result = self.assign_variable_name_with_context(
                                agent, host, registry, frame, atom, value,
                            );
                            let Some(()) = self.handle_vm_result(agent, assign_result)? else {
                                continue;
                            };
                            self.advance_instruction();
                        }
                        Opcode::DeleteName => {
                            let atom = self.read_atom_constant(frame.code(), bx)?;
                            let delete_result =
                                self.delete_name_with_context(agent, host, registry, frame, atom);
                            let Some(deleted) = self.handle_vm_result(agent, delete_result)? else {
                                continue;
                            };
                            self.write_register(frame, a, Value::from_bool(deleted))?;
                            self.advance_instruction();
                        }
                        Opcode::CaptureName => {
                            let atom = self.read_atom_constant(frame.code(), bx)?;
                            let capture_result = self
                                .capture_name_with_context(agent, host, registry, frame, a, atom);
                            let Some(()) = self.handle_vm_result(agent, capture_result)? else {
                                continue;
                            };
                            self.advance_instruction();
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
                            self.advance_instruction();
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
                            self.advance_instruction();
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
                            self.advance_instruction();
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
                            let Some(()) = self.handle_vm_result(agent, store_result)? else {
                                continue;
                            };
                            self.advance_instruction();
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
                            let Some(()) = self.handle_vm_result(agent, assign_result)? else {
                                continue;
                            };
                            self.advance_instruction();
                        }
                        Opcode::DeleteGlobal => {
                            let atom = self.read_atom_constant(frame.code(), bx)?;
                            let delete_result = Self::delete_global(agent, frame, atom);
                            let Some(deleted) = self.handle_vm_result(agent, delete_result)? else {
                                continue;
                            };
                            self.write_register(frame, a, Value::from_bool(deleted))?;
                            self.advance_instruction();
                        }
                        Opcode::LoadThis => {
                            let load_this = match agent.current_execution_context().map_or_else(
                                || ThisState::Value(frame.this_value()),
                                lyng_js_env::ExecutionContext::this_state,
                            ) {
                                ThisState::Value(value) => Ok(value),
                                ThisState::Uninitialized => {
                                    Err(VmError::Abrupt(errors::throw_reference_error(agent)))
                                }
                                ThisState::Lexical => {
                                    Self::resolve_this_binding(agent, frame.lexical_env(), frame)
                                }
                            };
                            let Some(value) = self.handle_vm_result(agent, load_this)? else {
                                continue;
                            };
                            self.write_register(frame, a, value)?;
                            self.advance_instruction();
                        }
                        Opcode::LoadCallee => {
                            let value = frame
                                .callee()
                                .map_or(Value::undefined(), Value::from_object_ref);
                            self.write_register(frame, a, value)?;
                            self.advance_instruction();
                        }
                        Opcode::LoadNewTarget => {
                            let value = frame
                                .new_target()
                                .map_or(Value::undefined(), Value::from_object_ref);
                            self.write_register(frame, a, value)?;
                            self.advance_instruction();
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
                                self.advance_instruction();
                            }
                        }
                        Opcode::CreateObject => {
                            let object = Self::create_object(
                                agent,
                                frame.realm(),
                                usize::try_from(bx).unwrap_or(usize::MAX),
                            )?;
                            self.write_register(frame, a, Value::from_object_ref(object))?;
                            self.advance_instruction();
                        }
                        Opcode::CreateArray => {
                            let length = usize::try_from(bx).unwrap_or(usize::MAX);
                            let object = Self::create_array(agent, frame.realm(), length)?;
                            let length = u32::try_from(length).unwrap_or(u32::MAX);
                            if length != 0 {
                                Self::define_length_property(agent, object, length, false)?;
                            }
                            self.write_register(frame, a, Value::from_object_ref(object))?;
                            self.advance_instruction();
                        }
                        Opcode::CheckObjectCoercible => {
                            let value = self.read_register(frame, a)?;
                            let coercible = Self::check_object_coercible(agent, value);
                            let Some(()) = self.handle_vm_result(agent, coercible)? else {
                                continue;
                            };
                            self.advance_instruction();
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
                            self.advance_instruction();
                        }
                        Opcode::CreateClosure => {
                            let closure_result = self.create_closure(agent, frame, bx);
                            let Some(closure) = self.handle_vm_result(agent, closure_result)?
                            else {
                                continue;
                            };
                            self.write_register(frame, a, Value::from_object_ref(closure))?;
                            self.advance_instruction();
                        }
                        Opcode::CloseForIn => {
                            let _ = self.for_in_states.remove(frame.registers().base(), a);
                            self.advance_instruction();
                        }
                        Opcode::CloseIterator => {
                            let close_result =
                                self.close_iterator_state(agent, host, registry, frame, a, bx != 0);
                            let Some(()) = self.handle_vm_result(agent, close_result)? else {
                                continue;
                            };
                            self.advance_instruction();
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
                #[allow(
                    clippy::match_same_arms,
                    reason = "opcode families stay grouped even when marker opcodes share dispatch behavior"
                )]
                Instruction::Ax { opcode, ax } => match opcode {
                    Opcode::Nop => self.advance_instruction(),
                    Opcode::LoopHeader => {
                        self.observe_tier_backedge_event(frame.code());
                        self.advance_instruction();
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
                        self.advance_instruction();
                    }
                    Opcode::PopClosureEnv => {
                        self.pop_loop_iteration_environment();
                        self.advance_instruction();
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
                        self.advance_instruction();
                    }
                    Opcode::PopWithEnv => {
                        self.pop_with_environment();
                        self.advance_instruction();
                    }
                    Opcode::TypeOf => {
                        let register =
                            u16::try_from(ax).map_err(|_| VmError::RegisterOutOfBounds {
                                code: frame.code(),
                                register: 0,
                            })?;
                        let value = self.read_register(frame, register)?;
                        let type_string = Self::type_of_value(agent, value);
                        self.write_register(frame, register, type_string)?;
                        self.advance_instruction();
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
                    Opcode::EnterHandler | Opcode::LeaveHandler => self.advance_instruction(),
                    Opcode::LoadException => {
                        let register =
                            u16::try_from(ax).map_err(|_| VmError::RegisterOutOfBounds {
                                code: frame.code(),
                                register: 0,
                            })?;
                        let value = self.current_exception.unwrap_or(Value::undefined());
                        self.write_register(frame, register, value)?;
                        self.advance_instruction();
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
                        self.advance_instruction();
                    }
                    Opcode::LoadResumeValue => {
                        let register =
                            u16::try_from(ax).map_err(|_| VmError::RegisterOutOfBounds {
                                code: frame.code(),
                                register: 0,
                            })?;
                        self.write_register(frame, register, frame.resume_value())?;
                        self.clear_active_resume();
                        self.advance_instruction();
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
}
