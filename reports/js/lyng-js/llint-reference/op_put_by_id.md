# JSC LLInt reference: `op_put_by_id`

Capture mode: Excerpt

```asm
llintOpWithMetadata(op_put_by_id, OpPutById, macro (size, get, dispatch, metadata, return)
    get(m_base, t3)
    loadConstantOrVariableCell(size, t3, t0, .opPutByIdSlow)
    metadata(t5, t2)
    loadi OpPutById::Metadata::m_oldStructureID[t5], t2
    bineq t2, JSCell::m_structureID[t0], .opPutByIdSlow

    # At this point, we have:
    # t0 -> object base
    # t2 -> current structure ID
    # t5 -> metadata

    loadi OpPutById::Metadata::m_newStructureID[t5], t1
    btiz t1, .opPutByIdNotTransition

    # This is the transition case. t1 holds the new structureID. t2 holds the old structure ID.
    # If we have a chain, we need to check it. t0 is the base. We may clobber t1 to use it as
    # scratch.
    loadp OpPutById::Metadata::m_structureChain[t5], t3
    btpz t3, .opPutByIdTransitionDirect

    structureIDToStructureWithScratch(t2, t1)

    loadp StructureChain::m_vector[t3], t3
    assert(macro (ok) btpnz t3, ok end)

    loadq Structure::m_prototype[t2], t2
    bqeq t2, ValueNull, .opPutByIdTransitionChainDone
.opPutByIdTransitionChainLoop:
    loadi JSCell::m_structureID[t2], t2
    bineq t2, [t3], .opPutByIdSlow
    addp 4, t3
    structureIDToStructureWithScratch(t2, t1)
    loadq Structure::m_prototype[t2], t2
    bqneq t2, ValueNull, .opPutByIdTransitionChainLoop

.opPutByIdTransitionChainDone:
    # Reload the new structure, since we clobbered it above.
    loadi OpPutById::Metadata::m_newStructureID[t5], t1
    # Reload base into t0
    get(m_base, t3)
    loadConstantOrVariable(size, t3, t0)

.opPutByIdTransitionDirect:
    storei t1, JSCell::m_structureID[t0]
    writeBarrierOnOperandWithReload(size, get, m_base, macro ()
        # Reload metadata into t5
        metadata(t5, t1)
        # Reload base into t0
        get(m_base, t1)
        loadConstantOrVariable(size, t1, t0)
    end)

.opPutByIdNotTransition:
    # The only thing live right now is t0, which holds the base.
    get(m_value, t1)
    loadConstantOrVariable(size, t1, t2)
    loadi OpPutById::Metadata::m_offset[t5], t1
    storePropertyAtVariableOffset(t1, t0, t2)
    writeBarrierOnOperands(size, get, m_base, m_value)
    dispatch()

.opPutByIdSlow:
    callSlowPath(_llint_slow_path_put_by_id)
    dispatch()

.osrReturnPoint:
    getterSetterOSRExitReturnPoint(op_put_by_id, size)
    dispatch()

end)


llintOpWithMetadata(op_get_by_val, OpGetByVal, macro (size, get, dispatch, metadata, return)
    macro finishGetByVal(result, scratch)
        get(m_dst, scratch)
        storeq result, [cfr, scratch, 8]
        valueProfile(size, OpGetByVal, m_valueProfile, result, scratch)
        dispatch()
    end
```
