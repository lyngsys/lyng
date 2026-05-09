use super::support::*;

#[test]
fn evaluate_script_string_literal_strict_equality_with_identifier() {
    let unit = compile_test_unit(
        2391,
        r#"
            var __10_4_2_1_1_1 = "str1";
            'str1' === __10_4_2_1_1_1;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_script_string_from_char_code_uses_uint16_code_units() {
    let unit = compile_test_unit(
        2392,
        r#"
            String.fromCharCode(65, 0x20ac, -1) === "A\u20ac\uffff";
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_script_string_add_preserves_mixed_encodings() {
    let unit = compile_test_unit(
        2393,
        r#"
            let text = "A" + String.fromCodePoint(0x1F600) + "\u00ff";
            text.length === 4 &&
                text.charCodeAt(0) === 0x0041 &&
                text.charCodeAt(1) === 0xD83D &&
                text.charCodeAt(2) === 0xDE00 &&
                text.charCodeAt(3) === 0x00FF;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_script_string_concat_preserves_utf16_code_units() {
    let unit = compile_test_unit(
        2394,
        r#"
            let lone = String.fromCharCode(0xD800);
            let text = "A".concat(lone, String.fromCodePoint(0x1F600), "\u00ff");
            text.length === 5 &&
                text.charCodeAt(0) === 0x0041 &&
                text.charCodeAt(1) === 0xD800 &&
                text.charCodeAt(2) === 0xD83D &&
                text.charCodeAt(3) === 0xDE00 &&
                text.charCodeAt(4) === 0x00FF;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_script_array_join_preserves_utf16_code_units() {
    let unit = compile_test_unit(
        2395,
        r#"
            let high = String.fromCharCode(0xD800);
            let low = String.fromCharCode(0xDFFF);
            let text = [high, "B"].join(low);
            text.length === 3 &&
                text.charCodeAt(0) === 0xD800 &&
                text.charCodeAt(1) === 0xDFFF &&
                text.charCodeAt(2) === 0x0042;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_script_typed_array_join_preserves_utf16_separator() {
    let unit = compile_test_unit(
        2396,
        r"
            let separator = String.fromCharCode(0xD800);
            let text = new Uint8Array([1, 2]).join(separator);
            text.length === 3 &&
                text.charCodeAt(0) === 0x0031 &&
                text.charCodeAt(1) === 0xD800 &&
                text.charCodeAt(2) === 0x0032;
        ",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_script_string_html_methods_preserve_utf16_code_units() {
    let unit = compile_test_unit(
        2397,
        r#"
            let high = String.fromCharCode(0xD800);
            let low = String.fromCharCode(0xDFFF);
            let bold = high.bold();
            let color = high.fontcolor('"' + low);
            bold.length === 8 &&
                bold.charCodeAt(3) === 0xD800 &&
                color.indexOf("&quot;") > 0 &&
                color.charCodeAt(color.indexOf("&quot;") + 6) === 0xDFFF &&
                color.charCodeAt(color.indexOf(">") + 1) === 0xD800;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_script_regexp_legacy_input_preserves_utf16_code_units() {
    let unit = compile_test_unit(
        2398,
        r"
            RegExp.input = String.fromCharCode(0xD800);
            RegExp.input.length === 1 && RegExp.input.charCodeAt(0) === 0xD800;
        ",
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn evaluate_script_regexp_exec_reuses_string_code_unit_scratch() {
    let unit = compile_test_unit(
        2399,
        r#"
            let source = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
            let re = /a/g;
            let count = 0;
            let current;
            while ((current = re.exec(source)) !== null) {
                count = count + current[0].length;
            }
            count;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    assert_eq!(vm.string_code_units_scratch_capacity(), 0);
    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_smi(64));
    assert!(vm.string_code_units_scratch_capacity() >= 64);
}

#[test]
fn evaluate_script_string_index_reads_do_not_allocate_primitive_wrappers() {
    let warmup = compile_test_unit(23929, "0;");
    let unit = compile_test_unit(
        23930,
        r#"
            var total = 0;
            for (var i = 0; i < 64; i++) {
                total += "0123456789ABCDEF"[i & 15].length;
            }
            total;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let _ = vm.evaluate_script(agent, realm, &warmup).unwrap();
    let before_objects = agent.heap().view().object_stats().occupied_slots;
    let before_strings = agent.heap().view().string_stats().occupied_slots;
    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let after_objects = agent.heap().view().object_stats().occupied_slots;
    let after_strings = agent.heap().view().string_stats().occupied_slots;

    assert_eq!(result, Value::from_smi(64));
    assert!(
        after_objects - before_objects < 16,
        "primitive string index reads allocated {} object slots",
        after_objects - before_objects
    );
    assert!(
        after_strings - before_strings < 32,
        "primitive string index reads allocated {} string slots",
        after_strings - before_strings
    );
}

#[test]
fn evaluate_script_repeated_short_latin1_concat_reuses_strings() {
    let warmup = compile_test_unit(23931, "0;");
    let unit = compile_test_unit(
        23932,
        r#"
            var total = 0;
            for (var i = 0; i < 64; i++) {
                total += ("%" + "8" + "F").length;
            }
            total;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let _ = vm.evaluate_script(agent, realm, &warmup).unwrap();
    let before_strings = agent.heap().view().string_stats().occupied_slots;
    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    let after_strings = agent.heap().view().string_stats().occupied_slots;

    assert_eq!(result, Value::from_smi(192));
    assert!(
        after_strings - before_strings < 16,
        "short Latin-1 concat allocated {} string slots",
        after_strings - before_strings
    );
}

#[test]
fn evaluate_script_string_search_uses_regexp_payloads() {
    let unit = compile_test_unit(
        2393,
        r#"
            "abc".search(/b/) === 1 && "\x00\u136c".search(/\0\u136c$/u) === 0;
        "#,
    );
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm.evaluate_script(agent, realm, &unit).unwrap();

    assert_eq!(result, Value::from_bool(true));
}
