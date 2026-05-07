use super::super::super::{
    uint8_array_from_base64_builtin, uint8_array_from_hex_builtin,
    uint8_array_set_from_base64_builtin, uint8_array_set_from_hex_builtin,
    uint8_array_to_base64_builtin, uint8_array_to_hex_builtin,
};
use super::super::buffers::allocate_array_buffer_object;
use super::super::{
    create_data_property_or_throw, property_key_from_text, range_error, string_value, syntax_error,
    to_boolean_for_builtin, type_error, PublicBuiltinDispatchContext,
};
use super::{allocate_typed_array_object, typed_array_this_record};
use crate::BuiltinInvocation;
use lyng_js_objects::{TypedArrayElementKind, TypedArrayObjectData};
use lyng_js_types::{BuiltinFunctionId, ObjectRef, RealmRef, Value};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Base64Alphabet {
    Standard,
    Url,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LastChunkHandling {
    Loose,
    Strict,
    StopBeforePartial,
}

pub(in crate::public::dispatch::binary_data) fn dispatch_uint8_array_base64_hex_builtin<
    Cx: PublicBuiltinDispatchContext,
>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == uint8_array_from_base64_builtin() {
        return uint8_array_from_base64_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == uint8_array_from_hex_builtin() {
        return uint8_array_from_hex_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == uint8_array_set_from_base64_builtin() {
        return uint8_array_set_from_base64_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == uint8_array_set_from_hex_builtin() {
        return uint8_array_set_from_hex_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == uint8_array_to_base64_builtin() {
        return uint8_array_to_base64_builtin_dispatch(context, invocation).map(Some);
    }
    if entry == uint8_array_to_hex_builtin() {
        return uint8_array_to_hex_builtin_dispatch(context, invocation).map(Some);
    }
    Ok(None)
}

fn require_string_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<Value, Cx::Error> {
    if !value.is_string() {
        return Err(type_error(cx));
    }
    Ok(value)
}

fn parse_alphabet_option<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    options: Value,
) -> Result<Base64Alphabet, Cx::Error> {
    if !options.is_object() {
        return Ok(Base64Alphabet::Standard);
    }
    let key = property_key_from_text(cx, "alphabet");
    let raw = cx.get_property_value(options, key)?;
    if raw.is_undefined() {
        return Ok(Base64Alphabet::Standard);
    }
    if !raw.is_string() {
        return Err(type_error(cx));
    }
    let text = cx.value_to_string_text(raw)?;
    match text.as_str() {
        "base64" => Ok(Base64Alphabet::Standard),
        "base64url" => Ok(Base64Alphabet::Url),
        _ => Err(type_error(cx)),
    }
}

fn parse_last_chunk_handling_option<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    options: Value,
) -> Result<LastChunkHandling, Cx::Error> {
    if !options.is_object() {
        return Ok(LastChunkHandling::Loose);
    }
    let key = property_key_from_text(cx, "lastChunkHandling");
    let raw = cx.get_property_value(options, key)?;
    if raw.is_undefined() {
        return Ok(LastChunkHandling::Loose);
    }
    if !raw.is_string() {
        return Err(type_error(cx));
    }
    let text = cx.value_to_string_text(raw)?;
    match text.as_str() {
        "loose" => Ok(LastChunkHandling::Loose),
        "strict" => Ok(LastChunkHandling::Strict),
        "stop-before-partial" => Ok(LastChunkHandling::StopBeforePartial),
        _ => Err(type_error(cx)),
    }
}

fn parse_omit_padding_option<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    options: Value,
) -> Result<bool, Cx::Error> {
    if !options.is_object() {
        return Ok(false);
    }
    let key = property_key_from_text(cx, "omitPadding");
    let raw = cx.get_property_value(options, key)?;
    if raw.is_undefined() {
        return Ok(false);
    }
    to_boolean_for_builtin(cx, raw)
}

fn options_argument(invocation: &BuiltinInvocation<'_>, index: usize) -> Value {
    invocation
        .arguments()
        .get(index)
        .copied()
        .unwrap_or(Value::undefined())
}

fn options_object_or_throw<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<Value, Cx::Error> {
    if value.is_undefined() {
        return Ok(Value::undefined());
    }
    if !value.is_object() {
        return Err(type_error(cx));
    }
    Ok(value)
}

fn require_uint8_array_record<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TypedArrayObjectData, Cx::Error> {
    let record = typed_array_this_record(cx, value)?;
    if record.kind() != TypedArrayElementKind::Uint8 {
        return Err(type_error(cx));
    }
    Ok(record)
}

const fn is_ascii_whitespace(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\u{000C}' | '\r' | ' ')
}

fn base64_alphabet_value(alphabet: Base64Alphabet, c: char) -> Option<u8> {
    let value = match c {
        'A'..='Z' => (c as u8) - b'A',
        'a'..='z' => (c as u8) - b'a' + 26,
        '0'..='9' => (c as u8) - b'0' + 52,
        '+' if alphabet == Base64Alphabet::Standard => 62,
        '/' if alphabet == Base64Alphabet::Standard => 63,
        '-' if alphabet == Base64Alphabet::Url => 62,
        '_' if alphabet == Base64Alphabet::Url => 63,
        _ => return None,
    };
    Some(value)
}

const fn hex_value(c: char) -> Option<u8> {
    match c {
        '0'..='9' => Some((c as u8) - b'0'),
        'a'..='f' => Some((c as u8) - b'a' + 10),
        'A'..='F' => Some((c as u8) - b'A' + 10),
        _ => None,
    }
}

#[derive(Default)]
struct DecodeBase64Result {
    bytes: Vec<u8>,
    /// Number of UTF-16 code units consumed from the source string.
    read: usize,
    /// `Some(())` if a `SyntaxError` should be raised after writing the
    /// already-decoded bytes; otherwise the decode finished cleanly.
    error: bool,
}

/// Decode `input` as base64 into bytes.
///
/// Mirrors the proposal's `FromBase64` abstract operation.
/// `max_length` of `None` means unbounded (used by `fromBase64`).
fn decode_base64(
    input: &str,
    alphabet: Base64Alphabet,
    last_chunk_handling: LastChunkHandling,
    max_length: Option<usize>,
) -> DecodeBase64Result {
    let mut result = DecodeBase64Result::default();
    if max_length == Some(0) {
        return result;
    }

    // Iterate over UTF-16 code units so `read` matches JS string indices.
    let units: Vec<u16> = input.encode_utf16().collect();
    // Map UTF-16 code units back to chars (BMP characters give single code unit).
    // Surrogate halves are not valid base64 chars, so we treat them as illegal.
    let mut chunk: [u8; 4] = [0; 4];
    let mut chunk_len: usize = 0;
    let mut i: usize = 0;

    loop {
        // Skip leading whitespace in this chunk position
        while i < units.len() {
            let unit = units[i];
            if !(0x20..0x7F).contains(&unit)
                && unit != 0x09
                && unit != 0x0A
                && unit != 0x0C
                && unit != 0x0D
            {
                // not ASCII printable or allowed whitespace -> illegal
                break;
            }
            let c = char::from_u32(u32::from(unit)).unwrap_or('\u{FFFD}');
            if is_ascii_whitespace(c) {
                i += 1;
                continue;
            }
            break;
        }

        if i >= units.len() {
            // End of input. If chunk_len > 0, handle remainder.
            if chunk_len == 0 {
                return result;
            }
            match last_chunk_handling {
                LastChunkHandling::Loose => {
                    // Treat remaining chunk as if padded sufficiently.
                    if chunk_len == 1 {
                        result.error = true;
                        return result;
                    }
                    if let Some(bytes) = decode_partial_chunk(chunk, chunk_len, false) {
                        // If the partial chunk would not fit, silently stop
                        // (no error, no write, do not update read).
                        if let Some(max) = max_length
                            && result.bytes.len() + bytes.len() > max
                        {
                            return result;
                        }
                        result.bytes.extend_from_slice(&bytes);
                        result.read = i;
                        return result;
                    }
                    result.error = true;
                    return result;
                }
                LastChunkHandling::Strict => {
                    // Strict requires explicit padding for non-multiple-of-4 input
                    // (and zero pad bits). Missing padding is an error.
                    result.error = true;
                    return result;
                }
                LastChunkHandling::StopBeforePartial => {
                    return result;
                }
            }
        }

        let unit = units[i];
        // Check non-ASCII printable
        if unit >= 0x80 {
            // Skip whitespace already handled above; non-ASCII is illegal.
            result.error = true;
            return result;
        }
        let c = char::from_u32(u32::from(unit)).unwrap_or('\u{FFFD}');

        if c == '=' {
            // Start of padding sequence.
            handle_base64_padding(
                &units,
                &mut i,
                chunk,
                chunk_len,
                last_chunk_handling,
                max_length,
                &mut result,
            );
            return result;
        }

        let Some(value) = base64_alphabet_value(alphabet, c) else {
            // Illegal char.
            result.error = true;
            return result;
        };
        chunk[chunk_len] = value;
        chunk_len += 1;
        i += 1;

        if chunk_len == 4 {
            // Complete 4-char chunk -> 3 output bytes.
            // Per spec FromBase64 step 10.l.ii: if the chunk would not fit,
            // return without writing and without updating read.
            if let Some(max) = max_length
                && result.bytes.len() + 3 > max
            {
                return result;
            }
            let bytes = decode_complete_chunk(chunk);
            result.bytes.extend_from_slice(&bytes);
            result.read = i;
            chunk_len = 0;

            if let Some(max) = max_length
                && result.bytes.len() == max
            {
                return result;
            }
        }
    }
}

fn byte_from_low_u32(value: u32) -> u8 {
    u8::try_from(value & 0xff).expect("masked byte should fit u8")
}

fn base64_table_char(table: &[u8; 64], index: u32) -> char {
    char::from(table[usize::try_from(index & 0x3F).expect("base64 sextet index should fit usize")])
}

fn decode_complete_chunk(chunk: [u8; 4]) -> [u8; 3] {
    let combined: u32 = (u32::from(chunk[0]) << 18)
        | (u32::from(chunk[1]) << 12)
        | (u32::from(chunk[2]) << 6)
        | u32::from(chunk[3]);
    [
        byte_from_low_u32(combined >> 16),
        byte_from_low_u32(combined >> 8),
        byte_from_low_u32(combined),
    ]
}

/// Decode a partial chunk of `chunk_len` chars (2 or 3) treating it as if
/// padded. Returns `None` if the trailing pad bits are non-zero and `strict_pad`
/// is true.
fn decode_partial_chunk(chunk: [u8; 4], chunk_len: usize, strict_pad: bool) -> Option<Vec<u8>> {
    debug_assert!(chunk_len == 2 || chunk_len == 3);
    if chunk_len == 2 {
        // 12 bits -> 1 byte (4 trailing bits dropped). Strict requires those to be zero.
        let combined: u32 = (u32::from(chunk[0]) << 6) | u32::from(chunk[1]);
        let byte = (combined >> 4) & 0xff;
        let trailing = combined & 0x0F;
        if strict_pad && trailing != 0 {
            return None;
        }
        Some(vec![byte_from_low_u32(byte)])
    } else {
        // 18 bits -> 2 bytes (2 trailing bits dropped).
        let combined: u32 =
            (u32::from(chunk[0]) << 12) | (u32::from(chunk[1]) << 6) | u32::from(chunk[2]);
        let trailing = combined & 0x03;
        if strict_pad && trailing != 0 {
            return None;
        }
        Some(vec![
            byte_from_low_u32(combined >> 10),
            byte_from_low_u32(combined >> 2),
        ])
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_base64_padding(
    units: &[u16],
    i: &mut usize,
    chunk: [u8; 4],
    chunk_len: usize,
    last_chunk_handling: LastChunkHandling,
    max_length: Option<usize>,
    result: &mut DecodeBase64Result,
) {
    // We're at units[*i] == '='. Determine pad count needed: 4 - chunk_len.
    let needed_pads: usize = match chunk_len {
        2 => 2,
        3 => 1,
        _ => {
            // chunk_len 0 or 1 with padding is illegal.
            result.error = true;
            return;
        }
    };

    // Consume the expected number of '=' signs (with allowed whitespace between).
    let mut consumed = 0_usize;
    while consumed < needed_pads {
        if *i >= units.len() {
            // Padding is insufficient. With stop-before-partial, bail silently
            // without writing or updating read. Loose/strict treat insufficient
            // padding as a SyntaxError.
            if last_chunk_handling == LastChunkHandling::StopBeforePartial {
                return;
            }
            result.error = true;
            return;
        }
        let unit = units[*i];
        let c = char::from_u32(u32::from(unit)).unwrap_or('\u{FFFD}');
        if is_ascii_whitespace(c) {
            *i += 1;
            continue;
        }
        if c != '=' {
            // A non-pad, non-whitespace char appearing when more padding is
            // expected is a SyntaxError (or a stop for stop-before-partial).
            if last_chunk_handling == LastChunkHandling::StopBeforePartial {
                return;
            }
            result.error = true;
            return;
        }
        *i += 1;
        consumed += 1;
    }

    // After consuming pad chars, only whitespace is allowed.
    while *i < units.len() {
        let unit = units[*i];
        let c = char::from_u32(u32::from(unit)).unwrap_or('\u{FFFD}');
        if is_ascii_whitespace(c) {
            *i += 1;
            continue;
        }
        // Trailing non-whitespace after padding is illegal.
        result.error = true;
        return;
    }

    // Produce bytes from the partial chunk, with strict padding if requested.
    let strict_pad = last_chunk_handling == LastChunkHandling::Strict;
    let Some(bytes) = decode_partial_chunk(chunk, chunk_len, strict_pad) else {
        result.error = true;
        return;
    };
    if let Some(max) = max_length
        && result.bytes.len() + bytes.len() > max
    {
        // Per spec, when there isn't room for the partial chunk, stop
        // without error.
        return;
    }
    result.bytes.extend_from_slice(&bytes);
    result.read = *i;
}

#[derive(Default)]
struct DecodeHexResult {
    bytes: Vec<u8>,
    read: usize,
    error: bool,
}

/// Decode `input` as hex into bytes.
///
/// Per the proposal, `Uint8Array.fromHex` and `prototype.setFromHex` use the
/// `FromHex` abstract operation which does not skip whitespace. Odd-length input
/// is a `SyntaxError`. Each pair must be `[0-9A-Fa-f]{2}`.
fn decode_hex(input: &str, max_length: Option<usize>) -> DecodeHexResult {
    let mut result = DecodeHexResult::default();
    let units: Vec<u16> = input.encode_utf16().collect();

    if !units.len().is_multiple_of(2) {
        // Even an empty `max_length=0` array still rejects odd-length input
        // (per `Uint8Array.fromHex('a')` -> SyntaxError). For setFromHex with
        // max_length=0, the spec also throws SyntaxError; the trailing-garbage
        // shortcut applies only to base64.
        result.error = true;
        return result;
    }

    let pair_count = units.len() / 2;
    let limit = max_length.map_or(pair_count, |max| pair_count.min(max));

    for pair_index in 0..pair_count {
        if result.bytes.len() == limit && max_length.is_some() {
            // Reached max_length: stop without error per spec (the remaining
            // valid characters are silently dropped).
            return result;
        }
        let high_unit = units[pair_index * 2];
        let low_unit = units[pair_index * 2 + 1];
        if high_unit >= 0x80 || low_unit >= 0x80 {
            result.error = true;
            return result;
        }
        let high_char = char::from_u32(u32::from(high_unit)).unwrap_or('\u{FFFD}');
        let low_char = char::from_u32(u32::from(low_unit)).unwrap_or('\u{FFFD}');
        let Some(high) = hex_value(high_char) else {
            result.error = true;
            return result;
        };
        let Some(low) = hex_value(low_char) else {
            result.error = true;
            return result;
        };
        result.bytes.push((high << 4) | low);
        result.read = (pair_index + 1) * 2;
    }
    result
}

fn encode_base64(bytes: &[u8], alphabet: Base64Alphabet, omit_padding: bool) -> String {
    let table: &[u8; 64] = match alphabet {
        Base64Alphabet::Standard => {
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
        }
        Base64Alphabet::Url => b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_",
    };
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    let chunks = bytes.chunks_exact(3);
    let remainder = chunks.remainder();
    for chunk in chunks {
        let combined =
            (u32::from(chunk[0]) << 16) | (u32::from(chunk[1]) << 8) | u32::from(chunk[2]);
        out.push(base64_table_char(table, combined >> 18));
        out.push(base64_table_char(table, combined >> 12));
        out.push(base64_table_char(table, combined >> 6));
        out.push(base64_table_char(table, combined));
    }
    match remainder.len() {
        0 => {}
        1 => {
            let combined = u32::from(remainder[0]) << 16;
            out.push(base64_table_char(table, combined >> 18));
            out.push(base64_table_char(table, combined >> 12));
            if !omit_padding {
                out.push('=');
                out.push('=');
            }
        }
        2 => {
            let combined = (u32::from(remainder[0]) << 16) | (u32::from(remainder[1]) << 8);
            out.push(base64_table_char(table, combined >> 18));
            out.push(base64_table_char(table, combined >> 12));
            out.push(base64_table_char(table, combined >> 6));
            if !omit_padding {
                out.push('=');
            }
        }
        _ => unreachable!(),
    }
    out
}

fn encode_hex(bytes: &[u8]) -> String {
    const TABLE: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes.iter().copied() {
        out.push(char::from(TABLE[usize::from((byte >> 4) & 0x0F)]));
        out.push(char::from(TABLE[usize::from(byte & 0x0F)]));
    }
    out
}

fn read_typed_array_bytes<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    record: TypedArrayObjectData,
) -> Option<Vec<u8>> {
    let length = record.length();
    let mut out = Vec::with_capacity(length);
    let store = record.backing_store();
    for index in 0..length {
        let byte_index = record.byte_offset().checked_add(index)?;
        let byte = cx.agent().backing_store_get_byte(store, byte_index)?;
        out.push(byte);
    }
    Some(out)
}

fn write_typed_array_bytes<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    record: TypedArrayObjectData,
    bytes: &[u8],
) -> Result<(), Cx::Error> {
    let store = record.backing_store();
    for (index, byte) in bytes.iter().copied().enumerate() {
        let byte_index = record
            .byte_offset()
            .checked_add(index)
            .ok_or_else(|| range_error(cx))?;
        if !cx.agent().backing_store_set_byte(store, byte_index, byte) {
            return Err(range_error(cx));
        }
    }
    Ok(())
}

fn allocate_uint8_array_from_bytes<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    bytes: &[u8],
) -> Result<ObjectRef, Cx::Error> {
    let array_buffer_prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().array_buffer_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let uint8_array_prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().uint8_array_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let length = bytes.len();
    let store = cx
        .agent()
        .allocate_backing_store(length)
        .ok_or_else(|| range_error(cx))?;
    for (index, byte) in bytes.iter().copied().enumerate() {
        if !cx.agent().backing_store_set_byte(store, index, byte) {
            return Err(range_error(cx));
        }
    }
    let buffer_object = allocate_array_buffer_object(cx, realm, array_buffer_prototype, store)?;
    let typed_array = TypedArrayObjectData::new(
        buffer_object,
        store,
        0,
        length,
        TypedArrayElementKind::Uint8,
    );
    allocate_typed_array_object(cx, realm, uint8_array_prototype, typed_array)
}

fn make_set_result<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    read: usize,
    written: usize,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().object_prototype())
        .ok_or_else(|| type_error(cx))?;
    let object = cx.allocate_ordinary_object_with_prototype(realm, Some(prototype))?;
    let read_key = property_key_from_text(cx, "read");
    let written_key = property_key_from_text(cx, "written");
    let read_value = u64_to_value(read);
    let written_value = u64_to_value(written);
    create_data_property_or_throw(cx, object, read_key, read_value)?;
    create_data_property_or_throw(cx, object, written_key, written_value)?;
    Ok(Value::from_object_ref(object))
}

fn u64_to_value(n: usize) -> Value {
    let as_u64 = u64::try_from(n).unwrap_or(u64::MAX);
    i32::try_from(as_u64).map_or_else(
        |_| {
            #[allow(
                clippy::cast_precision_loss,
                reason = "Uint8Array base64/hex result counts are exposed as ECMAScript Number values"
            )]
            let number = as_u64 as f64;
            Value::from_f64(number)
        },
        Value::from_smi,
    )
}

// ---------- Static methods ----------

fn uint8_array_from_base64_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let raw_string = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    require_string_value(cx, raw_string)?;
    let options = options_object_or_throw(cx, options_argument(&invocation, 1))?;
    let alphabet = parse_alphabet_option(cx, options)?;
    let last_chunk_handling = parse_last_chunk_handling_option(cx, options)?;
    let text = cx.value_to_string_text(raw_string)?;
    let result = decode_base64(&text, alphabet, last_chunk_handling, None);
    if result.error {
        return Err(syntax_error(cx));
    }
    let realm = cx.builtin_realm();
    let object = allocate_uint8_array_from_bytes(cx, realm, &result.bytes)?;
    Ok(Value::from_object_ref(object))
}

fn uint8_array_from_hex_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let raw_string = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    require_string_value(cx, raw_string)?;
    let text = cx.value_to_string_text(raw_string)?;
    let result = decode_hex(&text, None);
    if result.error {
        return Err(syntax_error(cx));
    }
    let realm = cx.builtin_realm();
    let object = allocate_uint8_array_from_bytes(cx, realm, &result.bytes)?;
    Ok(Value::from_object_ref(object))
}

// ---------- Prototype methods ----------

fn uint8_array_set_from_base64_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    // Spec order: ValidateUint8Array → require string → options → detached.
    let _ = require_uint8_array_record(cx, invocation.this_value())?;
    let raw_string = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    require_string_value(cx, raw_string)?;
    let options = options_object_or_throw(cx, options_argument(&invocation, 1))?;
    let alphabet = parse_alphabet_option(cx, options)?;
    let last_chunk_handling = parse_last_chunk_handling_option(cx, options)?;
    let record = require_uint8_array_record(cx, invocation.this_value())?;
    if cx
        .agent()
        .backing_store_is_detached(record.backing_store())
        .ok_or_else(|| type_error(cx))?
    {
        return Err(type_error(cx));
    }
    let max_length = record.length();
    let text = cx.value_to_string_text(raw_string)?;
    let result = decode_base64(&text, alphabet, last_chunk_handling, Some(max_length));
    let written = result.bytes.len();
    write_typed_array_bytes(cx, record, &result.bytes)?;
    if result.error {
        return Err(syntax_error(cx));
    }
    make_set_result(cx, result.read, written)
}

fn uint8_array_set_from_hex_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    // Spec order: ValidateUint8Array → require string → detached.
    let _ = require_uint8_array_record(cx, invocation.this_value())?;
    let raw_string = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    require_string_value(cx, raw_string)?;
    let record = require_uint8_array_record(cx, invocation.this_value())?;
    if cx
        .agent()
        .backing_store_is_detached(record.backing_store())
        .ok_or_else(|| type_error(cx))?
    {
        return Err(type_error(cx));
    }
    let max_length = record.length();
    let text = cx.value_to_string_text(raw_string)?;
    let result = decode_hex(&text, Some(max_length));
    let written = result.bytes.len();
    write_typed_array_bytes(cx, record, &result.bytes)?;
    if result.error {
        return Err(syntax_error(cx));
    }
    make_set_result(cx, result.read, written)
}

fn uint8_array_to_base64_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    // Per spec: validate receiver FIRST, then access options. This matches
    // receiver-not-uint8array.js (must throw TypeError without accessing
    // options.alphabet) and detached-buffer.js (alphabet getter runs once,
    // then detached check throws).
    let _ = require_uint8_array_record(cx, invocation.this_value())?;
    let options = options_object_or_throw(cx, options_argument(&invocation, 0))?;
    let alphabet = parse_alphabet_option(cx, options)?;
    let omit_padding = parse_omit_padding_option(cx, options)?;
    let record = require_uint8_array_record(cx, invocation.this_value())?;
    if cx
        .agent()
        .backing_store_is_detached(record.backing_store())
        .ok_or_else(|| type_error(cx))?
    {
        return Err(type_error(cx));
    }
    let bytes = read_typed_array_bytes(cx, record).ok_or_else(|| type_error(cx))?;
    let encoded = encode_base64(&bytes, alphabet, omit_padding);
    Ok(string_value(cx, &encoded))
}

fn uint8_array_to_hex_builtin_dispatch<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let record = require_uint8_array_record(cx, invocation.this_value())?;
    if cx
        .agent()
        .backing_store_is_detached(record.backing_store())
        .ok_or_else(|| type_error(cx))?
    {
        return Err(type_error(cx));
    }
    let bytes = read_typed_array_bytes(cx, record).ok_or_else(|| type_error(cx))?;
    let encoded = encode_hex(&bytes);
    Ok(string_value(cx, &encoded))
}
