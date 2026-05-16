# JSC LLInt reference: `op_jeq`

Capture mode: Excerpt

```asm

equalityJumpOp(
    jeq, OpJeq,
    macro (left, right, target) bieq left, right, target end)


equalityJumpOp(
    jneq, OpJneq,
    macro (left, right, target) bineq left, right, target end)


compareUnsignedJumpOp(
    jbelow, OpJbelow,
    macro (left, right, target) bib left, right, target end)


compareUnsignedJumpOp(
    jbeloweq, OpJbeloweq,
    macro (left, right, target) bibeq left, right, target end)


preOp(inc, OpInc,
    macro (value, slow) baddio 1, value, slow end)

preOp(dec, OpDec,
    macro (value, slow) bsubio 1, value, slow end)


llintOp(op_loop_hint, OpLoopHint, macro (unused, unused, dispatch)
    checkSwitchToJITForLoop()
    dispatch()
end)


macro checkTraps(dispatch)
    loadp CodeBlock[cfr], t1
    loadp CodeBlock::m_vm[t1], t1
    loadi VM::m_threadContext+VMThreadContext::m_traps+VMTraps::m_trapBits[t1], t0
    andi VMTrapsAsyncEvents, t0
    btpnz t0, .handleTraps
.afterHandlingTraps:
    dispatch()
.handleTraps:
    callTrapHandler(.throwHandler)
    jmp .afterHandlingTraps
.throwHandler:
    jmp _llint_throw_from_slow_path_trampoline
end

llintOp(op_check_traps, OpCheckTraps, macro (unused, unused, dispatch)
    checkTraps(dispatch)
end)


# Returns the packet pointer in t0.
macro acquireShadowChickenPacket(slow)
    loadp CodeBlock[cfr], t1
    loadp CodeBlock::m_vm[t1], t1
    loadp VM::m_shadowChicken[t1], t2
    loadp ShadowChicken::m_logCursor[t2], t0
    bpaeq t0, ShadowChicken::m_logEnd[t2], slow
    addp sizeof ShadowChicken::Packet, t0, t1
    storep t1, ShadowChicken::m_logCursor[t2]
end


llintOp(op_nop, OpNop, macro (unused, unused, dispatch)
    dispatch()
end)


# we can't use callOp because we can't pass `call` as the opcode name, since it's an instruction name
commonCallOp(op_call, OpCall, prepareForRegularCall, invokeForRegularCall, prepareForSlowRegularCall, macro (getu, metadata)
    arrayProfileForCall(OpCall, getu)
end, dispatchAfterRegularCall)

commonCallOp(op_construct, OpConstruct, prepareForRegularCall, invokeForRegularCall, prepareForSlowRegularCall, macro (getu, metadata)
end, dispatchAfterRegularCall)

commonCallOp(op_super_construct, OpSuperConstruct, prepareForRegularCall, invokeForRegularCall, prepareForSlowRegularCall, macro (getu, metadata)
```
