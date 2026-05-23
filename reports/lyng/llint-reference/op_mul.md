# JSC LLInt reference: `op_mul`

Capture mode: Excerpt

```asm
binaryOpCustomStore(mul, OpMul,
    macro (lhs, rhs, slow, index)
        # Assume t3 is scratchable.
        move lhs, t3
        bmulio rhs, t3, slow
        btinz t3, .done
        bilt rhs, 0, slow
        bilt lhs, 0, slow
    .done:
        orq numberTag, t3
        storeq t3, [cfr, index, 8]
    end,
    macro (lhs, rhs) muld rhs, lhs end)


macro binaryOp(opcodeName, opcodeStruct, integerOperation, doubleOperation)
    binaryOpCustomStore(opcodeName, opcodeStruct,
        macro (lhs, rhs, slow, index)
            integerOperation(lhs, rhs, slow)
            orq numberTag, lhs
            storeq lhs, [cfr, index, 8]
        end,
        doubleOperation)
end

binaryOp(add, OpAdd,
    macro (lhs, rhs, slow) baddio rhs, lhs, slow end,
    macro (lhs, rhs) addd rhs, lhs end)


binaryOp(sub, OpSub,
    macro (lhs, rhs, slow) bsubio rhs, lhs, slow end,
    macro (lhs, rhs) subd rhs, lhs end)

if X86_64
    llintOpWithReturn(op_mod, OpMod, macro (size, get, dispatch, return)
        get(m_rhs, t0)
        get(m_lhs, t2)
        loadConstantOrVariableInt32(size, t0, t1, .slow)
        loadConstantOrVariableInt32(size, t2, t0, .slow)

        # Assume t3 is scratchable.
        # r1 is always edx (even on Windows).
        btiz t1, .slow
        bineq t1, -1, .notNeg2ToThe31ModByNeg1
        bieq t0, -2147483648, .slow
    .notNeg2ToThe31ModByNeg1:
        move t1, t3
        bilt t0, 0, .needsNegZeroCheck
        cdqi
        idivi t3
        orq numberTag, r1
        return(r1)
    .needsNegZeroCheck:
        cdqi
        idivi t3
        btiz r1, .slow
        orq numberTag, r1
        return(r1)

    .slow:
        callSlowPath(_slow_path_mod)
        dispatch()
    end)
else
    slowPathOp(mod)
end

llintOpWithReturn(op_pow, OpPow, macro (size, get, dispatch, return)
    get(m_rhs, t0)
    get(m_lhs, t2)
    loadConstantOrVariableInt32(size, t0, t1, .slow)
    loadConstantOrVariable(size, t2, t0)

    bilt t1, 0, .slow
    bigt t1, (constexpr maxExponentForIntegerMathPow), .slow

    bqb t0, numberTag, .lhsNotInt
    ci2ds t0, ft0
    jmp .lhsReady
```
