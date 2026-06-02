//! DSL handler functions.
//!
//! Each opcode's asm-DSL handler is a `#[unsafe(naked)] extern "C" fn`
//! that the trampoline tail-jumps to via `DSL_DISPATCH_TABLE`. Handler
//! bodies are emitted by `lyng_vm_dsl::llint_handler!` across three
//! temperature families:
//!
//! - [`hot`]  — highest-frequency opcodes with inline SMI/bool fast paths.
//! - [`warm`] — mid-frequency opcodes that need a backedge safepoint poll
//!   or prefix decode.
//! - [`cold`] — all remaining opcodes lowered to `call_slow!`-only stubs.

#![allow(
    clippy::empty_loop,
    clippy::too_many_lines,
    reason = "The DSL dispatch table is intentionally a long const opcode map, and the placeholder handler is a non-returning invalid-bytecode sink"
)]

pub mod cold;
pub mod hot;
pub mod warm;

/// Calling convention for a DSL handler. The asm trampoline tail-calls
/// these via the dispatch table indexed by opcode byte; handlers never
/// return — they either tail-jump to the next handler or branch to
/// `_interpreter_exit`.
pub type DslHandler = unsafe extern "C" fn() -> !;

/// Placeholder for unimplemented opcode bytes (153..256). Valid bytecode
/// never reaches this; it fires only on corrupted bytecode.
const unsafe extern "C" fn unimplemented_dsl_handler() -> ! {
    loop {} // SAFETY: never reachable on valid bytecode.
}

// =====================================================================
// DSL dispatch table — one slot per opcode byte (0..256).
//
// The mapping is hand-written (not derived from the manifest) because
// static initializers can't loop in const context and the proc-macro
// emits function pointers directly. Adding an opcode requires:
// (a) extending `Opcode`, (b) regenerating cold.rs via
// `cargo run -p lyng-dsl-codegen`, and (c) adding the entry here.
// `dsl_dispatch_table_resolves_every_opcode` catches drift.
// =====================================================================

/// Dispatch table indexed by `Opcode as u8`.
#[allow(dead_code)]
#[cfg(target_arch = "aarch64")]
pub static DSL_DISPATCH_TABLE: [DslHandler; 256] = build_dispatch_table();

/// Non-aarch64 hosts get the all-placeholder table.
#[allow(dead_code)]
#[cfg(not(target_arch = "aarch64"))]
pub static DSL_DISPATCH_TABLE: [DslHandler; 256] = [unimplemented_dsl_handler; 256];

/// Build the dispatch table at const-eval time.
#[cfg(target_arch = "aarch64")]
const fn build_dispatch_table() -> [DslHandler; 256] {
    use lyng_bytecode::Opcode;

    let mut table: [DslHandler; 256] = [unimplemented_dsl_handler; 256];
    // Assign one slot; `Opcode as u8` is const-castable (`#[repr(u8)]`).
    macro_rules! assign {
        ($op:expr, $handler:path) => {
            table[$op as usize] = $handler;
        };
    }

    // Hot opcodes.
    assign!(Opcode::Move, hot::op_move);
    assign!(Opcode::Add, hot::op_add);
    assign!(Opcode::Jump, hot::op_jump);
    assign!(Opcode::Return, hot::op_return);

    // Warm opcodes.
    assign!(Opcode::Jump8, warm::op_jump8);
    assign!(Opcode::JumpIfTrue, warm::op_jump_if_true);
    assign!(Opcode::JumpIfTrue8, warm::op_jump_if_true8);
    assign!(Opcode::JumpIfFalse, warm::op_jump_if_false);
    assign!(Opcode::JumpIfFalse8, warm::op_jump_if_false8);
    assign!(Opcode::LoopHeader, warm::op_loop_header);
    assign!(Opcode::Wide, warm::op_wide);
    assign!(Opcode::ExtraWide, warm::op_extra_wide);

    // Cold opcodes — all carry the `_dsl` suffix.

    // -- loads family --
    assign!(Opcode::LoadUndefined, cold::op_load_undefined_dsl);
    assign!(
        Opcode::LoadUninitializedLexical,
        cold::op_load_uninitialized_lexical_dsl
    );
    assign!(Opcode::LoadNull, cold::op_load_null_dsl);
    assign!(Opcode::LoadTrue, cold::op_load_true_dsl);
    assign!(Opcode::LoadFalse, cold::op_load_false_dsl);
    assign!(Opcode::LoadZero, cold::op_load_zero_dsl);
    assign!(Opcode::LoadOne, cold::op_load_one_dsl);
    assign!(Opcode::LoadSmi, cold::op_load_smi_dsl);
    assign!(Opcode::LoadConst, cold::op_load_const_dsl);
    assign!(Opcode::LdaUndefined, cold::op_lda_undefined_dsl);
    assign!(Opcode::LdaNull, cold::op_lda_null_dsl);
    assign!(Opcode::LdaTrue, cold::op_lda_true_dsl);
    assign!(Opcode::LdaFalse, cold::op_lda_false_dsl);
    assign!(Opcode::LdaZero, cold::op_lda_zero_dsl);
    assign!(Opcode::LdaOne, cold::op_lda_one_dsl);
    assign!(Opcode::LdaSmi8, cold::op_lda_smi8_dsl);
    assign!(Opcode::LdaConst8, cold::op_lda_const8_dsl);
    assign!(Opcode::Ldar, cold::op_ldar_dsl);
    assign!(Opcode::LoadSmi8, cold::op_load_smi8_dsl);
    assign!(Opcode::LoadConst8, cold::op_load_const8_dsl);
    assign!(Opcode::Star0, cold::op_star_0_dsl);
    assign!(Opcode::Star1, cold::op_star_1_dsl);
    assign!(Opcode::Star2, cold::op_star_2_dsl);
    assign!(Opcode::Star3, cold::op_star_3_dsl);
    assign!(Opcode::Star4, cold::op_star_4_dsl);
    assign!(Opcode::Star5, cold::op_star_5_dsl);
    assign!(Opcode::Star6, cold::op_star_6_dsl);
    assign!(Opcode::Star7, cold::op_star_7_dsl);
    assign!(Opcode::LoadLocal0, cold::op_load_local_0_dsl);
    assign!(Opcode::LoadLocal1, cold::op_load_local_1_dsl);
    assign!(Opcode::LoadLocal2, cold::op_load_local_2_dsl);
    assign!(Opcode::LoadLocal3, cold::op_load_local_3_dsl);
    assign!(Opcode::StoreLocal0, cold::op_store_local_0_dsl);
    assign!(Opcode::StoreLocal1, cold::op_store_local_1_dsl);
    assign!(Opcode::StoreLocal2, cold::op_store_local_2_dsl);
    assign!(Opcode::StoreLocal3, cold::op_store_local_3_dsl);

    // -- arithmetic family --
    assign!(Opcode::AddSmi, cold::op_add_smi_dsl);
    assign!(Opcode::Sub, cold::op_sub_dsl);
    assign!(Opcode::SubSmi, cold::op_sub_smi_dsl);
    assign!(Opcode::Mul, cold::op_mul_dsl);
    assign!(Opcode::MulSmi, cold::op_mul_smi_dsl);
    assign!(Opcode::Div, cold::op_div_dsl);
    assign!(Opcode::DivSmi, cold::op_div_smi_dsl);
    assign!(Opcode::Mod, cold::op_mod_dsl);
    assign!(Opcode::ModSmi, cold::op_mod_smi_dsl);
    assign!(Opcode::Exp, cold::op_exp_dsl);
    assign!(Opcode::BitOr, cold::op_bit_or_dsl);
    assign!(Opcode::BitXor, cold::op_bit_xor_dsl);
    assign!(Opcode::BitAnd, cold::op_bit_and_dsl);
    assign!(Opcode::BitAndSmi, cold::op_bit_and_smi_dsl);
    assign!(Opcode::BitNot, cold::op_bit_not_dsl);
    assign!(Opcode::ShiftLeft, cold::op_shift_left_dsl);
    assign!(Opcode::ShiftRight, cold::op_shift_right_dsl);
    assign!(
        Opcode::UnsignedShiftRight,
        cold::op_unsigned_shift_right_dsl
    );
    assign!(Opcode::Negate, cold::op_negate_dsl);
    assign!(Opcode::Increment, cold::op_increment_dsl);
    assign!(Opcode::Decrement, cold::op_decrement_dsl);
    assign!(Opcode::Equal, cold::op_equal_dsl);
    assign!(Opcode::StrictEqual, cold::op_strict_equal_dsl);
    assign!(Opcode::EqualZero, cold::op_equal_zero_dsl);
    assign!(Opcode::LessThan, cold::op_less_than_dsl);
    assign!(Opcode::LessEqual, cold::op_less_equal_dsl);
    assign!(Opcode::GreaterThan, cold::op_greater_than_dsl);
    assign!(Opcode::GreaterEqual, cold::op_greater_equal_dsl);

    // -- control_flow family --
    assign!(Opcode::ReturnUndefined, cold::op_return_undefined_dsl);
    assign!(Opcode::Nop, cold::op_nop_dsl);

    // -- property family --
    assign!(Opcode::GetNamedProperty, cold::op_get_named_property_dsl);
    assign!(Opcode::SetNamedProperty, cold::op_set_named_property_dsl);
    assign!(
        Opcode::AssignNamedProperty,
        cold::op_assign_named_property_dsl
    );
    assign!(
        Opcode::StrictAssignNamedProperty,
        cold::op_strict_assign_named_property_dsl
    );
    assign!(Opcode::GetKeyedProperty, cold::op_get_keyed_property_dsl);
    assign!(Opcode::SetKeyedProperty, cold::op_set_keyed_property_dsl);
    assign!(
        Opcode::AssignKeyedProperty,
        cold::op_assign_keyed_property_dsl
    );
    assign!(
        Opcode::StrictAssignKeyedProperty,
        cold::op_strict_assign_keyed_property_dsl
    );
    assign!(
        Opcode::DefineNamedProperty,
        cold::op_define_named_property_dsl
    );
    assign!(
        Opcode::DefineKeyedProperty,
        cold::op_define_keyed_property_dsl
    );
    assign!(Opcode::CreateObject, cold::op_create_object_dsl);
    assign!(Opcode::CreateArray, cold::op_create_array_dsl);
    assign!(Opcode::StoreDenseElement, cold::op_store_dense_element_dsl);
    assign!(Opcode::LoadDenseElement, cold::op_load_dense_element_dsl);
    assign!(Opcode::DeleteProperty, cold::op_delete_property_dsl);
    assign!(Opcode::In, cold::op_in_dsl);
    assign!(Opcode::ToPropertyKey, cold::op_to_property_key_dsl);
    assign!(
        Opcode::CopyDataProperties,
        cold::op_copy_data_properties_dsl
    );
    assign!(Opcode::SetFunctionName, cold::op_set_function_name_dsl);
    assign!(
        Opcode::CheckObjectCoercible,
        cold::op_check_object_coercible_dsl
    );
    assign!(
        Opcode::ThrowIfUninitialized,
        cold::op_throw_if_uninitialized_dsl
    );

    // -- names family --
    assign!(Opcode::LoadGlobal, cold::op_load_global_dsl);
    assign!(Opcode::StoreGlobal, cold::op_store_global_dsl);
    assign!(Opcode::AssignGlobal, cold::op_assign_global_dsl);
    assign!(Opcode::DeleteGlobal, cold::op_delete_global_dsl);
    assign!(Opcode::LoadName, cold::op_load_name_dsl);
    assign!(Opcode::ResolveName, cold::op_resolve_name_dsl);
    assign!(Opcode::ResolveGlobal, cold::op_resolve_global_dsl);
    assign!(Opcode::AssignName, cold::op_assign_name_dsl);
    assign!(
        Opcode::AssignVariableName,
        cold::op_assign_variable_name_dsl
    );
    assign!(Opcode::DeleteName, cold::op_delete_name_dsl);
    assign!(Opcode::CaptureName, cold::op_capture_name_dsl);
    assign!(Opcode::LoadCapturedName, cold::op_load_captured_name_dsl);
    assign!(
        Opcode::LoadCapturedNameThis,
        cold::op_load_captured_name_this_dsl
    );
    assign!(
        Opcode::AssignCapturedName,
        cold::op_assign_captured_name_dsl
    );
    assign!(Opcode::LoadThis, cold::op_load_this_dsl);
    assign!(Opcode::LoadCallee, cold::op_load_callee_dsl);
    assign!(Opcode::LoadNewTarget, cold::op_load_new_target_dsl);

    // -- scope family --
    assign!(Opcode::LoadEnvSlot, cold::op_load_env_slot_dsl);
    assign!(Opcode::StoreEnvSlot, cold::op_store_env_slot_dsl);
    assign!(Opcode::AssignEnvSlot, cold::op_assign_env_slot_dsl);
    assign!(Opcode::EnterEnvScope, cold::op_enter_env_scope_dsl);
    assign!(Opcode::LeaveEnvScope, cold::op_leave_env_scope_dsl);
    assign!(Opcode::PushClosureEnv, cold::op_push_closure_env_dsl);
    assign!(Opcode::PopClosureEnv, cold::op_pop_closure_env_dsl);
    assign!(Opcode::PushWithEnv, cold::op_push_with_env_dsl);
    assign!(Opcode::PopWithEnv, cold::op_pop_with_env_dsl);
    assign!(Opcode::TypeOf, cold::op_type_of_dsl);

    // -- calls family --
    assign!(Opcode::Call0, cold::op_call0_dsl);
    assign!(Opcode::Call1, cold::op_call1_dsl);
    assign!(Opcode::Call2, cold::op_call2_dsl);
    assign!(Opcode::Call3, cold::op_call3_dsl);
    assign!(Opcode::Call, cold::op_call_dsl);
    assign!(Opcode::TailCall, cold::op_tail_call_dsl);
    assign!(Opcode::Construct, cold::op_construct_dsl);
    assign!(Opcode::CreateClosure, cold::op_create_closure_dsl);

    // -- iterators family --
    assign!(Opcode::CreateForIn, cold::op_create_for_in_dsl);
    assign!(Opcode::AdvanceForIn, cold::op_advance_for_in_dsl);
    assign!(Opcode::CloseForIn, cold::op_close_for_in_dsl);
    assign!(Opcode::CreateIterator, cold::op_create_iterator_dsl);
    assign!(Opcode::AdvanceIterator, cold::op_advance_iterator_dsl);
    assign!(Opcode::CloseIterator, cold::op_close_iterator_dsl);

    // -- generators family --
    assign!(
        Opcode::SuspendGeneratorStart,
        cold::op_suspend_generator_start_dsl
    );
    assign!(Opcode::Yield, cold::op_yield_dsl);
    assign!(Opcode::DelegateYield, cold::op_delegate_yield_dsl);
    assign!(Opcode::Await, cold::op_await_dsl);
    assign!(Opcode::LoadResumeKind, cold::op_load_resume_kind_dsl);
    assign!(Opcode::LoadResumeValue, cold::op_load_resume_value_dsl);

    // -- exceptions family --
    assign!(Opcode::Throw, cold::op_throw_dsl);
    assign!(Opcode::EnterHandler, cold::op_enter_handler_dsl);
    assign!(Opcode::LeaveHandler, cold::op_leave_handler_dsl);
    assign!(Opcode::LoadException, cold::op_load_exception_dsl);

    // -- misc --
    assign!(Opcode::InstanceOf, cold::op_instance_of_dsl);
    assign!(Opcode::CallMethod, cold::op_call_method_dsl);

    table
}
