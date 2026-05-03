use super::*;
use lyng_js_objects::{ObjectKind, PrimitiveWrapperKind};
use lyng_js_types::{ObjectRef, StringRef};
use std::fmt::Write as _;

pub(super) fn dispatch_json_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::json_parse_builtin() {
        return json_parse_builtin(context, invocation).map(Some);
    }
    if entry == super::json_stringify_builtin() {
        return json_stringify_builtin(context, invocation).map(Some);
    }
    if entry == super::json_raw_json_builtin() {
        return json_raw_json_builtin(context, invocation).map(Some);
    }
    if entry == super::json_is_raw_json_builtin() {
        return json_is_raw_json_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn json_raw_json_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let raw_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let raw_string = to_string_string_ref(cx, raw_value)?;
    let raw_text = string_ref_text(cx, raw_string)?;
    if !json_raw_text_is_valid(&raw_text) {
        return Err(syntax_error(cx));
    }
    let object = allocate_json_raw_object(cx, cx.builtin_realm(), raw_string)?;
    let frozen = cx.set_integrity_level(object, true)?;
    if !frozen {
        return Err(type_error(cx));
    }
    Ok(Value::from_object_ref(object))
}

fn json_is_raw_json_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let is_raw = value
        .as_object_ref()
        .is_some_and(|object| cx.agent().objects().is_json_raw_object(object));
    Ok(Value::from_bool(is_raw))
}

fn json_raw_text_is_valid(text: &str) -> bool {
    let bytes = text.as_bytes();
    let Some((&first, &last)) = bytes.first().zip(bytes.last()) else {
        return false;
    };
    if matches!(first, b'\t' | b'\n' | b'\r' | b' ') || matches!(last, b'\t' | b'\n' | b'\r' | b' ')
    {
        return false;
    }
    match first {
        b'n' => text == "null",
        b't' => text == "true",
        b'f' => text == "false",
        b'"' => json_raw_string_literal_is_valid(bytes),
        b'-' | b'0'..=b'9' => json_raw_number_literal_is_valid(bytes),
        _ => false,
    }
}

fn json_raw_string_literal_is_valid(bytes: &[u8]) -> bool {
    if bytes.len() < 2 || bytes.first() != Some(&b'"') || bytes.last() != Some(&b'"') {
        return false;
    }
    let mut index = 1;
    while index < bytes.len() - 1 {
        let byte = bytes[index];
        if byte == b'"' || byte < 0x20 {
            return false;
        }
        if byte == b'\\' {
            index += 1;
            let Some(&escape) = bytes.get(index) else {
                return false;
            };
            match escape {
                b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => {
                    index += 1;
                }
                b'u' => {
                    let Some(hex_digits) = bytes.get(index + 1..index + 5) else {
                        return false;
                    };
                    if !hex_digits.iter().all(u8::is_ascii_hexdigit) {
                        return false;
                    }
                    index += 5;
                }
                _ => return false,
            }
            continue;
        }
        if byte < 0x80 {
            index += 1;
            continue;
        }
        let Ok(text) = std::str::from_utf8(&bytes[index..]) else {
            return false;
        };
        let Some(ch) = text.chars().next() else {
            return false;
        };
        if u32::from(ch) <= 0x1F {
            return false;
        }
        index += ch.len_utf8();
    }
    true
}

fn json_raw_number_literal_is_valid(bytes: &[u8]) -> bool {
    let mut index = 0;
    if bytes.get(index) == Some(&b'-') {
        index += 1;
    }
    match bytes.get(index) {
        Some(b'0') => {
            index += 1;
        }
        Some(b'1'..=b'9') => {
            index += 1;
            while matches!(bytes.get(index), Some(b'0'..=b'9')) {
                index += 1;
            }
        }
        _ => return false,
    }
    if bytes.get(index) == Some(&b'.') {
        index += 1;
        if !matches!(bytes.get(index), Some(b'0'..=b'9')) {
            return false;
        }
        while matches!(bytes.get(index), Some(b'0'..=b'9')) {
            index += 1;
        }
    }
    if matches!(bytes.get(index), Some(b'e' | b'E')) {
        index += 1;
        if matches!(bytes.get(index), Some(b'+' | b'-')) {
            index += 1;
        }
        if !matches!(bytes.get(index), Some(b'0'..=b'9')) {
            return false;
        }
        while matches!(bytes.get(index), Some(b'0'..=b'9')) {
            index += 1;
        }
    }
    index == bytes.len()
}

const JSON_CHAR_QUOTE: u16 = b'"' as u16;
const JSON_CHAR_PLUS: u16 = b'+' as u16;
const JSON_CHAR_COMMA: u16 = b',' as u16;
const JSON_CHAR_MINUS: u16 = b'-' as u16;
const JSON_CHAR_DOT: u16 = b'.' as u16;
const JSON_CHAR_SLASH: u16 = b'/' as u16;
const JSON_CHAR_COLON: u16 = b':' as u16;
const JSON_CHAR_LEFT_BRACKET: u16 = b'[' as u16;
const JSON_CHAR_BACKSLASH: u16 = b'\\' as u16;
const JSON_CHAR_RIGHT_BRACKET: u16 = b']' as u16;
const JSON_CHAR_LEFT_BRACE: u16 = b'{' as u16;
const JSON_CHAR_RIGHT_BRACE: u16 = b'}' as u16;
const JSON_CHAR_ZERO: u16 = b'0' as u16;
const JSON_CHAR_NINE: u16 = b'9' as u16;
const JSON_CHAR_UPPER_A: u16 = b'A' as u16;
const JSON_CHAR_UPPER_E: u16 = b'E' as u16;
const JSON_CHAR_UPPER_F: u16 = b'F' as u16;
const JSON_CHAR_B: u16 = b'b' as u16;
const JSON_CHAR_E: u16 = b'e' as u16;
const JSON_CHAR_F: u16 = b'f' as u16;
const JSON_CHAR_N: u16 = b'n' as u16;
const JSON_CHAR_R: u16 = b'r' as u16;
const JSON_CHAR_T: u16 = b't' as u16;
const JSON_CHAR_U: u16 = b'u' as u16;
const JSON_CHAR_LOWER_A: u16 = b'a' as u16;
const JSON_CHAR_LOWER_F: u16 = b'f' as u16;
const JSON_CHAR_ONE: u16 = b'1' as u16;

enum JsonParseNode {
    Primitive {
        value: Value,
        source_units: Vec<u16>,
    },
    Array {
        object: ObjectRef,
        elements: Vec<JsonParseNode>,
    },
    Object {
        object: ObjectRef,
        entries: Vec<(PropertyKey, JsonParseNode)>,
    },
}

impl JsonParseNode {
    fn value(&self) -> Value {
        match self {
            Self::Primitive { value, .. } => *value,
            Self::Array { object, .. } | Self::Object { object, .. } => {
                Value::from_object_ref(*object)
            }
        }
    }

    fn source_units_for_value(&self, value: Value) -> Option<&[u16]> {
        match self {
            Self::Primitive {
                value: original,
                source_units,
            } if *original == value => Some(source_units),
            _ => None,
        }
    }

    fn array_elements_for_object(&self, object: ObjectRef) -> Option<&[JsonParseNode]> {
        match self {
            Self::Array {
                object: original,
                elements,
            } if *original == object => Some(elements),
            _ => None,
        }
    }

    fn object_entry_for_key(&self, object: ObjectRef, key: PropertyKey) -> Option<&JsonParseNode> {
        match self {
            Self::Object {
                object: original,
                entries,
            } if *original == object => entries
                .iter()
                .find(|(candidate, _)| *candidate == key)
                .map(|(_, node)| node),
            _ => None,
        }
    }
}

struct JsonParser<'a> {
    units: &'a [u16],
    index: usize,
}

impl<'a> JsonParser<'a> {
    fn new(units: &'a [u16]) -> Self {
        Self { units, index: 0 }
    }

    fn parse_text<Cx: PublicBuiltinDispatchContext>(
        &mut self,
        cx: &mut Cx,
    ) -> Result<JsonParseNode, Cx::Error> {
        self.skip_whitespace();
        let value = self.parse_value(cx)?;
        self.skip_whitespace();
        if self.index != self.units.len() {
            return Err(syntax_error(cx));
        }
        Ok(value)
    }

    fn parse_value<Cx: PublicBuiltinDispatchContext>(
        &mut self,
        cx: &mut Cx,
    ) -> Result<JsonParseNode, Cx::Error> {
        self.skip_whitespace();
        match self.peek() {
            Some(JSON_CHAR_N) => self.parse_keyword(cx, b"null", Value::null()),
            Some(JSON_CHAR_T) => self.parse_keyword(cx, b"true", Value::from_bool(true)),
            Some(JSON_CHAR_F) => self.parse_keyword(cx, b"false", Value::from_bool(false)),
            Some(JSON_CHAR_QUOTE) => {
                let start = self.index;
                let units = self.parse_string_units(cx)?;
                Ok(JsonParseNode::Primitive {
                    value: string_from_code_units(cx, &units),
                    source_units: self.units[start..self.index].to_vec(),
                })
            }
            Some(JSON_CHAR_LEFT_BRACKET) => self.parse_array(cx),
            Some(JSON_CHAR_LEFT_BRACE) => self.parse_object(cx),
            Some(JSON_CHAR_MINUS | JSON_CHAR_ZERO..=JSON_CHAR_NINE) => self.parse_number(cx),
            _ => Err(syntax_error(cx)),
        }
    }

    fn parse_keyword<Cx: PublicBuiltinDispatchContext>(
        &mut self,
        cx: &mut Cx,
        keyword: &[u8],
        value: Value,
    ) -> Result<JsonParseNode, Cx::Error> {
        let start = self.index;
        for expected in keyword {
            if self.advance() != Some(u16::from(*expected)) {
                return Err(syntax_error(cx));
            }
        }
        Ok(JsonParseNode::Primitive {
            value,
            source_units: self.units[start..self.index].to_vec(),
        })
    }

    fn parse_array<Cx: PublicBuiltinDispatchContext>(
        &mut self,
        cx: &mut Cx,
    ) -> Result<JsonParseNode, Cx::Error> {
        self.expect(cx, JSON_CHAR_LEFT_BRACKET)?;
        self.skip_whitespace();
        let mut elements = Vec::new();
        if self.peek() == Some(JSON_CHAR_RIGHT_BRACKET) {
            self.index += 1;
        } else {
            loop {
                elements.push(self.parse_value(cx)?);
                self.skip_whitespace();
                match self.advance() {
                    Some(JSON_CHAR_COMMA) => self.skip_whitespace(),
                    Some(JSON_CHAR_RIGHT_BRACKET) => break,
                    _ => return Err(syntax_error(cx)),
                }
            }
        }

        let array = create_array_result(cx, elements.len())?;
        for (index, element) in elements.iter().enumerate() {
            let key = array_like_index_property_key(cx, u64::try_from(index).unwrap_or(u64::MAX));
            create_data_property_or_throw(cx, array, key, element.value())?;
        }
        Ok(JsonParseNode::Array {
            object: array,
            elements,
        })
    }

    fn parse_object<Cx: PublicBuiltinDispatchContext>(
        &mut self,
        cx: &mut Cx,
    ) -> Result<JsonParseNode, Cx::Error> {
        self.expect(cx, JSON_CHAR_LEFT_BRACE)?;
        self.skip_whitespace();
        let mut entries = Vec::new();
        let prototype = json_object_prototype(cx)?;
        let object =
            cx.allocate_ordinary_object_with_prototype(cx.builtin_realm(), Some(prototype))?;
        if self.peek() == Some(JSON_CHAR_RIGHT_BRACE) {
            self.index += 1;
        } else {
            loop {
                let key_units = self.parse_string_units(cx)?;
                self.skip_whitespace();
                self.expect(cx, JSON_CHAR_COLON)?;
                let key_value = string_from_code_units(cx, &key_units);
                let key = cx.to_property_key(key_value)?;
                let value = self.parse_value(cx)?;
                create_data_property_or_throw(cx, object, key, value.value())?;
                if let Some(existing) = entries.iter_mut().find(|(candidate, _)| *candidate == key)
                {
                    existing.1 = value;
                } else {
                    entries.push((key, value));
                }
                self.skip_whitespace();
                match self.advance() {
                    Some(JSON_CHAR_COMMA) => self.skip_whitespace(),
                    Some(JSON_CHAR_RIGHT_BRACE) => break,
                    _ => return Err(syntax_error(cx)),
                }
            }
        }
        Ok(JsonParseNode::Object { object, entries })
    }

    fn parse_number<Cx: PublicBuiltinDispatchContext>(
        &mut self,
        cx: &mut Cx,
    ) -> Result<JsonParseNode, Cx::Error> {
        let start = self.index;
        if self.peek() == Some(JSON_CHAR_MINUS) {
            self.index += 1;
        }
        match self.peek() {
            Some(JSON_CHAR_ZERO) => {
                self.index += 1;
            }
            Some(JSON_CHAR_ONE..=JSON_CHAR_NINE) => {
                self.index += 1;
                while matches!(self.peek(), Some(JSON_CHAR_ZERO..=JSON_CHAR_NINE)) {
                    self.index += 1;
                }
            }
            _ => return Err(syntax_error(cx)),
        }
        if self.peek() == Some(JSON_CHAR_DOT) {
            self.index += 1;
            if !matches!(self.peek(), Some(JSON_CHAR_ZERO..=JSON_CHAR_NINE)) {
                return Err(syntax_error(cx));
            }
            while matches!(self.peek(), Some(JSON_CHAR_ZERO..=JSON_CHAR_NINE)) {
                self.index += 1;
            }
        }
        if matches!(self.peek(), Some(JSON_CHAR_E | JSON_CHAR_UPPER_E)) {
            self.index += 1;
            if matches!(self.peek(), Some(JSON_CHAR_PLUS | JSON_CHAR_MINUS)) {
                self.index += 1;
            }
            if !matches!(self.peek(), Some(JSON_CHAR_ZERO..=JSON_CHAR_NINE)) {
                return Err(syntax_error(cx));
            }
            while matches!(self.peek(), Some(JSON_CHAR_ZERO..=JSON_CHAR_NINE)) {
                self.index += 1;
            }
        }

        let text: String = self.units[start..self.index]
            .iter()
            .map(|unit| char::from_u32(u32::from(*unit)).expect("JSON numbers should stay ASCII"))
            .collect();
        let number = text.parse::<f64>().map_err(|_| syntax_error(cx))?;
        Ok(JsonParseNode::Primitive {
            value: number_value(number),
            source_units: self.units[start..self.index].to_vec(),
        })
    }

    fn parse_string_units<Cx: PublicBuiltinDispatchContext>(
        &mut self,
        cx: &mut Cx,
    ) -> Result<Vec<u16>, Cx::Error> {
        self.expect(cx, JSON_CHAR_QUOTE)?;
        let mut units = Vec::new();
        loop {
            let Some(unit) = self.advance() else {
                return Err(syntax_error(cx));
            };
            match unit {
                JSON_CHAR_QUOTE => return Ok(units),
                JSON_CHAR_BACKSLASH => {
                    let escaped = self.advance().ok_or_else(|| syntax_error(cx))?;
                    match escaped {
                        JSON_CHAR_QUOTE => units.push(JSON_CHAR_QUOTE),
                        JSON_CHAR_BACKSLASH => units.push(JSON_CHAR_BACKSLASH),
                        JSON_CHAR_SLASH => units.push(JSON_CHAR_SLASH),
                        JSON_CHAR_B => units.push(0x0008),
                        JSON_CHAR_F => units.push(0x000C),
                        JSON_CHAR_N => units.push(0x000A),
                        JSON_CHAR_R => units.push(0x000D),
                        JSON_CHAR_T => units.push(0x0009),
                        JSON_CHAR_U => units.push(self.parse_hex_escape(cx)?),
                        _ => return Err(syntax_error(cx)),
                    }
                }
                unit if unit < 0x0020 => return Err(syntax_error(cx)),
                unit => units.push(unit),
            }
        }
    }

    fn parse_hex_escape<Cx: PublicBuiltinDispatchContext>(
        &mut self,
        cx: &mut Cx,
    ) -> Result<u16, Cx::Error> {
        let mut value = 0u16;
        for _ in 0..4 {
            let digit = self.advance().ok_or_else(|| syntax_error(cx))?;
            let nibble = match digit {
                JSON_CHAR_ZERO..=JSON_CHAR_NINE => digit - JSON_CHAR_ZERO,
                JSON_CHAR_LOWER_A..=JSON_CHAR_LOWER_F => digit - JSON_CHAR_LOWER_A + 10,
                JSON_CHAR_UPPER_A..=JSON_CHAR_UPPER_F => digit - JSON_CHAR_UPPER_A + 10,
                _ => return Err(syntax_error(cx)),
            };
            value = (value << 4) | nibble;
        }
        Ok(value)
    }

    fn expect<Cx: PublicBuiltinDispatchContext>(
        &mut self,
        cx: &mut Cx,
        expected: u16,
    ) -> Result<(), Cx::Error> {
        if self.advance() == Some(expected) {
            Ok(())
        } else {
            Err(syntax_error(cx))
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(0x0009 | 0x000A | 0x000D | 0x0020)) {
            self.index += 1;
        }
    }

    fn peek(&self) -> Option<u16> {
        self.units.get(self.index).copied()
    }

    fn advance(&mut self) -> Option<u16> {
        let unit = self.peek()?;
        self.index += 1;
        Some(unit)
    }
}

fn json_object_prototype<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
) -> Result<ObjectRef, Cx::Error> {
    let realm = cx.builtin_realm();
    cx.agent()
        .realm(realm)
        .and_then(|realm| realm.intrinsics().object_prototype())
        .ok_or_else(|| type_error(cx))
}

fn json_enumerable_string_keys<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object: ObjectRef,
) -> Result<Vec<PropertyKey>, Cx::Error> {
    let keys = { proxy_own_property_keys(cx, object) };
    let keys = keys?;
    let mut enumerable = Vec::with_capacity(keys.len());
    for key in keys {
        if key.is_symbol() {
            continue;
        }
        let descriptor = { proxy_get_own_property(cx, object, key) };
        let Some(descriptor) = descriptor? else {
            continue;
        };
        if descriptor.enumerable() == Some(true) {
            enumerable.push(key);
        }
    }
    Ok(enumerable)
}

fn json_reviver_context_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    metadata: Option<&JsonParseNode>,
    value: Value,
) -> Result<Value, Cx::Error> {
    let prototype = json_object_prototype(cx)?;
    let context =
        cx.allocate_ordinary_object_with_prototype(cx.builtin_realm(), Some(prototype))?;
    if let Some(source_units) = metadata.and_then(|node| node.source_units_for_value(value)) {
        let source_key = property_key_from_text(cx, "source");
        let source_value = string_from_code_units(cx, source_units);
        create_data_property_or_throw(cx, context, source_key, source_value)?;
    }
    Ok(Value::from_object_ref(context))
}

fn json_internalize_property<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    holder: ObjectRef,
    name: PropertyKey,
    reviver: ObjectRef,
    metadata: Option<&JsonParseNode>,
) -> Result<Value, Cx::Error> {
    let value = get_property_from_object(cx, holder, name)?;
    if let Some(object) = value.as_object_ref() {
        if is_array_for_species(cx, object)? {
            let array_elements = metadata.and_then(|node| node.array_elements_for_object(object));
            let length = array_like_length(cx, object)?;
            for index in 0..length {
                let key = array_like_index_property_key(cx, u64::from(index));
                let child_metadata =
                    array_elements.and_then(|elements| elements.get(index as usize));
                let replacement =
                    json_internalize_property(cx, object, key, reviver, child_metadata)?;
                if replacement.is_undefined() {
                    let _ = try_delete_property_from_object(cx, object, key)?;
                } else {
                    let _ = try_create_data_property(cx, object, key, replacement)?;
                }
            }
        } else {
            let keys = json_enumerable_string_keys(cx, object)?;
            for key in keys {
                let child_metadata =
                    metadata.and_then(|node| node.object_entry_for_key(object, key));
                let replacement =
                    json_internalize_property(cx, object, key, reviver, child_metadata)?;
                if replacement.is_undefined() {
                    let _ = try_delete_property_from_object(cx, object, key)?;
                } else {
                    let _ = try_create_data_property(cx, object, key, replacement)?;
                }
            }
        }
    }

    let name_value = property_key_string_value(cx, name);
    let context = json_reviver_context_object(cx, metadata, value)?;
    cx.call_to_completion(
        reviver,
        Value::from_object_ref(holder),
        &[name_value, value, context],
    )
}

fn json_parse_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let text = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let text = to_string_string_ref(cx, text)?;
    let text_units = string_ref_code_units(cx, text)?;
    let mut parser = JsonParser::new(&text_units);
    let unfiltered = parser.parse_text(cx)?;

    let Some(reviver) = invocation
        .arguments()
        .get(1)
        .copied()
        .and_then(|value| callable_object_from_value(cx, value))
    else {
        return Ok(unfiltered.value());
    };

    let object_prototype = json_object_prototype(cx)?;
    let root =
        cx.allocate_ordinary_object_with_prototype(cx.builtin_realm(), Some(object_prototype))?;
    let root_name = property_key_from_text(cx, "");
    create_data_property_or_throw(cx, root, root_name, unfiltered.value())?;
    json_internalize_property(cx, root, root_name, reviver, Some(&unfiltered))
}

fn json_stringify_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let replacer = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let replacer_function = callable_object_from_value(cx, replacer);
    let property_list = if replacer_function.is_some() {
        None
    } else {
        json_property_list_from_replacer(cx, replacer)?
    };
    let gap = json_gap_from_value(
        cx,
        invocation
            .arguments()
            .get(2)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let object_prototype = json_object_prototype(cx)?;
    let wrapper =
        cx.allocate_ordinary_object_with_prototype(cx.builtin_realm(), Some(object_prototype))?;
    let root_name = property_key_from_text(cx, "");
    create_data_property_or_throw(cx, wrapper, root_name, value)?;
    let mut state = JsonStringifyState {
        stack: Vec::new(),
        indent: String::new(),
        gap,
        replacer_function,
        property_list,
    };
    let Some(text) = json_serialize_property(cx, wrapper, root_name, &mut state)? else {
        return Ok(Value::undefined());
    };
    Ok(string_value(cx, &text))
}

struct JsonStringifyState {
    stack: Vec<ObjectRef>,
    indent: String,
    gap: String,
    replacer_function: Option<ObjectRef>,
    property_list: Option<Vec<PropertyKey>>,
}

fn json_property_list_from_replacer<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<Option<Vec<PropertyKey>>, Cx::Error> {
    let Some(object) = value.as_object_ref() else {
        return Ok(None);
    };
    if !is_array_for_species(cx, object)? {
        return Ok(None);
    }

    let length = array_like_length(cx, object)?;
    let mut property_list = Vec::new();
    for index in 0..length {
        let item = get_property_from_object(cx, object, PropertyKey::Index(index))?;
        let Some(key) = json_property_list_item(cx, item)? else {
            continue;
        };
        if !property_list.contains(&key) {
            property_list.push(key);
        }
    }
    Ok(Some(property_list))
}

fn json_property_list_item<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<Option<PropertyKey>, Cx::Error> {
    let include = value.as_string_ref().is_some()
        || value.as_smi().is_some()
        || value.as_f64().is_some()
        || value.as_object_ref().is_some_and(|object| {
            matches!(
                cx.agent().objects().primitive_wrapper_kind(object),
                Some(PrimitiveWrapperKind::String | PrimitiveWrapperKind::Number)
            )
        });
    if !include {
        return Ok(None);
    }
    let item = to_string_string_ref(cx, value)?;
    let key = cx.to_property_key(Value::from_string_ref(item))?;
    Ok(Some(key))
}

fn json_gap_from_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    mut value: Value,
) -> Result<String, Cx::Error> {
    if let Some(object) = value.as_object_ref() {
        value = match cx.agent().objects().primitive_wrapper_kind(object) {
            Some(PrimitiveWrapperKind::Number) => to_number_value_for_builtin(cx, value)?,
            Some(PrimitiveWrapperKind::String) => {
                Value::from_string_ref(to_string_string_ref(cx, value)?)
            }
            _ => Value::undefined(),
        };
    }

    if value.as_smi().is_some() || value.as_f64().is_some() {
        let count = to_integer_or_infinity_for_builtin(cx, value)?;
        let count = if count <= 0.0 {
            0
        } else {
            usize::try_from(count.min(10.0) as u64).unwrap_or(10)
        };
        return Ok(" ".repeat(count));
    }

    let Some(string) = value.as_string_ref() else {
        return Ok(String::new());
    };
    let mut units = string_ref_code_units(cx, string)?;
    units.truncate(10);
    Ok(String::from_utf16_lossy(&units))
}

fn json_normalize_wrapper_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
    object: ObjectRef,
) -> Result<Option<Value>, Cx::Error> {
    let wrapper_value = {
        let agent = cx.agent();
        agent
            .objects()
            .primitive_wrapper_value(agent.heap().view(), object)
    };
    let normalized = match cx.agent().objects().primitive_wrapper_kind(object) {
        Some(PrimitiveWrapperKind::Number) => Some(to_number_value_for_builtin(cx, value)?),
        Some(PrimitiveWrapperKind::String) => {
            Some(Value::from_string_ref(to_string_string_ref(cx, value)?))
        }
        Some(PrimitiveWrapperKind::Boolean | PrimitiveWrapperKind::BigInt) => wrapper_value,
        _ => None,
    };
    Ok(normalized)
}

fn json_raw_json_string<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object: ObjectRef,
) -> Result<Option<String>, Cx::Error> {
    let raw_json_string = {
        let agent = cx.agent();
        if !agent.objects().is_json_raw_object(object) {
            None
        } else {
            agent
                .objects()
                .ordinary_payload_value(agent.heap().view(), object)
                .and_then(Value::as_string_ref)
        }
    };
    raw_json_string
        .map(|string| string_ref_text(cx, string))
        .transpose()
}

fn json_serialize_property<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    holder: ObjectRef,
    key: PropertyKey,
    state: &mut JsonStringifyState,
) -> Result<Option<String>, Cx::Error> {
    let mut value = get_property_from_object(cx, holder, key)?;
    if value.as_object_ref().is_some() || value.is_bigint() {
        let to_json_key = property_key_from_text(cx, "toJSON");
        let to_json = cx.get_property_value(value, to_json_key)?;
        if let Some(to_json) = callable_object_from_value(cx, to_json) {
            let key_value = property_key_string_value(cx, key);
            value = cx.call_to_completion(to_json, value, &[key_value])?;
        }
    }
    if let Some(replacer) = state.replacer_function {
        let key_value = property_key_string_value(cx, key);
        value = cx.call_to_completion(
            replacer,
            Value::from_object_ref(holder),
            &[key_value, value],
        )?;
    }
    if let Some(object) = value.as_object_ref() {
        if let Some(normalized) = json_normalize_wrapper_value(cx, value, object)? {
            value = normalized;
        }
    }

    if value.is_undefined() || value.as_symbol_ref().is_some() {
        return Ok(None);
    }
    if value.is_null() {
        return Ok(Some("null".to_owned()));
    }
    if let Some(boolean) = value.as_bool() {
        return Ok(Some(if boolean { "true" } else { "false" }.to_owned()));
    }
    if value.as_smi().is_some() || value.as_f64().is_some() {
        let number = to_number_for_builtin(cx, value)?;
        if !number.is_finite() {
            return Ok(Some("null".to_owned()));
        }
        return Ok(Some(cx.value_to_string_text(value)?));
    }
    if value.is_bigint() {
        return Err(type_error(cx));
    }
    if let Some(string) = value.as_string_ref() {
        return Ok(Some(json_quote_string_ref(cx, string)?));
    }

    let Some(object) = value.as_object_ref() else {
        return Ok(None);
    };
    if let Some(raw_json) = json_raw_json_string(cx, object)? {
        return Ok(Some(raw_json));
    }

    let is_function = {
        let agent = cx.agent();
        agent
            .objects()
            .object_header(agent.heap().view(), object)
            .is_some_and(|header| header.kind() == ObjectKind::Function)
    };
    if is_function {
        return Ok(None);
    }
    if state.stack.contains(&object) {
        return Err(type_error(cx));
    }

    state.stack.push(object);
    let result = if is_array_for_species(cx, object)? {
        json_serialize_array(cx, object, state)
    } else {
        json_serialize_object(cx, object, state)
    };
    let _ = state.stack.pop();
    result.map(Some)
}

fn json_serialize_array<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object: ObjectRef,
    state: &mut JsonStringifyState,
) -> Result<String, Cx::Error> {
    let length = array_like_length(cx, object)?;
    let stepback = state.indent.clone();
    state.indent.push_str(&state.gap);
    let indent = state.indent.clone();
    let gap_empty = state.gap.is_empty();
    let mut parts = Vec::with_capacity(usize::try_from(length).unwrap_or(usize::MAX));
    for index in 0..length {
        let key = array_like_index_property_key(cx, u64::from(index));
        parts.push(
            json_serialize_property(cx, object, key, state)?.unwrap_or_else(|| "null".to_owned()),
        );
    }
    state.indent = stepback.clone();
    if parts.is_empty() {
        return Ok("[]".to_owned());
    }
    if gap_empty {
        return Ok(format!("[{}]", parts.join(",")));
    }
    let separator = format!(",\n{indent}");
    Ok(format!(
        "[\n{}{}\n{}]",
        indent,
        parts.join(&separator),
        stepback
    ))
}

fn json_serialize_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object: ObjectRef,
    state: &mut JsonStringifyState,
) -> Result<String, Cx::Error> {
    let keys = if let Some(property_list) = state.property_list.as_ref() {
        property_list.clone()
    } else {
        json_enumerable_string_keys(cx, object)?
    };
    let stepback = state.indent.clone();
    state.indent.push_str(&state.gap);
    let indent = state.indent.clone();
    let gap_empty = state.gap.is_empty();
    let mut parts = Vec::new();
    for key in keys {
        let Some(serialized) = json_serialize_property(cx, object, key, state)? else {
            continue;
        };
        let separator = if gap_empty { ":" } else { ": " };
        parts.push(format!(
            "{}{}{}",
            json_quote_property_key(cx, key)?,
            separator,
            serialized
        ));
    }
    state.indent = stepback.clone();
    if parts.is_empty() {
        return Ok("{}".to_owned());
    }
    if gap_empty {
        return Ok(format!("{{{}}}", parts.join(",")));
    }
    let separator = format!(",\n{indent}");
    Ok(format!(
        "{{\n{}{}\n{}}}",
        indent,
        parts.join(&separator),
        stepback
    ))
}

fn json_quote_property_key<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    key: PropertyKey,
) -> Result<String, Cx::Error> {
    let name = property_key_string_value(cx, key);
    let Some(string) = name.as_string_ref() else {
        return Err(type_error(cx));
    };
    json_quote_string_ref(cx, string)
}

fn json_quote_string_ref<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    string: StringRef,
) -> Result<String, Cx::Error> {
    let units = string_ref_code_units(cx, string)?;
    Ok(json_quote_string_units(&units))
}

fn json_quote_string_units(units: &[u16]) -> String {
    let mut quoted = String::with_capacity(units.len() + 2);
    quoted.push('"');
    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        match unit {
            0x0022 => quoted.push_str("\\\""),
            0x005C => quoted.push_str("\\\\"),
            0x0008 => quoted.push_str("\\b"),
            0x0009 => quoted.push_str("\\t"),
            0x000A => quoted.push_str("\\n"),
            0x000C => quoted.push_str("\\f"),
            0x000D => quoted.push_str("\\r"),
            0x0000..=0x001F | 0xDC00..=0xDFFF => {
                let _ = write!(quoted, "\\u{unit:04x}");
            }
            0xD800..=0xDBFF => {
                if let Some(next) = units.get(index + 1).copied() {
                    if (0xDC00..=0xDFFF).contains(&next) {
                        let scalar =
                            0x10000 + (u32::from(unit - 0xD800) << 10) + u32::from(next - 0xDC00);
                        quoted.push(
                            char::from_u32(scalar)
                                .expect("valid UTF-16 surrogate pair should decode"),
                        );
                        index += 1;
                    } else {
                        let _ = write!(quoted, "\\u{unit:04x}");
                    }
                } else {
                    let _ = write!(quoted, "\\u{unit:04x}");
                }
            }
            unit => quoted.push(
                char::from_u32(u32::from(unit))
                    .expect("basic multilingual plane unit should decode"),
            ),
        }
        index += 1;
    }
    quoted.push('"');
    quoted
}
