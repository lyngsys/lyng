# JSC LLInt reference: `op_get_by_id`

Capture mode: Excerpt

```asm
llintOpWithMetadata(op_get_by_id, OpGetById, macro (size, get, dispatch, metadata, return)
    get(m_base, t0)
    loadConstantOrVariableCell(size, t0, t3, .opGetByIdSlow)
    metadata(t2, t1)
    performGetByIDHelper(OpGetById, m_modeMetadata, m_valueProfile, .opGetByIdSlow, size, return)

.opGetByIdSlow:
    callSlowPath(_llint_slow_path_get_by_id)
    dispatch()

.osrReturnPoint:
    getterSetterOSRExitReturnPoint(op_get_by_id, size)
    valueProfile(size, OpGetById, m_valueProfile, r0, t2)
    return(r0)
end)


llintOpWithMetadata(op_get_length, OpGetLength, macro (size, get, dispatch, metadata, return)
    get(m_base, t0)
    loadConstantOrVariableCell(size, t0, t3, .opGetLengthSlow)
    metadata(t2, t1)
    arrayProfile(OpGetLength::Metadata::m_arrayProfile, t3, t2, t5)
    performGetByIDHelper(OpGetLength, m_modeMetadata, m_valueProfile, .opGetLengthSlow, size, return)

.opGetLengthSlow:
    callSlowPath(_llint_slow_path_get_length)
    dispatch()

.osrReturnPoint:
    getterSetterOSRExitReturnPoint(op_get_length, size)
    valueProfile(size, OpGetLength, m_valueProfile, r0, t2)
    return(r0)
end)


llintOpWithProfile(op_get_prototype_of, OpGetPrototypeOf, macro (size, get, dispatch, return)
    get(m_value, t1)
    loadConstantOrVariable(size, t1, t0)

    btqnz t0, notCellMask, .opGetPrototypeOfSlow
    bbb JSCell::m_type[t0], ObjectType, .opGetPrototypeOfSlow
    btbnz JSCell::m_flags[t0], OverridesGetPrototype, .opGetPrototypeOfSlow

    loadStructureWithScratch(t0, t2, t1)
    loadq Structure::m_prototype[t2], t2
    btqz t2, .opGetPrototypeOfPolyProto
    return(t2)

.opGetPrototypeOfSlow:
    callSlowPath(_slow_path_get_prototype_of)
    dispatch()

.opGetPrototypeOfPolyProto:
    move knownPolyProtoOffset, t1
    loadInlineOffset(t1, t0, t3)
    return(t3)
end)


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

```
