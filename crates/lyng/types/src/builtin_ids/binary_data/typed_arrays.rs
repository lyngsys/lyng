use super::super::{builtin_id, BuiltinFunctionId};

builtin_id_accessors! {
    int8_array_builtin => super::super::INT8_ARRAY_RAW;
    int16_array_builtin => super::super::INT16_ARRAY_RAW;
    int32_array_builtin => super::super::INT32_ARRAY_RAW;
    uint32_array_builtin => super::super::UINT32_ARRAY_RAW;
    float32_array_builtin => super::super::FLOAT32_ARRAY_RAW;
    float16_array_builtin => super::super::FLOAT16_ARRAY_RAW;
    float64_array_builtin => super::super::FLOAT64_ARRAY_RAW;
    uint8_clamped_array_builtin => super::super::UINT8_CLAMPED_ARRAY_RAW;
    big_uint64_array_builtin => super::super::BIG_UINT64_ARRAY_RAW;
    big_int64_array_builtin => super::super::BIG_INT64_ARRAY_RAW;
}
