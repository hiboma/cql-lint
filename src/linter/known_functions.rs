/// カテゴリ付き関数エントリです。
pub struct FunctionEntry {
    pub name: &'static str,
    pub category: &'static str,
}

/// LogScale の組み込み関数のカテゴリ付きリストです。
/// https://library.humio.com/data-analysis/functions.html から取得しています。
pub static KNOWN_FUNCTION_ENTRIES: &[FunctionEntry] = &[
    // 集計関数
    FunctionEntry {
        name: "accumulate",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "avg",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "correlate",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "count",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "counterRate",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "groupBy",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "linReg",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "max",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "min",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "movingAvg",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "peek",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "percentile",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "range",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "rdns",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "sample",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "selectFromMax",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "selectFromMin",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "selectLast",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "session",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "stats",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "stdDev",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "sum",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "table",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "timeChart",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "top",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "uniq",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "variance",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "window",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "worldMap",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "callFunction",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "counterAsRate",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "partition",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "series",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "slidingTimeWindow",
        category: "Aggregate",
    },
    FunctionEntry {
        name: "slidingWindow",
        category: "Aggregate",
    },
    // 配列関数
    FunctionEntry {
        name: "array:append",
        category: "Array",
    },
    FunctionEntry {
        name: "array:collect",
        category: "Array",
    },
    FunctionEntry {
        name: "array:contains",
        category: "Array",
    },
    FunctionEntry {
        name: "array:dedup",
        category: "Array",
    },
    FunctionEntry {
        name: "array:drop",
        category: "Array",
    },
    FunctionEntry {
        name: "array:eval",
        category: "Array",
    },
    FunctionEntry {
        name: "array:exists",
        category: "Array",
    },
    FunctionEntry {
        name: "array:filter",
        category: "Array",
    },
    FunctionEntry {
        name: "array:flatten",
        category: "Array",
    },
    FunctionEntry {
        name: "array:intersection",
        category: "Array",
    },
    FunctionEntry {
        name: "array:join",
        category: "Array",
    },
    FunctionEntry {
        name: "array:length",
        category: "Array",
    },
    FunctionEntry {
        name: "array:reduceAll",
        category: "Array",
    },
    FunctionEntry {
        name: "array:regex",
        category: "Array",
    },
    FunctionEntry {
        name: "array:sort",
        category: "Array",
    },
    FunctionEntry {
        name: "array:union",
        category: "Array",
    },
    FunctionEntry {
        name: "objectArray:eval",
        category: "Array",
    },
    FunctionEntry {
        name: "objectArray:exists",
        category: "Array",
    },
    FunctionEntry {
        name: "array:reduceColumn",
        category: "Array",
    },
    FunctionEntry {
        name: "array:reduceRow",
        category: "Array",
    },
    FunctionEntry {
        name: "array:rename",
        category: "Array",
    },
    FunctionEntry {
        name: "matchAsArray",
        category: "Array",
    },
    // 条件関数
    FunctionEntry {
        name: "case",
        category: "Conditional",
    },
    FunctionEntry {
        name: "coalesce",
        category: "Conditional",
    },
    FunctionEntry {
        name: "if",
        category: "Conditional",
    },
    FunctionEntry {
        name: "in",
        category: "Conditional",
    },
    FunctionEntry {
        name: "match",
        category: "Conditional",
    },
    FunctionEntry {
        name: "test",
        category: "Conditional",
    },
    // 暗号・ハッシュ関数
    FunctionEntry {
        name: "crypto:md5",
        category: "Crypto",
    },
    FunctionEntry {
        name: "crypto:sha1",
        category: "Crypto",
    },
    FunctionEntry {
        name: "crypto:sha256",
        category: "Crypto",
    },
    FunctionEntry {
        name: "hash",
        category: "Crypto",
    },
    FunctionEntry {
        name: "tokenHash",
        category: "Crypto",
    },
    // Geolocation 関数
    FunctionEntry {
        name: "geography:distance",
        category: "Geolocation",
    },
    FunctionEntry {
        name: "geohash",
        category: "Geolocation",
    },
    // 日付・時刻関数
    FunctionEntry {
        name: "duration",
        category: "DateTime",
    },
    FunctionEntry {
        name: "findTimestamp",
        category: "DateTime",
    },
    FunctionEntry {
        name: "formatDuration",
        category: "DateTime",
    },
    FunctionEntry {
        name: "formatTime",
        category: "DateTime",
    },
    FunctionEntry {
        name: "now",
        category: "DateTime",
    },
    FunctionEntry {
        name: "parseTimestamp",
        category: "DateTime",
    },
    FunctionEntry {
        name: "start",
        category: "DateTime",
    },
    FunctionEntry {
        name: "end",
        category: "DateTime",
    },
    FunctionEntry {
        name: "time:dayOfMonth",
        category: "DateTime",
    },
    FunctionEntry {
        name: "time:dayOfWeek",
        category: "DateTime",
    },
    FunctionEntry {
        name: "time:dayOfWeekName",
        category: "DateTime",
    },
    FunctionEntry {
        name: "time:dayOfYear",
        category: "DateTime",
    },
    FunctionEntry {
        name: "time:hour",
        category: "DateTime",
    },
    FunctionEntry {
        name: "time:millisecond",
        category: "DateTime",
    },
    FunctionEntry {
        name: "time:minute",
        category: "DateTime",
    },
    FunctionEntry {
        name: "time:month",
        category: "DateTime",
    },
    FunctionEntry {
        name: "time:monthName",
        category: "DateTime",
    },
    FunctionEntry {
        name: "time:second",
        category: "DateTime",
    },
    FunctionEntry {
        name: "time:year",
        category: "DateTime",
    },
    FunctionEntry {
        name: "time:weekOfYear",
        category: "DateTime",
    },
    FunctionEntry {
        name: "setTimeInterval",
        category: "DateTime",
    },
    // フィルタ関数
    FunctionEntry {
        name: "cidr",
        category: "Filter",
    },
    FunctionEntry {
        name: "contains",
        category: "Filter",
    },
    FunctionEntry {
        name: "field",
        category: "Filter",
    },
    FunctionEntry {
        name: "ipLocation",
        category: "Filter",
    },
    FunctionEntry {
        name: "regex",
        category: "Filter",
    },
    FunctionEntry {
        name: "text:contains",
        category: "Filter",
    },
    FunctionEntry {
        name: "wildcard",
        category: "Filter",
    },
    FunctionEntry {
        name: "text:endsWith",
        category: "Filter",
    },
    FunctionEntry {
        name: "text:startsWith",
        category: "Filter",
    },
    // フォーマット関数
    FunctionEntry {
        name: "format",
        category: "Format",
    },
    FunctionEntry {
        name: "lowerCase",
        category: "Format",
    },
    FunctionEntry {
        name: "upperCase",
        category: "Format",
    },
    // JSON 関数
    FunctionEntry {
        name: "json:prettyPrint",
        category: "JSON",
    },
    FunctionEntry {
        name: "parseJson",
        category: "JSON",
    },
    FunctionEntry {
        name: "toJson",
        category: "JSON",
    },
    // 数学関数
    FunctionEntry {
        name: "math:abs",
        category: "Math",
    },
    FunctionEntry {
        name: "math:arccos",
        category: "Math",
    },
    FunctionEntry {
        name: "math:arcsin",
        category: "Math",
    },
    FunctionEntry {
        name: "math:arctan",
        category: "Math",
    },
    FunctionEntry {
        name: "math:arctan2",
        category: "Math",
    },
    FunctionEntry {
        name: "math:ceil",
        category: "Math",
    },
    FunctionEntry {
        name: "math:cos",
        category: "Math",
    },
    FunctionEntry {
        name: "math:cosh",
        category: "Math",
    },
    FunctionEntry {
        name: "math:exp",
        category: "Math",
    },
    FunctionEntry {
        name: "math:expm1",
        category: "Math",
    },
    FunctionEntry {
        name: "math:floor",
        category: "Math",
    },
    FunctionEntry {
        name: "math:log",
        category: "Math",
    },
    FunctionEntry {
        name: "math:log10",
        category: "Math",
    },
    FunctionEntry {
        name: "math:log2",
        category: "Math",
    },
    FunctionEntry {
        name: "math:mod",
        category: "Math",
    },
    FunctionEntry {
        name: "math:pow",
        category: "Math",
    },
    FunctionEntry {
        name: "math:sin",
        category: "Math",
    },
    FunctionEntry {
        name: "math:sinh",
        category: "Math",
    },
    FunctionEntry {
        name: "math:sqrt",
        category: "Math",
    },
    FunctionEntry {
        name: "math:tan",
        category: "Math",
    },
    FunctionEntry {
        name: "math:tanh",
        category: "Math",
    },
    FunctionEntry {
        name: "round",
        category: "Math",
    },
    FunctionEntry {
        name: "math:deg2rad",
        category: "Math",
    },
    FunctionEntry {
        name: "math:log1p",
        category: "Math",
    },
    FunctionEntry {
        name: "math:rad2deg",
        category: "Math",
    },
    FunctionEntry {
        name: "parseInt",
        category: "Math",
    },
    // ネットワーク関数
    FunctionEntry {
        name: "asn",
        category: "Network",
    },
    FunctionEntry {
        name: "communityId",
        category: "Network",
    },
    FunctionEntry {
        name: "subnet",
        category: "Network",
    },
    FunctionEntry {
        name: "reverseDns",
        category: "Network",
    },
    FunctionEntry {
        name: "shannonEntropy",
        category: "Network",
    },
    // パース関数
    FunctionEntry {
        name: "base64Decode",
        category: "Parse",
    },
    FunctionEntry {
        name: "base64Encode",
        category: "Parse",
    },
    FunctionEntry {
        name: "kvParse",
        category: "Parse",
    },
    FunctionEntry {
        name: "parseCsv",
        category: "Parse",
    },
    FunctionEntry {
        name: "parseFixedWidth",
        category: "Parse",
    },
    FunctionEntry {
        name: "parseHexString",
        category: "Parse",
    },
    FunctionEntry {
        name: "parseUrl",
        category: "Parse",
    },
    FunctionEntry {
        name: "parseXml",
        category: "Parse",
    },
    FunctionEntry {
        name: "split",
        category: "Parse",
    },
    FunctionEntry {
        name: "splitString",
        category: "Parse",
    },
    FunctionEntry {
        name: "urlDecode",
        category: "Parse",
    },
    FunctionEntry {
        name: "urlEncode",
        category: "Parse",
    },
    FunctionEntry {
        name: "parseCEF",
        category: "Parse",
    },
    FunctionEntry {
        name: "parseLEEF",
        category: "Parse",
    },
    FunctionEntry {
        name: "parseUri",
        category: "Parse",
    },
    // フィールド操作関数
    FunctionEntry {
        name: "concat",
        category: "Field",
    },
    FunctionEntry {
        name: "concatArray",
        category: "Field",
    },
    FunctionEntry {
        name: "drop",
        category: "Field",
    },
    FunctionEntry {
        name: "dropEvent",
        category: "Field",
    },
    FunctionEntry {
        name: "eval",
        category: "Field",
    },
    FunctionEntry {
        name: "fieldset",
        category: "Field",
    },
    FunctionEntry {
        name: "getField",
        category: "Field",
    },
    FunctionEntry {
        name: "head",
        category: "Field",
    },
    FunctionEntry {
        name: "length",
        category: "Field",
    },
    FunctionEntry {
        name: "lower",
        category: "Field",
    },
    FunctionEntry {
        name: "replace",
        category: "Field",
    },
    FunctionEntry {
        name: "rename",
        category: "Field",
    },
    FunctionEntry {
        name: "select",
        category: "Field",
    },
    FunctionEntry {
        name: "setField",
        category: "Field",
    },
    FunctionEntry {
        name: "tail",
        category: "Field",
    },
    FunctionEntry {
        name: "trim",
        category: "Field",
    },
    FunctionEntry {
        name: "upper",
        category: "Field",
    },
    FunctionEntry {
        name: "copyEvent",
        category: "Field",
    },
    FunctionEntry {
        name: "neighbor",
        category: "Field",
    },
    FunctionEntry {
        name: "stripAnsiCodes",
        category: "Field",
    },
    FunctionEntry {
        name: "text:editDistance",
        category: "Field",
    },
    FunctionEntry {
        name: "text:editDistanceAsArray",
        category: "Field",
    },
    FunctionEntry {
        name: "text:length",
        category: "Field",
    },
    FunctionEntry {
        name: "text:positionOf",
        category: "Field",
    },
    FunctionEntry {
        name: "text:substring",
        category: "Field",
    },
    FunctionEntry {
        name: "text:trim",
        category: "Field",
    },
    FunctionEntry {
        name: "lowercase",
        category: "Field",
    },
    FunctionEntry {
        name: "uppercase",
        category: "Field",
    },
    // Event Info 関数
    FunctionEntry {
        name: "eventInternals",
        category: "EventInfo",
    },
    FunctionEntry {
        name: "fieldstats",
        category: "EventInfo",
    },
    // ソート関数
    FunctionEntry {
        name: "sort",
        category: "Sort",
    },
    FunctionEntry {
        name: "reverse",
        category: "Sort",
    },
    // 集合関数
    FunctionEntry {
        name: "bucket",
        category: "Collection",
    },
    FunctionEntry {
        name: "collect",
        category: "Collection",
    },
    FunctionEntry {
        name: "selfJoin",
        category: "Collection",
    },
    FunctionEntry {
        name: "transpose",
        category: "Collection",
    },
    // ユーティリティ関数
    FunctionEntry {
        name: "createEvents",
        category: "Utility",
    },
    FunctionEntry {
        name: "default",
        category: "Utility",
    },
    FunctionEntry {
        name: "eventFieldCount",
        category: "Utility",
    },
    FunctionEntry {
        name: "eventSize",
        category: "Utility",
    },
    FunctionEntry {
        name: "hashMatch",
        category: "Utility",
    },
    FunctionEntry {
        name: "hashRewrite",
        category: "Utility",
    },
    FunctionEntry {
        name: "ioc:lookup",
        category: "Utility",
    },
    FunctionEntry {
        name: "readFile",
        category: "Utility",
    },
    FunctionEntry {
        name: "sankey",
        category: "Utility",
    },
    FunctionEntry {
        name: "saved",
        category: "Utility",
    },
    FunctionEntry {
        name: "typeOf",
        category: "Utility",
    },
    FunctionEntry {
        name: "unit:convert",
        category: "Utility",
    },
    FunctionEntry {
        name: "writeJson",
        category: "Utility",
    },
    FunctionEntry {
        name: "explain:asTable",
        category: "Utility",
    },
    // join 関数
    FunctionEntry {
        name: "defineTable",
        category: "Join",
    },
    FunctionEntry {
        name: "join",
        category: "Join",
    },
    FunctionEntry {
        name: "selfJoinFilter",
        category: "Join",
    },
    // lookup 関数
    FunctionEntry {
        name: "lookup",
        category: "Lookup",
    },
    FunctionEntry {
        name: "lookupFile",
        category: "Lookup",
    },
];

/// LogScale の組み込み関数名のリストです。
/// https://library.humio.com/data-analysis/functions.html から取得しています。
pub static KNOWN_FUNCTIONS: &[&str] = &[
    // 集計関数
    "accumulate",
    "avg",
    "correlate",
    "count",
    "counterRate",
    "groupBy",
    "linReg",
    "max",
    "min",
    "movingAvg",
    "peek",
    "percentile",
    "range",
    "rdns",
    "sample",
    "selectFromMax",
    "selectFromMin",
    "selectLast",
    "session",
    "stats",
    "stdDev",
    "sum",
    "table",
    "timeChart",
    "top",
    "uniq",
    "variance",
    "window",
    "worldMap",
    "callFunction",
    "counterAsRate",
    "partition",
    "series",
    "slidingTimeWindow",
    "slidingWindow",
    // 配列関数
    "array:append",
    "array:collect",
    "array:contains",
    "array:dedup",
    "array:drop",
    "array:eval",
    "array:exists",
    "array:filter",
    "array:flatten",
    "array:intersection",
    "array:join",
    "array:length",
    "array:reduceAll",
    "array:regex",
    "array:sort",
    "array:union",
    "objectArray:eval",
    "objectArray:exists",
    "array:reduceColumn",
    "array:reduceRow",
    "array:rename",
    "matchAsArray",
    // 条件関数
    "case",
    "coalesce",
    "if",
    "in",
    "match",
    "test",
    // 暗号・ハッシュ関数
    "crypto:md5",
    "crypto:sha1",
    "crypto:sha256",
    "hash",
    "tokenHash",
    // Geolocation 関数
    "geography:distance",
    "geohash",
    // 日付・時刻関数
    "duration",
    "findTimestamp",
    "formatDuration",
    "formatTime",
    "now",
    "parseTimestamp",
    "start",
    "end",
    "time:dayOfMonth",
    "time:dayOfWeek",
    "time:dayOfWeekName",
    "time:dayOfYear",
    "time:hour",
    "time:millisecond",
    "time:minute",
    "time:month",
    "time:monthName",
    "time:second",
    "time:year",
    "time:weekOfYear",
    "setTimeInterval",
    // フィルタ関数
    "cidr",
    "contains",
    "field",
    "ipLocation",
    "regex",
    "text:contains",
    "wildcard",
    "text:endsWith",
    "text:startsWith",
    // フォーマット関数
    "format",
    "lowerCase",
    "upperCase",
    // JSON 関数
    "json:prettyPrint",
    "parseJson",
    "toJson",
    // 数学関数
    "math:abs",
    "math:arccos",
    "math:arcsin",
    "math:arctan",
    "math:arctan2",
    "math:ceil",
    "math:cos",
    "math:cosh",
    "math:exp",
    "math:expm1",
    "math:floor",
    "math:log",
    "math:log10",
    "math:log2",
    "math:mod",
    "math:pow",
    "math:sin",
    "math:sinh",
    "math:sqrt",
    "math:tan",
    "math:tanh",
    "round",
    "math:deg2rad",
    "math:log1p",
    "math:rad2deg",
    "parseInt",
    // ネットワーク関数
    "asn",
    "communityId",
    "subnet",
    "reverseDns",
    "shannonEntropy",
    // パース関数
    "base64Decode",
    "base64Encode",
    "kvParse",
    "parseCsv",
    "parseFixedWidth",
    "parseHexString",
    "parseUrl",
    "parseXml",
    "split",
    "splitString",
    "urlDecode",
    "urlEncode",
    "parseCEF",
    "parseLEEF",
    "parseUri",
    // フィールド操作関数
    "concat",
    "concatArray",
    "drop",
    "dropEvent",
    "eval",
    "fieldset",
    "getField",
    "head",
    "length",
    "lower",
    "replace",
    "rename",
    "select",
    "setField",
    "tail",
    "trim",
    "upper",
    "copyEvent",
    "neighbor",
    "stripAnsiCodes",
    "text:editDistance",
    "text:editDistanceAsArray",
    "text:length",
    "text:positionOf",
    "text:substring",
    "text:trim",
    "lowercase",
    "uppercase",
    // Event Info 関数
    "eventInternals",
    "fieldstats",
    // ソート関数
    "sort",
    "reverse",
    // 集合関数
    "bucket",
    "collect",
    "groupBy",
    "selfJoin",
    "transpose",
    // ユーティリティ関数
    "createEvents",
    "default",
    "eventFieldCount",
    "eventSize",
    "hashMatch",
    "hashRewrite",
    "ioc:lookup",
    "readFile",
    "sankey",
    "saved",
    "typeOf",
    "unit:convert",
    "writeJson",
    "explain:asTable",
    // join 関数
    "defineTable",
    "join",
    "selfJoinFilter",
    // lookup 関数
    "lookup",
    "lookupFile",
];

/// 関数名が組み込み関数として認識されるか判定します。
pub fn is_known_function(name: &str) -> bool {
    KNOWN_FUNCTIONS
        .iter()
        .any(|&f| f.eq_ignore_ascii_case(name))
}

/// 集約関数の一覧です。
static AGGREGATE_FUNCTIONS: &[&str] = &[
    "accumulate",
    "avg",
    "correlate",
    "count",
    "counterRate",
    "groupBy",
    "linReg",
    "max",
    "min",
    "movingAvg",
    "peek",
    "percentile",
    "range",
    "sample",
    "selectFromMax",
    "selectFromMin",
    "selectLast",
    "session",
    "stats",
    "stdDev",
    "sum",
    "table",
    "timeChart",
    "top",
    "uniq",
    "variance",
    "window",
    "worldMap",
    "bucket",
    "collect",
    "selfJoin",
    "transpose",
    "sankey",
    "callFunction",
    "counterAsRate",
    "partition",
    "series",
    "slidingTimeWindow",
    "slidingWindow",
];

/// 関数名が集約関数か判定します。
pub fn is_aggregate_function(name: &str) -> bool {
    AGGREGATE_FUNCTIONS
        .iter()
        .any(|&f| f.eq_ignore_ascii_case(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_function() {
        assert!(is_known_function("count"));
        assert!(is_known_function("groupBy"));
        assert!(is_known_function("array:contains"));
    }

    #[test]
    fn test_known_function_case_insensitive() {
        assert!(is_known_function("Count"));
        assert!(is_known_function("GROUPBY"));
    }

    #[test]
    fn test_unknown_function() {
        assert!(!is_known_function("foobar"));
        assert!(!is_known_function("notAFunction"));
    }
}
