
var TemporalHelpers = {};

(function () {
  function prefix(description) {
    return description ? description + ": " : "";
  }

  function assertInstance(value, constructor, name, description) {
    assert(value instanceof constructor, prefix(description) + "expected a " + name);
  }

  TemporalHelpers.assertUnreachable = function (description) {
    var message = "This code should not be executed";
    if (description) {
      message += ": " + description;
    }
    throw new Test262Error(message);
  };

  TemporalHelpers.assertDuration = function (duration, years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds, description) {
    var p = prefix(description);
    assertInstance(duration, Temporal.Duration, "Temporal.Duration", description);
    assert.sameValue(duration.years, years, p + "years result:");
    assert.sameValue(duration.months, months, p + "months result:");
    assert.sameValue(duration.weeks, weeks, p + "weeks result:");
    assert.sameValue(duration.days, days, p + "days result:");
    assert.sameValue(duration.hours, hours, p + "hours result:");
    assert.sameValue(duration.minutes, minutes, p + "minutes result:");
    assert.sameValue(duration.seconds, seconds, p + "seconds result:");
    assert.sameValue(duration.milliseconds, milliseconds, p + "milliseconds result:");
    assert.sameValue(duration.microseconds, microseconds, p + "microseconds result:");
    assert.sameValue(duration.nanoseconds, nanoseconds, p + "nanoseconds result:");
  };

  TemporalHelpers.assertDateDuration = function (duration, years, months, weeks, days, description) {
    TemporalHelpers.assertDuration(duration, years, months, weeks, days, 0, 0, 0, 0, 0, 0, description);
  };

  TemporalHelpers.assertDurationsEqual = function (actual, expected, description) {
    assertInstance(expected, Temporal.Duration, "Temporal.Duration", description);
    TemporalHelpers.assertDuration(
      actual,
      expected.years,
      expected.months,
      expected.weeks,
      expected.days,
      expected.hours,
      expected.minutes,
      expected.seconds,
      expected.milliseconds,
      expected.microseconds,
      expected.nanoseconds,
      description
    );
  };

  TemporalHelpers.assertInstantsEqual = function (actual, expected, description) {
    assertInstance(expected, Temporal.Instant, "Temporal.Instant", description);
    assertInstance(actual, Temporal.Instant, "Temporal.Instant", description);
    assert(actual.equals(expected), prefix(description) + "equals method");
  };

  TemporalHelpers.assertPlainDate = function (date, year, month, monthCode, day, description, era, eraYear) {
    var p = prefix(description);
    assertInstance(date, Temporal.PlainDate, "Temporal.PlainDate", description);
    assert.sameValue(date.year, year, p + "year result:");
    assert.sameValue(date.month, month, p + "month result:");
    assert.sameValue(date.monthCode, monthCode, p + "monthCode result:");
    assert.sameValue(date.day, day, p + "day result:");
    if (arguments.length > 6) {
      assert.sameValue(date.era, era, p + "era result:");
      assert.sameValue(date.eraYear, eraYear, p + "eraYear result:");
    }
  };

  TemporalHelpers.assertPlainDatesEqual = function (actual, expected, description) {
    assertInstance(expected, Temporal.PlainDate, "Temporal.PlainDate", description);
    assertInstance(actual, Temporal.PlainDate, "Temporal.PlainDate", description);
    assert(actual.equals(expected), prefix(description) + "equals method");
    assert.sameValue(actual.calendarId, expected.calendarId, prefix(description) + "calendar same value:");
  };

  TemporalHelpers.assertPlainDateTime = function (dateTime, year, month, monthCode, day, hour, minute, second, millisecond, microsecond, nanosecond, description, era, eraYear) {
    var p = prefix(description);
    assertInstance(dateTime, Temporal.PlainDateTime, "Temporal.PlainDateTime", description);
    assert.sameValue(dateTime.year, year, p + "year result:");
    assert.sameValue(dateTime.month, month, p + "month result:");
    assert.sameValue(dateTime.monthCode, monthCode, p + "monthCode result:");
    assert.sameValue(dateTime.day, day, p + "day result:");
    assert.sameValue(dateTime.hour, hour, p + "hour result:");
    assert.sameValue(dateTime.minute, minute, p + "minute result:");
    assert.sameValue(dateTime.second, second, p + "second result:");
    assert.sameValue(dateTime.millisecond, millisecond, p + "millisecond result:");
    assert.sameValue(dateTime.microsecond, microsecond, p + "microsecond result:");
    assert.sameValue(dateTime.nanosecond, nanosecond, p + "nanosecond result:");
    if (arguments.length > 12) {
      assert.sameValue(dateTime.era, era, p + "era result:");
      assert.sameValue(dateTime.eraYear, eraYear, p + "eraYear result:");
    }
  };

  TemporalHelpers.assertPlainDateTimesEqual = function (actual, expected, description) {
    assertInstance(expected, Temporal.PlainDateTime, "Temporal.PlainDateTime", description);
    assertInstance(actual, Temporal.PlainDateTime, "Temporal.PlainDateTime", description);
    assert(actual.equals(expected), prefix(description) + "equals method");
    assert.sameValue(actual.calendarId, expected.calendarId, prefix(description) + "calendar same value:");
  };

  TemporalHelpers.assertPlainTime = function (time, hour, minute, second, millisecond, microsecond, nanosecond, description) {
    var p = prefix(description);
    assertInstance(time, Temporal.PlainTime, "Temporal.PlainTime", description);
    assert.sameValue(time.hour, hour, p + "hour result:");
    assert.sameValue(time.minute, minute, p + "minute result:");
    assert.sameValue(time.second, second, p + "second result:");
    assert.sameValue(time.millisecond, millisecond, p + "millisecond result:");
    assert.sameValue(time.microsecond, microsecond, p + "microsecond result:");
    assert.sameValue(time.nanosecond, nanosecond, p + "nanosecond result:");
  };

  TemporalHelpers.assertPlainTimesEqual = function (actual, expected, description) {
    assertInstance(expected, Temporal.PlainTime, "Temporal.PlainTime", description);
    assertInstance(actual, Temporal.PlainTime, "Temporal.PlainTime", description);
    assert(actual.equals(expected), prefix(description) + "equals method");
  };

  TemporalHelpers.assertPlainMonthDay = function (monthDay, monthCode, day, description, referenceISOYear) {
    var p = prefix(description);
    assertInstance(monthDay, Temporal.PlainMonthDay, "Temporal.PlainMonthDay", description);
    assert.sameValue(monthDay.monthCode, monthCode, p + "monthCode result:");
    assert.sameValue(monthDay.day, day, p + "day result:");
    if (referenceISOYear !== undefined) {
      var isoYear = Number(monthDay.toString({ calendarName: "always" }).split("-")[0]);
      assert.sameValue(isoYear, referenceISOYear, p + "referenceISOYear result:");
    }
  };

  TemporalHelpers.assertPlainYearMonth = function (yearMonth, year, month, monthCode, description, era, eraYear, referenceISODay) {
    var p = prefix(description);
    assertInstance(yearMonth, Temporal.PlainYearMonth, "Temporal.PlainYearMonth", description);
    assert.sameValue(yearMonth.year, year, p + "year result:");
    assert.sameValue(yearMonth.month, month, p + "month result:");
    assert.sameValue(yearMonth.monthCode, monthCode, p + "monthCode result:");
    if (arguments.length > 5) {
      assert.sameValue(yearMonth.era, era, p + "era result:");
      assert.sameValue(yearMonth.eraYear, eraYear, p + "eraYear result:");
    }
    if (referenceISODay !== undefined && referenceISODay !== null) {
      var parts = yearMonth.toString({ calendarName: "always" }).slice(1).split("-");
      var isoDay = Number(parts[2].slice(0, 2));
      assert.sameValue(isoDay, referenceISODay, p + "referenceISODay result:");
    }
  };

  TemporalHelpers.assertZonedDateTimesEqual = function (actual, expected, description) {
    assertInstance(expected, Temporal.ZonedDateTime, "Temporal.ZonedDateTime", description);
    assertInstance(actual, Temporal.ZonedDateTime, "Temporal.ZonedDateTime", description);
    assert(actual.equals(expected), prefix(description) + "equals method");
    assert.sameValue(actual.timeZoneId, expected.timeZoneId, prefix(description) + "time zone same value:");
    assert.sameValue(actual.calendarId, expected.calendarId, prefix(description) + "calendar same value:");
  };

  TemporalHelpers.checkToTemporalCalendarFastPath = function (func) {
    var plainDate = new Temporal.PlainDate(2000, 5, 2, "iso8601");
    var plainDateTime = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 987, 654, 321, "iso8601");
    var plainMonthDay = new Temporal.PlainMonthDay(5, 2, "iso8601");
    var plainYearMonth = new Temporal.PlainYearMonth(2000, 5, "iso8601");
    var zonedDateTime = new Temporal.ZonedDateTime(1000000000000000000n, "UTC", "iso8601");
    var temporalObjects = [plainDate, plainDateTime, plainMonthDay, plainYearMonth, zonedDateTime];

    for (var i = 0; i < temporalObjects.length; i++) {
      var temporalObject = temporalObjects[i];
      Object.defineProperty(temporalObject, "calendar", {
        get: function () {
          throw new Test262Error("should not get 'calendar' property");
        }
      });
      Object.defineProperty(temporalObject, "calendarId", {
        get: function () {
          throw new Test262Error("should not get 'calendarId' property");
        }
      });
      func(temporalObject);
    }
  };

  TemporalHelpers.checkToTemporalInstantFastPath = function (func) {
    var actual = [];
    var expected = [];
    var datetime = new Temporal.ZonedDateTime(1000000000987654321n, "UTC");
    Object.defineProperty(datetime, "toString", {
      get: function () {
        actual.push("get toString");
        return function (options) {
          actual.push("call toString");
          return Temporal.ZonedDateTime.prototype.toString.call(this, options);
        };
      }
    });
    func(datetime);
    assert.compareArray(actual, expected, "toString not called");
  };

  TemporalHelpers.checkPlainDateTimeConversionFastPath = function (func, message) {
    var actual = [];
    var expected = [];
    var calendar = "iso8601";
    var datetime = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 987, 654, 321, calendar);
    var properties = ["year", "month", "monthCode", "day", "hour", "minute", "second", "millisecond", "microsecond", "nanosecond"];
    for (var i = 0; i < properties.length; i++) {
      var property = properties[i];
      var prototypeDescr = Object.getOwnPropertyDescriptor(Temporal.PlainDateTime.prototype, property);
      Object.defineProperty(datetime, property, {
        get: (function (propertyName, descriptor) {
          return function () {
            var value;
            actual.push("get " + formatPropertyName(propertyName));
            value = descriptor.get.call(this);
            return {
              toString: function () {
                actual.push("toString " + formatPropertyName(propertyName));
                return value.toString();
              },
              valueOf: function () {
                actual.push("valueOf " + formatPropertyName(propertyName));
                return value;
              }
            };
          };
        }(property, prototypeDescr))
      });
    }
    Object.defineProperty(datetime, "calendar", {
      get: function () {
        actual.push("get calendar");
        return calendar;
      }
    });
    func(datetime);
    assert.compareArray(actual, expected, (message || "checkPlainDateTimeConversionFastPath") + ": property getters not called");
  };

  TemporalHelpers.checkToTemporalPlainDateTimeFastPath = function (func) {
    var actual = [];
    var expected = [];
    var date = new Temporal.PlainDate(2000, 5, 2, "iso8601");
    var dateProperties = ["year", "month", "monthCode", "day"];
    for (var i = 0; i < dateProperties.length; i++) {
      var dateProperty = dateProperties[i];
      var datePrototypeDescr = Object.getOwnPropertyDescriptor(Temporal.PlainDate.prototype, dateProperty);
      Object.defineProperty(date, dateProperty, {
        get: (function (propertyName, descriptor) {
          return function () {
            var value;
            actual.push("get " + formatPropertyName(propertyName));
            value = descriptor.get.call(this);
            return TemporalHelpers.toPrimitiveObserver(actual, value, propertyName);
          };
        }(dateProperty, datePrototypeDescr))
      });
    }
    var timeProperties = ["hour", "minute", "second", "millisecond", "microsecond", "nanosecond"];
    for (var j = 0; j < timeProperties.length; j++) {
      var timeProperty = timeProperties[j];
      Object.defineProperty(date, timeProperty, {
        get: (function (propertyName) {
          return function () {
            actual.push("get " + formatPropertyName(propertyName));
            return undefined;
          };
        }(timeProperty))
      });
    }
    Object.defineProperty(date, "calendar", {
      get: function () {
        actual.push("get calendar");
        return "iso8601";
      }
    });
    func(date);
    assert.compareArray(actual, expected, "property getters not called");
  };

  TemporalHelpers.checkSubclassingIgnored = function (construct, constructArgs, method, methodArgs, resultAssertions) {
    function constructInstance(C) {
      switch (constructArgs.length) {
        case 0: return new C();
        case 1: return new C(constructArgs[0]);
        case 2: return new C(constructArgs[0], constructArgs[1]);
        case 3: return new C(constructArgs[0], constructArgs[1], constructArgs[2]);
        case 4: return new C(constructArgs[0], constructArgs[1], constructArgs[2], constructArgs[3]);
        case 5: return new C(constructArgs[0], constructArgs[1], constructArgs[2], constructArgs[3], constructArgs[4]);
        case 6: return new C(constructArgs[0], constructArgs[1], constructArgs[2], constructArgs[3], constructArgs[4], constructArgs[5]);
        case 7: return new C(constructArgs[0], constructArgs[1], constructArgs[2], constructArgs[3], constructArgs[4], constructArgs[5], constructArgs[6]);
        case 8: return new C(constructArgs[0], constructArgs[1], constructArgs[2], constructArgs[3], constructArgs[4], constructArgs[5], constructArgs[6], constructArgs[7]);
        case 9: return new C(constructArgs[0], constructArgs[1], constructArgs[2], constructArgs[3], constructArgs[4], constructArgs[5], constructArgs[6], constructArgs[7], constructArgs[8]);
        case 10: return new C(constructArgs[0], constructArgs[1], constructArgs[2], constructArgs[3], constructArgs[4], constructArgs[5], constructArgs[6], constructArgs[7], constructArgs[8], constructArgs[9]);
      }
      throw new Test262Error("unsupported Temporal helper constructor arity");
    }

    function assertIntrinsicResult(instance, description) {
      var result = instance[method].apply(instance, methodArgs);
      assert.sameValue(Object.getPrototypeOf(result), construct.prototype, description);
      resultAssertions(result);
    }

    function checkInstanceConstructor(value, description) {
      var instance = constructInstance(construct);
      instance.constructor = value;
      assertIntrinsicResult(instance, description);
    }

    checkInstanceConstructor(null, "constructor null");
    checkInstanceConstructor(true, "constructor boolean");
    checkInstanceConstructor("test", "constructor string");
    checkInstanceConstructor(Symbol(), "constructor symbol");
    checkInstanceConstructor(7, "constructor number");
    checkInstanceConstructor(7n, "constructor bigint");

    var throwingInstance = constructInstance(construct);
    Object.defineProperty(throwingInstance, "constructor", {
      get: function () {
        throw new Test262Error("constructor getter should be ignored");
      }
    });
    assertIntrinsicResult(throwingInstance, "constructor getter ignored");

    class MySubclass extends construct {}
    var subclassInstance = constructInstance(MySubclass);
    assertIntrinsicResult(subclassInstance, "subclass constructor ignored by method");

    function checkSpecies(value, description) {
      var instance = constructInstance(construct);
      instance.constructor = {};
      instance.constructor[Symbol.species] = value;
      assertIntrinsicResult(instance, description);
    }

    checkSpecies(undefined, "species undefined");
    checkSpecies(null, "species null");
    checkSpecies(true, "species boolean");
    checkSpecies("test", "species string");
    checkSpecies(Symbol(), "species symbol");
    checkSpecies(7, "species number");
    checkSpecies(7n, "species bigint");
    checkSpecies({}, "species object");
    checkSpecies(function () {
      throw new Test262Error("species constructor should be ignored");
    }, "species constructor ignored");

    var speciesThrowingInstance = constructInstance(construct);
    speciesThrowingInstance.constructor = {};
    Object.defineProperty(speciesThrowingInstance.constructor, Symbol.species, {
      get: function () {
        throw new Test262Error("species getter should be ignored");
      }
    });
    assertIntrinsicResult(speciesThrowingInstance, "species getter ignored");
  };

  TemporalHelpers.checkSubclassingIgnoredStatic = function (construct, method, methodArgs, resultAssertions) {
    var result = construct[method].apply(construct, methodArgs);
    assert.sameValue(Object.getPrototypeOf(result), construct.prototype);
    resultAssertions(result);

    var called = false;
    class MySubclass extends construct {
      constructor() {
        called = true;
        super();
      }
    }
    result = construct[method].apply(MySubclass, methodArgs);
    assert.sameValue(called, false);
    assert.sameValue(Object.getPrototypeOf(result), construct.prototype);
    resultAssertions(result);
  };

  function formatPropertyName(propertyName, objectName) {
    if (objectName === undefined) {
      objectName = "";
    }
    if (typeof propertyName === "symbol") {
      if (Symbol.keyFor(propertyName) !== undefined) {
        return objectName + "[Symbol.for('" + Symbol.keyFor(propertyName) + "')]";
      }
      if (propertyName.description.startsWith("Symbol.")) {
        return objectName + "[" + propertyName.description + "]";
      }
      return objectName + "[Symbol('" + propertyName.description + "')]";
    }
    if (typeof propertyName === "string" && propertyName !== String(Number(propertyName))) {
      return objectName ? objectName + "." + propertyName : propertyName;
    }
    return objectName + "[" + propertyName + "]";
  }

  TemporalHelpers.toPrimitiveObserver = function (calls, primitiveValue, propertyName) {
    var observer = {};
    Object.defineProperty(observer, "valueOf", {
      get: function () {
        calls.push("get " + propertyName + ".valueOf");
        return function () {
          calls.push("call " + propertyName + ".valueOf");
          return primitiveValue;
        };
      }
    });
    Object.defineProperty(observer, "toString", {
      get: function () {
        calls.push("get " + propertyName + ".toString");
        return function () {
          calls.push("call " + propertyName + ".toString");
          if (primitiveValue === undefined) {
            return undefined;
          }
          return primitiveValue.toString();
        };
      }
    });
    return observer;
  };

  TemporalHelpers.observeProperty = function (calls, object, propertyName, value, objectName) {
    Object.defineProperty(object, propertyName, {
      get: function () {
        calls.push("get " + formatPropertyName(propertyName, objectName));
        return value;
      },
      set: function () {
        calls.push("set " + formatPropertyName(propertyName, objectName));
      }
    });
  };

  TemporalHelpers.observeMethod = function (calls, object, propertyName, objectName) {
    var method = object[propertyName];
    object[propertyName] = function () {
      calls.push("call " + formatPropertyName(propertyName, objectName));
      return method.apply(object, arguments);
    };
  };

  TemporalHelpers.propertyBagObserver = function (calls, propertyBag, objectName, skipToPrimitive) {
    return new Proxy(propertyBag, {
      ownKeys: function (target) {
        calls.push("ownKeys " + objectName);
        return Reflect.ownKeys(target);
      },
      getOwnPropertyDescriptor: function (target, key) {
        calls.push("getOwnPropertyDescriptor " + formatPropertyName(key, objectName));
        return Reflect.getOwnPropertyDescriptor(target, key);
      },
      get: function (target, key, receiver) {
        var result;
        calls.push("get " + formatPropertyName(key, objectName));
        result = Reflect.get(target, key, receiver);
        if (result === undefined) {
          return undefined;
        }
        if ((result !== null && typeof result === "object") || typeof result === "function") {
          return result;
        }
        if (skipToPrimitive && skipToPrimitive.indexOf(key) >= 0) {
          return result;
        }
        return TemporalHelpers.toPrimitiveObserver(calls, result, formatPropertyName(key, objectName));
      },
      has: function (target, key) {
        calls.push("has " + formatPropertyName(key, objectName));
        return Reflect.has(target, key);
      }
    });
  };

  TemporalHelpers.checkStringOptionWrongType = function (propertyName, value, checkFunc, assertFunc) {
    var expected;
    var actual;
    var observer;
    var result;
    assert.throws(RangeError, function () { checkFunc(null); }, "null");
    assert.throws(RangeError, function () { checkFunc(true); }, "true");
    assert.throws(RangeError, function () { checkFunc(false); }, "false");
    assert.throws(TypeError, function () { checkFunc(Symbol()); }, "symbol");
    assert.throws(RangeError, function () { checkFunc(2); }, "number");
    assert.throws(RangeError, function () { checkFunc(2n); }, "bigint");
    assert.throws(RangeError, function () { checkFunc({}); }, "plain object");
    expected = ["get " + propertyName + ".toString", "call " + propertyName + ".toString"];
    actual = [];
    observer = TemporalHelpers.toPrimitiveObserver(actual, value, propertyName);
    result = checkFunc(observer);
    assertFunc(result, "object with toString");
    assert.compareArray(actual, expected, "order of operations");
  };

  TemporalHelpers.checkRoundingIncrementOptionWrongType = function (checkFunc, assertTrueResultFunc, assertObjectResultFunc) {
    var expected;
    var actual;
    var observer;
    var objectResult;
    assert.throws(RangeError, function () { checkFunc(null); }, "null");
    assertTrueResultFunc(checkFunc(true), "true");
    assert.throws(RangeError, function () { checkFunc(false); }, "false");
    assert.throws(TypeError, function () { checkFunc(Symbol()); }, "symbol");
    assert.throws(TypeError, function () { checkFunc(2n); }, "bigint");
    assert.throws(RangeError, function () { checkFunc({}); }, "plain object");
    expected = ["get roundingIncrement.valueOf", "call roundingIncrement.valueOf"];
    actual = [];
    observer = TemporalHelpers.toPrimitiveObserver(actual, 2, "roundingIncrement");
    objectResult = checkFunc(observer);
    assertObjectResultFunc(objectResult, "object with valueOf");
    assert.compareArray(actual, expected, "order of operations");
  };

  TemporalHelpers.checkPluralUnitsAccepted = function (func, validSingularUnits) {
    var plurals = {
      year: "years",
      month: "months",
      week: "weeks",
      day: "days",
      hour: "hours",
      minute: "minutes",
      second: "seconds",
      millisecond: "milliseconds",
      microsecond: "microseconds",
      nanosecond: "nanoseconds"
    };
    for (var i = 0; i < validSingularUnits.length; i++) {
      var unit = validSingularUnits[i];
      var singularValue = func(unit);
      var pluralValue = func(plurals[unit]);
      var desc = "Plural " + plurals[unit] + " produces the same result as singular " + unit;
      if (singularValue instanceof Temporal.Duration) {
        TemporalHelpers.assertDurationsEqual(pluralValue, singularValue, desc);
      } else if (singularValue instanceof Temporal.Instant) {
        TemporalHelpers.assertInstantsEqual(pluralValue, singularValue, desc);
      } else if (singularValue instanceof Temporal.PlainDateTime) {
        TemporalHelpers.assertPlainDateTimesEqual(pluralValue, singularValue, desc);
      } else if (singularValue instanceof Temporal.PlainTime) {
        TemporalHelpers.assertPlainTimesEqual(pluralValue, singularValue, desc);
      } else if (singularValue instanceof Temporal.ZonedDateTime) {
        TemporalHelpers.assertZonedDateTimesEqual(pluralValue, singularValue, desc);
      } else {
        assert.sameValue(pluralValue, singularValue, desc);
      }
    }
  };

  TemporalHelpers.ISOMonths = [
    { month: 1, monthCode: "M01", daysInMonth: 31 },
    { month: 2, monthCode: "M02", daysInMonth: 29 },
    { month: 3, monthCode: "M03", daysInMonth: 31 },
    { month: 4, monthCode: "M04", daysInMonth: 30 },
    { month: 5, monthCode: "M05", daysInMonth: 31 },
    { month: 6, monthCode: "M06", daysInMonth: 30 },
    { month: 7, monthCode: "M07", daysInMonth: 31 },
    { month: 8, monthCode: "M08", daysInMonth: 31 },
    { month: 9, monthCode: "M09", daysInMonth: 30 },
    { month: 10, monthCode: "M10", daysInMonth: 31 },
    { month: 11, monthCode: "M11", daysInMonth: 30 },
    { month: 12, monthCode: "M12", daysInMonth: 31 }
  ];

  TemporalHelpers.ISO = {};
  TemporalHelpers.ISO.plainMonthDayStringsInvalid = function () {
    return [
      "11-18junk",
      "11-18[u-ca=gregory]",
      "11-18[u-ca=hebrew]",
      "11-18[U-CA=iso8601]",
      "11-18[u-CA=iso8601]",
      "11-18[FOO=bar]",
      "-999999-01-01[u-ca=gregory]",
      "-999999-01-01[u-ca=chinese]",
      "+999999-01-01[u-ca=gregory]",
      "+999999-01-01[u-ca=chinese]"
    ];
  };
  TemporalHelpers.ISO.plainMonthDayStringsValid = function () {
    return [
      "10-01",
      "1001",
      "1965-10-01",
      "1976-10-01T152330.1+00:00",
      "19761001T15:23:30.1+00:00",
      "1976-10-01T15:23:30.1+0000",
      "1976-10-01T152330.1+0000",
      "19761001T15:23:30.1+0000",
      "19761001T152330.1+00:00",
      "19761001T152330.1+0000",
      "+001976-10-01T152330.1+00:00",
      "+0019761001T15:23:30.1+00:00",
      "+001976-10-01T15:23:30.1+0000",
      "+001976-10-01T152330.1+0000",
      "+0019761001T15:23:30.1+0000",
      "+0019761001T152330.1+00:00",
      "+0019761001T152330.1+0000",
      "1976-10-01T15:23:00",
      "1976-10-01T15:23",
      "1976-10-01T15",
      "1976-10-01",
      "--10-01",
      "--1001",
      "-999999-10-01",
      "-999999-10-01[u-ca=iso8601]",
      "+999999-10-01",
      "+999999-10-01[u-ca=iso8601]"
    ];
  };
  TemporalHelpers.ISO.plainTimeStringsAmbiguous = function () {
    var ambiguousStrings = [
      "2021-12",
      "2021-12[-12:00]",
      "1214",
      "0229",
      "1130",
      "12-14",
      "12-14[-14:00]",
      "202112",
      "202112[UTC]"
    ];
    var stringsWithCalendar = ambiguousStrings.map(function (s) {
      return s + "[u-ca=iso8601]";
    });
    return ambiguousStrings.concat(stringsWithCalendar);
  };
  TemporalHelpers.ISO.plainTimeStringsUnambiguous = function () {
    return [
      "2021-13",
      "202113",
      "2021-13[-13:00]",
      "202113[-13:00]",
      "0000-00",
      "000000",
      "0000-00[UTC]",
      "000000[UTC]",
      "1314",
      "13-14",
      "1232",
      "0230",
      "0631",
      "0000",
      "00-00"
    ];
  };
  TemporalHelpers.ISO.plainYearMonthStringsInvalid = function () {
    return [
      "2020-13",
      "1976-11[u-ca=gregory]",
      "1976-11[u-ca=hebrew]",
      "1976-11[U-CA=iso8601]",
      "1976-11[u-CA=iso8601]",
      "1976-11[FOO=bar]",
      "+999999-01",
      "-999999-01"
    ];
  };
  TemporalHelpers.ISO.plainYearMonthStringsValid = function () {
    return [
      "1976-11",
      "1976-11-10",
      "1976-11-01T09:00:00+00:00",
      "1976-11-01T00:00:00+05:00",
      "197611",
      "+00197611",
      "1976-11-18T15:23:30.1-02:00",
      "1976-11-18T152330.1+00:00",
      "19761118T15:23:30.1+00:00",
      "1976-11-18T15:23:30.1+0000",
      "1976-11-18T152330.1+0000",
      "19761118T15:23:30.1+0000",
      "19761118T152330.1+00:00",
      "19761118T152330.1+0000",
      "+001976-11-18T152330.1+00:00",
      "+0019761118T15:23:30.1+00:00",
      "+001976-11-18T15:23:30.1+0000",
      "+001976-11-18T152330.1+0000",
      "+0019761118T15:23:30.1+0000",
      "+0019761118T152330.1+00:00",
      "+0019761118T152330.1+0000",
      "1976-11-18T15:23",
      "1976-11-18T15",
      "1976-11-18"
    ];
  };
  TemporalHelpers.ISO.plainYearMonthStringsValidNegativeYear = function () {
    return [
      "-009999-11"
    ];
  };
}());
