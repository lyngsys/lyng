# JSC LLInt reference: `op_get_by_val`

Capture mode: Excerpt

```asm
llintOpWithMetadata(op_get_by_val, OpGetByVal, macro (size, get, dispatch, metadata, return)
    macro finishGetByVal(result, scratch)
        get(m_dst, scratch)
        storeq result, [cfr, scratch, 8]
        valueProfile(size, OpGetByVal, m_valueProfile, result, scratch)
        dispatch()
    end

    macro finishIntGetByVal(result, scratch)
        orq numberTag, result
        finishGetByVal(result, scratch)
    end

    macro finishDoubleGetByVal(result, scratch1, scratch2, unused)
        fd2q result, scratch1
        subq numberTag, scratch1
        finishGetByVal(scratch1, scratch2)
    end

    macro setLargeTypedArray(scratch)
        loadi OpGetByVal::Metadata::m_arrayProfile.m_arrayProfileFlags[t5], scratch
        ori constexpr ArrayProfileFlag::MayBeLargeTypedArray, scratch
        storei scratch, OpGetByVal::Metadata::m_arrayProfile.m_arrayProfileFlags[t5]
    end

    metadata(t5, t2)

    get(m_base, t2)
    loadConstantOrVariableCell(size, t2, t0, .opGetByValSlow)

    move t0, t2
    arrayProfile(OpGetByVal::Metadata::m_arrayProfile, t2, t5, t1)
    loadb JSCell::m_indexingTypeAndMisc[t2], t2

    get(m_property, t3)
    loadConstantOrVariableInt32(size, t3, t1, .opGetByValSlow)
    # This sign-extension makes the bounds-checking in getByValTypedArray work even on 4GB TypedArray.
    sxi2q t1, t1

    loadCagedJSValue(JSObjectWithButterfly::m_butterfly[t0], t3, numberTag)
    move TagNumber, numberTag

    andi IndexingShapeMask, t2
    bieq t2, Int32Shape, .opGetByValIsContiguous
    bineq t2, ContiguousShape, .opGetByValNotContiguous

.opGetByValIsContiguous:
    biaeq t1, -sizeof IndexingHeader + IndexingHeader::u.lengths.publicLength[t3], .opGetByValSlow
    get(m_dst, t0)
    loadq [t3, t1, 8], t2
    btqz t2, .opGetByValSlow
    jmp .opGetByValDone

.opGetByValNotContiguous:
    bineq t2, DoubleShape, .opGetByValNotDouble
    biaeq t1, -sizeof IndexingHeader + IndexingHeader::u.lengths.publicLength[t3], .opGetByValSlow
    get(m_dst, t0)
    loadd [t3, t1, 8], ft0
    bdnequn ft0, ft0, .opGetByValSlow
    fd2q ft0, t2
    subq numberTag, t2
    jmp .opGetByValDone
    
.opGetByValNotDouble:
    subi ArrayStorageShape, t2
    bia t2, SlowPutArrayStorageShape - ArrayStorageShape, .opGetByValNotIndexedStorage
    biaeq t1, -sizeof IndexingHeader + IndexingHeader::u.lengths.vectorLength[t3], .opGetByValSlow
    get(m_dst, t0)
    loadq ArrayStorage::m_vector[t3, t1, 8], t2
    btqz t2, .opGetByValSlow

.opGetByValDone:
    storeq t2, [cfr, t0, 8]
    valueProfile(size, OpGetByVal, m_valueProfile, t2, t5)
    dispatch()

.opGetByValNotIndexedStorage:
    getByValTypedArray(t0, t1, finishIntGetByVal, finishDoubleGetByVal, setLargeTypedArray, .opGetByValSlow)

.opGetByValSlow:
```
