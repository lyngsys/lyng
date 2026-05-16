# JSC LLInt reference: `op_call`

Capture mode: Excerpt

```asm
commonCallOp(op_call, OpCall, prepareForRegularCall, invokeForRegularCall, prepareForSlowRegularCall, macro (getu, metadata)
    arrayProfileForCall(OpCall, getu)
end, dispatchAfterRegularCall)

commonCallOp(op_construct, OpConstruct, prepareForRegularCall, invokeForRegularCall, prepareForSlowRegularCall, macro (getu, metadata)
end, dispatchAfterRegularCall)

commonCallOp(op_super_construct, OpSuperConstruct, prepareForRegularCall, invokeForRegularCall, prepareForSlowRegularCall, macro (getu, metadata)
    if JSVALUE64
        getu(m_argv, t1)
        lshifti 3, t1
        negp t1
        addp cfr, t1
        loadp ThisArgumentOffset + PayloadOffset[t1], t1
        loadp OpSuperConstruct::Metadata::m_cachedCallee[t5], t2
        bqeq t1, t2, .done
        btqz t2, .store
    .invalidate:
        move SeenMultipleCalleeObjects, t1
    .store:
        storep t1, OpSuperConstruct::Metadata::m_cachedCallee[t5]
    .done:
    end
end, dispatchAfterRegularCall)

commonCallOp(op_tail_call, OpTailCall, prepareForTailCall, invokeForTailCall, prepareForSlowTailCall, macro (getu, metadata)
    arrayProfileForCall(OpTailCall, getu)
    checkSwitchToJITForEpilogue()
    # reload metadata since checkSwitchToJITForEpilogue() might have trashed t5
    metadata(t5, t0)
end, dispatchAfterTailCall)

commonCallOp(op_call_ignore_result, OpCallIgnoreResult, prepareForRegularCall, invokeForRegularCallIgnoreResult, prepareForSlowRegularCall, macro (getu, metadata)
    arrayProfileForCall(OpCallIgnoreResult, getu)
end, dispatchAfterRegularCallIgnoreResult)

macro branchIfException(exceptionTarget)
    loadp CodeBlock[cfr], t3
    loadp CodeBlock::m_vm[t3], t3
    btpz VM::m_exception[t3], .noException
    jmp exceptionTarget
.noException:    
end


llintOpWithMetadata(op_call_varargs, OpCallVarargs, macro (size, get, dispatch, metadata, return)
    doCallVarargs(op_call_varargs, size, get, OpCallVarargs, m_valueProfile, m_dst, dispatch, metadata, _llint_slow_path_size_frame_for_varargs, _llint_slow_path_call_varargs, prepareForRegularCall, invokeForRegularCall, prepareForSlowRegularCall, dispatchAfterRegularCall)
end)

llintOpWithMetadata(op_tail_call_varargs, OpTailCallVarargs, macro (size, get, dispatch, metadata, return)
    checkSwitchToJITForEpilogue()
    # We lie and perform the tail call instead of preparing it since we can't
    # prepare the frame for a call opcode
    doCallVarargs(op_tail_call_varargs, size, get, OpTailCallVarargs, m_valueProfile, m_dst, dispatch, metadata, _llint_slow_path_size_frame_for_varargs, _llint_slow_path_tail_call_varargs, prepareForTailCall, invokeForTailCall, prepareForSlowTailCall, dispatchAfterTailCall)
end)


llintOpWithMetadata(op_construct_varargs, OpConstructVarargs, macro (size, get, dispatch, metadata, return)
    doCallVarargs(op_construct_varargs, size, get, OpConstructVarargs, m_valueProfile, m_dst, dispatch, metadata, _llint_slow_path_size_frame_for_varargs, _llint_slow_path_construct_varargs, prepareForRegularCall, invokeForRegularCall, prepareForSlowRegularCall, dispatchAfterRegularCall)
end)

# Eval is executed in one of two modes:
#
# 1) We find that we're really invoking eval() in which case the
#    execution is perfomed entirely inside the slow_path, and it
#    returns the PC of a function that just returns the return value
#    that the eval returned.
#
# 2) We find that we're invoking something called eval() that is not
#    the real eval. Then the slow_path returns the PC of the thing to
#    call, and we call it.
#
# This allows us to handle two cases, which would require a total of
# up to four pieces of state that cannot be easily packed into two
# registers (C functions can return up to two registers, easily):
#
# - The call frame register. This may or may not have been modified
#   by the slow_path, but the convention is that it returns it. It's not
#   totally clear if that's necessary, since the cfr is callee save.
#   But that's our style in this here interpreter so we stick with it.
```
