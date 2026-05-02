use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::metadata::{has_async_flag, variants_for_metadata, TestMetadata, TestVariant};

const LOCAL_TEMPORAL_HELPERS_SOURCE: &str = include_str!("temporal_helpers.js");
const DECIMAL_TO_HEX_STRING_ADAPTER_SOURCE: &str = r#"
function toUint32DecimalHelper(value) {
  var number = Number(value);
  if (!(number >= 0 || number < 0) || number === 0 || number === Infinity || number === -Infinity) {
    return 0;
  }
  // JS3 does not lower unsigned right shift yet, so keep the helper upstream-shaped
  // while expressing ToUint32 in terms of arithmetic the runtime already supports.
  var integer = number - (number % 1);
  var modulo = integer % 4294967296;
  if (modulo < 0) {
    modulo += 4294967296;
  }
  return modulo;
}

function decimalToHexString(n) {
  var hex = "0123456789ABCDEF";
  n = toUint32DecimalHelper(n);
  var s = "";
  while (n > 0) {
    var digit = n % 16;
    s = hex.charAt(digit) + s;
    n = (n - digit) / 16;
  }
  while (s.length < 4) {
    s = "0" + s;
  }
  return s;
}

function decimalToPercentHexString(n) {
  var hex = "0123456789ABCDEF";
  n = toUint32DecimalHelper(n) % 256;
  var low = n % 16;
  return "%" + hex.charAt((n - low) / 16) + hex.charAt(low);
}
"#;
const ASYNC_DONE_GLOBAL_BRIDGE_SOURCE: &str = r#"
globalThis.$DONE = $DONE;
"#;
const DATE_DST_OFFSET_FRESH_OBJECT_SOURCE: &str = r#"  function tzOffsetFromUnixTimestamp(timestamp)
  {
    var d = new Date(NaN);
    d.setTime(timestamp); // local slot = NaN, UTC slot = timestamp
    return d.getTimezoneOffset(); // get UTC, calculate local => diff in minutes
  }"#;
const DATE_DST_OFFSET_REUSED_OBJECT_SOURCE: &str = r#"  var lyngDSTOffsetDate = new Date(NaN);
  function tzOffsetFromUnixTimestamp(timestamp)
  {
    lyngDSTOffsetDate.setTime(timestamp); // local slot = NaN, UTC slot = timestamp
    return lyngDSTOffsetDate.getTimezoneOffset(); // get UTC, calculate local => diff in minutes
  }"#;
const DATE_DST_CLEAR_CACHE_SOURCE: &str = r#"  function clearDSTOffsetCache(undesiredTimestamp)
  {
    var opposite = (undesiredTimestamp + MAX_UNIX_TIMET / 2) % MAX_UNIX_TIMET;

    // Generic purge to known, but not necessarily desired, state
    tzOffsetFromUnixTimestamp(0);
    tzOffsetFromUnixTimestamp(MAX_UNIX_TIMET);

    // Purge to desired state.  Cycle 2x in case opposite or undesiredTimestamp
    // is close to 0 or MAX_UNIX_TIMET.
    tzOffsetFromUnixTimestamp(opposite);
    tzOffsetFromUnixTimestamp(undesiredTimestamp);
    tzOffsetFromUnixTimestamp(opposite);
    tzOffsetFromUnixTimestamp(undesiredTimestamp);
  }"#;
const DATE_DST_CLEAR_CACHE_NOOP_SOURCE: &str = r#"  function clearDSTOffsetCache(undesiredTimestamp)
  {
    // Lyng computes Date offsets directly through host hooks; there is no
    // SpiderMonkey DST offset cache to purge between deterministic lookups.
  }"#;
pub(crate) const SUPPORTED_INCLUDES: &[&str] = &[
    "compareArray.js",
    "deepEqual.js",
    "propertyHelper.js",
    "promiseHelper.js",
    "asyncHelpers.js",
    "isConstructor.js",
    "wellKnownIntrinsicObjects.js",
    "fnGlobalObject.js",
    "testTypedArray.js",
    "byteConversionValues.js",
    "detachArrayBuffer.js",
    "nans.js",
    "temporalHelpers.js",
    "regExpUtils.js",
    "nativeFunctionMatcher.js",
    "decimalToHexString.js",
    "compareIterator.js",
    "proxyTrapsHelper.js",
    "assertRelativeDateMs.js",
    "dateConstants.js",
    "atomicsHelper.js",
    "iteratorZipUtils.js",
    "resizableArrayBufferUtils.js",
    "testAtomics.js",
    "tcoHelper.js",
    "sm/assertThrowsValue.js",
    "sm/non262-Date-shell.js",
    "sm/non262-JSON-shell.js",
    "sm/non262-Math-shell.js",
    "sm/non262-Reflect-shell.js",
    "sm/non262-Set-shell.js",
    "sm/non262-Temporal-PlainMonthDay-shell.js",
    "sm/non262-TypedArray-shell.js",
    "sm/non262-expressions-shell.js",
    "sm/non262-generators-shell.js",
    "sm/non262-strict-shell.js",
];

#[derive(Clone)]
pub(crate) struct HelperCatalog {
    base_source: String,
    async_done_source: String,
    include_sources: HashMap<&'static str, String>,
    test262_root: PathBuf,
}

impl HelperCatalog {
    pub(crate) fn load(workspace_root: &Path) -> Result<Self, String> {
        let test262_root = resolve_test262_root(workspace_root)?;
        let harness_root = test262_root.join("harness");
        let mut include_sources = HashMap::new();
        for include in SUPPORTED_INCLUDES {
            let source = match *include {
                "temporalHelpers.js" => LOCAL_TEMPORAL_HELPERS_SOURCE.to_string(),
                name => adapt_helper_source(name, read_helper_file(&harness_root, name)?),
            };
            include_sources.insert(*include, source);
        }

        let assert_source = read_helper_file(&harness_root, "assert.js")?;
        let sta_source = read_helper_file(&harness_root, "sta.js")?;
        let async_done_source = format!(
            "{}\n{}",
            read_helper_file(&harness_root, "doneprintHandle.js")?,
            ASYNC_DONE_GLOBAL_BRIDGE_SOURCE
        );
        let base_source = format!("{assert_source}\n{sta_source}");

        Ok(Self {
            base_source,
            async_done_source,
            include_sources,
            test262_root,
        })
    }

    pub(crate) fn build_runtime_source(
        &self,
        metadata: &TestMetadata,
        source: &str,
    ) -> Result<String, String> {
        let variant = variants_for_metadata(metadata)
            .into_iter()
            .next()
            .unwrap_or(TestVariant::Default);
        self.build_runtime_source_for_variant(metadata, variant, source)
    }

    pub(crate) fn build_runtime_source_for_variant(
        &self,
        metadata: &TestMetadata,
        variant: TestVariant,
        source: &str,
    ) -> Result<String, String> {
        if variant.is_raw() {
            return Ok(source.to_string());
        }

        let mut full = String::with_capacity(
            self.base_source.len()
                + source.len()
                + metadata.includes.len() * 128
                + usize::from(has_async_flag(metadata)) * self.async_done_source.len(),
        );
        if variant.uses_strict_directive() {
            full.push_str("\"use strict\";\n");
        }
        full.push_str(&self.base_source);
        if has_async_flag(metadata) {
            full.push('\n');
            full.push_str(&self.async_done_source);
        }
        for include in &metadata.includes {
            let extra = self
                .source_for(include)
                .ok_or_else(|| format!("unsupported harness include: {include}"))?;
            if !extra.is_empty() {
                full.push('\n');
                full.push_str(extra);
            }
        }
        full.push('\n');
        full.push_str(source);
        Ok(full)
    }

    pub(crate) fn supports_include(&self, include: &str) -> bool {
        self.include_sources.contains_key(include)
    }

    pub(crate) fn test_dir(&self) -> PathBuf {
        self.test262_root.join("test")
    }

    fn source_for(&self, include: &str) -> Option<&str> {
        self.include_sources.get(include).map(String::as_str)
    }
}

pub(crate) fn resolve_test262_root(workspace_root: &Path) -> Result<PathBuf, String> {
    for candidate in workspace_root.ancestors() {
        let test262_root = candidate.join("testdata/test262");
        if test262_root.join("harness/assert.js").is_file() && test262_root.join("test").is_dir() {
            return Ok(test262_root);
        }
    }

    Err(format!(
        "test262 fixture root not found from workspace {}",
        workspace_root.display()
    ))
}

fn read_helper_file(harness_root: &Path, name: &str) -> Result<String, String> {
    let path = harness_root.join(name);
    fs::read_to_string(&path)
        .map_err(|error| format!("failed to read harness helper {}: {error}", path.display()))
}

fn adapt_helper_source(name: &str, source: String) -> String {
    match name {
        "decimalToHexString.js" => DECIMAL_TO_HEX_STRING_ADAPTER_SOURCE.to_string(),
        "sm/non262-Date-shell.js" => source
            .replace(
                DATE_DST_OFFSET_FRESH_OBJECT_SOURCE,
                DATE_DST_OFFSET_REUSED_OBJECT_SOURCE,
            )
            .replace(
                DATE_DST_CLEAR_CACHE_SOURCE,
                DATE_DST_CLEAR_CACHE_NOOP_SOURCE,
            )
            .replace("assert.sameValue(", "$262.sameValue("),
        _ => source,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::metadata::parse_metadata;

    use super::{resolve_test262_root, HelperCatalog};

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("workspace root should exist")
    }

    #[test]
    fn resolves_test262_root_from_worktree_or_checkout() {
        let test262_root = resolve_test262_root(&workspace_root()).expect("test262 root");
        assert!(test262_root.join("harness/assert.js").is_file());
        assert!(test262_root.join("test").is_dir());
    }

    #[test]
    fn build_runtime_source_uses_upstream_base_helpers_and_async_done_selectively() {
        let catalog = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        let sync_metadata = parse_metadata(
            r"
            /*---
            includes: [propertyHelper.js]
            ---*/
            ",
        );
        let sync_source = catalog
            .build_runtime_source(&sync_metadata, "verifyProperty({}, 'x');")
            .expect("sync harness source");
        assert!(sync_source.contains("function Test262Error"));
        assert!(sync_source.contains("function assert("));
        assert!(sync_source.contains("verifyProperty({}, 'x');"));
        assert!(!sync_source.contains("function $DONE("));

        let async_metadata = parse_metadata(
            r"
            /*---
            flags: [async]
            includes: [asyncHelpers.js]
            ---*/
            ",
        );
        let async_source = catalog
            .build_runtime_source(&async_metadata, "asyncTest(async function () {});")
            .expect("async harness source");
        assert!(async_source.contains("function $DONE("));
        assert!(async_source.contains("globalThis.$DONE = $DONE;"));
        assert!(async_source.contains("assert.throwsAsync = function"));
    }

    #[test]
    fn build_runtime_source_loads_assert_before_sta() {
        let catalog = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        let metadata = parse_metadata("");
        let source = catalog
            .build_runtime_source(&metadata, "")
            .expect("base harness source");

        let assert_index = source.find("function assert(").expect("assert.js source");
        let sta_index = source.find("function Test262Error").expect("sta.js source");
        assert!(
            assert_index < sta_index,
            "assert.js must be evaluated before sta.js"
        );
    }

    #[test]
    fn build_runtime_source_uses_doneprint_handle_for_async_tests() {
        let catalog = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        let metadata = parse_metadata(
            r"
            /*---
            flags: [async]
            ---*/
            ",
        );
        let source = catalog
            .build_runtime_source(&metadata, "$DONE();")
            .expect("async harness source");

        assert!(source.contains("Test262:AsyncTestComplete"));
        assert!(source.contains("Test262:AsyncTestFailure:"));
    }

    #[test]
    fn build_runtime_source_leaves_raw_tests_unmodified() {
        let catalog = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        let metadata = parse_metadata(
            r"
            /*---
            flags: [raw]
            includes: [propertyHelper.js]
            ---*/
            ",
        );
        let test_source = "'use strict'\n[0]\n's'.p = null;";
        let source = catalog
            .build_runtime_source(&metadata, test_source)
            .expect("raw source should build");

        assert_eq!(source, test_source);
    }

    #[test]
    fn build_runtime_source_uses_upstream_well_known_intrinsics_helper() {
        let catalog = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        let metadata = parse_metadata(
            r"
            /*---
            includes: [wellKnownIntrinsicObjects.js]
            ---*/
            ",
        );
        let source = catalog
            .build_runtime_source(&metadata, "getWellKnownIntrinsicObject('%Array%');")
            .expect("well-known intrinsic harness source");
        assert!(source.contains("name: '%Array%'"));
        assert!(source.contains("new Function(\"return \" + wkio.source)()"));
    }

    #[test]
    fn build_runtime_source_adapts_decimal_helper_without_forking_behavior() {
        let catalog = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        let metadata = parse_metadata(
            r"
            /*---
            includes: [decimalToHexString.js]
            ---*/
            ",
        );
        let source = catalog
            .build_runtime_source(&metadata, "decimalToHexString(100);")
            .expect("decimal helper harness source");
        assert!(source.contains("toUint32DecimalHelper"));
        assert!(source.contains("var integer = number - (number % 1);"));
        assert!(source.contains("return \"%\" + hex.charAt((n - low) / 16) + hex.charAt(low);"));
        assert!(!source.contains("Math.floor"));
    }

    #[test]
    fn supports_current_spidermonkey_non262_helpers() {
        let catalog = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        for include in [
            "sm/non262-TypedArray-shell.js",
            "sm/non262-strict-shell.js",
            "sm/assertThrowsValue.js",
            "sm/non262-Math-shell.js",
            "sm/non262-Date-shell.js",
            "sm/non262-JSON-shell.js",
            "sm/non262-Set-shell.js",
            "sm/non262-expressions-shell.js",
            "sm/non262-generators-shell.js",
            "sm/non262-Reflect-shell.js",
        ] {
            assert!(
                catalog.supports_include(include),
                "missing SpiderMonkey helper include {include}"
            );
        }
    }

    #[test]
    fn adapts_spidermonkey_date_helper_to_native_same_value_fast_path() {
        let catalog = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        let source = catalog
            .source_for("sm/non262-Date-shell.js")
            .expect("date helper source");

        assert!(source.contains("$262.sameValue(tzo1, CORRECT_TZOFFSETS[i]);"));
        assert!(!source.contains("assert.sameValue(tzo1, CORRECT_TZOFFSETS[i]);"));
        assert!(source.contains("var lyngDSTOffsetDate = new Date(NaN);"));
        assert!(!source.contains("var d = new Date(NaN);"));
        assert!(source.contains("Lyng computes Date offsets directly through host hooks"));
        assert!(!source.contains("tzOffsetFromUnixTimestamp(opposite);"));
    }

    #[test]
    fn supports_iterator_zip_helper() {
        let catalog = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        assert!(catalog.supports_include("iteratorZipUtils.js"));
    }

    #[test]
    fn supports_resizable_arraybuffer_helper() {
        let catalog = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        assert!(catalog.supports_include("resizableArrayBufferUtils.js"));
    }
}
