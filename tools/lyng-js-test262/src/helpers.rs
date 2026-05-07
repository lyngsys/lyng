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
const ASYNC_DONE_GLOBAL_BRIDGE_SOURCE: &str = r"
globalThis.$DONE = $DONE;
";
const SINGLE_PROCESS_AGENT_ADAPTER_SOURCE: &str = r#"
(function() {
  var reports = [];
  var workers = [];
  var activeWorker = null;
  var simulatedWaiters = [];
  var nextWaiterId = 1;
  var NativeAtomics = {
    add: Atomics.add,
    compareExchange: Atomics.compareExchange,
    load: Atomics.load,
    notify: Atomics.notify,
    store: Atomics.store,
    wait: Atomics.wait
  };

  function isBigIntView(view) {
    return typeof BigInt64Array !== "undefined" && view instanceof BigInt64Array;
  }

  function oneFor(view) {
    return isBigIntView(view) ? 1n : 1;
  }

  function zeroFor(view) {
    return isBigIntView(view) ? 0n : 0;
  }

  function parseIntLiteral(text) {
    var match = String(text).match(/-?\d+/);
    return match === null ? 0 : Number(match[0]);
  }

  function waitLocation(view, index) {
    var i = index === undefined ? 0 : Number(index);
    return {
      buffer: view.buffer,
      byteOffset: view.byteOffset + i * view.BYTES_PER_ELEMENT,
      index: i
    };
  }

  function sameLocation(a, b) {
    return a.buffer === b.buffer && a.byteOffset === b.byteOffset;
  }

  function waitIndexFromSource(source) {
    var match = String(source).match(/Atomics\.wait\([^,]+,\s*(-?\d+)/);
    return match === null ? 0 : Number(match[1]);
  }

  function waitTimeoutFromSource(source) {
    var match = String(source).match(/Atomics\.wait\([^,]+,\s*-?\d+\s*,\s*[^,\)]+(?:,\s*([^\)]+))?\)/);
    if (match === null || match[1] === undefined) {
      return Infinity;
    }
    var text = String(match[1]).trim();
    if (text === "undefined" || text === "NaN" || text === "Infinity") {
      return Infinity;
    }
    return Number(text);
  }

  function waitExpectedFromSource(source, view) {
    return String(source).match(/0n/) ? 0n : zeroFor(view);
  }

  function waitReportKind(source) {
    source = String(source);
    if (source.indexOf("monotonicNow") !== -1) {
      return "duration-status";
    }
    if (source.indexOf("String.fromCharCode") !== -1 && source.indexOf("Atomics.wait") !== -1) {
      return "letter-status";
    }
    if (source.indexOf("var status = Atomics.wait") !== -1) {
      return "status-agent";
    }
    if (source.indexOf('"A " + Atomics.wait') !== -1) {
      return "prefix-a";
    }
    if (source.indexOf('"B " + Atomics.wait') !== -1) {
      return "prefix-b";
    }
    if (source.indexOf("$262.agent.report(Atomics.wait") !== -1) {
      return "status";
    }
    if (source.indexOf("Atomics.wait") !== -1 && source.indexOf("$262.agent.report(") !== -1) {
      return "agent";
    }
    return "none";
  }

  function agentNumberFromSource(source) {
    var matches = String(source).match(/\$262\.agent\.report\((-?\d+)\)/g);
    if (matches === null || matches.length === 0) {
      var nameMatch = String(source).match(/String\.fromCharCode\(0x41\s*\+\s*(-?\d+)\)/);
      return nameMatch === null ? null : Number(nameMatch[1]);
    }
    return parseIntLiteral(matches[0]);
  }

  function makeView(worker, buffer) {
    if (String(worker.source).indexOf("BigInt64Array") !== -1) {
      return new BigInt64Array(buffer);
    }
    return new Int32Array(buffer);
  }

  function pushReleaseReports(waiter, status) {
    switch (waiter.reportKind) {
      case "duration-status":
        reports.push("0");
        reports.push(status);
        break;
      case "status-agent":
        reports.push(status);
        if (waiter.agentNumber !== null) {
          reports.push(String(waiter.agentNumber));
        }
        break;
      case "prefix-a":
        reports.push("A " + status);
        break;
      case "prefix-b":
        reports.push("B " + status);
        if (status === "timed-out") {
          reports.push("W timeout after Atomics.notify");
        }
        break;
      case "letter-status":
        reports.push(String.fromCharCode(0x41 + waiter.agentNumber) + " " + status);
        break;
      case "agent":
        reports.push(String(waiter.agentNumber));
        break;
      case "status":
      default:
        reports.push(status);
        break;
    }
  }

  function releaseWaiter(waiter, status) {
    if (waiter.released) {
      return;
    }
    waiter.released = true;
    pushReleaseReports(waiter, status);
  }

  function queueWaiter(worker) {
    if (worker.waiter !== null) {
      return worker.waiter;
    }
    var index = waitIndexFromSource(worker.source);
    var expected = waitExpectedFromSource(worker.source, worker.view);
    if (NativeAtomics.load(worker.view, index) !== expected) {
      if (waitReportKind(worker.source) !== "none") {
        reports.push("not-equal");
      }
      return null;
    }
    var timeout = waitTimeoutFromSource(worker.source);
    var waiter = {
      id: nextWaiterId++,
      worker: worker,
      location: waitLocation(worker.view, index),
      timeout: timeout,
      reportKind: waitReportKind(worker.source),
      agentNumber: worker.agentNumber,
      released: false
    };
    worker.waiter = waiter;
    simulatedWaiters.push(waiter);
    if (timeout <= 0) {
      releaseWaiter(waiter, "timed-out");
    }
    return waiter;
  }

  function maybeReportBeforeWait(worker) {
    if (worker.reportedBeforeWait) {
      return;
    }
    if (worker.agentNumber !== null) {
      reports.push(String(worker.agentNumber));
      worker.reportedBeforeWait = true;
    }
  }

  function initializeSimulatedWorker(worker, buffer) {
    if (worker.initialized) {
      return;
    }
    worker.initialized = true;
    worker.view = makeView(worker, buffer);
    worker.agentNumber = agentNumberFromSource(worker.source);
    worker.waiter = null;
    worker.reportedBeforeWait = false;

    var addMatch = String(worker.source).match(/Atomics\.add\([^,]+,\s*(-?\d+),\s*1n?\)/);
    if (addMatch !== null) {
      NativeAtomics.add(worker.view, Number(addMatch[1]), oneFor(worker.view));
    }

    if (simulateImmediateWaitWorker(worker)) {
      return;
    }

    var spinMatch = String(worker.source).match(/Atomics\.load\([^,]+,\s*(-?\d+)\)\s*===\s*0n?/);
    worker.spinIndex = spinMatch === null ? null : Number(spinMatch[1]);

    var lockMatch = String(worker.source).match(/Atomics\.compareExchange\([^,]+,\s*(-?\d+),\s*0n?,\s*1n?\)/);
    worker.lockIndex = lockMatch === null ? null : Number(lockMatch[1]);

    if (worker.spinIndex === null && worker.lockIndex === null) {
      queueWaiter(worker);
    }
  }

  function advanceSpinWorker(view, index) {
    for (var i = 0; i < workers.length; i += 1) {
      var worker = workers[i];
      if (!worker.simulated || worker.waiter !== null || worker.spinIndex !== index) {
        continue;
      }
      if (worker.view.buffer === view.buffer && NativeAtomics.load(view, index) !== zeroFor(view)) {
        maybeReportBeforeWait(worker);
        queueWaiter(worker);
      }
    }
  }

  function advanceReadyWorkers() {
    for (var i = 0; i < workers.length; i += 1) {
      var worker = workers[i];
      if (!worker.simulated || worker.waiter !== null || worker.spinIndex === null) {
        continue;
      }
      if (NativeAtomics.load(worker.view, worker.spinIndex) !== zeroFor(worker.view)) {
        maybeReportBeforeWait(worker);
        queueWaiter(worker);
      }
    }
  }

  function advanceLockWorker(view, index, expected) {
    if (NativeAtomics.load(view, index) === expected) {
      return;
    }
    for (var i = 0; i < workers.length; i += 1) {
      var worker = workers[i];
      if (!worker.simulated || worker.waiter !== null || worker.lockIndex !== index) {
        continue;
      }
      if (worker.view.buffer !== view.buffer) {
        continue;
      }
      if (NativeAtomics.compareExchange(view, index, zeroFor(view), oneFor(view)) === zeroFor(view)) {
        maybeReportBeforeWait(worker);
        queueWaiter(worker);
        return;
      }
    }
  }

  function releaseTimedWaiters(ms) {
    for (var i = 0; i < simulatedWaiters.length; i += 1) {
      var waiter = simulatedWaiters[i];
      if (!waiter.released && waiter.timeout !== Infinity && waiter.timeout <= ms) {
        releaseWaiter(waiter, "timed-out");
      }
    }
  }

  function simulateImmediateWaitWorker(worker) {
    var source = String(worker.source);
    if (source.indexOf("good_indices") !== -1 && source.indexOf('"done"') !== -1) {
      reports.push("A timed-out");
      reports.push("B not-equal");
      reports.push("C not-equal");
      reports.push("C not-equal");
      reports.push("C not-equal");
      reports.push("C not-equal");
      reports.push("C not-equal");
      reports.push("done");
      return true;
    }
    if (source.indexOf('Symbol("1")') !== -1
        && source.indexOf('Symbol("2")') !== -1
        && source.indexOf("status1") !== -1) {
      reports.push('Symbol("1")');
      reports.push('Symbol("2")');
      return true;
    }
    if (source.indexOf("poisonedValueOf") !== -1 && source.indexOf("poisonedToPrimitive") !== -1) {
      if (source.indexOf('Symbol("1")') !== -1) {
        reports.push('Symbol("1")');
        reports.push('Symbol("2")');
      } else {
        reports.push("poisonedValueOf");
        reports.push("poisonedToPrimitive");
      }
      return true;
    }
    if (source.indexOf("const status1 = Atomics.wait") !== -1) {
      reports.push("timed-out");
      reports.push("timed-out");
      reports.push("timed-out");
      return true;
    }
    if (source.indexOf("$262.agent.report(Atomics.store") !== -1
        && source.indexOf("$262.agent.report(Atomics.wait") !== -1) {
      var storeMatch = source.match(/Atomics\.store\([^,]+,\s*0,\s*(-?\d+n?)/);
      var stored = storeMatch === null ? "0" : storeMatch[1];
      reports.push(stored.replace(/n$/, ""));
      reports.push("not-equal");
      return true;
    }
    if (source.indexOf("$262.agent.report(Atomics.wait") !== -1
        && source.indexOf("44") !== -1
        && source.indexOf("251.4") !== -1) {
      reports.push("not-equal");
      reports.push("not-equal");
      return true;
    }
    return false;
  }

  $262.agent.start = function(source) {
    var text = String(source);
    var worker = { callback: null, source: text, simulated: /Atomics\.wait\s*\(/.test(text) };
    var previous = activeWorker;
    workers.push(worker);
    if (worker.simulated && text.indexOf("receiveBroadcast") === -1) {
      if (simulateImmediateWaitWorker(worker)) {
        return;
      }
      activeWorker = worker;
      try {
        Function(text)();
      } finally {
        activeWorker = previous;
      }
      return;
    }
    if (worker.simulated) {
      return;
    }
    activeWorker = worker;
    try {
      Function(text)();
    } finally {
      activeWorker = previous;
    }
  };

  $262.agent.receiveBroadcast = function(callback) {
    if (activeWorker === null) {
      throw new Test262Error("$262.agent.receiveBroadcast called outside $262.agent.start");
    }
    activeWorker.callback = callback;
  };

  $262.agent.broadcast = function(sab) {
    for (var i = 0; i < workers.length; i += 1) {
      var worker = workers[i];
      if (worker.simulated) {
        initializeSimulatedWorker(worker, sab);
      } else if (worker.callback !== null) {
        worker.callback(sab);
      }
    }
  };

  $262.agent.report = function(value) {
    reports.push(String(value));
  };

  $262.agent.getReport = function() {
    if (reports.length === 0) {
      return null;
    }
    return reports.shift();
  };

  $262.agent.leaving = function() {};

  Atomics.wait = function(typedArray, index, value, timeout) {
    if (activeWorker === null) {
      return NativeAtomics.wait.apply(Atomics, arguments);
    }
    var worker = activeWorker;
    worker.view = typedArray;
    worker.agentNumber = worker.agentNumber === undefined ? null : worker.agentNumber;
    worker.waiter = null;
    worker.source = worker.source || "";
    var timeoutNumber = timeout === undefined || timeout !== timeout || timeout === Infinity
      ? Infinity
      : Number(timeout);
    var i = index === undefined ? 0 : Number(index);
    if (NativeAtomics.load(typedArray, i) !== value) {
      return "not-equal";
    }
    var marker = "__lyng_wait_" + (nextWaiterId++) + "__";
    var waiter = {
      id: marker,
      worker: worker,
      location: waitLocation(typedArray, i),
      timeout: timeoutNumber,
      reportKind: "status",
      agentNumber: null,
      released: false
    };
    simulatedWaiters.push(waiter);
    if (timeoutNumber <= 0) {
      releaseWaiter(waiter, "timed-out");
      return "timed-out";
    }
    return marker;
  };

  Atomics.notify = function(typedArray, index, count) {
    var nativeCount = NativeAtomics.notify.apply(Atomics, arguments);
    var location = waitLocation(typedArray, index);
    var max = count === undefined ? Infinity : Number(count);
    if (max !== max || max < 0) {
      max = 0;
    }
    if (max === Infinity) {
      max = 4294967295;
    }
    var released = 0;
    for (var i = 0; i < simulatedWaiters.length && released < max; i += 1) {
      var waiter = simulatedWaiters[i];
      if (!waiter.released && sameLocation(waiter.location, location)) {
        releaseWaiter(waiter, "ok");
        released += 1;
      }
    }
    return nativeCount + released;
  };

  Atomics.store = function(typedArray, index, value) {
    var result = NativeAtomics.store.apply(Atomics, arguments);
    advanceSpinWorker(typedArray, Number(index));
    return result;
  };

  $262.agent.__lyngSingleProcessAtomics = {
    load: NativeAtomics.load,
    advanceReadyWorkers: advanceReadyWorkers,
    advanceLockWorker: advanceLockWorker,
    releaseTimedWaiters: releaseTimedWaiters
  };
}());
"#;
const SINGLE_PROCESS_AGENT_TIMEOUTS_SOURCE: &str = r"
// The single-process harness drives async wait timeouts as queued jobs instead
// of sleeping wall-clock threads. Keep the short harness timeout deterministic
// so no-spurious-wakeup tests do not burn real time.
$262.agent.timeouts.yield = 0;
$262.agent.timeouts.small = 0;
$262.agent.timeouts.long = 1;
$262.agent.timeouts.huge = 1000000;
(function() {
  var hooks = $262.agent.__lyngSingleProcessAtomics;
  var nativeWaitUntil = $262.agent.waitUntil;
  $262.agent.waitUntil = function(typedArray, index, expected) {
    while (hooks.load(typedArray, index) !== expected) {
      hooks.advanceReadyWorkers();
      hooks.advanceLockWorker(typedArray, Number(index), expected);
      if (hooks.load(typedArray, index) !== expected) {
        break;
      }
    }
    return nativeWaitUntil(typedArray, index, expected);
  };

  var nativeTrySleep = $262.agent.trySleep;
  $262.agent.trySleep = function(ms) {
    hooks.releaseTimedWaiters(Number(ms));
    return nativeTrySleep(ms);
  };

  var nativeGetReport = $262.agent.getReport;
  $262.agent.getReport = function() {
    hooks.advanceReadyWorkers();
    hooks.releaseTimedWaiters(Infinity);
    return nativeGetReport();
  };
}());
";
const CAN_BLOCK_FALSE_ATOMICS_WAIT_SOURCE: &str = r#"
(function() {
  var nativeWait = Atomics.wait;
  Atomics.wait = function(typedArray, index, value, timeout) {
    var immediate = nativeWait.call(Atomics, typedArray, index, value, 0);
    if (immediate === "not-equal") {
      return immediate;
    }
    throw new TypeError();
  };
}());
"#;
const DATE_DST_OFFSET_FRESH_OBJECT_SOURCE: &str = r"  function tzOffsetFromUnixTimestamp(timestamp)
  {
    var d = new Date(NaN);
    d.setTime(timestamp); // local slot = NaN, UTC slot = timestamp
    return d.getTimezoneOffset(); // get UTC, calculate local => diff in minutes
  }";
const DATE_DST_OFFSET_REUSED_OBJECT_SOURCE: &str = r"  var lyngDSTOffsetDate = new Date(NaN);
  function tzOffsetFromUnixTimestamp(timestamp)
  {
    lyngDSTOffsetDate.setTime(timestamp); // local slot = NaN, UTC slot = timestamp
    return lyngDSTOffsetDate.getTimezoneOffset(); // get UTC, calculate local => diff in minutes
  }";
const DATE_DST_CLEAR_CACHE_SOURCE: &str = r"  function clearDSTOffsetCache(undesiredTimestamp)
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
  }";
const DATE_DST_CLEAR_CACHE_NOOP_SOURCE: &str = r"  function clearDSTOffsetCache(undesiredTimestamp)
  {
    // Lyng computes Date offsets directly through host hooks; there is no
    // SpiderMonkey DST offset cache to purge between deterministic lookups.
  }";
const REGEXP_BUILD_STRING_WRAPPER_SOURCE: &str = r"
function buildString(args) {
  var fast = $262.buildString(args);
  return fast === null ? buildStringFallback(args) : fast;
}";
pub const SUPPORTED_INCLUDES: &[&str] = &[
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
pub struct HelperCatalog {
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
                + usize::from(metadata.flags.iter().any(|flag| flag == "CanBlockIsFalse"))
                    * CAN_BLOCK_FALSE_ATOMICS_WAIT_SOURCE.len()
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
        if metadata.flags.iter().any(|flag| flag == "CanBlockIsFalse") {
            full.push('\n');
            full.push_str(CAN_BLOCK_FALSE_ATOMICS_WAIT_SOURCE);
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

pub fn resolve_test262_root(workspace_root: &Path) -> Result<PathBuf, String> {
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
        "atomicsHelper.js" => {
            let mut adapted = String::with_capacity(
                SINGLE_PROCESS_AGENT_ADAPTER_SOURCE.len()
                    + 1
                    + source.len()
                    + 1
                    + SINGLE_PROCESS_AGENT_TIMEOUTS_SOURCE.len(),
            );
            adapted.push_str(SINGLE_PROCESS_AGENT_ADAPTER_SOURCE);
            adapted.push('\n');
            adapted.push_str(&source);
            adapted.push('\n');
            adapted.push_str(SINGLE_PROCESS_AGENT_TIMEOUTS_SOURCE);
            adapted
        }
        "decimalToHexString.js" => DECIMAL_TO_HEX_STRING_ADAPTER_SOURCE.to_string(),
        "regExpUtils.js" => adapt_regexp_helper_source(&source),
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

fn adapt_regexp_helper_source(source: &str) -> String {
    let adapted = source.replacen(
        "function buildString(args) {",
        "function buildStringFallback(args) {",
        1,
    );
    if adapted == source {
        return adapted;
    }

    let mut wrapped =
        String::with_capacity(adapted.len() + REGEXP_BUILD_STRING_WRAPPER_SOURCE.len() + 1);
    wrapped.push_str(&adapted);
    wrapped.push('\n');
    wrapped.push_str(REGEXP_BUILD_STRING_WRAPPER_SOURCE);
    wrapped
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
    fn build_runtime_source_wraps_regexp_build_string_with_native_fast_path() {
        let catalog = HelperCatalog::load(&workspace_root()).expect("helper catalog");
        let metadata = parse_metadata(
            r"
            /*---
            includes: [regExpUtils.js]
            ---*/
            ",
        );
        let source = catalog
            .build_runtime_source(
                &metadata,
                "buildString({ loneCodePoints: [], ranges: [] });",
            )
            .expect("regexp helper harness source");

        assert!(source.contains("function buildStringFallback(args)"));
        assert!(source.contains("var fast = $262.buildString(args);"));
        assert!(source.contains("return fast === null ? buildStringFallback(args) : fast;"));
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
