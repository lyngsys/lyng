use crate::function::BytecodeFunction;
use crate::instruction::Instruction;
use crate::metadata::{
    CallRange, ConstantValue, ExceptionHandlerKind, FeedbackSiteMetadata, ThisMode,
    WideAbcOperands, WideAbxOperands,
};
use std::fmt::Write;

fn format_constant(constant: &ConstantValue) -> String {
    match constant {
        ConstantValue::Smi(value) => format!("Smi({value})"),
        ConstantValue::Float64Bits(bits) => format!("Float64({})", f64::from_bits(*bits)),
        ConstantValue::Atom(atom) => format!("Atom({atom:?})"),
        ConstantValue::Builtin(id) => format!("Builtin({id:?})"),
    }
}

fn format_atom_operand(function: &BytecodeFunction, index: u32) -> String {
    match usize::try_from(index)
        .ok()
        .and_then(|index| function.constants().get(index))
    {
        Some(ConstantValue::Atom(atom)) => format!("atom[{index}] ; Atom({atom:?})"),
        _ => format!("atom[{index}]"),
    }
}

fn format_this_mode(this_mode: ThisMode) -> &'static str {
    match this_mode {
        ThisMode::Lexical => "Lexical",
        ThisMode::Strict => "Strict",
        ThisMode::Global => "Global",
    }
}

fn wide_payload(function: &BytecodeFunction, instruction_offset: usize) -> Option<u32> {
    let instruction_offset = u32::try_from(instruction_offset).ok()?;
    function.wide_operands().iter().find_map(|operand| {
        (operand.instruction_offset() == instruction_offset).then_some(operand.payload())
    })
}

fn abc_operands(
    function: &BytecodeFunction,
    instruction_offset: usize,
    a: u8,
    b: u8,
    c: u8,
) -> WideAbcOperands {
    wide_payload(function, instruction_offset).map_or_else(
        || WideAbcOperands::narrow(a, b, c),
        |payload| WideAbcOperands::decode(a, b, c, payload),
    )
}

fn abx_operands(
    function: &BytecodeFunction,
    instruction_offset: usize,
    a: u8,
    bx: u16,
) -> WideAbxOperands {
    wide_payload(function, instruction_offset).map_or_else(
        || WideAbxOperands::narrow(a, bx),
        |payload| WideAbxOperands::decode(a, bx, payload),
    )
}

fn format_call_range(range: CallRange) -> String {
    format!(
        "[r{}..r{})",
        range.argument_base(),
        range.argument_base().saturating_add(range.argument_count())
    )
}

fn format_call_like_instruction(
    opcode: &str,
    instruction_offset: usize,
    function: &BytecodeFunction,
    result: u8,
    callee: u8,
    this_value: u8,
    has_result: bool,
) -> String {
    let result_prefix = if has_result {
        format!("r{result}, ")
    } else {
        String::new()
    };
    if let Some(payload) = wide_payload(function, instruction_offset) {
        let range = format_call_range(CallRange::decode(payload));
        return format!(
            "{opcode}{result_prefix}callee=r{callee}, this=r{this_value}, args={range}"
        );
    }
    format!("{opcode}{result_prefix}callee=r{callee}, this=r{this_value}")
}

fn format_construct_instruction(
    opcode: &str,
    instruction_offset: usize,
    function: &BytecodeFunction,
    result: u8,
    callee: u8,
) -> String {
    if let Some(payload) = wide_payload(function, instruction_offset) {
        let range = format_call_range(CallRange::decode(payload));
        return format!("{opcode}r{result}, callee=r{callee}, args={range}");
    }
    format!("{opcode}r{result}, callee=r{callee}")
}

fn format_named_property_instruction(
    opcode: &str,
    function: &BytecodeFunction,
    operands: WideAbcOperands,
) -> String {
    format!(
        "{opcode}r{}, r{}, {}",
        operands.a(),
        operands.b(),
        format_atom_operand(function, u32::from(operands.c()))
    )
}

fn format_dense_element_instruction(opcode: &str, operands: WideAbcOperands) -> String {
    format!(
        "{opcode}r{}, r{}, [{}]",
        operands.a(),
        operands.b(),
        operands.c()
    )
}

fn format_abc_instruction(
    opcode: &str,
    bytecode_opcode: crate::Opcode,
    operands: WideAbcOperands,
    function: &BytecodeFunction,
) -> String {
    match bytecode_opcode {
        crate::Opcode::Move => format!("{opcode}r{}, r{}", operands.a(), operands.b()),
        crate::Opcode::Negate
        | crate::Opcode::BitNot
        | crate::Opcode::Increment
        | crate::Opcode::Decrement => format!("{opcode}r{}, r{}", operands.a(), operands.b()),
        crate::Opcode::Add
        | crate::Opcode::Sub
        | crate::Opcode::Mul
        | crate::Opcode::Div
        | crate::Opcode::Mod
        | crate::Opcode::BitAnd
        | crate::Opcode::BitXor
        | crate::Opcode::ShiftLeft
        | crate::Opcode::ShiftRight
        | crate::Opcode::UnsignedShiftRight
        | crate::Opcode::Equal
        | crate::Opcode::StrictEqual
        | crate::Opcode::LessThan
        | crate::Opcode::LessEqual
        | crate::Opcode::GreaterThan
        | crate::Opcode::GreaterEqual
        | crate::Opcode::GetKeyedProperty
        | crate::Opcode::SetKeyedProperty
        | crate::Opcode::AssignKeyedProperty
        | crate::Opcode::DefineKeyedProperty
        | crate::Opcode::CopyDataProperties => {
            format!(
                "{opcode}r{}, r{}, r{}",
                operands.a(),
                operands.b(),
                operands.c()
            )
        }
        crate::Opcode::SetFunctionName | crate::Opcode::ToPropertyKey => {
            format!("{opcode}r{}, r{}", operands.a(), operands.b())
        }
        crate::Opcode::GetNamedProperty
        | crate::Opcode::SetNamedProperty
        | crate::Opcode::AssignNamedProperty
        | crate::Opcode::DefineNamedProperty => {
            format_named_property_instruction(opcode, function, operands)
        }
        crate::Opcode::StoreDenseElement | crate::Opcode::LoadDenseElement => {
            format_dense_element_instruction(opcode, operands)
        }
        _ => format!(
            "{opcode}a={}, b={}, c={}",
            operands.a(),
            operands.b(),
            operands.c()
        ),
    }
}

fn decode_smi_operand(bx: u32) -> i16 {
    let narrow = u16::try_from(bx).expect("load-smi operand should fit into u16");
    i16::from_le_bytes(narrow.to_le_bytes())
}

fn format_abx_instruction(
    opcode: &str,
    bytecode_opcode: crate::Opcode,
    operands: WideAbxOperands,
    function: &BytecodeFunction,
) -> String {
    match bytecode_opcode {
        crate::Opcode::LoadConst => {
            let constant = function
                .constants()
                .get(usize::try_from(operands.bx()).unwrap_or(usize::MAX))
                .map_or_else(|| format!("const[{}]", operands.bx()), format_constant);
            format!(
                "{opcode}r{}, const[{}] ; {constant}",
                operands.a(),
                operands.bx()
            )
        }
        crate::Opcode::LoadSmi => {
            let value = decode_smi_operand(operands.bx());
            format!("{opcode}r{}, {value}", operands.a())
        }
        crate::Opcode::CreateClosure => {
            format!("{opcode}r{}, child[{}]", operands.a(), operands.bx())
        }
        crate::Opcode::LoadThis
        | crate::Opcode::LoadCallee
        | crate::Opcode::LoadNewTarget
        | crate::Opcode::CheckObjectCoercible => {
            format!("{opcode}r{}", operands.a())
        }
        crate::Opcode::LoadGlobal
        | crate::Opcode::StoreGlobal
        | crate::Opcode::AssignGlobal
        | crate::Opcode::DeleteGlobal
        | crate::Opcode::ResolveGlobal
        | crate::Opcode::CaptureName => {
            format!(
                "{opcode}r{}, {}",
                operands.a(),
                format_atom_operand(function, operands.bx())
            )
        }
        crate::Opcode::LoadCapturedName | crate::Opcode::AssignCapturedName => {
            format!("{opcode}r{}, r{}", operands.a(), operands.bx())
        }
        crate::Opcode::LoadEnvSlot | crate::Opcode::StoreEnvSlot | crate::Opcode::AssignEnvSlot => {
            let depth = (operands.bx() >> 24) as u8;
            let slot = operands.bx() & 0x00ff_ffff;
            format!("{opcode}r{}, depth={depth}, slot={slot}", operands.a())
        }
        crate::Opcode::JumpIfTrue | crate::Opcode::JumpIfFalse => {
            let delta = i32::from_le_bytes(operands.bx().to_le_bytes());
            format!("{opcode}r{}, {delta:+}", operands.a())
        }
        _ => format!("{opcode}r{}, {}", operands.a(), operands.bx()),
    }
}

/// Disassemble an entire bytecode template into a stable, human-readable string.
pub fn disassemble(function: &BytecodeFunction) -> String {
    let header = function.header();
    let mut output = String::new();
    let _ = writeln!(
        output,
        "function {:?} kind={:?} this={} args={:?} params={} min_args={} regs={} hidden={} env={} env_slots={} rest={}",
        header.id(),
        header.kind(),
        format_this_mode(header.this_mode()),
        header.arguments_mode(),
        header.parameter_count(),
        header.minimum_argument_count(),
        header.register_count(),
        header.hidden_register_count(),
        header.needs_environment(),
        header.environment_slot_count(),
        header.has_rest_parameter(),
    );

    for (index, instruction) in function.instructions().iter().enumerate() {
        let _ = writeln!(
            output,
            "{index:04}: {}",
            disassemble_instruction_at(index, *instruction, function)
        );
    }

    if !function.constants().is_empty() {
        output.push_str("constants:\n");
        for (index, constant) in function.constants().iter().enumerate() {
            let _ = writeln!(output, "  [{index}] {}", format_constant(constant));
        }
    }

    if !function.child_functions().is_empty() {
        output.push_str("children:\n");
        for (index, child) in function.child_functions().iter().enumerate() {
            let _ = writeln!(output, "  [{index}] {child:?}");
        }
    }

    if !function.exception_handlers().is_empty() {
        output.push_str("handlers:\n");
        for (index, handler) in function.exception_handlers().iter().enumerate() {
            let kind = match handler.kind() {
                ExceptionHandlerKind::Catch => "catch",
                ExceptionHandlerKind::Finally => "finally",
            };
            let target = handler
                .target_register()
                .map_or_else(|| "-".to_owned(), |register| format!("r{register}"));
            let _ = writeln!(
                output,
                "  [{index}] {kind} [{}..{}) -> {} depth={} target={target}",
                handler.protected_start(),
                handler.protected_end(),
                handler.handler(),
                handler.stack_depth(),
            );
        }
    }

    if !function.feedback_sites().is_empty() {
        output.push_str("feedback:\n");
        for descriptor in function.feedback_sites() {
            let metadata = match descriptor.metadata() {
                FeedbackSiteMetadata::None => String::new(),
                FeedbackSiteMetadata::ExpectedArity(arity) => format!(" arity={arity}"),
                FeedbackSiteMetadata::CallArguments {
                    expected_arity,
                    spread_mask,
                } => format!(" arity={expected_arity} spread=0x{spread_mask:016x}"),
                FeedbackSiteMetadata::NamedProperty(atom) => format!(" atom={atom:?}"),
                FeedbackSiteMetadata::KeyedProperty => " keyed".to_owned(),
            };
            let _ = writeln!(
                output,
                "  [{:?}] @{:04} {:?}{}",
                descriptor.slot(),
                descriptor.instruction_offset(),
                descriptor.kind(),
                metadata,
            );
        }
    }

    output
}

/// Disassemble a single instruction in the context of one bytecode template.
pub fn disassemble_instruction(instruction: &Instruction, function: &BytecodeFunction) -> String {
    disassemble_instruction_at(usize::MAX, *instruction, function)
}

fn disassemble_instruction_at(
    instruction_offset: usize,
    instruction: Instruction,
    function: &BytecodeFunction,
) -> String {
    let opcode = instruction.opcode().name();
    let opcode = format!("{opcode:<16}");
    match instruction {
        Instruction::Abc { opcode: _, a, b, c } => match instruction.opcode() {
            crate::Opcode::Call => {
                format_call_like_instruction(&opcode, instruction_offset, function, a, b, c, true)
            }
            crate::Opcode::Construct => {
                format_construct_instruction(&opcode, instruction_offset, function, a, b)
            }
            crate::Opcode::TailCall => {
                format_call_like_instruction(&opcode, instruction_offset, function, 0, a, b, false)
            }
            bytecode_opcode => {
                let operands = abc_operands(function, instruction_offset, a, b, c);
                format_abc_instruction(&opcode, bytecode_opcode, operands, function)
            }
        },
        Instruction::Abx { opcode: _, a, bx } => {
            let operands = abx_operands(function, instruction_offset, a, bx);
            format_abx_instruction(&opcode, instruction.opcode(), operands, function)
        }
        Instruction::Ax { opcode, ax } => match opcode {
            crate::Opcode::Jump | crate::Opcode::JumpIfTrue | crate::Opcode::JumpIfFalse => {
                format!("{opcode_name:<16}{ax:+}", opcode_name = opcode.name())
            }
            crate::Opcode::Return
            | crate::Opcode::LoopHeader
            | crate::Opcode::PushClosureEnv
            | crate::Opcode::PopClosureEnv
            | crate::Opcode::Throw
            | crate::Opcode::LoadException => {
                format!("{opcode_name:<16}r{ax}", opcode_name = opcode.name())
            }
            crate::Opcode::ReturnUndefined => opcode.name().to_owned(),
            _ => format!("{opcode_name:<16}{ax}", opcode_name = opcode.name()),
        },
    }
}
