# JSC LLInt reference: `op_bitor`

Capture mode: Excerpt

```asm
bitOp(bitor, OpBitor,
    macro (lhs, rhs) ori rhs, lhs end)

bitOp(bitxor, OpBitxor,
    macro (lhs, rhs) xori rhs, lhs end)

llintOpWithReturn(op_bitnot, OpBitnot, macro (size, get, dispatch, return)
    get(m_operand, t0)
    loadConstantOrVariableInt32(size, t0, t3, .opBitNotSlow)
    noti t3
    orq numberTag, t3
    return(t3)
.opBitNotSlow:
    callSlowPath(_slow_path_bitnot)
    dispatch()
end)


llintOpWithReturn(op_is_empty, OpIsEmpty, macro (size, get, dispatch, return)
    get(m_operand, t1)
    loadConstantOrVariable(size, t1, t0)
    cqeq t0, ValueEmpty, t3
    orq ValueFalse, t3
    return(t3)
end)


llintOpWithReturn(op_typeof_is_undefined, OpTypeofIsUndefined, macro (size, get, dispatch, return)
    get(m_operand, t1)
    loadConstantOrVariable(size, t1, t0)
    btqz t0, notCellMask, .opIsUndefinedCell
    cqeq t0, ValueUndefined, t3
    orq ValueFalse, t3
    return(t3)
.opIsUndefinedCell:
    btbnz JSCell::m_flags[t0], MasqueradesAsUndefined, .masqueradesAsUndefined
    move ValueFalse, t1
    return(t1)
.masqueradesAsUndefined:
    loadStructureWithScratch(t0, t3, t1)
    loadp CodeBlock[cfr], t1
    loadp CodeBlock::m_globalObject[t1], t1
    cpeq Structure::m_realm[t3], t1, t0
    orq ValueFalse, t0
    return(t0)
end)

llintOpWithReturn(op_typeof_is_function, OpTypeofIsFunction, macro (size, get, dispatch, return)
    get(m_operand, t1)
    loadConstantOrVariable(size, t1, t0)
    btqnz t0, notCellMask, .opTypeOfIsFunctionIsImm
    bbaeq JSCell::m_type[t0], ObjectType, .opTypeOfIsFunctionSlowCase
.opTypeOfIsFunctionIsImm:
    move ValueFalse, t0
    return(t0)
.opTypeOfIsFunctionSlowCase:
    callSlowPath(_slow_path_typeof_is_function)
    dispatch()
end)

llintOpWithReturn(op_is_boolean, OpIsBoolean, macro (size, get, dispatch, return)
    get(m_operand, t1)
    loadConstantOrVariable(size, t1, t0)
    xorq ValueFalse, t0
    tqz t0, ~1, t0
    orq ValueFalse, t0
    return(t0)
end)


llintOpWithReturn(op_is_number, OpIsNumber, macro (size, get, dispatch, return)
    get(m_operand, t1)
    loadConstantOrVariable(size, t1, t0)
    tqnz t0, numberTag, t1
    orq ValueFalse, t1
    return(t1)
end)

if BIGINT32
    llintOpWithReturn(op_is_big_int, OpIsBigInt, macro(size, get, dispatch, return)
```
