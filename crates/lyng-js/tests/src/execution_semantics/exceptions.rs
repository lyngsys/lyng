use super::support::{compile_and_run, compile_and_run_string};
use lyng_js_types::Value;

#[test]
fn phase4_try_catch_finally_runs_in_order() {
    let result = compile_and_run(
        r#"
        let marker = 0;
        try {
            throw 6;
        } catch (error) {
            marker = error;
        } finally {
            marker = marker + 1;
        }
        marker;
        "#,
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn phase4_try_catch_handles_throws_from_called_functions() {
    let result = compile_and_run(
        r#"
        function fail() {
            throw 6;
        }
        let marker = 0;
        try {
            fail();
        } catch (error) {
            marker = error;
        }
        marker;
        "#,
    );

    assert_eq!(result, Value::from_smi(6));
}

#[test]
fn phase4_try_catch_handles_primitive_string_throws() {
    let result = compile_and_run_string(
        r#"
        try {
            throw "error";
        } catch (error) {
            error;
        }
        "#,
    );

    assert_eq!(result, "error");
}

#[test]
fn phase4_finally_preserves_return_completion() {
    let result = compile_and_run(
        r#"
        function read() {
            let box = { marker: 0 };
            try {
                return box;
            } finally {
                box.marker = 5;
            }
        }
        let result = read();
        result.marker;
        "#,
    );

    assert_eq!(result, Value::from_smi(5));
}

#[test]
fn phase4_catch_parameters_shadow_outer_bindings() {
    let result = compile_and_run(
        r#"
        function run(value) {
            try {
                throw 2;
            } catch (value) {
                return value;
            }
        }
        run(1);
        "#,
    );

    assert_eq!(result, Value::from_smi(2));
}

#[test]
fn phase4_labeled_break_and_continue_run_finally_cleanup() {
    let result = compile_and_run(
        r#"
        let total = 0;
        outer: for (let i = 0; i < 4; i = i + 1) {
            try {
                if (i == 1) continue outer;
                if (i == 3) break outer;
                total = total + i;
            } finally {
                total = total + 10;
            }
        }
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(42));
}

#[test]
fn phase4_typeof_identifier_reference_probes_without_throwing() {
    let result = compile_and_run_string("typeof missingName;");

    assert_eq!(result, "undefined");
}

#[test]
fn phase4_sloppy_delete_identifier_reference_uses_phase4_special_cases() {
    let result = compile_and_run(
        r#"
        let local = 1;
        var globalVar = 2;
        let localDeleted = delete local;
        let globalDeleted = delete globalVar;
        let missingDeleted = delete missingName;
        (localDeleted ? 1 : 0) + (globalDeleted ? 2 : 0) + (missingDeleted ? 4 : 0);
        "#,
    );

    assert_eq!(result, Value::from_smi(4));
}
