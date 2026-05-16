# JSC LLInt reference: `op_jmp`

Capture mode: Excerpt

```asm
llintOpWithJump(op_jmp, OpJmp, macro (size, get, jump, dispatch)
    jump(m_targetLabel)
end)


llintJumpTrueOrFalseOp(jtrue, OpJtrue, 
    # Misc primitive
    macro (value, target) btinz value, 1, target end,
    # Truthy Cell
    macro (dispatch) end)


llintJumpTrueOrFalseOp(jfalse, OpJfalse,
    # Misc primitive
    macro (value, target) btiz value, 1, target end,
    # Truthy Cell
    macro (dispatch) dispatch() end)

compareOp(greater, OpGreater,
    macro (left, right, result) cigt left, right, result end,
    macro (left, right, result) cdgt left, right, result end)

compareOp(greatereq, OpGreatereq,
    macro (left, right, result) cigteq left, right, result end,
    macro (left, right, result) cdgteq left, right, result end)

compareOp(less, OpLess,
    macro (left, right, result) cilt left, right, result end,
    macro (left, right, result) cdlt left, right, result end)

compareOp(lesseq, OpLesseq,
    macro (left, right, result) cilteq left, right, result end,
    macro (left, right, result) cdlteq left, right, result end)

compareJumpOp(
    jless, OpJless,
    macro (left, right, target) bilt left, right, target end,
    macro (left, right, target) bdlt left, right, target end)


compareJumpOp(
    jnless, OpJnless,
    macro (left, right, target) bigteq left, right, target end,
    macro (left, right, target) bdgtequn left, right, target end)


compareJumpOp(
    jgreater, OpJgreater,
    macro (left, right, target) bigt left, right, target end,
    macro (left, right, target) bdgt left, right, target end)


compareJumpOp(
    jngreater, OpJngreater,
    macro (left, right, target) bilteq left, right, target end,
    macro (left, right, target) bdltequn left, right, target end)


compareJumpOp(
    jlesseq, OpJlesseq,
    macro (left, right, target) bilteq left, right, target end,
    macro (left, right, target) bdlteq left, right, target end)


compareJumpOp(
    jnlesseq, OpJnlesseq,
    macro (left, right, target) bigt left, right, target end,
    macro (left, right, target) bdgtun left, right, target end)


compareJumpOp(
    jgreatereq, OpJgreatereq,
    macro (left, right, target) bigteq left, right, target end,
    macro (left, right, target) bdgteq left, right, target end)


compareJumpOp(
    jngreatereq, OpJngreatereq,
    macro (left, right, target) bilt left, right, target end,
    macro (left, right, target) bdltun left, right, target end)
```
