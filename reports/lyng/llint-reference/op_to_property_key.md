# JSC LLInt reference: `op_to_property_key`

Capture mode: Excerpt

```asm
llintOpWithReturn(op_to_property_key, OpToPropertyKey, macro (size, get, dispatch, return)
    get(m_src, t2)
    loadConstantOrVariable(size, t2, t0)

    btqnz t0, notCellMask, .opToPropertyKeySlow
    bbeq JSCell::m_type[t0], SymbolType, .done
    bbneq JSCell::m_type[t0], StringType, .opToPropertyKeySlow

.done:
    return(t0)

.opToPropertyKeySlow:
    callSlowPath(_slow_path_to_property_key)
    dispatch()
end)

llintOpWithReturn(op_to_property_key_or_number, OpToPropertyKeyOrNumber, macro (size, get, dispatch, return)
    get(m_src, t2)
    loadConstantOrVariable(size, t2, t0)

    btqnz t0, numberTag, .done
    btqnz t0, notCellMask, .opToPropertyKeyOrNumberSlow
    bbeq JSCell::m_type[t0], SymbolType, .done
    bbneq JSCell::m_type[t0], StringType, .opToPropertyKeyOrNumberSlow

.done:
    return(t0)

.opToPropertyKeyOrNumberSlow:
    callSlowPath(_slow_path_to_property_key_or_number)
    dispatch()
end)


commonOp(llint_op_catch, macro () end, macro (size)
    # This is where we end up from the JIT's throw trampoline (because the
    # machine code return address will be set to _llint_op_catch), and from
    # the interpreter's throw trampoline (see _llint_throw_trampoline).
    # The throwing code must have known that we were throwing to the interpreter,
    # and have set VM::targetInterpreterPCForThrow.
    getVMFromCallFrame(t3, t0)
    restoreCalleeSavesFromVMEntryFrameCalleeSavesBuffer(t3, t0)
    loadp VM::callFrameForCatch[t3], cfr
    storep 0, VM::callFrameForCatch[t3]
    restoreStackPointerAfterCall()

    loadp CodeBlock[cfr], PB
    loadp CodeBlock::m_metadata[PB], metadataTable
    loadp CodeBlock::m_instructionsRawPointer[PB], PB
    loadp VM::targetInterpreterPCForThrow[t3], PC
    subp PB, PC

    callSlowPath(_llint_slow_path_retrieve_and_clear_exception_if_catchable)
    bpneq r1, 0, .isCatchableException
    jmp _llint_throw_from_slow_path_trampoline

.isCatchableException:
    move r1, t0
    get(size, OpCatch, m_exception, t2)
    storeq t0, [cfr, t2, 8]

    loadq Exception::m_value[t0], t3
    get(size, OpCatch, m_thrownValue, t2)
    storeq t3, [cfr, t2, 8]

    traceExecution()

    callSlowPath(_llint_slow_path_profile_catch)

    dispatchOp(size, op_catch)
end)

op(llint_throw_from_slow_path_trampoline, macro ()
    getVMFromCallFrame(t1, t2)
    copyCalleeSavesToVMEntryFrameCalleeSavesBuffer(t1, t2)

    callSlowPath(_llint_slow_path_handle_exception)

    # When throwing from the interpreter (i.e. throwing from LLIntSlowPaths), so
    # the throw target is not necessarily interpreted code, we come to here.
```
