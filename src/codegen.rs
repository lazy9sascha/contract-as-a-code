use std::fs;
use std::path::Path;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Clause {
    pub id: String,
    pub actor: String,
    pub obligation: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Operation {
    pub name: String,
    pub params: String,
    #[serde(default = "default_returns")]
    pub returns: String,
    #[serde(default)]
    pub clauses: Vec<String>,
}

fn default_returns() -> String {
    "()".to_owned()
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Case {
    pub name: String,
    pub input: String,
    pub expected: String,
    #[serde(default)]
    pub clauses: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContractSchema {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub terms: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    pub operations: Vec<Operation>,
    #[serde(default)]
    pub clauses: Vec<Clause>,
    #[serde(default)]
    pub cases: Vec<Case>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclusions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extends: Vec<String>,
}

fn default_version() -> String {
    "0.1.0".to_owned()
}

/// Parse a contract file (JSON schema or Markdown) and extract structure.
pub fn parse_json(content: &str) -> Result<ContractSchema, String> {
    serde_json::from_str(content).map_err(|e| format!("JSON parse error: {}", e))
}

/// Generate a Rust trait from a contract schema.
pub fn generate_rust_trait(schema: &ContractSchema) -> String {
    let trait_name: String = schema
        .name
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .collect();
    let mut out = String::new();
    out.push_str(&format!("/// Generated from contract: {}\n", schema.name));
    out.push_str(&format!("/// Version: {}\n", schema.version));
    if !schema.description.is_empty() {
        out.push_str(&format!("/// {}\n", schema.description));
    }
    out.push_str("/// Do not edit manually. Regenerate with `caac gen`.\n");
    out.push_str(&format!("pub trait {} {{\n", trait_name));

    for op in &schema.operations {
        // Find clauses for this operation
        let op_clauses: Vec<&Clause> = schema
            .clauses
            .iter()
            .filter(|c| op.clauses.contains(&c.id))
            .collect();
        for c in &op_clauses {
            out.push_str(&format!("    /// {}: {} {}\n", c.id, c.actor, c.obligation));
        }
        let params = if op.params.is_empty() {
            "&self".to_owned()
        } else if op.params.trim_start().starts_with("self") {
            op.params.clone()
        } else {
            format!("&self, {}", op.params)
        };
        let ret = if op.returns == "()" || op.returns.is_empty() {
            String::new()
        } else {
            format!(" -> {}", op.returns)
        };
        out.push_str(&format!("    fn {}({}){};\n\n", op.name, params, ret));
    }

    out.push_str("}\n");
    out
}

/// Generate a Markdown contract from a JSON schema.
pub fn generate_markdown(schema: &ContractSchema) -> String {
    let mut out = String::new();
    out.push_str(&format!("---\ntitle: \"{}\"\ndoc_type: \"contract\"\naudience: \"users\"\nstatus: \"maintained\"\npublication: \"review-required\"\n---\n\n", schema.name));
    out.push_str(&format!("# {}\n\n", schema.name));
    out.push_str(&format!(
        "Revision: {}. Status: draft. Owner acceptance: pending.\n\n",
        schema.version
    ));

    if !schema.description.is_empty() {
        out.push_str(&format!("{}\n\n", schema.description));
    }

    // Terms
    if !schema.terms.is_empty() {
        out.push_str("## Terms and inputs\n\n");
        for (k, v) in &schema.terms {
            let def = if v.ends_with('.') {
                v.clone()
            } else {
                format!("{}.", v)
            };
            out.push_str(&format!("An `{}` is {} \n\n", k, def));
        }
    }

    // Operations
    if !schema.operations.is_empty() {
        out.push_str("## Operation signatures\n\n");
        for op in &schema.operations {
            out.push_str(&format!(
                "- `{}({})` -> `{}`\n",
                op.name, op.params, op.returns
            ));
        }
        out.push('\n');
    }

    // Clauses grouped by operation
    out.push_str("## Clauses\n\n");
    for op in &schema.operations {
        out.push_str(&format!("### {}\n\n", op.name));
        for cid in &op.clauses {
            if let Some(c) = schema.clauses.iter().find(|cl| cl.id == *cid) {
                out.push_str(&format!("{} | {} {}\n", c.id, c.actor, c.obligation));
            }
        }
        out.push('\n');
    }
    // Standalone clauses (not assigned to any operation)
    let op_clause_ids: std::collections::HashSet<&str> = schema
        .operations
        .iter()
        .flat_map(|o| o.clauses.iter().map(|s| s.as_str()))
        .collect();
    let standalone: Vec<&Clause> = schema
        .clauses
        .iter()
        .filter(|c| !op_clause_ids.contains(c.id.as_str()))
        .collect();
    if !standalone.is_empty() {
        out.push_str("### Invariants\n\n");
        for c in &standalone {
            out.push_str(&format!("{} | {} {}\n", c.id, c.actor, c.obligation));
        }
        out.push('\n');
    }

    // Cases
    if !schema.cases.is_empty() {
        out.push_str("## Contract cases\n\n");
        out.push_str("| Case | Input | Expected result | Clauses |\n");
        out.push_str("| --- | --- | --- | --- |\n");
        for case in &schema.cases {
            out.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                case.name,
                case.input,
                case.expected,
                case.clauses.join(", ")
            ));
        }
        out.push('\n');
    }

    // Exclusions
    if !schema.exclusions.is_empty() {
        out.push_str("## Exclusions\n\n");
        for e in &schema.exclusions {
            out.push_str(&format!("- {}\n", e));
        }
        out.push('\n');
    }

    out.push_str(
        "These cases describe expected results. They are not records of executed tests.\n",
    );
    out
}

pub fn load_schema(input: &Path) -> Result<ContractSchema, i32> {
    let content = match fs::read_to_string(input) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("ERROR: cannot read {}: {}", input.display(), e);
            return Err(2);
        }
    };
    match parse_json(&content) {
        Ok(s) => Ok(s),
        Err(e) => {
            eprintln!("ERROR: {}", e);
            Err(1)
        }
    }
}

pub fn merge_schemas(schemas: &[ContractSchema]) -> ContractSchema {
    let mut merged = ContractSchema {
        name: schemas.first().map(|s| s.name.clone()).unwrap_or_default(),
        version: schemas
            .first()
            .map(|s| s.version.clone())
            .unwrap_or_default(),
        description: schemas
            .first()
            .map(|s| s.description.clone())
            .unwrap_or_default(),
        terms: std::collections::BTreeMap::new(),
        operations: Vec::new(),
        clauses: Vec::new(),
        cases: Vec::new(),
        exclusions: Vec::new(),
        extends: Vec::new(),
    };
    for s in schemas {
        for (k, v) in &s.terms {
            merged.terms.insert(k.clone(), v.clone());
        }
        merged.operations.extend(s.operations.clone());
        merged.clauses.extend(s.clauses.clone());
        merged.cases.extend(s.cases.clone());
        merged.exclusions.extend(s.exclusions.clone());
    }
    merged
}

pub fn run_gen(inputs: &[&Path], output: &Path) -> i32 {
    let mut schemas = Vec::new();
    for input in inputs {
        match load_schema(input) {
            Ok(s) => schemas.push(s),
            Err(code) => return code,
        }
    }
    if schemas.is_empty() {
        eprintln!("ERROR: no input files");
        return 2;
    }
    let schema = merge_schemas(&schemas);

    if schema.operations.is_empty() {
        eprintln!("ERROR: no operations found in inputs");
        return 1;
    }

    let trait_code = generate_rust_trait(&schema);

    if let Some(parent) = output.parent() {
        let _ = fs::create_dir_all(parent);
    }
    match fs::write(output, trait_code) {
        Ok(_) => {
            println!("Generated: {}", output.display());
            println!("Operations: {}", schema.operations.len());
            for op in &schema.operations {
                println!("  - {}({}) -> {}", op.name, op.params, op.returns);
            }
            0
        }
        Err(e) => {
            eprintln!("ERROR: cannot write {}: {}", output.display(), e);
            2
        }
    }
}

pub fn run_md(inputs: &[&Path], output: &Path) -> i32 {
    let mut schemas = Vec::new();
    for input in inputs {
        match load_schema(input) {
            Ok(s) => schemas.push(s),
            Err(code) => return code,
        }
    }
    if schemas.is_empty() {
        eprintln!("ERROR: no input files");
        return 2;
    }
    let schema = merge_schemas(&schemas);
    let md = generate_markdown(&schema);

    if let Some(parent) = output.parent() {
        let _ = fs::create_dir_all(parent);
    }
    match fs::write(output, md) {
        Ok(_) => {
            println!("Generated: {}", output.display());
            0
        }
        Err(e) => {
            eprintln!("ERROR: cannot write {}: {}", output.display(), e);
            2
        }
    }
}
