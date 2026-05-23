# JSC LLInt reference: `op_negate`

Capture mode: Excerpt

```asm
llintOpWithMetadata(op_negate, OpNegate, macro (size, get, dispatch, metadata, return)
    get(m_operand, t0)
    loadConstantOrVariable(size, t0, t3)
    bqb t3, numberTag, .opNegateNotInt
    btiz t3, 0x7fffffff, .opNegateSlow
    negi t3
    orq numberTag, t3
    updateUnaryArithProfile(size, OpNegate, ArithProfileInt, t1, t2)
    return(t3)
.opNegateNotInt:
    btqz t3, numberTag, .opNegateSlow
    xorq 0x8000000000000000, t3
    updateUnaryArithProfile(size, OpNegate, ArithProfileNumber, t1, t2)
    return(t3)

.opNegateSlow:
    callSlowPath(_slow_path_negate)
    dispatch()
end)


macro binaryOpCustomStore(opcodeName, opcodeStruct, integerOperationAndStore, doubleOperation)
    llintOpWithMetadata(op_%opcodeName%, opcodeStruct, macro (size, get, dispatch, metadata, return)
        get(m_rhs, t0)
        get(m_lhs, t2)
        loadConstantOrVariable(size, t0, t1)
        loadConstantOrVariable(size, t2, t0)
        bqb t0, numberTag, .op1NotInt
        bqb t1, numberTag, .op2NotInt
        get(m_dst, t2)
        integerOperationAndStore(t0, t1, .slow, t2)

        updateBinaryArithProfile(size, opcodeStruct, ArithProfileIntInt, t5, t2)
        dispatch()

    .op1NotInt:
        # First operand is definitely not an int, the second operand could be anything.
        btqz t0, numberTag, .slow
        bqaeq t1, numberTag, .op1NotIntOp2Int
        btqz t1, numberTag, .slow
        addq numberTag, t1
        fq2d t1, ft1
        updateBinaryArithProfile(size, opcodeStruct, ArithProfileNumberNumber, t5, t2)
        jmp .op1NotIntReady
    .op1NotIntOp2Int:
        updateBinaryArithProfile(size, opcodeStruct, ArithProfileNumberInt, t5, t2)
        ci2ds t1, ft1
    .op1NotIntReady:
        get(m_dst, t2)
        addq numberTag, t0
        fq2d t0, ft0
        doubleOperation(ft0, ft1)
        fd2q ft0, t0
        subq numberTag, t0
        storeq t0, [cfr, t2, 8]
        dispatch()

    .op2NotInt:
        # First operand is definitely an int, the second is definitely not.
        btqz t1, numberTag, .slow
        updateBinaryArithProfile(size, opcodeStruct, ArithProfileIntNumber, t5, t2)
        get(m_dst, t2)
        ci2ds t0, ft0
        addq numberTag, t1
        fq2d t1, ft1
        doubleOperation(ft0, ft1)
        fd2q ft0, t0
        subq numberTag, t0
        storeq t0, [cfr, t2, 8]
        dispatch()

    .slow:
        callSlowPath(_slow_path_%opcodeName%)
        dispatch()
    end)
end

if X86_64
    binaryOpCustomStore(div, OpDiv,
        macro (lhs, rhs, slow, index)
```
