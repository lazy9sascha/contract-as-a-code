use std::collections::BTreeMap;
use std::path::Path;

/// A prepared pattern with slots and a code template.
#[derive(Debug, Clone)]
pub struct Pattern {
    pub id: &'static str,
    pub name: &'static str,
    pub family: &'static str,
    pub slots: &'static [&'static str],
    pub required_slots: &'static [&'static str],
    pub template: &'static str,
}

pub const PATTERNS: &[Pattern] = &[
    // === Abstraction (4) ===
    Pattern {
        id: "ABS-INTERFACE",
        name: "declare and bind abstract responsibilities",
        family: "abstraction",
        slots: &["trait_name", "method_name", "input_type", "output_type"],
        required_slots: &["trait_name", "method_name"],
        template: "// ABS-INTERFACE: declare and bind abstract responsibilities\npub trait {trait_name} {\n    fn {method_name}(&self, input: {input_type}) -> {output_type};\n}\n",
    },
    Pattern {
        id: "ABS-GENERIC",
        name: "instantiate parameterized meaning",
        family: "abstraction",
        slots: &["fn_name", "type_param", "input_type", "output_type"],
        required_slots: &["fn_name", "type_param"],
        template: "// ABS-GENERIC: instantiate parameterized meaning\npub fn {fn_name}<{type_param}>(value: {type_param}) -> {type_param} {\n    value\n}\n",
    },
    Pattern {
        id: "ABS-INHERIT",
        name: "derive at declared extension points",
        family: "abstraction",
        slots: &["base_trait", "extended_trait", "method_name"],
        required_slots: &["base_trait", "extended_trait", "method_name"],
        template: "// ABS-INHERIT: derive at declared extension points\npub trait {base_trait} { fn {method_name}(&self, x: i64) -> i64; }\npub trait {extended_trait}: {base_trait} {{}}\n",
    },
    Pattern {
        id: "ABS-COMPOSE",
        name: "connect typed operations",
        family: "abstraction",
        slots: &["fn_name", "f_type", "g_type"],
        required_slots: &["fn_name"],
        template: "// ABS-COMPOSE: connect typed operations\npub fn {fn_name}<F, G>(f: F, g: G) -> impl Fn(i64) -> i64\nwhere F: Fn(i64) -> i64, G: Fn(i64) -> i64\n{\n    move |x| g(f(x))\n}\n",
    },
    // === Concurrency (2) ===
    Pattern {
        id: "CON-TASK",
        name: "start asynchronous bounded or ongoing work",
        family: "concurrency",
        slots: &["fn_name", "body_type"],
        required_slots: &["fn_name"],
        template: "// CON-TASK: start async work\npub fn {fn_name}<F: FnOnce() + Send + 'static>(f: F) {\n    std::thread::spawn(f);\n}\n",
    },
    Pattern {
        id: "CON-COORDINATE",
        name: "join tasks and protect shared access",
        family: "concurrency",
        slots: &["fn_name", "result_type"],
        required_slots: &["fn_name"],
        template: "// CON-COORDINATE: join tasks\npub fn {fn_name}<T>(handles: Vec<std::thread::JoinHandle<T>>) -> Vec<T> {\n    handles.into_iter().map(|h| h.join().unwrap()).collect()\n}\n",
    },
    // === Control (6) ===
    Pattern {
        id: "FLOW-SEQUENCE",
        name: "execute finite ordered work",
        family: "control",
        slots: &["fn_name", "step_type", "result_type"],
        required_slots: &["fn_name"],
        template: "// FLOW-SEQUENCE: execute steps in order\npub fn {fn_name}(steps: Vec<Box<dyn Fn()>>) {\n    for step in steps { step(); }\n}\n",
    },
    Pattern {
        id: "FLOW-BRANCH",
        name: "select one Boolean path",
        family: "control",
        slots: &["fn_name", "result_type"],
        required_slots: &["fn_name"],
        template: "// FLOW-BRANCH: select one path\npub fn {fn_name}<T>(cond: bool, then_fn: impl Fn() -> T, else_fn: impl Fn() -> T) -> T {\n    if cond { then_fn() } else { else_fn() }\n}\n",
    },
    Pattern {
        id: "FLOW-MATCH",
        name: "choose one tagged alternative",
        family: "control",
        slots: &["enum_name", "fn_name"],
        required_slots: &["enum_name", "fn_name"],
        template: "// FLOW-MATCH: choose one alternative\npub enum {enum_name} { A, B }\npub fn {fn_name}(tag: {enum_name}) -> &'static str {\n    match tag { {enum_name}::A => \"a\", {enum_name}::B => \"b\" }\n}\n",
    },
    Pattern {
        id: "FLOW-LOOP",
        name: "repeat bounded or ongoing work",
        family: "control",
        slots: &["fn_name", "item_type", "result_type"],
        required_slots: &["fn_name"],
        template: "// FLOW-LOOP: repeat bounded work\npub fn {fn_name}<T, F: FnMut() -> Option<T>>(mut f: F) -> Vec<T> {\n    let mut results = Vec::new();\n    while let Some(v) = f() { results.push(v); }\n    results\n}\n",
    },
    Pattern {
        id: "FLOW-EXIT",
        name: "transfer to one declared target",
        family: "control",
        slots: &["enum_name"],
        required_slots: &["enum_name"],
        template: "// FLOW-EXIT: transfer to declared target\npub enum {enum_name} { Return(i64), Break, Continue }\n",
    },
    Pattern {
        id: "FLOW-RECURSION",
        name: "call with bounded progress",
        family: "control",
        slots: &["fn_name", "max_depth"],
        required_slots: &["fn_name"],
        template: "// FLOW-RECURSION: bounded recursion\npub fn {fn_name}(n: i64, max: i64) -> i64 {\n    if n <= 0 || n > max { return 0; }\n    n + {fn_name}(n - 1, max)\n}\n",
    },
    // === Effects (6) ===
    Pattern {
        id: "EXT-IO",
        name: "perform one bounded external interaction",
        family: "effects",
        slots: &["fn_name", "input_type", "output_type", "error_type"],
        required_slots: &["fn_name"],
        template: "// EXT-IO: bounded external interaction\npub fn {fn_name}(input: &str) -> Result<String, String> {\n    Ok(input.to_owned())\n}\n",
    },
    Pattern {
        id: "EXT-PERSIST",
        name: "read or change stored state",
        family: "effects",
        slots: &["store_fn", "load_fn", "key_type", "value_type"],
        required_slots: &["store_fn", "load_fn"],
        template: "// EXT-PERSIST: read or change stored state\npub fn {store_fn}(key: &str, value: &str) {}\npub fn {load_fn}(key: &str) -> Option<String> { None }\n",
    },
    Pattern {
        id: "EXT-TRANSACTION",
        name: "group effects under a declared atomic boundary",
        family: "effects",
        slots: &["fn_name", "result_type"],
        required_slots: &["fn_name"],
        template: "// EXT-TRANSACTION: atomic boundary\npub fn {fn_name}<F: FnOnce() -> bool>(f: F) -> bool {\n    f()\n}\n",
    },
    Pattern {
        id: "EXT-MESSAGE",
        name: "send or consume a bounded message",
        family: "effects",
        slots: &["send_fn", "recv_fn", "msg_type"],
        required_slots: &["send_fn", "recv_fn"],
        template: "// EXT-MESSAGE: send or consume a message\npub fn {send_fn}(msg: &str) {}\npub fn {recv_fn}() -> Option<String> { None }\n",
    },
    Pattern {
        id: "EXT-IDEMPOTENCY",
        name: "bound duplicate observable effects",
        family: "effects",
        slots: &["fn_name", "key_type"],
        required_slots: &["fn_name"],
        template: "// EXT-IDEMPOTENCY: bound duplicate effects\nuse std::collections::HashSet;\npub fn {fn_name}(key: &str, seen: &mut HashSet<String>) -> bool {\n    seen.insert(key.to_owned())\n}\n",
    },
    Pattern {
        id: "EXT-COMPENSATION",
        name: "compensate a failed step",
        family: "effects",
        slots: &["fn_name", "compensate_fn"],
        required_slots: &["fn_name", "compensate_fn"],
        template: "// EXT-COMPENSATION: compensate a failed step\npub fn {fn_name}<F: Fn()>(f: F) { f(); }\npub fn {compensate_fn}() {}\n",
    },
    // === Expressions (1) ===
    Pattern {
        id: "EXP-EVAL",
        name: "evaluate a name, operator, or conversion",
        family: "expressions",
        slots: &["fn_name", "input_type", "output_type"],
        required_slots: &["fn_name"],
        template: "// EXP-EVAL: evaluate an expression\npub fn {fn_name}(x: i64, y: i64) -> i64 { x + y }\n",
    },
    // === Failure (6) ===
    Pattern {
        id: "FAIL-RESULT",
        name: "construct or propagate tagged errors",
        family: "failure",
        slots: &["ok_type", "err_type"],
        required_slots: &[],
        template: "// FAIL-RESULT: construct or propagate tagged errors\npub enum Result<T, E> { Ok(T), Err(E) }\n",
    },
    Pattern {
        id: "FAIL-EXCEPTION",
        name: "raise or handle a nonlocal error",
        family: "failure",
        slots: &["fn_name", "result_type", "error_type"],
        required_slots: &["fn_name"],
        template: "// FAIL-EXCEPTION: raise or handle a nonlocal error\npub fn {fn_name}<F: Fn() -> Result<(), String>>(f: F) -> Result<(), String> { f() }\n",
    },
    Pattern {
        id: "FAIL-RETRY",
        name: "repeat eligible failed attempts",
        family: "failure",
        slots: &["fn_name", "max_type"],
        required_slots: &["fn_name"],
        template: "// FAIL-RETRY: retry on failure\npub fn {fn_name}<F, T, E>(mut f: F, max: u32) -> Result<T, E>\nwhere F: FnMut() -> Result<T, E>\n{\n    let mut last = None;\n    for _ in 0..max { match f() { Ok(v) => return Ok(v), Err(e) => last = Some(e) } }\n    Err(last.unwrap())\n}\n",
    },
    Pattern {
        id: "FAIL-DEADLINE",
        name: "bound an operation by elapsed time",
        family: "failure",
        slots: &["fn_name", "body_type", "duration_type"],
        required_slots: &["fn_name"],
        template: "// FAIL-DEADLINE: bound by elapsed time\npub fn {fn_name}<F: FnOnce()>(f: F, _ms: u64) { f(); }\n",
    },
    Pattern {
        id: "FAIL-CANCEL",
        name: "cancel one declared scope",
        family: "failure",
        slots: &["fn_name", "flag_type"],
        required_slots: &["fn_name"],
        template: "// FAIL-CANCEL: cancel one scope\npub fn {fn_name}(flag: &mut bool) { *flag = true; }\n",
    },
    Pattern {
        id: "FAIL-CLEANUP",
        name: "clean up after failure",
        family: "failure",
        slots: &["fn_name", "cleanup_fn"],
        required_slots: &["fn_name", "cleanup_fn"],
        template: "// FAIL-CLEANUP: clean up after failure\npub fn {fn_name}<F: Fn()>(f: F) { f(); }\npub fn {cleanup_fn}() {}\n",
    },
    // === Functions (2) ===
    Pattern {
        id: "FN-OPERATION",
        name: "expose one callable responsibility",
        family: "functions",
        slots: &["fn_name", "input_type", "output_type"],
        required_slots: &["fn_name"],
        template: "// FN-OPERATION: expose one callable responsibility\npub fn {fn_name}(input: i64) -> i64 { input }\n",
    },
    Pattern {
        id: "FN-CALLBACK",
        name: "call an operation supplied as a value",
        family: "functions",
        slots: &["fn_name", "callback_type"],
        required_slots: &["fn_name"],
        template: "// FN-CALLBACK: call an operation supplied as a value\npub fn {fn_name}<F: Fn(i64) -> i64>(f: F, x: i64) -> i64 { f(x) }\n",
    },
    // === Organization (1) ===
    Pattern {
        id: "ORG-MODULE",
        name: "expose definitions and initialize dependencies",
        family: "organization",
        slots: &["module_name"],
        required_slots: &["module_name"],
        template: "// ORG-MODULE: expose definitions\npub mod {module_name} { pub fn init() {} }\n",
    },
    // === State (2) ===
    Pattern {
        id: "STATE-MACHINE",
        name: "control observable state transitions",
        family: "state",
        slots: &["enum_name", "fn_name", "state_type"],
        required_slots: &["enum_name", "fn_name"],
        template: "// STATE-MACHINE: control state transitions\npub enum {enum_name} { Idle, Done }\npub fn {fn_name}(s: {enum_name}, e: &str) -> {enum_name} {\n    match (s, e) { ({enum_name}::Idle, \"finish\") => {enum_name}::Done, (s, _) => s }\n}\n",
    },
    Pattern {
        id: "STATE-RESOURCE",
        name: "acquire, own, and release one resource",
        family: "state",
        slots: &["acquire_fn", "release_fn", "handle_type"],
        required_slots: &["acquire_fn", "release_fn"],
        template: "// STATE-RESOURCE: acquire and release\npub fn {acquire_fn}() -> i64 { 0 }\npub fn {release_fn}(handle: i64) {}\n",
    },
    // === Values (4) ===
    Pattern {
        id: "VAL-LITERAL",
        name: "admit one value",
        family: "values",
        slots: &["fn_name", "type_name", "param_name"],
        required_slots: &["fn_name"],
        template: "// VAL-LITERAL: admit one value\npub fn {fn_name}<{type_name}>(v: {type_name}) -> {type_name} { v }\n",
    },
    Pattern {
        id: "VAL-RECORD",
        name: "construct named fields",
        family: "values",
        slots: &["struct_name", "field1", "field2"],
        required_slots: &["struct_name"],
        template: "// VAL-RECORD: construct named fields\npub struct {struct_name} { pub {field1}: i64, pub {field2}: i64 }\n",
    },
    Pattern {
        id: "VAL-COLLECTION",
        name: "construct finite ordered or unique values",
        family: "values",
        slots: &["fn_name", "item_type"],
        required_slots: &["fn_name"],
        template: "// VAL-COLLECTION: construct collection\npub fn {fn_name}<T>(items: Vec<T>) -> Vec<T> { items }\n",
    },
    Pattern {
        id: "VAL-VARIANT",
        name: "construct a tagged, optional, or nullable value",
        family: "values",
        slots: &["enum_name", "variant1", "variant2"],
        required_slots: &["enum_name"],
        template: "// VAL-VARIANT: construct a tagged value\npub enum {enum_name} { {variant1}(i64), {variant2}(String) }\n",
    },
    // === Concurrency (2) ===
    // === Abstraction (additional) ===
];

pub fn list_patterns() -> &'static [Pattern] {
    PATTERNS
}

pub fn find_pattern(id: &str) -> Option<&'static Pattern> {
    PATTERNS.iter().find(|p| p.id == id)
}

pub fn run_list() -> i32 {
    println!("Available patterns ({}):", PATTERNS.len());
    println!();
    let mut families: BTreeMap<&str, Vec<&Pattern>> = BTreeMap::new();
    for p in PATTERNS {
        families.entry(p.family).or_default().push(p);
    }
    for (family, patterns) in &families {
        println!("  {}:", family);
        for p in patterns {
            println!("    {} - {} (slots: {})", p.id, p.name, p.slots.join(", "));
        }
    }
    0
}

pub fn run_gen(pattern_id: &str, output: &Path) -> i32 {
    let pattern = match find_pattern(pattern_id) {
        Some(p) => p,
        None => {
            eprintln!("ERROR: unknown pattern: {}", pattern_id);
            eprintln!(
                "Available: {}",
                PATTERNS.iter().map(|p| p.id).collect::<Vec<_>>().join(", ")
            );
            return 1;
        }
    };

    if let Some(parent) = output.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match std::fs::write(output, pattern.template) {
        Ok(_) => {
            println!("Generated: {}", output.display());
            println!("Pattern: {} ({})", pattern.id, pattern.name);
            println!("Slots: {}", pattern.slots.join(", "));
            0
        }
        Err(e) => {
            eprintln!("ERROR: cannot write {}: {}", output.display(), e);
            2
        }
    }
}

pub fn run_gen_with_slots(pattern_id: &str, slots_json: &str, output: &Path) -> i32 {
    let pattern = match find_pattern(pattern_id) {
        Some(p) => p,
        None => {
            eprintln!("ERROR: unknown pattern: {}", pattern_id);
            eprintln!(
                "Available: {}",
                PATTERNS.iter().map(|p| p.id).collect::<Vec<_>>().join(", ")
            );
            return 1;
        }
    };

    // Validate required slots
    let slots: std::collections::BTreeMap<String, String> = match serde_json::from_str(slots_json) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ERROR: invalid slots JSON: {}", e);
            return 2;
        }
    };

    for req in pattern.required_slots {
        if !slots.contains_key(*req) {
            eprintln!("ERROR: required slot '{}' not provided.", req);
            eprintln!("Required: {:?}", pattern.required_slots);
            return 2;
        }
    }

    // Fill slots in template
    let mut code = pattern.template.to_owned();
    for (key, value) in &slots {
        let placeholder = format!("{{{}}}", key);
        code = code.replace(&placeholder, value);
    }

    // Report unfilled optional slots
    for slot in pattern.slots {
        let placeholder = format!("{{{}}}", slot);
        if code.contains(&placeholder) {
            eprintln!("WARNING: optional slot '{}' not provided.", slot);
        }
    }

    if let Some(parent) = output.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match std::fs::write(output, code) {
        Ok(_) => {
            println!("Generated: {}", output.display());
            println!("Pattern: {} ({})", pattern.id, pattern.name);
            println!("Filled slots: {}", slots.len());
            0
        }
        Err(e) => {
            eprintln!("ERROR: cannot write {}: {}", output.display(), e);
            2
        }
    }
}
