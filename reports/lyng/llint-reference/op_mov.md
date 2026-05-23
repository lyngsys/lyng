# JSC LLInt reference: `op_mov`

Capture mode: Excerpt

```asm
llintOpWithReturn(op_mov, OpMov, macro (size, get, dispatch, return)
    get(m_src, t1)
    loadConstantOrVariable(size, t1, t2)
    return(t2)
end)


llintOpWithReturn(op_not, OpNot, macro (size, get, dispatch, return)
    get(m_operand, t0)
    loadConstantOrVariable(size, t0, t2)
    xorq ValueFalse, t2
    btqnz t2, ~1, .opNotSlow
    xorq ValueTrue, t2
    return(t2)

.opNotSlow:
    callSlowPath(_slow_path_not)
    dispatch()
end)


macro equalityComparisonOp(opcodeName, opcodeStruct, integerComparison)
    llintOpWithReturn(op_%opcodeName%, opcodeStruct, macro (size, get, dispatch, return)
        get(m_rhs, t0)
        get(m_lhs, t2)
        loadConstantOrVariableInt32(size, t0, t1, .slow)
        loadConstantOrVariableInt32(size, t2, t0, .slow)
        integerComparison(t0, t1, t0)
        orq ValueFalse, t0
        return(t0)

    .slow:
        callSlowPath(_slow_path_%opcodeName%)
        dispatch()
    end)
end


macro equalNullComparisonOp(opcodeName, opcodeStruct, fn)
    llintOpWithReturn(opcodeName, opcodeStruct, macro (size, get, dispatch, return)
        get(m_operand, t1)
        loadConstantOrVariable(size, t1, t0)
        btqnz t0, notCellMask, .immediate
        btbnz JSCell::m_flags[t0], MasqueradesAsUndefined, .masqueradesAsUndefined
        move 0, t0
        jmp .done
    .masqueradesAsUndefined:
        loadStructureWithScratch(t0, t2, t1)
        loadp CodeBlock[cfr], t0
        loadp CodeBlock::m_globalObject[t0], t0
        cpeq Structure::m_realm[t2], t0, t0
        jmp .done
    .immediate:
        andq ~TagUndefined, t0
        cqeq t0, ValueNull, t0
    .done:
        fn(t0)
        return(t0)
    end)
end

equalNullComparisonOp(op_eq_null, OpEqNull,
    macro (value) orq ValueFalse, value end)


equalNullComparisonOp(op_neq_null, OpNeqNull,
    macro (value) xorq ValueTrue, value end)


llintOpWithReturn(op_is_undefined_or_null, OpIsUndefinedOrNull, macro (size, get, dispatch, return)
    get(m_operand, t1)
    loadConstantOrVariable(size, t1, t0)
    andq ~TagUndefined, t0
    cqeq t0, ValueNull, t0
    orq ValueFalse, t0
    return(t0)
end)


macro strictEqOp(opcodeName, opcodeStruct, createBoolean)
```
