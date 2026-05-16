# JSC LLInt reference: `op_sub`

Capture mode: Excerpt

```asm
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
.lhsNotInt:
    btqz t0, numberTag, .slow
    addq numberTag, t0
    fq2d t0, ft0
.lhsReady:
    get(m_dst, t2)
    move 1, t0
    ci2ds t0, ft1

.loop:
    btiz t1, 0x1, .exponentIsEven
    muld ft0, ft1
.exponentIsEven:
    muld ft0, ft0
    rshifti 1, t1
    btinz t1, .loop

    fd2q ft1, t0
    subq numberTag, t0
    storeq t0, [cfr, t2, 8]
    dispatch()

.slow:
    callSlowPath(_slow_path_pow)
    dispatch()
end)

llintOpWithReturn(op_unsigned, OpUnsigned, macro (size, get, dispatch, return)
    get(m_operand, t1)
    loadConstantOrVariable(size, t1, t2)
```
