use super::{
    install_public_builtin_function, FamilyInstallContext, PromiseDisposalFamilyBuiltins,
    PromiseDisposalFamilyPrototypes,
};
use crate::public::PublicRealmBuiltins;
use lyng_js_env::Agent;
use lyng_js_types::{
    js3_add_async_disposable_resource_builtin, js3_add_sync_disposable_resource_builtin,
    js3_async_disposable_stack_adopt_builtin, js3_async_disposable_stack_builtin,
    js3_async_disposable_stack_defer_builtin, js3_async_disposable_stack_dispose_async_builtin,
    js3_async_disposable_stack_disposed_getter_builtin, js3_async_disposable_stack_move_builtin,
    js3_async_disposable_stack_use_builtin, js3_create_async_disposal_scope_builtin,
    js3_create_sync_disposal_scope_builtin, js3_disposable_stack_adopt_builtin,
    js3_disposable_stack_builtin, js3_disposable_stack_defer_builtin,
    js3_disposable_stack_dispose_builtin, js3_disposable_stack_disposed_getter_builtin,
    js3_disposable_stack_move_builtin, js3_disposable_stack_use_builtin,
    js3_dispose_scope_async_builtin, js3_dispose_scope_builtin, js3_promise_all_builtin,
    js3_promise_all_settled_builtin, js3_promise_any_builtin, js3_promise_builtin,
    js3_promise_catch_builtin, js3_promise_finally_builtin, js3_promise_race_builtin,
    js3_promise_reject_builtin, js3_promise_resolve_builtin, js3_promise_species_getter_builtin,
    js3_promise_then_builtin, BuiltinFunctionId, ObjectRef,
};

pub(in crate::public) fn install_promise_disposal_family(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    prototypes: PromiseDisposalFamilyPrototypes,
) -> PromiseDisposalFamilyBuiltins {
    let promise = install_promise_family(agent, cx, prototypes.promise_prototype);
    let disposable_stack =
        install_disposable_stack_family(agent, cx, prototypes.disposable_stack_prototype);
    let async_disposable_stack = install_async_disposable_stack_family(
        agent,
        cx,
        prototypes.async_disposable_stack_prototype,
    );
    let disposal_helpers = install_disposal_helper_family(agent, cx);

    PromiseDisposalFamilyBuiltins {
        promise: promise.promise,
        promise_prototype: promise.prototype,
        disposable_stack: disposable_stack.disposable_stack,
        disposable_stack_prototype: disposable_stack.prototype,
        async_disposable_stack: async_disposable_stack.async_disposable_stack,
        async_disposable_stack_prototype: async_disposable_stack.prototype,
        disposable_stack_use: disposable_stack.use_method,
        disposable_stack_adopt: disposable_stack.adopt,
        disposable_stack_defer: disposable_stack.defer,
        disposable_stack_move: disposable_stack.move_method,
        disposable_stack_disposed_getter: disposable_stack.disposed_getter,
        disposable_stack_dispose: disposable_stack.dispose,
        async_disposable_stack_use: async_disposable_stack.use_method,
        async_disposable_stack_adopt: async_disposable_stack.adopt,
        async_disposable_stack_defer: async_disposable_stack.defer,
        async_disposable_stack_move: async_disposable_stack.move_method,
        async_disposable_stack_disposed_getter: async_disposable_stack.disposed_getter,
        async_disposable_stack_dispose_async: async_disposable_stack.dispose_async,
        create_sync_disposal_scope: disposal_helpers.create_sync_disposal_scope,
        create_async_disposal_scope: disposal_helpers.create_async_disposal_scope,
        add_sync_disposable_resource: disposal_helpers.add_sync_disposable_resource,
        add_async_disposable_resource: disposal_helpers.add_async_disposable_resource,
        dispose_scope: disposal_helpers.dispose_scope,
        dispose_scope_async: disposal_helpers.dispose_scope_async,
        promise_then: promise.then,
        promise_catch: promise.catch,
        promise_finally: promise.finally,
        promise_resolve: promise.resolve,
        promise_reject: promise.reject,
        promise_all: promise.all,
        promise_all_settled: promise.all_settled,
        promise_race: promise.race,
        promise_any: promise.any,
        promise_species_getter: promise.species_getter,
    }
}

pub(in crate::public) fn promise_disposal_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    promise_builtin_object(builtins, entry)
        .or_else(|| disposable_stack_builtin_object(builtins, entry))
        .or_else(|| async_disposable_stack_builtin_object(builtins, entry))
        .or_else(|| disposal_helper_builtin_object(builtins, entry))
}

fn promise_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    [
        (js3_promise_builtin(), builtins.promise),
        (js3_promise_then_builtin(), builtins.promise_then),
        (js3_promise_catch_builtin(), builtins.promise_catch),
        (js3_promise_finally_builtin(), builtins.promise_finally),
        (js3_promise_resolve_builtin(), builtins.promise_resolve),
        (js3_promise_reject_builtin(), builtins.promise_reject),
        (js3_promise_all_builtin(), builtins.promise_all),
        (
            js3_promise_all_settled_builtin(),
            builtins.promise_all_settled,
        ),
        (js3_promise_race_builtin(), builtins.promise_race),
        (js3_promise_any_builtin(), builtins.promise_any),
        (
            js3_promise_species_getter_builtin(),
            builtins.promise_species_getter,
        ),
    ]
    .into_iter()
    .find_map(|(id, object)| (entry == id).then_some(object))
}

fn disposable_stack_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    [
        (js3_disposable_stack_builtin(), builtins.disposable_stack),
        (
            js3_disposable_stack_use_builtin(),
            builtins.disposable_stack_use,
        ),
        (
            js3_disposable_stack_adopt_builtin(),
            builtins.disposable_stack_adopt,
        ),
        (
            js3_disposable_stack_defer_builtin(),
            builtins.disposable_stack_defer,
        ),
        (
            js3_disposable_stack_move_builtin(),
            builtins.disposable_stack_move,
        ),
        (
            js3_disposable_stack_disposed_getter_builtin(),
            builtins.disposable_stack_disposed_getter,
        ),
        (
            js3_disposable_stack_dispose_builtin(),
            builtins.disposable_stack_dispose,
        ),
    ]
    .into_iter()
    .find_map(|(id, object)| (entry == id).then_some(object))
}

fn async_disposable_stack_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    [
        (
            js3_async_disposable_stack_builtin(),
            builtins.async_disposable_stack,
        ),
        (
            js3_async_disposable_stack_use_builtin(),
            builtins.async_disposable_stack_use,
        ),
        (
            js3_async_disposable_stack_adopt_builtin(),
            builtins.async_disposable_stack_adopt,
        ),
        (
            js3_async_disposable_stack_defer_builtin(),
            builtins.async_disposable_stack_defer,
        ),
        (
            js3_async_disposable_stack_move_builtin(),
            builtins.async_disposable_stack_move,
        ),
        (
            js3_async_disposable_stack_disposed_getter_builtin(),
            builtins.async_disposable_stack_disposed_getter,
        ),
        (
            js3_async_disposable_stack_dispose_async_builtin(),
            builtins.async_disposable_stack_dispose_async,
        ),
    ]
    .into_iter()
    .find_map(|(id, object)| (entry == id).then_some(object))
}

fn disposal_helper_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    [
        (
            js3_create_sync_disposal_scope_builtin(),
            builtins.create_sync_disposal_scope,
        ),
        (
            js3_create_async_disposal_scope_builtin(),
            builtins.create_async_disposal_scope,
        ),
        (
            js3_add_sync_disposable_resource_builtin(),
            builtins.add_sync_disposable_resource,
        ),
        (
            js3_add_async_disposable_resource_builtin(),
            builtins.add_async_disposable_resource,
        ),
        (js3_dispose_scope_builtin(), builtins.dispose_scope),
        (
            js3_dispose_scope_async_builtin(),
            builtins.dispose_scope_async,
        ),
    ]
    .into_iter()
    .find_map(|(id, object)| (entry == id).then_some(object))
}

#[derive(Clone, Copy, Debug)]
struct PromiseFamilyBuiltins {
    promise: ObjectRef,
    prototype: ObjectRef,
    then: ObjectRef,
    catch: ObjectRef,
    finally: ObjectRef,
    resolve: ObjectRef,
    reject: ObjectRef,
    all: ObjectRef,
    all_settled: ObjectRef,
    race: ObjectRef,
    any: ObjectRef,
    species_getter: ObjectRef,
}

fn install_promise_family(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    prototype: ObjectRef,
) -> PromiseFamilyBuiltins {
    PromiseFamilyBuiltins {
        promise: install_public_builtin_function(agent, cx, js3_promise_builtin(), Some(prototype)),
        prototype,
        then: install_public_builtin_function(agent, cx, js3_promise_then_builtin(), None),
        catch: install_public_builtin_function(agent, cx, js3_promise_catch_builtin(), None),
        finally: install_public_builtin_function(agent, cx, js3_promise_finally_builtin(), None),
        resolve: install_public_builtin_function(agent, cx, js3_promise_resolve_builtin(), None),
        reject: install_public_builtin_function(agent, cx, js3_promise_reject_builtin(), None),
        all: install_public_builtin_function(agent, cx, js3_promise_all_builtin(), None),
        all_settled: install_public_builtin_function(
            agent,
            cx,
            js3_promise_all_settled_builtin(),
            None,
        ),
        race: install_public_builtin_function(agent, cx, js3_promise_race_builtin(), None),
        any: install_public_builtin_function(agent, cx, js3_promise_any_builtin(), None),
        species_getter: install_public_builtin_function(
            agent,
            cx,
            js3_promise_species_getter_builtin(),
            None,
        ),
    }
}

#[derive(Clone, Copy, Debug)]
struct DisposableStackFamilyBuiltins {
    disposable_stack: ObjectRef,
    prototype: ObjectRef,
    use_method: ObjectRef,
    adopt: ObjectRef,
    defer: ObjectRef,
    move_method: ObjectRef,
    disposed_getter: ObjectRef,
    dispose: ObjectRef,
}

fn install_disposable_stack_family(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    prototype: ObjectRef,
) -> DisposableStackFamilyBuiltins {
    DisposableStackFamilyBuiltins {
        disposable_stack: install_public_builtin_function(
            agent,
            cx,
            js3_disposable_stack_builtin(),
            Some(prototype),
        ),
        prototype,
        use_method: install_public_builtin_function(
            agent,
            cx,
            js3_disposable_stack_use_builtin(),
            None,
        ),
        adopt: install_public_builtin_function(
            agent,
            cx,
            js3_disposable_stack_adopt_builtin(),
            None,
        ),
        defer: install_public_builtin_function(
            agent,
            cx,
            js3_disposable_stack_defer_builtin(),
            None,
        ),
        move_method: install_public_builtin_function(
            agent,
            cx,
            js3_disposable_stack_move_builtin(),
            None,
        ),
        disposed_getter: install_public_builtin_function(
            agent,
            cx,
            js3_disposable_stack_disposed_getter_builtin(),
            None,
        ),
        dispose: install_public_builtin_function(
            agent,
            cx,
            js3_disposable_stack_dispose_builtin(),
            None,
        ),
    }
}

#[derive(Clone, Copy, Debug)]
struct AsyncDisposableStackFamilyBuiltins {
    async_disposable_stack: ObjectRef,
    prototype: ObjectRef,
    use_method: ObjectRef,
    adopt: ObjectRef,
    defer: ObjectRef,
    move_method: ObjectRef,
    disposed_getter: ObjectRef,
    dispose_async: ObjectRef,
}

fn install_async_disposable_stack_family(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    prototype: ObjectRef,
) -> AsyncDisposableStackFamilyBuiltins {
    AsyncDisposableStackFamilyBuiltins {
        async_disposable_stack: install_public_builtin_function(
            agent,
            cx,
            js3_async_disposable_stack_builtin(),
            Some(prototype),
        ),
        prototype,
        use_method: install_public_builtin_function(
            agent,
            cx,
            js3_async_disposable_stack_use_builtin(),
            None,
        ),
        adopt: install_public_builtin_function(
            agent,
            cx,
            js3_async_disposable_stack_adopt_builtin(),
            None,
        ),
        defer: install_public_builtin_function(
            agent,
            cx,
            js3_async_disposable_stack_defer_builtin(),
            None,
        ),
        move_method: install_public_builtin_function(
            agent,
            cx,
            js3_async_disposable_stack_move_builtin(),
            None,
        ),
        disposed_getter: install_public_builtin_function(
            agent,
            cx,
            js3_async_disposable_stack_disposed_getter_builtin(),
            None,
        ),
        dispose_async: install_public_builtin_function(
            agent,
            cx,
            js3_async_disposable_stack_dispose_async_builtin(),
            None,
        ),
    }
}

#[derive(Clone, Copy, Debug)]
struct DisposalHelperFamilyBuiltins {
    create_sync_disposal_scope: ObjectRef,
    create_async_disposal_scope: ObjectRef,
    add_sync_disposable_resource: ObjectRef,
    add_async_disposable_resource: ObjectRef,
    dispose_scope: ObjectRef,
    dispose_scope_async: ObjectRef,
}

fn install_disposal_helper_family(
    agent: &mut Agent,
    cx: FamilyInstallContext,
) -> DisposalHelperFamilyBuiltins {
    DisposalHelperFamilyBuiltins {
        create_sync_disposal_scope: install_public_builtin_function(
            agent,
            cx,
            js3_create_sync_disposal_scope_builtin(),
            None,
        ),
        create_async_disposal_scope: install_public_builtin_function(
            agent,
            cx,
            js3_create_async_disposal_scope_builtin(),
            None,
        ),
        add_sync_disposable_resource: install_public_builtin_function(
            agent,
            cx,
            js3_add_sync_disposable_resource_builtin(),
            None,
        ),
        add_async_disposable_resource: install_public_builtin_function(
            agent,
            cx,
            js3_add_async_disposable_resource_builtin(),
            None,
        ),
        dispose_scope: install_public_builtin_function(
            agent,
            cx,
            js3_dispose_scope_builtin(),
            None,
        ),
        dispose_scope_async: install_public_builtin_function(
            agent,
            cx,
            js3_dispose_scope_async_builtin(),
            None,
        ),
    }
}
