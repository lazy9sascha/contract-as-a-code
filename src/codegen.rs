use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Clause {
    pub id: String,
    pub actor: String,
    pub obligation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub name: String,
    pub input: String,
    pub expected: String,
    #[serde(default)]
    pub clauses: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractSchema {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(
        default,
        deserialize_with = "unique_terms",
        skip_serializing_if = "std::collections::BTreeMap::is_empty"
    )]
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

// A repeated term key is ambiguous input, even when both definitions match.
fn unique_terms<'de, D>(
    deserializer: D,
) -> Result<std::collections::BTreeMap<String, String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct TermsVisitor;
    impl<'de> serde::de::Visitor<'de> for TermsVisitor {
        type Value = std::collections::BTreeMap<String, String>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("an object with unique term names")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            let mut terms = Self::Value::new();
            while let Some((name, definition)) = map.next_entry::<String, String>()? {
                if terms.insert(name.clone(), definition).is_some() {
                    return Err(serde::de::Error::custom(format!(
                        "duplicate term definition: {name}"
                    )));
                }
            }
            Ok(terms)
        }
    }
    deserializer.deserialize_map(TermsVisitor)
}

fn default_version() -> String {
    "0.1.0".to_owned()
}

/// Parse a contract file (JSON schema or Markdown) and extract structure.
pub fn parse_json(content: &str) -> Result<ContractSchema, String> {
    serde_json::from_str(content).map_err(|e| format!("JSON parse error: {}", e))
}

/// Generate a Rust trait from a contract schema.
pub fn generate_rust_trait(schema: &ContractSchema) -> Result<String, String> {
    validate_schema(schema)?;
    let trait_name: String = schema
        .name
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .collect();
    let mut out = format!("// caac-schema-v1:{}\n", schema_hex(schema));
    out.push_str(&format!(
        "/// Generated from contract: {}\n",
        comment_text(&schema.name)
    ));
    out.push_str(&format!("/// Version: {}\n", comment_text(&schema.version)));
    if !schema.description.is_empty() {
        out.push_str(&format!("/// {}\n", comment_text(&schema.description)));
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
            out.push_str(&format!(
                "    /// {}: {} {}\n",
                c.id,
                comment_text(&c.actor),
                comment_text(&c.obligation)
            ));
        }
        out.push_str(&format!("    {};\n\n", operation_signature(op)?));
    }

    out.push_str("}\n");
    Ok(out)
}

/// Generate a Markdown contract from a JSON schema.
pub fn generate_markdown(schema: &ContractSchema) -> Result<String, String> {
    validate_schema(schema)?;
    let mut out = format!("<!-- caac-schema-v1:{} -->\n", schema_hex(schema));
    out.push_str(&format!("---\ntitle: {}\ndoc_type: \"contract\"\naudience: \"users\"\nstatus: \"maintained\"\npublication: \"review-required\"\n---\n\n", serde_json::to_string(&schema.name).expect("string serialization")));
    out.push_str(&format!("# {}\n\n", markdown_text(&schema.name)));
    out.push_str(&format!(
        "Revision: {}. Status: draft. Owner acceptance: pending.\n\n",
        markdown_text(&schema.version)
    ));

    if !schema.description.is_empty() {
        out.push_str("## Description (non-normative)\n\n");
        for line in schema.description.lines() {
            out.push_str(&format!("> {}\n", markdown_text(line)));
        }
        out.push('\n');
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
            out.push_str(&format!("An `{}` is {} \n\n", k, markdown_text(&def)));
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
    let mut emitted = std::collections::HashSet::new();
    for op in &schema.operations {
        out.push_str(&format!("### {}\n\n", op.name));
        for cid in &op.clauses {
            if !emitted.insert(cid) {
                out.push_str(&format!("Applies: @{}\n", cid));
                continue;
            }
            if let Some(c) = schema.clauses.iter().find(|cl| cl.id == *cid) {
                out.push_str(&format!(
                    "{} | {} {}\n",
                    c.id,
                    markdown_text(&c.actor),
                    markdown_text(&c.obligation)
                ));
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
            out.push_str(&format!(
                "{} | {} {}\n",
                c.id,
                markdown_text(&c.actor),
                markdown_text(&c.obligation)
            ));
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
                markdown_cell(&case.name),
                markdown_cell(&case.input),
                markdown_cell(&case.expected),
                markdown_cell(&case.clauses.join(", "))
            ));
        }
        out.push('\n');
    }

    if !schema.extends.is_empty() {
        out.push_str("## Extends\n\n");
        for base in &schema.extends {
            out.push_str(&format!("- {}\n", markdown_text(base)));
        }
        out.push('\n');
    }

    // Exclusions
    if !schema.exclusions.is_empty() {
        out.push_str("## Exclusions\n\n");
        for e in &schema.exclusions {
            out.push_str(&format!("- {}\n", markdown_text(e)));
        }
        out.push('\n');
    }

    out.push_str(
        "## Verification status (non-normative)\n\nThese cases describe expected results. They are not records of executed tests.\n",
    );
    Ok(out)
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

pub fn merge_schemas(schemas: &[ContractSchema]) -> Result<ContractSchema, String> {
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
        if s.name != merged.name
            || s.version != merged.version
            || s.description != merged.description
        {
            return Err(
                "schema name/version/description conflicts; reconcile before merging".into(),
            );
        }
        for (k, v) in &s.terms {
            if merged.terms.get(k).is_some_and(|old| old != v) {
                return Err(format!("conflicting term: {k}"));
            }
            merged.terms.insert(k.clone(), v.clone());
        }
        merged.operations.extend(s.operations.clone());
        merged.clauses.extend(s.clauses.clone());
        merged.cases.extend(s.cases.clone());
        merged.exclusions.extend(s.exclusions.clone());
        merged.extends.extend(s.extends.clone());
    }
    validate_schema(&merged)?;
    Ok(merged)
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
    let schema = match merge_schemas(&schemas) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ERROR: {e}");
            return 1;
        }
    };

    if schema.operations.is_empty() {
        eprintln!("ERROR: no operations found in inputs");
        return 1;
    }

    let trait_code = match generate_rust_trait(&schema) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("ERROR: {e}");
            return 1;
        }
    };

    match crate::output::write_new(output, trait_code.as_bytes()) {
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
    let schema = match merge_schemas(&schemas) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ERROR: {e}");
            return 1;
        }
    };
    let md = match generate_markdown(&schema) {
        Ok(md) => md,
        Err(e) => {
            eprintln!("ERROR: {e}");
            return 1;
        }
    };

    match crate::output::write_new(output, md.as_bytes()) {
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

fn comment_text(text: &str) -> String {
    text.replace('\r', " ").replace('\n', "\n/// ")
}

fn schema_hex(schema: &ContractSchema) -> String {
    serde_json::to_vec(schema)
        .expect("schema serialization")
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Decode versioned, inert schema metadata. Callers must also verify the derived body.
pub fn embedded_schema(
    content: &str,
    prefix: &str,
    suffix: &str,
) -> Result<Option<ContractSchema>, String> {
    let first = content.lines().next().unwrap_or("");
    let Some(rest) = first.strip_prefix(prefix) else {
        if content.contains("caac-schema-") {
            return Err("unsupported or misplaced CAAC metadata".into());
        }
        return Ok(None);
    };
    let hex = rest.strip_suffix(suffix).ok_or("malformed CAAC metadata")?;
    if !hex.len().is_multiple_of(2) || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("invalid CAAC metadata encoding".into());
    }
    let bytes: Result<Vec<_>, _> = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
        .collect();
    let bytes = bytes.map_err(|_| "invalid metadata bytes")?;
    let schema =
        serde_json::from_slice(&bytes).map_err(|e| format!("invalid schema metadata: {e}"))?;
    Ok(Some(schema))
}

pub(crate) fn identifier(s: &str) -> bool {
    s.len() <= 4096
        && !matches!(s, "gen" | "union")
        && s.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
        && syn::parse_str::<syn::Ident>(s).is_ok()
}

// Bound parser work before parsing. This is a complexity budget, not a grammar.
fn bounded_rust(s: &str) -> bool {
    s.len() <= 4096
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_ :,&<>()".contains(&c))
        && s.bytes().filter(|c| b"<&(".contains(c)).count() <= 64
}

pub(crate) fn supported_type(s: &str) -> bool {
    bounded_rust(s) && syn::parse_str::<syn::Type>(s).is_ok_and(|ty| type_borrows(&ty).is_some())
}

// Admit the declared type profile and count elided reference lifetimes together.
fn type_borrows(ty: &syn::Type) -> Option<usize> {
    match ty {
        syn::Type::Tuple(t) if t.elems.is_empty() => Some(0),
        syn::Type::Reference(r) if r.lifetime.is_none() => Some(1 + type_borrows(&r.elem)?),
        syn::Type::Path(p) if p.qself.is_none() && p.path.is_ident("Self") => Some(0),
        syn::Type::Path(p) if p.qself.is_none() && p.path.leading_colon.is_none() => {
            let mut borrows = 0;
            for (i, segment) in p.path.segments.iter().enumerate() {
                let name = segment.ident.to_string();
                if !identifier(&name) {
                    return None;
                }
                match &segment.arguments {
                    syn::PathArguments::None => {}
                    syn::PathArguments::AngleBracketed(a)
                        if i + 1 == p.path.segments.len()
                            && a.colon2_token.is_none()
                            && !a.args.is_empty() =>
                    {
                        for arg in &a.args {
                            let syn::GenericArgument::Type(ty) = arg else {
                                return None;
                            };
                            borrows += type_borrows(ty)?;
                        }
                    }
                    _ => return None,
                }
            }
            Some(borrows)
        }
        _ => None,
    }
}

pub(crate) fn validate_operation(op: &Operation) -> Result<(), String> {
    operation_signature(op).map(|_| ())
}

fn operation_signature(op: &Operation) -> Result<String, String> {
    if !identifier(&op.name) || !bounded_rust(&op.params) || !bounded_rust(&op.returns) {
        return Err(format!("unsupported signature in {}", op.name));
    }
    let returns = if op.returns.is_empty() || op.returns == "()" {
        String::new()
    } else {
        format!(" -> {}", op.returns)
    };
    let signature =
        syn::parse_str::<syn::Signature>(&format!("fn {}({}){}", op.name, op.params, returns))
            .map_err(|e| format!("invalid signature in {}: {e}", op.name))?;
    if signature.generics.where_clause.is_some() {
        return Err("where clauses are unsupported".into());
    }
    let mut names = std::collections::HashSet::new();
    let mut receiver = false;
    let mut borrowed_receiver = false;
    let mut input_borrows = 0;
    for (i, arg) in signature.inputs.iter().enumerate() {
        match arg {
            syn::FnArg::Receiver(r) => {
                if i != 0 || !r.attrs.is_empty() || r.mutability.is_some() {
                    return Err("unsupported receiver".into());
                }
                borrowed_receiver = match &r.kind {
                    syn::ReceiverKind::Value => false,
                    syn::ReceiverKind::Reference(_, None, _) => true,
                    _ => return Err("unsupported receiver".into()),
                };
                receiver = true;
            }
            syn::FnArg::Typed(p) => {
                let syn::Pat::Ident(name) = &*p.pat else {
                    return Err("parameters must be named and typed".into());
                };
                if !p.attrs.is_empty()
                    || !name.attrs.is_empty()
                    || name.by_ref.is_some()
                    || name.mutability.is_some()
                    || name.subpat.is_some()
                    || !identifier(&name.ident.to_string())
                    || !names.insert(name.ident.to_string())
                {
                    return Err("unsupported or duplicate parameter".into());
                }
                input_borrows += type_borrows(&p.ty).ok_or("unsupported parameter type")?;
            }
        }
    }
    let output_borrows = match &signature.output {
        syn::ReturnType::Default => 0,
        syn::ReturnType::Type(_, ty) => type_borrows(ty).ok_or("unsupported return type")?,
    };
    if receiver && !borrowed_receiver && output_borrows > 0 && input_borrows != 1 {
        return Err(format!(
            "borrowed return in {} requires a borrowed receiver or exactly one input lifetime",
            op.name
        ));
    }
    let params = if receiver {
        op.params.clone()
    } else if op.params.trim().is_empty() {
        "&self".into()
    } else {
        format!("&self, {}", op.params)
    };
    Ok(format!("fn {}({}){}", op.name, params, returns))
}

pub(crate) fn validate_clause_references(clauses: &[Clause]) -> Result<(), String> {
    let ids: std::collections::HashSet<_> =
        clauses.iter().map(|clause| clause.id.as_str()).collect();
    for clause in clauses {
        let body = format!("{} {}", clause.actor, clause.obligation);
        for id in crate::checker::find_references(&body) {
            if !ids.contains(id.as_str()) {
                return Err(format!(
                    "clause {} references unknown clause {id}",
                    clause.id
                ));
            }
        }
    }
    Ok(())
}

pub fn validate_schema(schema: &ContractSchema) -> Result<(), String> {
    for (label, text) in [
        ("name", schema.name.as_str()),
        ("version", schema.version.as_str()),
    ] {
        single_line(label, text)?;
    }
    for (name, definition) in &schema.terms {
        single_line("term name", name)?;
        single_line("term definition", definition)?;
        if name.contains(['`', '<', '>']) {
            return Err("term names cannot contain backticks or HTML delimiters".into());
        }
    }
    for text in schema.exclusions.iter().chain(&schema.extends) {
        single_line("exclusion/inheritance", text)?;
    }

    let trait_name: String = schema
        .name
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .collect();
    if !identifier(&trait_name) {
        return Err("name must produce a valid Rust identifier".into());
    }
    let mut ids = std::collections::HashSet::new();
    for c in &schema.clauses {
        single_line("clause ID", &c.id)?;
        single_line("actor", &c.actor)?;
        single_line("obligation", &c.obligation)?;
        let visible = format!("{} | {} {}", c.id, c.actor, c.obligation);
        let parsed = crate::parse::parse_clause_line(&visible)
            .ok_or_else(|| format!("invalid actor or obligation in clause {}", c.id))?;
        if parsed.actor != c.actor.trim() || parsed.obligation != c.obligation.trim() {
            return Err(format!("ambiguous actor or obligation in clause {}", c.id));
        }
        let rendered = format!(
            "{} {}",
            markdown_text(&c.actor),
            markdown_text(&c.obligation)
        );
        if crate::checker::find_references(&visible) != crate::checker::find_references(&rendered) {
            return Err(format!(
                "Markdown escaping changes reference roles in clause {}",
                c.id
            ));
        }
        if !valid_clause_id(&c.id) || !ids.insert(c.id.as_str()) {
            return Err(format!("duplicate or empty clause ID: {}", c.id));
        }
    }
    validate_clause_references(&schema.clauses)?;
    let mut ops = std::collections::HashSet::new();
    for op in &schema.operations {
        validate_operation(op)?;
        if op.name == "Invariants" {
            return Err("Invariants is reserved for standalone clauses".into());
        }
        if !ops.insert(&op.name) {
            return Err(format!("duplicate operation: {}", op.name));
        }
        for id in &op.clauses {
            if !ids.contains(id.as_str()) {
                return Err(format!("undefined clause: {id}"));
            }
        }
    }
    for case in &schema.cases {
        for id in &case.clauses {
            if !ids.contains(id.as_str()) {
                return Err(format!("undefined case clause: {id}"));
            }
        }
    }
    Ok(())
}

fn valid_clause_id(id: &str) -> bool {
    crate::checker::valid_id(id)
}

fn markdown_cell(value: &str) -> String {
    value
        .lines()
        .map(markdown_text)
        .collect::<Vec<_>>()
        .join("<br>")
}

fn single_line(label: &str, value: &str) -> Result<(), String> {
    if value.chars().any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}' | '\u{200b}'..='\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2060}'..='\u{206f}' | '\u{feff}')) {
        return Err(format!("{label} must be a single line without control or formatting characters"));
    }
    Ok(())
}

fn markdown_text(value: &str) -> String {
    let mut escaped = String::new();
    for c in value.chars() {
        match c {
            '&' => escaped.push_str(r"\&"),
            '<' => escaped.push_str(r"\<"),
            '>' => escaped.push_str(r"\>"),
            '\\' | '`' | '*' | '_' | '[' | ']' | '!' | '#' | '~' | '|' => {
                escaped.push('\\');
                escaped.push(c);
            }
            c if c.is_control()
                || matches!(c, '\u{2028}' | '\u{2029}' | '\u{200b}'..='\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2060}'..='\u{206f}' | '\u{feff}') =>
            {
                escaped.push_str(&format!("[U+{:04X}]", c as u32))
            }
            c => escaped.push(c),
        }
    }
    escaped
}
