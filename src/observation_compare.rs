//! Compare caller-supplied observations without executing their target or oracle.
use serde::{Deserialize, Serialize, de, ser::SerializeMap, ser::SerializeSeq};
use serde_json::value::RawValue;
use std::{collections::BTreeMap, fmt, path::Path};

const MAX_FILE: usize = 8 * 1024 * 1024;
const MAX_NODES: usize = 100_000;
const MAX_DEPTH: usize = 64;
const MAX_DIFFERENCES: usize = 100_000;
const MAX_REPORT: usize = 64 * 1024 * 1024;

// Parsing each object separately lets us reject duplicate keys at every depth.
struct RawObject<'a>(BTreeMap<String, &'a RawValue>);
impl<'de> Deserialize<'de> for RawObject<'de> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = RawObject<'de>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a JSON object without duplicate keys")
            }

            fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut entries = BTreeMap::new();
                while let Some((key, value)) = map.next_entry::<String, &'de RawValue>()? {
                    if entries.insert(key, value).is_some() {
                        return Err(de::Error::custom("Duplicate JSON key"));
                    }
                }
                Ok(RawObject(entries))
            }
        }
        deserializer.deserialize_map(Visitor)
    }
}

struct Node<'a> {
    raw: &'a RawValue,
    value: Data<'a>,
}

enum Data<'a> {
    Null,
    Bool(bool),
    Number(&'a RawValue),
    String(String),
    Array(Vec<Node<'a>>),
    Object(BTreeMap<String, Node<'a>>),
}

impl PartialEq for Node<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (&self.value, &other.value) {
            (Data::Null, Data::Null) => true,
            (Data::Bool(a), Data::Bool(b)) => a == b,
            (Data::Number(a), Data::Number(b)) => a.get() == b.get(),
            (Data::String(a), Data::String(b)) => a == b,
            (Data::Array(a), Data::Array(b)) => a == b,
            (Data::Object(a), Data::Object(b)) => a == b,
            _ => false,
        }
    }
}

impl Serialize for Node<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match &self.value {
            Data::Null => serializer.serialize_unit(),
            Data::Bool(value) => serializer.serialize_bool(*value),
            Data::Number(value) => value.serialize(serializer),
            Data::String(value) => serializer.serialize_str(value),
            Data::Array(values) => {
                let mut seq = serializer.serialize_seq(Some(values.len()))?;
                for value in values {
                    seq.serialize_element(value)?;
                }
                seq.end()
            }
            Data::Object(values) => {
                let mut map = serializer.serialize_map(Some(values.len()))?;
                for (key, value) in values {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            }
        }
    }
}

fn decode<'a>(raw: &'a RawValue, depth: usize, nodes: &mut usize) -> Result<Node<'a>, String> {
    *nodes += 1;
    if *nodes > MAX_NODES || depth > MAX_DEPTH {
        return Err("Observation JSON exceeds node or depth limit".into());
    }
    let text = raw.get();
    let value = match text.as_bytes()[0] {
        b'{' => {
            let entries: RawObject<'a> = serde_json::from_str(text).map_err(|e| e.to_string())?;
            Data::Object(
                entries
                    .0
                    .into_iter()
                    .map(|(key, value)| Ok((key, decode(value, depth + 1, nodes)?)))
                    .collect::<Result<_, String>>()?,
            )
        }
        b'[' => {
            let entries: Vec<&'a RawValue> =
                serde_json::from_str(text).map_err(|e| e.to_string())?;
            Data::Array(
                entries
                    .into_iter()
                    .map(|value| decode(value, depth + 1, nodes))
                    .collect::<Result<_, _>>()?,
            )
        }
        b'"' => Data::String(serde_json::from_str(text).map_err(|e| e.to_string())?),
        b't' => Data::Bool(true),
        b'f' => Data::Bool(false),
        b'n' => Data::Null,
        _ => Data::Number(raw),
    };
    Ok(Node { raw, value })
}

fn object<'a, 'b>(node: &'b Node<'a>) -> Result<&'b BTreeMap<String, Node<'a>>, String> {
    match &node.value {
        Data::Object(value) => Ok(value),
        _ => Err("Expected a JSON object".into()),
    }
}

fn fields<'a, 'b>(
    node: &'b Node<'a>,
    names: &[&str],
) -> Result<&'b BTreeMap<String, Node<'a>>, String> {
    let object = object(node)?;
    if object.len() != names.len() || names.iter().any(|name| !object.contains_key(*name)) {
        return Err(format!(
            "Expected exactly these fields: {}",
            names.join(", ")
        ));
    }
    Ok(object)
}

fn string<'a, 'b>(node: &'b Node<'a>) -> Result<&'b str, String> {
    match &node.value {
        Data::String(value) if !value.trim().is_empty() => Ok(value),
        _ => Err("Expected a nonempty string".into()),
    }
}

struct Case<'a, 'b> {
    input: &'b Node<'a>,
    expected: &'b Node<'a>,
    actual: &'b Node<'a>,
}

struct Run<'a, 'b> {
    oracle: &'b Node<'a>,
    scope: &'b Node<'a>,
    cases: BTreeMap<&'b str, Case<'a, 'b>>,
}

fn run<'a, 'b>(node: &'b Node<'a>) -> Result<Run<'a, 'b>, String> {
    let root = fields(node, &["format", "oracle", "scope", "cases"])?;
    if string(&root["format"])? != "caac-observation-run-v1" {
        return Err("Unsupported observation run format".into());
    }
    let oracle = &root["oracle"];
    let oracle_fields = fields(oracle, &["id", "revision"])?;
    string(&oracle_fields["id"])?;
    string(&oracle_fields["revision"])?;
    let scope = &root["scope"];
    if object(scope)?.is_empty() {
        return Err("Scope must be a nonempty object".into());
    }
    let Data::Array(cases) = &root["cases"].value else {
        return Err("Cases must be an array".into());
    };
    if cases.is_empty() {
        return Err("At least one case is required".into());
    }
    let mut by_id = BTreeMap::new();
    for case in cases {
        let values = fields(case, &["id", "input", "expected", "actual"])?;
        let id = string(&values["id"])?;
        let case = Case {
            input: &values["input"],
            expected: &values["expected"],
            actual: &values["actual"],
        };
        if by_id.insert(id, case).is_some() {
            return Err("Duplicate case ID".into());
        }
    }
    Ok(Run {
        oracle,
        scope,
        cases: by_id,
    })
}

#[derive(Serialize)]
struct Presence<'a> {
    present: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    raw_json: Option<&'a str>,
}

impl<'a> From<Option<&'a Node<'a>>> for Presence<'a> {
    fn from(node: Option<&'a Node<'a>>) -> Self {
        Self {
            present: node.is_some(),
            raw_json: node.map(|value| value.raw.get()),
        }
    }
}

#[derive(Serialize)]
struct Difference<'a> {
    path: String,
    left: Presence<'a>,
    right: Presence<'a>,
}

#[derive(Default)]
struct Budget {
    differences: usize,
    bytes: usize,
}

impl Budget {
    fn add_bytes(&mut self, bytes: usize) -> Result<(), String> {
        self.bytes = self.bytes.saturating_add(bytes);
        if self.bytes > MAX_REPORT {
            return Err("Observation comparison exceeds report size limit".into());
        }
        Ok(())
    }
}

fn differences<'a>(
    left: Option<&'a Node<'a>>,
    right: Option<&'a Node<'a>>,
    path: &mut String,
    output: &mut Vec<Difference<'a>>,
    budget: &mut Budget,
) -> Result<(), String> {
    if left == right {
        return Ok(());
    }
    match (left.map(|n| &n.value), right.map(|n| &n.value)) {
        (Some(Data::Object(a)), Some(Data::Object(b))) => {
            for key in a
                .keys()
                .chain(b.keys())
                .collect::<std::collections::BTreeSet<_>>()
            {
                let length = path.len();
                path.push('/');
                for c in key.chars() {
                    match c {
                        '~' => path.push_str("~0"),
                        '/' => path.push_str("~1"),
                        _ => path.push(c),
                    }
                }
                differences(a.get(key), b.get(key), path, output, budget)?;
                path.truncate(length);
            }
        }
        (Some(Data::Array(a)), Some(Data::Array(b))) => {
            for index in 0..a.len().max(b.len()) {
                let length = path.len();
                path.push('/');
                path.push_str(&index.to_string());
                differences(a.get(index), b.get(index), path, output, budget)?;
                path.truncate(length);
            }
        }
        _ => {
            budget.differences += 1;
            if budget.differences > MAX_DIFFERENCES {
                return Err("Observation comparison exceeds difference limit".into());
            }
            let raw_length = |node: Option<&Node<'_>>| node.map_or(0, |n| n.raw.get().len());
            budget.add_bytes(
                path.len()
                    .saturating_add(raw_length(left))
                    .saturating_add(raw_length(right))
                    .saturating_add(128),
            )?;
            output.push(Difference {
                path: path.clone(),
                left: left.into(),
                right: right.into(),
            });
        }
    }
    Ok(())
}

#[derive(Serialize)]
struct CaseReport<'a> {
    id: &'a str,
    status: &'static str,
    before_vs_expected: Vec<Difference<'a>>,
    after_vs_expected: Vec<Difference<'a>>,
    before_vs_after: Vec<Difference<'a>>,
}

#[derive(Serialize)]
struct Report<'a> {
    format: &'static str,
    before_sha256: String,
    after_sha256: String,
    oracle: &'a Node<'a>,
    scope: &'a Node<'a>,
    counts: BTreeMap<&'static str, usize>,
    cases: Vec<CaseReport<'a>>,
    limits: [&'static str; 3],
}

/// Compare two snapshots without executing a target.
///
/// Objects ignore key order, arrays retain order, and strings use decoded values.
/// Numbers compare by exact JSON text. Errors return no report.
pub fn compare_files(before_path: &Path, after_path: &Path) -> Result<String, String> {
    let before_bytes = crate::source_intake::read_regular(before_path, MAX_FILE)?;
    let after_bytes = crate::source_intake::read_regular(after_path, MAX_FILE)?;
    compare(&before_bytes, &after_bytes)
}

fn compare(before_bytes: &[u8], after_bytes: &[u8]) -> Result<String, String> {
    let before_raw: &RawValue = serde_json::from_slice(before_bytes).map_err(|e| e.to_string())?;
    let after_raw: &RawValue = serde_json::from_slice(after_bytes).map_err(|e| e.to_string())?;
    let before_node = decode(before_raw, 0, &mut 0)?;
    let after_node = decode(after_raw, 0, &mut 0)?;
    let before = run(&before_node)?;
    let after = run(&after_node)?;

    if before.oracle != after.oracle || before.scope != after.scope {
        return Err("Oracle or scope differs between runs".into());
    }
    if !before.cases.keys().eq(after.cases.keys()) {
        return Err("Case IDs differ between runs".into());
    }
    for (id, case) in &before.cases {
        let other = &after.cases[id];
        if case.input != other.input || case.expected != other.expected {
            return Err("Case input or expected result differs between runs".into());
        }
    }

    let mut budget = Budget::default();
    budget.add_bytes(
        before
            .oracle
            .raw
            .get()
            .len()
            .saturating_add(before.scope.raw.get().len())
            .saturating_add(1024),
    )?;
    let mut counts = BTreeMap::from([
        ("fixed", 0),
        ("regressed", 0),
        ("still-matches", 0),
        ("still-differs", 0),
    ]);
    let mut cases = Vec::new();
    for (id, case) in &before.cases {
        budget.add_bytes(id.len().saturating_add(192))?;
        let other = &after.cases[id];
        let status = match (case.actual == case.expected, other.actual == other.expected) {
            (false, true) => "fixed",
            (true, false) => "regressed",
            (true, true) => "still-matches",
            (false, false) => "still-differs",
        };
        *counts.get_mut(status).unwrap() += 1;
        let mut before_vs_expected = Vec::new();
        let mut after_vs_expected = Vec::new();
        let mut before_vs_after = Vec::new();
        let mut path = String::new();
        differences(
            Some(case.actual),
            Some(case.expected),
            &mut path,
            &mut before_vs_expected,
            &mut budget,
        )?;
        differences(
            Some(other.actual),
            Some(case.expected),
            &mut path,
            &mut after_vs_expected,
            &mut budget,
        )?;
        differences(
            Some(case.actual),
            Some(other.actual),
            &mut path,
            &mut before_vs_after,
            &mut budget,
        )?;
        cases.push(CaseReport {
            id,
            status,
            before_vs_expected,
            after_vs_expected,
            before_vs_after,
        });
    }
    let report = Report {
        format: "caac-observation-comparison-v1",
        before_sha256: crate::candidate::digest(before_bytes),
        after_sha256: crate::candidate::digest(after_bytes),
        oracle: before.oracle,
        scope: before.scope,
        counts,
        cases,
        limits: [
            "Oracle, scope, input, expected result and actual result are caller declarations.",
            "This comparison does not execute code or verify the declarations, completeness or acceptance.",
            "Numbers compare by exact JSON text. Object order and string escape spelling do not affect equality.",
        ],
    };
    let encoded = crate::suggest::safe_json(&report);
    if encoded.len() > MAX_REPORT {
        return Err("Observation comparison exceeds report size limit".into());
    }
    Ok(encoded)
}
