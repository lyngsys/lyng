mod atomics;
mod buffers;
mod data_view;
mod typed_arrays;

use crate::bootstrap::{install_descriptor_tables, BuiltinBootstrapError};
use crate::public::{BuiltinCache, PublicRealmBuiltins};
use crate::{BuiltinDescriptorTable, BuiltinInstallTarget, BuiltinIntrinsic};
use lyng_js_env::Agent;
use lyng_js_gc::AllocationLifetime;
use lyng_js_types::{RealmRef, Value};

#[allow(clippy::too_many_lines)]
pub(in crate::public) fn install_binary_data_family_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    builtins: &PublicRealmBuiltins,
) -> Result<(), BuiltinBootstrapError> {
    let bootstrap_atoms = agent.bootstrap_atoms();
    let array_buffer_tag = Value::from_string_ref(agent.alloc_runtime_string(
        "ArrayBuffer",
        Some(bootstrap_atoms.array_buffer()),
        AllocationLifetime::Default,
    ));
    let shared_array_buffer_tag = Value::from_string_ref(agent.alloc_runtime_string(
        "SharedArrayBuffer",
        Some(bootstrap_atoms.shared_array_buffer()),
        AllocationLifetime::Default,
    ));
    let data_view_tag = Value::from_string_ref(agent.alloc_runtime_string(
        "DataView",
        Some(bootstrap_atoms.data_view()),
        AllocationLifetime::Default,
    ));
    let atomics_tag = Value::from_string_ref(agent.alloc_runtime_string(
        "Atomics",
        Some(bootstrap_atoms.atomics()),
        AllocationLifetime::Default,
    ));
    let int8_array_tag = Value::from_string_ref(agent.alloc_runtime_string(
        "Int8Array",
        Some(bootstrap_atoms.int8_array()),
        AllocationLifetime::Default,
    ));
    let int16_array_tag = Value::from_string_ref(agent.alloc_runtime_string(
        "Int16Array",
        Some(bootstrap_atoms.int16_array()),
        AllocationLifetime::Default,
    ));
    let int32_array_tag = Value::from_string_ref(agent.alloc_runtime_string(
        "Int32Array",
        Some(bootstrap_atoms.int32_array()),
        AllocationLifetime::Default,
    ));
    let float32_array_tag = Value::from_string_ref(agent.alloc_runtime_string(
        "Float32Array",
        Some(bootstrap_atoms.float32_array()),
        AllocationLifetime::Default,
    ));
    let float64_array_tag = Value::from_string_ref(agent.alloc_runtime_string(
        "Float64Array",
        Some(bootstrap_atoms.float64_array()),
        AllocationLifetime::Default,
    ));
    let big_int64_array_tag = Value::from_string_ref(agent.alloc_runtime_string(
        "BigInt64Array",
        Some(bootstrap_atoms.big_int64_array()),
        AllocationLifetime::Default,
    ));
    let big_uint64_array_tag = Value::from_string_ref(agent.alloc_runtime_string(
        "BigUint64Array",
        Some(bootstrap_atoms.big_uint64_array()),
        AllocationLifetime::Default,
    ));
    let uint32_array_tag = Value::from_string_ref(agent.alloc_runtime_string(
        "Uint32Array",
        Some(bootstrap_atoms.uint32_array()),
        AllocationLifetime::Default,
    ));
    let uint16_array_tag = Value::from_string_ref(agent.alloc_runtime_string(
        "Uint16Array",
        Some(bootstrap_atoms.uint16_array()),
        AllocationLifetime::Default,
    ));
    let uint8_clamped_array_tag = Value::from_string_ref(agent.alloc_runtime_string(
        "Uint8ClampedArray",
        Some(bootstrap_atoms.uint8_clamped_array()),
        AllocationLifetime::Default,
    ));
    let uint8_array_tag = Value::from_string_ref(agent.alloc_runtime_string(
        "Uint8Array",
        Some(bootstrap_atoms.uint8_array()),
        AllocationLifetime::Default,
    ));
    let last_index_of_atom = agent.atoms_mut().intern("lastIndexOf");
    let copy_within_atom = agent.atoms_mut().intern("copyWithin");
    let entries_atom = agent.atoms_mut().intern("entries");
    let every_atom = agent.atoms_mut().intern("every");
    let fill_atom = agent.atoms_mut().intern("fill");
    let filter_atom = agent.atoms_mut().intern("filter");
    let find_atom = agent.atoms_mut().intern("find");
    let find_index_atom = agent.atoms_mut().intern("findIndex");
    let find_last_atom = agent.atoms_mut().intern("findLast");
    let find_last_index_atom = agent.atoms_mut().intern("findLastIndex");
    let from_atom = agent.atoms_mut().intern("from");
    let for_each_atom = agent.atoms_mut().intern("forEach");
    let includes_atom = agent.atoms_mut().intern("includes");
    let index_of_atom = agent.atoms_mut().intern("indexOf");
    let join_atom = agent.atoms_mut().intern("join");
    let keys_atom = agent.atoms_mut().intern("keys");
    let map_atom = agent.atoms_mut().intern("map");
    let of_atom = agent.atoms_mut().intern("of");
    let reduce_atom = agent.atoms_mut().intern("reduce");
    let reduce_right_atom = agent.atoms_mut().intern("reduceRight");
    let reverse_atom = agent.atoms_mut().intern("reverse");
    let some_atom = agent.atoms_mut().intern("some");
    let at_atom = agent.atoms_mut().intern("at");
    let slice_atom = agent.atoms_mut().intern("slice");
    let buffer_atom = agent.atoms_mut().intern("buffer");
    let byte_length_atom = agent.atoms_mut().intern("byteLength");
    let byte_offset_atom = agent.atoms_mut().intern("byteOffset");
    let bytes_per_element_atom = agent.atoms_mut().intern("BYTES_PER_ELEMENT");
    let is_view_atom = agent.atoms_mut().intern("isView");
    let sort_atom = agent.atoms_mut().intern("sort");
    let to_locale_string_atom = agent.atoms_mut().intern("toLocaleString");
    let to_reversed_atom = agent.atoms_mut().intern("toReversed");
    let to_sorted_atom = agent.atoms_mut().intern("toSorted");
    let values_atom = agent.atoms_mut().intern("values");
    let with_atom = agent.atoms_mut().intern("with");
    let get_float32_atom = agent.atoms_mut().intern("getFloat32");
    let get_float64_atom = agent.atoms_mut().intern("getFloat64");
    let get_int16_atom = agent.atoms_mut().intern("getInt16");
    let get_int32_atom = agent.atoms_mut().intern("getInt32");
    let get_int8_atom = agent.atoms_mut().intern("getInt8");
    let get_uint16_atom = agent.atoms_mut().intern("getUint16");
    let get_uint32_atom = agent.atoms_mut().intern("getUint32");
    let get_uint8_atom = agent.atoms_mut().intern("getUint8");
    let add_atom = agent.atoms_mut().intern("add");
    let and_atom = agent.atoms_mut().intern("and");
    let compare_exchange_atom = agent.atoms_mut().intern("compareExchange");
    let exchange_atom = agent.atoms_mut().intern("exchange");
    let is_lock_free_atom = agent.atoms_mut().intern("isLockFree");
    let load_atom = agent.atoms_mut().intern("load");
    let notify_atom = agent.atoms_mut().intern("notify");
    let or_atom = agent.atoms_mut().intern("or");
    let set_atom = agent.atoms_mut().intern("set");
    let set_float32_atom = agent.atoms_mut().intern("setFloat32");
    let set_float64_atom = agent.atoms_mut().intern("setFloat64");
    let set_int16_atom = agent.atoms_mut().intern("setInt16");
    let set_int32_atom = agent.atoms_mut().intern("setInt32");
    let set_int8_atom = agent.atoms_mut().intern("setInt8");
    let set_uint16_atom = agent.atoms_mut().intern("setUint16");
    let set_uint32_atom = agent.atoms_mut().intern("setUint32");
    let set_uint8_atom = agent.atoms_mut().intern("setUint8");
    let store_atom = agent.atoms_mut().intern("store");
    let sub_atom = agent.atoms_mut().intern("sub");
    let subarray_atom = agent.atoms_mut().intern("subarray");
    let wait_atom = agent.atoms_mut().intern("wait");
    let wait_async_atom = agent.atoms_mut().intern("waitAsync");
    let xor_atom = agent.atoms_mut().intern("xor");
    let buffer_descriptor_sets = buffers::descriptor_sets(
        builtins,
        buffers::BufferDescriptorAtoms {
            is_view: is_view_atom,
            byte_length: byte_length_atom,
            slice: slice_atom,
        },
        buffers::BufferDescriptorTags {
            array_buffer: array_buffer_tag,
            shared_array_buffer: shared_array_buffer_tag,
        },
    );
    let atomics_descriptors = atomics::descriptors(
        atomics::AtomicsDescriptorAtoms {
            add: add_atom,
            and: and_atom,
            compare_exchange: compare_exchange_atom,
            exchange: exchange_atom,
            is_lock_free: is_lock_free_atom,
            load: load_atom,
            notify: notify_atom,
            or: or_atom,
            store: store_atom,
            sub: sub_atom,
            wait: wait_atom,
            wait_async: wait_async_atom,
            xor: xor_atom,
        },
        atomics_tag,
    );
    let data_view_descriptor_sets = data_view::descriptor_sets(
        builtins,
        data_view::DataViewDescriptorAtoms {
            buffer: buffer_atom,
            byte_length: byte_length_atom,
            byte_offset: byte_offset_atom,
            get_float32: get_float32_atom,
            get_float64: get_float64_atom,
            get_int16: get_int16_atom,
            get_int32: get_int32_atom,
            get_int8: get_int8_atom,
            get_uint16: get_uint16_atom,
            get_uint32: get_uint32_atom,
            get_uint8: get_uint8_atom,
            set_float32: set_float32_atom,
            set_float64: set_float64_atom,
            set_int16: set_int16_atom,
            set_int32: set_int32_atom,
            set_int8: set_int8_atom,
            set_uint16: set_uint16_atom,
            set_uint32: set_uint32_atom,
            set_uint8: set_uint8_atom,
        },
        data_view_tag,
    );
    let typed_array_descriptor_sets = typed_arrays::descriptor_sets(
        builtins,
        typed_arrays::TypedArrayDescriptorAtoms {
            last_index_of: last_index_of_atom,
            copy_within: copy_within_atom,
            entries: entries_atom,
            every: every_atom,
            fill: fill_atom,
            filter: filter_atom,
            find: find_atom,
            find_index: find_index_atom,
            find_last: find_last_atom,
            find_last_index: find_last_index_atom,
            from: from_atom,
            for_each: for_each_atom,
            includes: includes_atom,
            index_of: index_of_atom,
            join: join_atom,
            keys: keys_atom,
            map: map_atom,
            of: of_atom,
            reduce: reduce_atom,
            reduce_right: reduce_right_atom,
            reverse: reverse_atom,
            some: some_atom,
            at: at_atom,
            slice: slice_atom,
            buffer: buffer_atom,
            byte_length: byte_length_atom,
            byte_offset: byte_offset_atom,
            bytes_per_element: bytes_per_element_atom,
            sort: sort_atom,
            to_locale_string: to_locale_string_atom,
            to_reversed: to_reversed_atom,
            to_sorted: to_sorted_atom,
            values: values_atom,
            with: with_atom,
            set: set_atom,
            subarray: subarray_atom,
        },
        typed_arrays::TypedArrayDescriptorTags {
            int8_array: int8_array_tag,
            int16_array: int16_array_tag,
            int32_array: int32_array_tag,
            float32_array: float32_array_tag,
            float64_array: float64_array_tag,
            big_int64_array: big_int64_array_tag,
            big_uint64_array: big_uint64_array_tag,
            uint32_array: uint32_array_tag,
            uint16_array: uint16_array_tag,
            uint8_clamped_array: uint8_clamped_array_tag,
            uint8_array: uint8_array_tag,
        },
    );
    let tables = [
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::ArrayBuffer),
            &buffer_descriptor_sets.array_buffer,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::ArrayBufferPrototype),
            &buffer_descriptor_sets.array_buffer_prototype,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::SharedArrayBuffer),
            &buffer_descriptor_sets.shared_array_buffer,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::SharedArrayBufferPrototype),
            &buffer_descriptor_sets.shared_array_buffer_prototype,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Atomics),
            &atomics_descriptors,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::DataView),
            &data_view_descriptor_sets.data_view,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::DataViewPrototype),
            &data_view_descriptor_sets.data_view_prototype,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::TypedArray),
            &typed_array_descriptor_sets.typed_array_descriptors,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::TypedArrayPrototype),
            &typed_array_descriptor_sets.typed_array_prototype_descriptors,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Int8Array),
            &typed_array_descriptor_sets.int8_array_descriptors,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Int8ArrayPrototype),
            &typed_array_descriptor_sets.int8_array_prototype_descriptors,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Int16Array),
            &typed_array_descriptor_sets.int16_array_descriptors,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Int16ArrayPrototype),
            &typed_array_descriptor_sets.int16_array_prototype_descriptors,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Int32Array),
            &typed_array_descriptor_sets.int32_array_descriptors,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Int32ArrayPrototype),
            &typed_array_descriptor_sets.int32_array_prototype_descriptors,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Float32Array),
            &typed_array_descriptor_sets.float32_array_descriptors,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Float32ArrayPrototype),
            &typed_array_descriptor_sets.float32_array_prototype_descriptors,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Float64Array),
            &typed_array_descriptor_sets.float64_array_descriptors,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Float64ArrayPrototype),
            &typed_array_descriptor_sets.float64_array_prototype_descriptors,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::BigInt64Array),
            &typed_array_descriptor_sets.big_int64_array_descriptors,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::BigInt64ArrayPrototype),
            &typed_array_descriptor_sets.big_int64_array_prototype_descriptors,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::BigUint64Array),
            &typed_array_descriptor_sets.big_uint64_array_descriptors,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::BigUint64ArrayPrototype),
            &typed_array_descriptor_sets.big_uint64_array_prototype_descriptors,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Uint32Array),
            &typed_array_descriptor_sets.uint32_array_descriptors,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Uint32ArrayPrototype),
            &typed_array_descriptor_sets.uint32_array_prototype_descriptors,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Uint16Array),
            &typed_array_descriptor_sets.uint16_array_descriptors,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Uint16ArrayPrototype),
            &typed_array_descriptor_sets.uint16_array_prototype_descriptors,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Uint8ClampedArray),
            &typed_array_descriptor_sets.uint8_clamped_array_descriptors,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Uint8ClampedArrayPrototype),
            &typed_array_descriptor_sets.uint8_clamped_array_prototype_descriptors,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Uint8Array),
            &typed_array_descriptor_sets.uint8_array_descriptors,
        ),
        BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Uint8ArrayPrototype),
            &typed_array_descriptor_sets.uint8_array_prototype_descriptors,
        ),
    ];
    install_descriptor_tables(agent, cache, realm, &tables)
}
