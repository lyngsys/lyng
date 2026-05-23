# JSC LLInt reference: `op_put_by_val`

Capture mode: Excerpt

```asm
putByValOp(put_by_val, OpPutByVal, macro (size, dispatch)
.osrReturnPoint:
    getterSetterOSRExitReturnPoint(op_put_by_val, size)
    dispatch()
end)

putByValOp(put_by_val_direct, OpPutByValDirect, macro (size, dispatch)
.osrReturnPoint:
    getterSetterOSRExitReturnPoint(op_put_by_val_direct, size)
    dispatch()
end)

macro llintJumpTrueOrFalseOp(opcodeName, opcodeStruct, miscConditionOp, truthyCellConditionOp)
    llintOpWithJump(op_%opcodeName%, opcodeStruct, macro (size, get, jump, dispatch)
        get(m_condition, t1)
        loadConstantOrVariable(size, t1, t0)
        btqnz t0, ~0xf, .maybeCell
        miscConditionOp(t0, .target)
        dispatch()

    .maybeCell:
        btqnz t0, notCellMask, .slow
        bbbeq JSCell::m_type[t0], constexpr LastMaybeFalsyCellPrimitive, .slow
        btbnz JSCell::m_flags[t0], constexpr MasqueradesAsUndefined, .slow
        truthyCellConditionOp(dispatch)

    .target:
        jump(m_targetLabel)

    .slow:
        callSlowPath(_llint_slow_path_%opcodeName%)
        nextInstruction()
    end)
end


macro equalNullJumpOp(opcodeName, opcodeStruct, cellHandler, immediateHandler)
    llintOpWithJump(op_%opcodeName%, opcodeStruct, macro (size, get, jump, dispatch)
        get(m_value, t1)
        loadConstantOrVariable(size, t1, t0)
        btqnz t0, notCellMask, .immediate
        loadStructureWithScratch(t0, t2, t1)
        cellHandler(t2, JSCell::m_flags[t0], .target)
        dispatch()

    .target:
        jump(m_targetLabel)

    .immediate:
        andq ~TagUndefined, t0
        immediateHandler(t0, .target)
        dispatch()
    end)
end

equalNullJumpOp(jeq_null, OpJeqNull,
    macro (structure, value, target) 
        btbz value, MasqueradesAsUndefined, .notMasqueradesAsUndefined
        loadp CodeBlock[cfr], t0
        loadp CodeBlock::m_globalObject[t0], t0
        bpeq Structure::m_realm[structure], t0, target
.notMasqueradesAsUndefined:
    end,
    macro (value, target) bqeq value, ValueNull, target end)


equalNullJumpOp(jneq_null, OpJneqNull,
    macro (structure, value, target) 
        btbz value, MasqueradesAsUndefined, target
        loadp CodeBlock[cfr], t0
        loadp CodeBlock::m_globalObject[t0], t0
        bpneq Structure::m_realm[structure], t0, target
    end,
    macro (value, target) bqneq value, ValueNull, target end)

macro undefinedOrNullJumpOp(opcodeName, opcodeStruct, fn)
    llintOpWithJump(op_%opcodeName%, opcodeStruct, macro (size, get, jump, dispatch)
        get(m_value, t1)
        loadConstantOrVariable(size, t1, t0)
        andq ~TagUndefined, t0
```
