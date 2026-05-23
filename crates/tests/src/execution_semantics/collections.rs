use super::support::compile_and_run_string;

#[test]
fn phase6_map_group_by_groups_iterable_values_by_map_keys() {
    let result = compile_and_run_string(
        r#"
        let objectKey = {};
        let grouped = Map.groupBy([1, "1", objectKey, -0, +0], function(value, index) {
            return index < 3 ? value : value;
        });
        let zeroValues = grouped.get(0);

        [
            typeof Map.groupBy,
            String(Map.groupBy.length),
            String(grouped instanceof Map),
            String(grouped.size),
            grouped.get(1).join(","),
            grouped.get("1").join(","),
            String(grouped.get(objectKey)[0] === objectKey),
            String(Object.is(zeroValues[0], -0)),
            String(Object.is(zeroValues[1], +0)),
            Array.from(grouped.keys()).map(function(key) {
                return key === objectKey ? "object" : String(key);
            }).join(",")
        ].join("|");
        "#,
    );

    assert_eq!(result, "function|2|true|4|1|1|true|true|true|1,1,object,0");
}
