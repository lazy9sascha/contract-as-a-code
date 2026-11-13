use caac::{author, checker};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

const ROOT_USAGE: &str = "Usage:\n  caac scratch OUTPUT --title TITLE\n  caac plugin OUTPUT --title TITLE --source FILE...\n  caac migration OUTPUT --title TITLE --source FILE...\n  caac check [--json] FILE...\n  caac watch [--interval SECS] FILE...\n  caac interview [--auto] [--suggest] [--source FILE] FILE...\n  caac resume FILE...\n  caac run [--source FILE] [--max-iterations N] FILE...\n  caac gen INPUT... --output FILE\n  caac md INPUT... --output FILE\n  caac parse INPUT --output FILE\n  caac extract INPUT --output FILE\n  caac sync INPUT --code FILE --docs FILE [--watch] [--interval SECS]\n  caac pattern list\n  caac pattern gen ID --output FILE\n  caac sync INPUT --code FILE --docs FILE [--watch] [--interval SECS]\n  caac --help";

pub fn run<I>(args: I) -> i32
where
    I: IntoIterator<Item = String>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    let Some((command, rest)) = args.split_first() else {
        return argument_error("A command is required.", ROOT_USAGE);
    };

    if matches!(command.as_str(), "--help" | "-h" | "help") {
        print_root_help();
        return 0;
    }

    match command.as_str() {
        "scratch" | "plugin" | "migration" => run_authoring(command, rest),
        "check" => run_check(rest),
        "watch" => run_watch(rest),
        "interview" => run_interview_cmd(rest),
        "resume" => run_resume_cmd(rest),
        "gen" => run_gen_cmd(rest),
        "md" => run_md_cmd(rest),
        "parse" => run_parse_cmd(rest),
        "extract" => run_extract_cmd(rest),
        "sync" => run_sync_cmd(rest),
        "pattern" => run_pattern_cmd(rest),
        "run" => run_run_cmd(rest),
        _ => argument_error(&format!("Unknown command: {command}"), ROOT_USAGE),
    }
}

fn run_authoring(mode: &str, args: &[String]) -> i32 {
    if args
        .iter()
        .any(|argument| matches!(argument.as_str(), "--help" | "-h"))
    {
        print_authoring_help(mode);
        return 0;
    }

    let (destination, title, sources) = match parse_authoring_args(mode, args) {
        Ok(parsed) => parsed,
        Err(message) => return argument_error(&message, &authoring_usage(mode)),
    };

    match author::create(mode, &destination, &title, &sources) {
        Ok(path) => {
            println!(
                "Created draft: {}. Resolve OPEN decisions before implementation.",
                path.display()
            );
            0
        }
        Err(message) => {
            eprintln!("ERROR: {message}");
            2
        }
    }
}

fn parse_authoring_args(
    mode: &str,
    args: &[String],
) -> Result<(PathBuf, String, Vec<PathBuf>), String> {
    let mut destination = None;
    let mut title = None;
    let mut sources = Vec::new();
    let mut index = 0;

    while index < args.len() {
        let argument = &args[index];
        if argument == "--title" {
            index += 1;
            title = Some(
                args.get(index)
                    .ok_or_else(|| "--title requires a value".to_owned())?
                    .clone(),
            );
        } else if let Some(value) = argument.strip_prefix("--title=") {
            title = Some(value.to_owned());
        } else if argument == "--source" {
            if mode == "scratch" {
                return Err("scratch does not accept --source".to_owned());
            }
            index += 1;
            let source = args
                .get(index)
                .ok_or_else(|| "--source requires a file path".to_owned())?;
            sources.push(PathBuf::from(source));
        } else if let Some(value) = argument.strip_prefix("--source=") {
            if mode == "scratch" {
                return Err("scratch does not accept --source".to_owned());
            }
            sources.push(PathBuf::from(value));
        } else if argument.starts_with('-') {
            return Err(format!("Unknown option: {argument}"));
        } else if destination.is_none() {
            destination = Some(PathBuf::from(argument));
        } else {
            return Err(format!("Unexpected argument: {argument}"));
        }
        index += 1;
    }

    let destination = destination.ok_or_else(|| "An output path is required".to_owned())?;
    let title = title.ok_or_else(|| "--title is required".to_owned())?;
    if mode != "scratch" && sources.is_empty() {
        return Err("At least one --source file is required".to_owned());
    }
    Ok((destination, title, sources))
}

fn run_watch(args: &[String]) -> i32 {
    let mut interval = 2u64;
    let mut files = Vec::new();
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--interval" {
            index += 1;
            interval = match args
                .get(index)
                .and_then(|v| v.parse::<u64>().ok())
                .filter(|v| *v > 0)
            {
                Some(value) => value,
                None => {
                    return argument_error("--interval requires a positive integer", ROOT_USAGE);
                }
            };
        } else if args[index].starts_with("--interval=") {
            interval = match args[index]
                .trim_start_matches("--interval=")
                .parse::<u64>()
                .ok()
                .filter(|v| *v > 0)
            {
                Some(value) => value,
                None => {
                    return argument_error("--interval requires a positive integer", ROOT_USAGE);
                }
            };
        } else if matches!(args[index].as_str(), "--help" | "-h") {
            println!(
                "Watch files and re-run syntax checks on change.\n\nUsage: caac watch [--interval SECS] FILE..."
            );
            return 0;
        } else {
            files.push(PathBuf::from(&args[index]));
        }
        index += 1;
    }

    if files.is_empty() {
        return argument_error(
            "At least one input file is required.",
            "Usage: caac watch [--interval SECS] FILE...",
        );
    }

    let mut last_mtimes: Vec<std::time::SystemTime> = files.iter().map(file_mtime).collect();
    let mut iteration = 0u64;

    loop {
        iteration += 1;
        println!("--- watch iteration {iteration} ---");
        match checker::check(&files) {
            Ok(diagnostics) => {
                for d in &diagnostics {
                    println!("{}:{}: {}: {}", d.path, d.line, d.code, d.message);
                }
                if diagnostics.is_empty() {
                    println!("PASS: declared syntax only. Semantic review pending.");
                } else {
                    println!(
                        "FAIL: {} finding(s). Repair and save to re-check.",
                        diagnostics.len()
                    );
                }
            }
            Err(message) => eprintln!("ERROR: {message}"),
        }

        // Poll for changes
        loop {
            std::thread::sleep(std::time::Duration::from_secs(interval));
            let current: Vec<std::time::SystemTime> = files.iter().map(file_mtime).collect();
            if current != last_mtimes {
                last_mtimes = current;
                break;
            }
        }
    }
}

fn file_mtime(path: &PathBuf) -> std::time::SystemTime {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
}

fn run_interview_cmd(args: &[String]) -> i32 {
    let mut auto = false;
    let mut suggest = false;
    let mut sources: Vec<String> = Vec::new();
    let mut files = Vec::new();
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--auto" {
            auto = true;
        } else if args[index] == "--suggest" {
            suggest = true;
        } else if args[index] == "--source" {
            index += 1;
            if let Some(src) = args.get(index) {
                sources.push(src.clone());
            }
        } else if matches!(args[index].as_str(), "--help" | "-h") {
            println!(
                "Interview: walk through open decisions in a contract.\n\nUsage: caac interview [--auto] [--suggest] [--source FILE] FILE..."
            );
            return 0;
        } else {
            files.push(PathBuf::from(&args[index]));
        }
        index += 1;
    }
    if files.is_empty() {
        return argument_error(
            "At least one input file is required.",
            "Usage: caac interview [--auto] [--suggest] [--source FILE] FILE...",
        );
    }

    let source_contents: Vec<String> = sources
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect();
    let source_refs: Vec<&str> = source_contents.iter().map(|s| s.as_str()).collect();

    let mut result = 0;
    for file in &files {
        if suggest {
            let content = std::fs::read_to_string(file).unwrap_or_default();
            let open_lines: Vec<&str> = content
                .lines()
                .filter(|l| l.trim().starts_with("OPEN:"))
                .collect();
            if !open_lines.is_empty() {
                println!("Suggestions for {}:", file.display());
                for ol in &open_lines {
                    let decision = ol.trim().trim_start_matches("OPEN:").trim();
                    if let Some(suggestion) =
                        caac::suggest::suggest_for_decision(decision, &source_refs)
                    {
                        println!("  Q: {}", decision);
                        println!("  Suggested: {}", suggestion);
                    }
                }
                println!();
            }
        }
        result = result.max(caac::interview::run_interview(file, auto));
    }
    result
}

fn run_resume_cmd(args: &[String]) -> i32 {
    if args.iter().any(|a| matches!(a.as_str(), "--help" | "-h")) {
        println!("Resume a previous interview.\n\nUsage: caac resume FILE...");
        return 0;
    }
    if args.is_empty() {
        return argument_error(
            "At least one input file is required.",
            "Usage: caac resume FILE...",
        );
    }
    let mut result = 0;
    for file in args {
        result = result.max(caac::interview::run_resume(&PathBuf::from(file)));
    }
    result
}

fn run_gen_cmd(args: &[String]) -> i32 {
    let mut inputs = Vec::new();
    let mut output = None;
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--output" || args[index] == "-o" {
            index += 1;
            output = args.get(index).map(PathBuf::from);
        } else if args[index].starts_with("--output=") {
            output = Some(PathBuf::from(args[index].trim_start_matches("--output=")));
        } else if matches!(args[index].as_str(), "--help" | "-h") {
            println!(
                "Generate Rust trait from contract schema(s).\n\nUsage: caac gen INPUT... --output FILE"
            );
            return 0;
        } else if args[index].starts_with('-') {
            return argument_error(
                &format!("Unknown option: {}", args[index]),
                "Usage: caac gen INPUT... --output FILE",
            );
        } else {
            inputs.push(PathBuf::from(&args[index]));
        }
        index += 1;
    }
    if inputs.is_empty() {
        return argument_error(
            "At least one input contract file is required.",
            "Usage: caac gen INPUT... --output FILE",
        );
    }
    let output = match output {
        Some(p) => p,
        None => {
            return argument_error(
                "--output is required.",
                "Usage: caac gen INPUT... --output FILE",
            );
        }
    };
    let input_refs: Vec<&Path> = inputs.iter().map(|p| p.as_path()).collect();
    caac::codegen::run_gen(&input_refs, &output)
}

fn run_md_cmd(args: &[String]) -> i32 {
    let mut inputs = Vec::new();
    let mut output = None;
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--output" || args[index] == "-o" {
            index += 1;
            output = args.get(index).map(PathBuf::from);
        } else if args[index].starts_with("--output=") {
            output = Some(PathBuf::from(args[index].trim_start_matches("--output=")));
        } else if matches!(args[index].as_str(), "--help" | "-h") {
            println!(
                "Generate Markdown contract from schema(s).\n\nUsage: caac md INPUT... --output FILE"
            );
            return 0;
        } else if args[index].starts_with('-') {
            return argument_error(
                &format!("Unknown option: {}", args[index]),
                "Usage: caac md INPUT... --output FILE",
            );
        } else {
            inputs.push(PathBuf::from(&args[index]));
        }
        index += 1;
    }
    if inputs.is_empty() {
        return argument_error(
            "At least one input contract file is required.",
            "Usage: caac md INPUT... --output FILE",
        );
    }
    let output = match output {
        Some(p) => p,
        None => {
            return argument_error(
                "--output is required.",
                "Usage: caac md INPUT... --output FILE",
            );
        }
    };
    let input_refs: Vec<&Path> = inputs.iter().map(|p| p.as_path()).collect();
    caac::codegen::run_md(&input_refs, &output)
}

fn run_parse_cmd(args: &[String]) -> i32 {
    let mut input = None;
    let mut output = None;
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--output" || args[index] == "-o" {
            index += 1;
            output = args.get(index).map(PathBuf::from);
        } else if args[index].starts_with("--output=") {
            output = Some(PathBuf::from(args[index].trim_start_matches("--output=")));
        } else if matches!(args[index].as_str(), "--help" | "-h") {
            println!(
                "Parse Markdown contract to JSON schema.\n\nUsage: caac parse INPUT --output FILE"
            );
            return 0;
        } else if input.is_none() {
            input = Some(PathBuf::from(&args[index]));
        } else {
            return argument_error(
                &format!("Unexpected argument: {}", args[index]),
                "Usage: caac parse INPUT --output FILE",
            );
        }
        index += 1;
    }
    let input = match input {
        Some(p) => p,
        None => {
            return argument_error(
                "An input contract file is required.",
                "Usage: caac parse INPUT --output FILE",
            );
        }
    };
    let output = match output {
        Some(p) => p,
        None => {
            return argument_error(
                "--output is required.",
                "Usage: caac parse INPUT --output FILE",
            );
        }
    };
    caac::parse::run_parse(&input, &output)
}

fn run_extract_cmd(args: &[String]) -> i32 {
    let mut input = None;
    let mut output = None;
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--output" || args[index] == "-o" {
            index += 1;
            output = args.get(index).map(PathBuf::from);
        } else if args[index].starts_with("--output=") {
            output = Some(PathBuf::from(args[index].trim_start_matches("--output=")));
        } else if matches!(args[index].as_str(), "--help" | "-h") {
            println!(
                "Recover schema from unchanged CAAC-generated Rust (metadata required).\n\nUsage: caac extract INPUT --output FILE"
            );
            return 0;
        } else if input.is_none() {
            input = Some(PathBuf::from(&args[index]));
        } else {
            return argument_error(
                &format!("Unexpected argument: {}", args[index]),
                "Usage: caac extract INPUT --output FILE",
            );
        }
        index += 1;
    }
    let input = match input {
        Some(p) => p,
        None => {
            return argument_error(
                "An input source file is required.",
                "Usage: caac extract INPUT --output FILE",
            );
        }
    };
    let output = match output {
        Some(p) => p,
        None => {
            return argument_error(
                "--output is required.",
                "Usage: caac extract INPUT --output FILE",
            );
        }
    };
    caac::extract::run_extract(&input, &output)
}

fn run_run_cmd(args: &[String]) -> i32 {
    let mut sources: Vec<String> = Vec::new();
    let mut max_iterations = 10u32;
    let mut files = Vec::new();
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--source" {
            index += 1;
            if let Some(src) = args.get(index) {
                sources.push(src.clone());
            }
        } else if args[index] == "--max-iterations" {
            index += 1;
            if let Some(n) = args.get(index) {
                max_iterations = n.parse().unwrap_or(10);
            }
        } else if matches!(args[index].as_str(), "--help" | "-h") {
            println!(
                "Run syntax and case-reference prechecks; semantic review remains pending.\n\nUsage: caac run [--source FILE] [--max-iterations N] FILE..."
            );
            return 0;
        } else if args[index].starts_with('-') {
            return argument_error(
                &format!("Unknown option: {}", args[index]),
                "Usage: caac run [--source FILE] [--max-iterations N] FILE...",
            );
        } else {
            files.push(PathBuf::from(&args[index]));
        }
        index += 1;
    }
    if files.is_empty() {
        return argument_error(
            "At least one input file is required.",
            "Usage: caac run [--source FILE] [--max-iterations N] FILE...",
        );
    }
    caac::harness::run_loop(&files, &sources, max_iterations)
}

fn run_pattern_cmd(args: &[String]) -> i32 {
    if args.is_empty() || matches!(args[0].as_str(), "--help" | "-h") {
        println!(
            "Pattern library commands.\n\nUsage: caac pattern list\n       caac pattern gen ID --output FILE"
        );
        return 0;
    }
    match args[0].as_str() {
        "list" => caac::patterns::run_list(),
        "gen" => {
            if args.len() < 2 {
                return argument_error(
                    "Pattern ID required.",
                    "Usage: caac pattern gen ID --output FILE",
                );
            }
            let id = &args[1];
            let mut slots = String::new();
            let mut output = None;
            let mut index = 2;
            while index < args.len() {
                if args[index] == "--slots" {
                    index += 1;
                    slots = args.get(index).cloned().unwrap_or_default();
                } else if args[index].starts_with("--slots=") {
                    slots = args[index].trim_start_matches("--slots=").to_owned();
                } else if args[index] == "--output" || args[index] == "-o" {
                    index += 1;
                    output = args.get(index).map(PathBuf::from);
                } else if args[index].starts_with("--output=") {
                    output = Some(PathBuf::from(args[index].trim_start_matches("--output=")));
                }
                index += 1;
            }
            let output = match output {
                Some(p) => p,
                None => {
                    return argument_error(
                        "--output is required.",
                        "Usage: caac pattern gen ID [--slots JSON] --output FILE",
                    );
                }
            };
            if slots.is_empty() {
                caac::patterns::run_gen(id, &output)
            } else {
                caac::patterns::run_gen_with_slots(id, &slots, &output)
            }
        }
        _ => argument_error(
            &format!("Unknown subcommand: {}", args[0]),
            "Usage: caac pattern list | gen ID --output FILE",
        ),
    }
}

fn run_sync_cmd(args: &[String]) -> i32 {
    let mut input = None;
    let mut output_code = None;
    let mut output_docs = None;
    let mut watch = false;
    let mut interval = 2u64;
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--code" || args[index] == "--output-code" {
            index += 1;
            output_code = args.get(index).map(PathBuf::from);
        } else if args[index] == "--docs" || args[index] == "--output-docs" {
            index += 1;
            output_docs = args.get(index).map(PathBuf::from);
        } else if args[index] == "--watch" {
            watch = true;
        } else if args[index] == "--interval" {
            index += 1;
            interval = match args
                .get(index)
                .and_then(|v| v.parse::<u64>().ok())
                .filter(|v| *v > 0)
            {
                Some(value) => value,
                None => {
                    return argument_error("--interval requires a positive integer", ROOT_USAGE);
                }
            };
        } else if matches!(args[index].as_str(), "--help" | "-h") {
            println!(
                "Generate code/docs at absent paths; watch updates only this session outputs.\n\nUsage: caac sync INPUT --code FILE --docs FILE [--watch] [--interval SECS]"
            );
            return 0;
        } else if args[index].starts_with('-') {
            return argument_error(
                &format!("Unknown option: {}", args[index]),
                "Usage: caac sync INPUT --code FILE --docs FILE [--watch] [--interval SECS]",
            );
        } else if input.is_none() {
            input = Some(PathBuf::from(&args[index]));
        } else {
            return argument_error(
                &format!("Unexpected argument: {}", args[index]),
                "Usage: caac sync INPUT --code FILE --docs FILE [--watch] [--interval SECS]",
            );
        }
        index += 1;
    }
    let input = match input {
        Some(p) => p,
        None => {
            return argument_error(
                "An input schema file is required.",
                "Usage: caac sync INPUT --code FILE --docs FILE [--watch] [--interval SECS]",
            );
        }
    };
    caac::sync::run_sync(
        &input,
        output_code.as_deref(),
        output_docs.as_deref(),
        watch,
        interval,
    )
}

fn run_check(args: &[String]) -> i32 {
    let mut json_output = false;
    let mut positional_only = false;
    let mut files = Vec::new();
    for argument in args {
        if positional_only {
            files.push(PathBuf::from(argument));
        } else if argument == "--" {
            positional_only = true;
        } else if matches!(argument.as_str(), "--help" | "-h") {
            print_check_help();
            return 0;
        } else if argument == "--json" {
            json_output = true;
        } else if argument.starts_with('-') {
            return argument_error(
                &format!("Unknown option: {argument}"),
                "Usage: caac check [--json] FILE...",
            );
        } else {
            files.push(PathBuf::from(argument));
        }
    }

    if files.is_empty() {
        return argument_error(
            "At least one input file is required.",
            "Usage: caac check [--json] FILE...",
        );
    }

    match checker::check(&files) {
        Ok(diagnostics) => {
            if json_output {
                let diagnostic_values = diagnostics
                    .iter()
                    .map(|diagnostic| {
                        json!({
                            "path": diagnostic.path.as_str(),
                            "line": diagnostic.line,
                            "code": diagnostic.code.as_str(),
                            "message": diagnostic.message.as_str(),
                        })
                    })
                    .collect::<Vec<_>>();
                print_json(&json!({
                    "syntax": if diagnostics.is_empty() { "passed" } else { "failed" },
                    "semantic_review": "pending",
                    "diagnostics": diagnostic_values,
                }));
            } else {
                for diagnostic in &diagnostics {
                    println!(
                        "{}:{}: {}: {}",
                        diagnostic.path, diagnostic.line, diagnostic.code, diagnostic.message
                    );
                }
                println!(
                    "{}: declared syntax only. Semantic review pending.",
                    if diagnostics.is_empty() {
                        "PASS"
                    } else {
                        "FAIL"
                    }
                );
            }
            if diagnostics.is_empty() { 0 } else { 1 }
        }
        Err(message) => {
            if json_output {
                print_json(&json!({
                    "error": message,
                    "semantic_review": "pending",
                }));
            } else {
                eprintln!("ERROR: {message}");
            }
            2
        }
    }
}

fn print_json(value: &Value) {
    match serde_json::to_string_pretty(value) {
        Ok(text) => println!("{text}"),
        Err(error) => {
            eprintln!("ERROR: Could not serialize the JSON report: {error}");
        }
    }
}

fn argument_error(message: &str, usage: &str) -> i32 {
    eprintln!("ERROR: {message}\n{usage}");
    2
}

fn authoring_usage(mode: &str) -> String {
    if mode == "scratch" {
        "Usage: caac scratch OUTPUT --title TITLE".to_owned()
    } else {
        format!("Usage: caac {mode} OUTPUT --title TITLE --source FILE...")
    }
}

fn print_root_help() {
    println!(
        "Contract authoring, declared syntax checks, repair watch, and interview.\n\n{ROOT_USAGE}"
    );
}

fn print_authoring_help(mode: &str) {
    println!(
        "Create an isolated draft contract.\n\n{}",
        authoring_usage(mode)
    );
    println!("  --title TITLE   Set the selected contract scope.");
    if mode != "scratch" {
        println!("  --source FILE   Select a source file. Repeat for more files.");
    }
}

fn print_check_help() {
    println!(
        "Check declared Markdown syntax. Semantic review remains separate.\n\n\
         Usage: caac check [--json] FILE...\n\n\
         --json   Print machine-readable diagnostics."
    );
}
