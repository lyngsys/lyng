use super::support::compile_and_run_string;

#[test]
fn regexp_scoped_ignore_case_applies_to_backrefs_boundaries_and_properties() {
    let result = compile_and_run_string(
        r#"
        let longS = String.fromCharCode(0x017f);
        [
            /(a)(?i:\1)/.test("aA"),
            /(a)(?-i:\1)/i.test("aA"),
            /(a)(?-i:\1)/i.test("AA"),
            /(?i:\b)/u.test(longS),
            /(?-i:\b)/ui.test(longS),
            /(?i:Z\B)/u.test("Z" + longS),
            /(?-i:Z\B)/ui.test("Z" + longS),
            /(?i:\p{Lu})/u.test("a"),
            /(?i:\P{Lu})/u.test("A")
        ].join("|");
        "#,
    );

    assert_eq!(result, "true|false|true|true|false|true|false|true|true");
}

#[test]
fn regexp_duplicate_named_backrefs_and_escaped_astral_names_match_spec() {
    let result = compile_and_run_string(
        r#"
        function captureEscapedAstralName() {
            try {
                let match = "quick brown fox".match(/(?<\ud835\udcd1\ud835\udcfb\ud835\udcf8\ud835\udd00\ud835\udcf7>brown)/);
                return match.groups["\u{1d4d1}\u{1d4fb}\u{1d4f8}\u{1d500}\u{1d4f7}"];
            } catch (error) {
                return Object.prototype.toString.call(error);
            }
        }

        let simple = /(?:(?<x>a)|(?<x>b))\k<x>/;
        let repeated = /(?:(?:(?<x>a)|(?<x>b))\k<x>){2}/;
        let repeatedMatch = repeated.exec("aabb");

        [
            simple.exec("bb")[0],
            simple.exec("abab") === null,
            repeatedMatch[0],
            repeatedMatch.groups.x,
            repeated.exec("abab") === null,
            captureEscapedAstralName()
        ].join("|");
        "#,
    );

    assert_eq!(result, "bb|true|aabb|b|true|brown");
}

#[test]
fn regexp_group_names_are_readable_through_escaped_identifier_properties() {
    let result = compile_and_run_string(
        r#"
        try {
            let match = "quick brown fox".match(/(?<\u{1d4d1}\u{1d4fb}\u{1d4f8}\u{1d500}\u{1d4f7}>brown)/);
            match === null
                ? "null"
                : match.groups.\u{1d4d1}\u{1d4fb}\u{1d4f8}\u{1d500}\u{1d4f7};
        } catch (error) {
            error.name + ":" + Object.prototype.toString.call(error);
        }
        "#,
    );

    assert_eq!(result, "brown");
}
