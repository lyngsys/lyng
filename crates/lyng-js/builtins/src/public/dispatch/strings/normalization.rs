use super::super::{
    range_error, string_ref_text, string_this_ref, string_value, to_string_string_ref,
    PublicBuiltinDispatchContext,
};
use crate::BuiltinInvocation;
use lyng_js_types::Value;

pub(super) fn string_locale_compare_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let source_ref = string_this_ref(cx, invocation.this_value())?;
    let source_text = string_ref_text(cx, source_ref)?;
    let that_ref = to_string_string_ref(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let that_text = string_ref_text(cx, that_ref)?;
    let source_key = normalize_text_for_form(&source_text, "NFD").ok_or_else(|| range_error(cx))?;
    let that_key = normalize_text_for_form(&that_text, "NFD").ok_or_else(|| range_error(cx))?;
    let result = match source_key.cmp(&that_key) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    };
    Ok(Value::from_smi(result))
}

const fn canonical_combining_class(code_point: u32) -> u8 {
    match code_point {
        0x093C => 7,
        0x031B => 216,
        0x0323 => 220,
        0x0327 => 202,
        0x0301 | 0x0302 | 0x0304 | 0x0306 | 0x0307 | 0x0308 | 0x030A => 230,
        _ => 0,
    }
}

fn decompose_hangul(code_point: u32, output: &mut Vec<u32>) -> bool {
    const S_BASE: u32 = 0xAC00;
    const L_BASE: u32 = 0x1100;
    const V_BASE: u32 = 0x1161;
    const T_BASE: u32 = 0x11A7;
    const L_COUNT: u32 = 19;
    const V_COUNT: u32 = 21;
    const T_COUNT: u32 = 28;
    const N_COUNT: u32 = V_COUNT * T_COUNT;
    const S_COUNT: u32 = L_COUNT * N_COUNT;

    if !(S_BASE..S_BASE + S_COUNT).contains(&code_point) {
        return false;
    }
    let s_index = code_point - S_BASE;
    output.push(L_BASE + s_index / N_COUNT);
    output.push(V_BASE + (s_index % N_COUNT) / T_COUNT);
    let trailing = s_index % T_COUNT;
    if trailing != 0 {
        output.push(T_BASE + trailing);
    }
    true
}

fn decompose_code_point(code_point: u32, compatibility: bool, output: &mut Vec<u32>) {
    if decompose_hangul(code_point, output) {
        return;
    }
    let decomposition: Option<&'static [u32]> = match code_point {
        0x00C5 => Some(&[0x0041, 0x030A]),
        0x00C7 => Some(&[0x0043, 0x0327]),
        0x00C9 => Some(&[0x0045, 0x0301]),
        0x00E1 => Some(&[0x0061, 0x0301]),
        0x00E4 => Some(&[0x0061, 0x0308]),
        0x00E9 => Some(&[0x0065, 0x0301]),
        0x00F4 => Some(&[0x006F, 0x0302]),
        0x00F6 => Some(&[0x006F, 0x0308]),
        0x0100 => Some(&[0x0041, 0x0304]),
        0x0103 => Some(&[0x0061, 0x0306]),
        0x01B0 => Some(&[0x0075, 0x031B]),
        0x0344 => Some(&[0x0308, 0x0301]),
        0x0958 => Some(&[0x0915, 0x093C]),
        0x1E0B => Some(&[0x0064, 0x0307]),
        0x1E0D => Some(&[0x0064, 0x0323]),
        0x1E63 => Some(&[0x0073, 0x0323]),
        0x1E69 => Some(&[0x0073, 0x0323, 0x0307]),
        0x1E9B => Some(&[0x017F, 0x0307]),
        0x1EA1 => Some(&[0x0061, 0x0323]),
        0x1EE5 => Some(&[0x0075, 0x0323]),
        0x1EF1 => Some(&[0x0075, 0x031B, 0x0323]),
        0x2126 => Some(&[0x03A9]),
        0x212B => Some(&[0x00C5]),
        0x2ADC => Some(&[0x2ADD, 0x0338]),
        0x017F if compatibility => Some(&[0x0073]),
        _ => None,
    };
    if let Some(decomposition) = decomposition {
        for point in decomposition {
            decompose_code_point(*point, compatibility, output);
        }
    } else {
        output.push(code_point);
    }
}

fn reorder_combining_marks(points: &mut [u32]) {
    let mut index = 1;
    while index < points.len() {
        let class = canonical_combining_class(points[index]);
        if class == 0 {
            index += 1;
            continue;
        }
        let mut scan = index;
        while scan > 0 {
            let previous = canonical_combining_class(points[scan - 1]);
            if previous == 0 || previous <= class {
                break;
            }
            points.swap(scan - 1, scan);
            scan -= 1;
        }
        index += 1;
    }
}

fn compose_hangul_pair(left: u32, right: u32) -> Option<u32> {
    const S_BASE: u32 = 0xAC00;
    const L_BASE: u32 = 0x1100;
    const V_BASE: u32 = 0x1161;
    const T_BASE: u32 = 0x11A7;
    const L_COUNT: u32 = 19;
    const V_COUNT: u32 = 21;
    const T_COUNT: u32 = 28;
    const N_COUNT: u32 = V_COUNT * T_COUNT;
    const S_COUNT: u32 = L_COUNT * N_COUNT;

    if (L_BASE..L_BASE + L_COUNT).contains(&left) && (V_BASE..V_BASE + V_COUNT).contains(&right) {
        return Some(S_BASE + (left - L_BASE) * N_COUNT + (right - V_BASE) * T_COUNT);
    }
    if (S_BASE..S_BASE + S_COUNT).contains(&left)
        && (left - S_BASE).is_multiple_of(T_COUNT)
        && (T_BASE + 1..T_BASE + T_COUNT).contains(&right)
    {
        return Some(left + (right - T_BASE));
    }
    None
}

fn compose_pair(left: u32, right: u32) -> Option<u32> {
    if let Some(hangul) = compose_hangul_pair(left, right) {
        return Some(hangul);
    }
    match (left, right) {
        (0x0041, 0x030A) => Some(0x00C5),
        (0x0041, 0x0304) => Some(0x0100),
        (0x0043, 0x0327) => Some(0x00C7),
        (0x0045, 0x0301) => Some(0x00C9),
        (0x0061, 0x0301) => Some(0x00E1),
        (0x0061, 0x0306) => Some(0x0103),
        (0x0061, 0x0308) => Some(0x00E4),
        (0x0061, 0x0323) => Some(0x1EA1),
        (0x0064, 0x0307) => Some(0x1E0B),
        (0x0064, 0x0323) => Some(0x1E0D),
        (0x0065, 0x0301) => Some(0x00E9),
        (0x006F, 0x0302) => Some(0x00F4),
        (0x006F, 0x0308) => Some(0x00F6),
        (0x0073, 0x0323) => Some(0x1E63),
        (0x0075, 0x031B) => Some(0x01B0),
        (0x0075, 0x0323) => Some(0x1EE5),
        (0x017F, 0x0307) => Some(0x1E9B),
        (0x01B0, 0x0323) => Some(0x1EF1),
        (0x1E63, 0x0307) => Some(0x1E69),
        _ => None,
    }
}

fn compose_normalized_code_points(points: &[u32]) -> Vec<u32> {
    let mut result: Vec<u32> = Vec::with_capacity(points.len());
    let mut starter_index: Option<usize> = None;
    let mut previous_class = 0;
    for point in points {
        let class = canonical_combining_class(*point);
        if let Some(starter) = starter_index
            && (previous_class == 0 || previous_class < class)
            && let Some(composed) = compose_pair(result[starter], *point)
        {
            result[starter] = composed;
            continue;
        }
        if class == 0 {
            starter_index = Some(result.len());
        }
        previous_class = class;
        result.push(*point);
    }
    result
}

fn code_points_to_string(points: &[u32]) -> String {
    let mut text = String::new();
    for point in points {
        if let Some(ch) = char::from_u32(*point) {
            text.push(ch);
        } else {
            text.push('\u{FFFD}');
        }
    }
    text
}

fn normalize_text_for_form(text: &str, form: &str) -> Option<String> {
    let (compatibility, compose) = match form {
        "NFC" => (false, true),
        "NFD" => (false, false),
        "NFKC" => (true, true),
        "NFKD" => (true, false),
        _ => return None,
    };
    let mut points = Vec::with_capacity(text.chars().count());
    for ch in text.chars() {
        decompose_code_point(ch as u32, compatibility, &mut points);
    }
    reorder_combining_marks(&mut points);
    let normalized = if compose {
        compose_normalized_code_points(&points)
    } else {
        points
    };
    Some(code_points_to_string(&normalized))
}

pub(super) fn string_normalize_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let string = string_this_ref(cx, invocation.this_value())?;
    let text = string_ref_text(cx, string)?;
    let form = if let Some(value) = invocation.arguments().first().copied() {
        if value.is_undefined() {
            "NFC".to_owned()
        } else {
            let form_ref = to_string_string_ref(cx, value)?;
            string_ref_text(cx, form_ref)?
        }
    } else {
        "NFC".to_owned()
    };
    let normalized = normalize_text_for_form(&text, &form).ok_or_else(|| range_error(cx))?;
    Ok(string_value(cx, &normalized))
}
