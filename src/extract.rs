use std::fs;
use std::path::Path;

fn valid_id(identifier: &str) -> bool {
    let mut parts = identifier.split('-').collect::<Vec<_>>();
    if parts.len() < 2 {
        return false;
    }
    let sequence = parts.pop().unwrap();
    if sequence.len() != 3 || !sequence.bytes().all(|b| b.is_ascii_digit()) {
        return false;
    }
    parts.iter().all(|part| {
        let mut bytes = part.bytes();
        matches!(bytes.next(), Some(b'A'..=b'Z'))
            && bytes.all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
    })
}

/// Extract a contract schema from a Rust source file containing a trait.
pub fn extract_rust(content: &str) -> crate::codegen::ContractSchema {
    let mut name = String::from("Extracted");
    let mut operations = Vec::new();
    let mut clauses = Vec::new();
    let mut clause_counter = 0u32;

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    let mut in_trait = false;
    let mut pending_docs: Vec<String> = Vec::new();

    while i < lines.len() {
        let line = lines[i].trim();

        // Detect trait definition
        if line.starts_with("pub trait ") || line.starts_with("trait ") {
            let rest = line
                .trim_start_matches("pub ")
                .trim_start_matches("trait ")
                .trim();
            let tn = rest
                .split_whitespace()
                .next()
                .unwrap_or("Trait")
                .trim_end_matches('{')
                .to_owned();
            if !tn.is_empty() {
                name = tn;
            }
            in_trait = true;
            i += 1;
            continue;
        }

        if in_trait && line == "}" {
            in_trait = false;
            i += 1;
            continue;
        }

        if !in_trait {
            i += 1;
            continue;
        }

        // Collect doc comments
        if line.starts_with("///") {
            pending_docs.push(line.trim_start_matches("///").trim().to_owned());
            i += 1;
            continue;
        }

        // Detect method signature: fn name(params) -> ReturnType;
        if line.starts_with("fn ") {
            let sig = line.trim_end_matches(';').trim();
            let sig = sig.trim_start_matches("fn ").trim();
            // Parse name(params) -> returns
            if let Some(paren) = sig.find('(') {
                let method_name = sig[..paren].trim().to_owned();
                let rest = &sig[paren..];
                let (params, returns) = if let Some(arrow) = rest.find("->") {
                    let raw = rest[..arrow]
                        .trim()
                        .trim_matches(|c| c == '(' || c == ')')
                        .trim()
                        .to_owned();
                    let params = raw
                        .trim_start_matches("&self,")
                        .trim_start_matches("&self ,")
                        .trim()
                        .to_owned();
                    let params = if params == "&self" {
                        String::new()
                    } else {
                        params
                    };
                    let ret = rest[arrow + 2..].trim().trim_matches(';').trim().to_owned();
                    (params, ret)
                } else {
                    let raw = rest
                        .trim_matches(|c| c == '(' || c == ')')
                        .trim()
                        .to_owned();
                    let params = raw
                        .trim_start_matches("&self,")
                        .trim_start_matches("&self ,")
                        .trim()
                        .to_owned();
                    let params = if params == "&self" {
                        String::new()
                    } else {
                        params
                    };
                    (params, "()".to_owned())
                };

                // Create clauses from pending docs
                let mut clause_ids = Vec::new();
                for doc in &pending_docs {
                    // Try to extract clause ID from doc comment (e.g., "YOCTO-001: Queue must...")
                    let (id, obligation, actor) = if let Some(colon) = doc.find(": ") {
                        let possible_id = &doc[..colon];
                        if valid_id(possible_id) {
                            let rest = doc[colon + 2..].trim();
                            // Extract actor (first word) and obligation (rest)
                            let mut parts = rest.splitn(2, ' ');
                            let actor = parts.next().unwrap_or(&name).to_owned();
                            let obligation = parts.next().unwrap_or(rest).to_owned();
                            (possible_id.to_owned(), obligation, actor)
                        } else {
                            clause_counter += 1;
                            let id = format!(
                                "{}-{:03}",
                                name.to_uppercase().replace(' ', ""),
                                clause_counter
                            );
                            (id, doc.clone(), name.clone())
                        }
                    } else {
                        clause_counter += 1;
                        let id = format!(
                            "{}-{:03}",
                            name.to_uppercase().replace(' ', ""),
                            clause_counter
                        );
                        (id, doc.clone(), name.clone())
                    };
                    clauses.push(crate::codegen::Clause {
                        id: id.clone(),
                        actor,
                        obligation,
                    });
                    clause_ids.push(id);
                }

                operations.push(crate::codegen::Operation {
                    name: method_name,
                    params,
                    returns,
                    clauses: clause_ids,
                });
            }
            pending_docs.clear();
        }

        i += 1;
    }

    crate::codegen::ContractSchema {
        name,
        version: "0.1.0".to_owned(),
        description: "Extracted from Rust source.".to_owned(),
        terms: std::collections::BTreeMap::new(),
        operations,
        clauses,
        cases: Vec::new(),
        exclusions: Vec::new(),
        extends: Vec::new(),
    }
}

pub fn run_extract(input: &Path, output: &Path) -> i32 {
    let content = match fs::read_to_string(input) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("ERROR: cannot read {}: {}", input.display(), e);
            return 2;
        }
    };

    let schema = extract_rust(&content);
    let json = match serde_json::to_string_pretty(&schema) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("ERROR: cannot serialize: {}", e);
            return 2;
        }
    };

    if let Some(parent) = output.parent() {
        let _ = fs::create_dir_all(parent);
    }
    match fs::write(output, json) {
        Ok(_) => {
            println!("Extracted: {}", output.display());
            println!(
                "Operations: {}, Clauses: {}",
                schema.operations.len(),
                schema.clauses.len()
            );
            0
        }
        Err(e) => {
            eprintln!("ERROR: cannot write {}: {}", output.display(), e);
            2
        }
    }
}
