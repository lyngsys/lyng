//! Copyable runtime-facing data types for the lyng-js Phase 2 primitive runtime.
//!
//! Ownership: `lyng_js_types` owns representation-only runtime data. Allocation,
//! rooting, tracing, and heap dereference remain in `lyng_js_gc`.

#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::return_self_not_must_use
)]

mod builtin_ids;
mod completion;
mod ids;
mod marker;
mod property;
mod value;

pub use builtin_ids::*;
pub use completion::{AbruptCompletion, Completion};
pub use ids::{
    BackingStoreRef, BigIntRef, BuiltinFunctionId, CodeRef, EmbeddingFunctionId, EnvironmentRef,
    FeedbackSlotId, NativeFunctionId, ObjectRef, RealmRef, ShapeId, StringRef,
    SuspendedExecutionRef, SymbolRef, WellKnownSymbolId,
};
pub use marker::TypeOwnershipMarker;
pub use property::{DescriptorAttributes, DescriptorPresent, PropertyDescriptor, PropertyKey};
pub use value::{InternalSentinel, Value};

const _: [(); 8] = [(); std::mem::size_of::<PropertyKey>()];
const _: [(); 16] = [(); std::mem::size_of::<AbruptCompletion>()];

#[cfg(test)]
mod tests {
    use super::*;
    use lyng_js_common::AtomId;
    use std::any::TypeId;
    use std::collections::HashSet;
    use std::mem::size_of;
    use std::num::NonZeroU32;

    macro_rules! assert_runtime_id_contract {
        ($name:ident, $raw:expr) => {{
            let raw = $raw;
            let id = $name::from_raw(raw).expect("non-zero runtime ID");
            assert_eq!(id.get(), raw);
            assert_eq!(id.raw().get(), raw);
            assert_eq!($name::new(NonZeroU32::new(raw).unwrap()), id);
            assert_eq!($name::from_raw(id.get()), Some(id));
            assert_eq!(size_of::<$name>(), size_of::<u32>());
            assert_eq!(size_of::<Option<$name>>(), size_of::<u32>());

            let mut seen = HashSet::new();
            seen.insert(id);
            assert!(seen.contains(&$name::from_raw(raw).unwrap()));
        }};
    }

    #[test]
    fn type_marker_round_trips_property_name() {
        let property_name = AtomId::from_raw(17);
        let marker = TypeOwnershipMarker::new(property_name);

        assert_eq!(marker.property_name(), property_name);
    }

    #[test]
    fn type_marker_stays_compact_and_copyable() {
        let property_name = AtomId::from_raw(23);
        let marker = TypeOwnershipMarker::new(property_name);
        let copy = marker;

        assert_eq!(size_of::<TypeOwnershipMarker>(), size_of::<AtomId>());
        assert_eq!(copy, marker);
    }

    #[test]
    fn type_marker_hashes_by_value() {
        let property_name = AtomId::from_raw(29);
        let marker = TypeOwnershipMarker::new(property_name);

        let mut seen = HashSet::new();
        seen.insert(marker);

        assert!(seen.contains(&marker));
        assert!(seen.contains(&TypeOwnershipMarker::new(property_name)));
    }

    #[test]
    fn well_known_symbol_ids_expose_stable_descriptions() {
        assert_eq!(
            WellKnownSymbolId::HasInstance.description(),
            "Symbol.hasInstance"
        );
        assert_eq!(
            WellKnownSymbolId::IsConcatSpreadable.description(),
            "Symbol.isConcatSpreadable"
        );
        assert_eq!(WellKnownSymbolId::Iterator.description(), "Symbol.iterator");
        assert_eq!(
            WellKnownSymbolId::AsyncIterator.description(),
            "Symbol.asyncIterator"
        );
        assert_eq!(WellKnownSymbolId::Species.description(), "Symbol.species");
        assert_eq!(
            WellKnownSymbolId::ToPrimitive.description(),
            "Symbol.toPrimitive"
        );
        assert_eq!(
            WellKnownSymbolId::ToStringTag.description(),
            "Symbol.toStringTag"
        );
        assert_eq!(
            WellKnownSymbolId::Unscopables.description(),
            "Symbol.unscopables"
        );
        assert_eq!(WellKnownSymbolId::Dispose.description(), "Symbol.dispose");
        assert_eq!(
            WellKnownSymbolId::AsyncDispose.description(),
            "Symbol.asyncDispose"
        );
    }

    #[test]
    fn typed_handles_round_trip_and_stay_compact() {
        assert_runtime_id_contract!(ObjectRef, 1);
        assert_runtime_id_contract!(StringRef, 2);
        assert_runtime_id_contract!(SymbolRef, 3);
        assert_runtime_id_contract!(BigIntRef, 4);
        assert_runtime_id_contract!(EnvironmentRef, 5);
        assert_runtime_id_contract!(CodeRef, 6);
        assert_runtime_id_contract!(SuspendedExecutionRef, 7);
        assert_runtime_id_contract!(BackingStoreRef, 8);
        assert_runtime_id_contract!(RealmRef, 9);
        assert_runtime_id_contract!(ShapeId, 10);
        assert_runtime_id_contract!(FeedbackSlotId, 11);
        assert_runtime_id_contract!(BuiltinFunctionId, 12);
        assert_runtime_id_contract!(EmbeddingFunctionId, 13);
    }

    #[test]
    fn zero_payload_is_invalid_for_all_runtime_ids() {
        assert_eq!(ObjectRef::from_raw(0), None);
        assert_eq!(StringRef::from_raw(0), None);
        assert_eq!(SymbolRef::from_raw(0), None);
        assert_eq!(BigIntRef::from_raw(0), None);
        assert_eq!(EnvironmentRef::from_raw(0), None);
        assert_eq!(CodeRef::from_raw(0), None);
        assert_eq!(SuspendedExecutionRef::from_raw(0), None);
        assert_eq!(BackingStoreRef::from_raw(0), None);
        assert_eq!(RealmRef::from_raw(0), None);
        assert_eq!(ShapeId::from_raw(0), None);
        assert_eq!(FeedbackSlotId::from_raw(0), None);
        assert_eq!(BuiltinFunctionId::from_raw(0), None);
        assert_eq!(EmbeddingFunctionId::from_raw(0), None);
    }

    #[test]
    fn builtin_internal_namespace_classifies_helper_ids() {
        let internal = internal_function_call_builtin();
        let dynamic_import = internal_dynamic_import_builtin();
        let public = boolean_builtin();
        let phase6 = promise_builtin();

        assert!(is_internal_builtin(internal));
        assert!(is_internal_builtin(dynamic_import));
        assert!(!is_internal_builtin(public));
        assert!(!is_internal_builtin(phase6));
        assert!(is_core_builtin(public));
        assert!(!is_core_builtin(phase6));
        assert!(is_completion_builtin(phase6));
        assert_eq!(INTERNAL_BUILTIN_NAMESPACE_START, 1_001);
        assert_eq!(INTERNAL_BUILTIN_NAMESPACE_END, 1_036);
        assert_eq!(CORE_BUILTIN_NAMESPACE_START, 2_001);
        assert_eq!(CORE_BUILTIN_NAMESPACE_END, 2_284);
        assert_eq!(COMPLETION_BUILTIN_NAMESPACE_START, 3_101);
        assert_eq!(COMPLETION_BUILTIN_NAMESPACE_END, 3_500);
    }

    #[test]
    fn native_function_ids_preserve_builtin_and_embedding_lanes() {
        let builtin = function_builtin();
        let embedding = EmbeddingFunctionId::from_raw(17).expect("non-zero embedding id");

        assert_eq!(
            NativeFunctionId::builtin(builtin).builtin_entry(),
            Some(builtin)
        );
        assert_eq!(NativeFunctionId::builtin(builtin).embedding_entry(), None);
        assert_eq!(NativeFunctionId::embedding(embedding).builtin_entry(), None);
        assert_eq!(
            NativeFunctionId::embedding(embedding).embedding_entry(),
            Some(embedding)
        );
    }

    #[test]
    fn runtime_id_families_remain_type_separated() {
        assert_ne!(TypeId::of::<ObjectRef>(), TypeId::of::<StringRef>());
        assert_ne!(TypeId::of::<StringRef>(), TypeId::of::<SymbolRef>());
        assert_ne!(TypeId::of::<BigIntRef>(), TypeId::of::<EnvironmentRef>());
        assert_ne!(TypeId::of::<CodeRef>(), TypeId::of::<RealmRef>());
        assert_ne!(
            TypeId::of::<SuspendedExecutionRef>(),
            TypeId::of::<BackingStoreRef>()
        );
        assert_ne!(TypeId::of::<ShapeId>(), TypeId::of::<FeedbackSlotId>());
        assert_ne!(
            TypeId::of::<FeedbackSlotId>(),
            TypeId::of::<BuiltinFunctionId>()
        );
        assert_ne!(
            TypeId::of::<BuiltinFunctionId>(),
            TypeId::of::<EmbeddingFunctionId>()
        );
    }

    #[test]
    fn runtime_id_debug_and_equality_are_raw_value_based() {
        let a = ObjectRef::from_raw(11).unwrap();
        let b = ObjectRef::from_raw(11).unwrap();
        let c = ObjectRef::from_raw(12).unwrap();

        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(format!("{a:?}"), "ObjectRef(11)");
        assert_eq!(
            format!("{:?}", BuiltinFunctionId::from_raw(13).unwrap()),
            "BuiltinFunctionId(13)"
        );
    }
}
