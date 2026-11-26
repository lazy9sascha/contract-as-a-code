use caac::{author, checker};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

const ROOT_USAGE: &str = "Usage:\n  caac review-findings --collection FILE --packet FILE --findings FILE --base FILE [--contract FILE]... [--run-input FILE]... [--root FILE | --root-assessment FILE] [--baseline FILE] [--crawk-modules FILE --crawk-deps FILE --crawk-snapshot FILE --crawk-source-root DIR] --output NEW_DIRECTORY\n  caac review-packet --collection FILE [--contract FILE]... [--run-input FILE]... [--root FILE | --root-assessment FILE] [--baseline FILE] [--crawk-modules FILE --crawk-deps FILE --crawk-snapshot FILE --crawk-source-root DIR] --output NEW_FILE\n  caac discover --directory DIR [--directory DIR...] [--depth 0..4]\n  caac collect [--contract FILE]... [--run-input FILE]... [--root FILE | --root-assessment FILE] [--baseline FILE] [--crawk-modules FILE --crawk-deps FILE --crawk-snapshot FILE --crawk-source-root DIR] --output NEW_FILE\n  caac scratch OUTPUT --title TITLE [--size XS|S|M]\n  caac plugin OUTPUT --title TITLE --source FILE... [--size XS|S|M]\n  caac migration OUTPUT --title TITLE --source FILE... [--size XS|S|M]\n  caac candidate schema\n  caac candidate apply --base DRAFT --answers ANSWERS --output NEW_DIRECTORY [--review-receipt RECEIPT --findings FILE --collection FILE --packet FILE ...]\n  caac candidate interview --base DRAFT --answers JOURNAL --output NEW_DIRECTORY [--resume]\n  caac candidate verify --candidate FILE --provenance FILE --review FILE [--trace ID]\n  caac candidate compare-review --candidate FILE --provenance FILE --review ORIGINAL --edited EDITED --output NEW_DIRECTORY\n  caac candidate revise --candidate FILE --provenance FILE --review ORIGINAL (--proposal PROPOSAL | --presentation REQUEST) --output NEW_DIRECTORY\n  caac baseline check BASELINE\n  caac check [--json] FILE...\n  caac lint [--json] FILE...\n  caac mcp\n  caac watch [--interval SECS] FILE...\n  caac interview [--auto] [--suggest] [--source FILE | --baseline FILE] FILE...\n  caac resume FILE...\n  caac run [--source FILE | --baseline FILE] [--max-iterations N] FILE...\n  caac gen INPUT... --output FILE\n  caac md INPUT... --output FILE\n  caac parse INPUT --output FILE\n  caac extract INPUT --output FILE\n  caac sync INPUT --code FILE --docs FILE [--watch] [--interval SECS]\n  caac pattern list\n  caac pattern gen ID --output FILE\n  caac sync INPUT --code FILE --docs FILE [--watch] [--interval SECS]\n  caac --help";

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
        "discover" => run_discover(rest),
        "compare-observations" => run_compare_observations(rest),
        "collect" => run_collect(rest),
        "review-packet" => run_review_packet(rest),
        "review-findings" => run_review_findings(rest),
        "check" | "lint" => run_check(command, rest),
        "mcp" => run_mcp(rest),
        "baseline" => run_baseline(rest),
        "candidate" => run_candidate(rest),
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

fn run_mcp(args: &[String]) -> i32 {
    let usage = "Usage: caac mcp";
    if args.len() == 1 && matches!(args[0].as_str(), "--help" | "-h") {
        println!("{usage}\nServe the local contract lint tool over MCP stdio.");
        return 0;
    }
    if !args.is_empty() {
        return argument_error("The MCP server takes no arguments.", usage);
    }
    match crate::mcp::serve_stdio() {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("ERROR: {error}");
            2
        }
    }
}

fn run_compare_observations(args: &[String]) -> i32 {
    let usage = "Usage: caac compare-observations --before FILE --after FILE";
    if args.len() == 1 && matches!(args[0].as_str(), "--help" | "-h") {
        println!(
            "{usage}\nCompare JSON observations from a test harness. See docs/observation-comparison.md for the input and equality rules. Exit 0 for a report or 2 for an input or limit error."
        );
        return 0;
    }
    let [first_flag, first_path, second_flag, second_path] = args else {
        return argument_error("Both run files are required", usage);
    };
    let mut before = None;
    let mut after = None;
    for (flag, path) in [(first_flag, first_path), (second_flag, second_path)] {
        if path.is_empty() || path.starts_with('-') {
            return argument_error("Run file paths must be nonempty", usage);
        }
        match flag.as_str() {
            "--before" if before.is_none() => before = Some(Path::new(path)),
            "--after" if after.is_none() => after = Some(Path::new(path)),
            _ => return argument_error("Unknown or duplicate comparison option", usage),
        }
    }
    let (Some(before), Some(after)) = (before, after) else {
        return argument_error("Both run files are required", usage);
    };
    match caac::observation_compare::compare_files(before, after) {
        Ok(report) => {
            println!("{report}");
            0
        }
        Err(error) => {
            eprintln!("{}", caac::suggest::safe_json(&error));
            2
        }
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

    let options = match parse_authoring_args(mode, args) {
        Ok(parsed) => parsed,
        Err(message) => return argument_error(&message, &authoring_usage(mode)),
    };

    match author::create_with_scope_size(
        mode,
        &options.destination,
        &options.title,
        &options.sources,
        options.scope_size.as_deref(),
    ) {
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

struct AuthoringArgs {
    destination: PathBuf,
    title: String,
    sources: Vec<PathBuf>,
    scope_size: Option<String>,
}

fn parse_authoring_args(mode: &str, args: &[String]) -> Result<AuthoringArgs, String> {
    let mut destination = None;
    let mut title = None;
    let mut sources = Vec::new();
    let mut scope_size = None;
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
        } else if argument == "--size" || argument.starts_with("--size=") {
            if scope_size.is_some() {
                return Err("Specify --size only once".into());
            }
            let value = if let Some(value) = argument.strip_prefix("--size=") {
                Some(value)
            } else {
                index += 1;
                args.get(index).map(String::as_str)
            };
            scope_size = Some(
                value
                    .filter(|value| !value.is_empty() && !value.starts_with('-'))
                    .ok_or("--size requires a value")?
                    .to_owned(),
            );
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
    Ok(AuthoringArgs {
        destination,
        title,
        sources,
        scope_size,
    })
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
    let usage =
        "Usage: caac interview [--auto] [--suggest] [--source FILE | --baseline FILE] FILE...";
    let mut auto = false;
    let mut suggest = false;
    let mut sources = Vec::new();
    let mut baseline = None;
    let mut files = Vec::new();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--" => {
                files.extend(args[index + 1..].iter().map(PathBuf::from));
                break;
            }
            "--auto" => auto = true,
            "--suggest" => suggest = true,
            "--source" | "--baseline" => {
                let option = args[index].as_str();
                index += 1;
                let Some(value) = args.get(index).filter(|v| !v.starts_with('-')) else {
                    return argument_error("Source options require a file path.", usage);
                };
                if option == "--source" {
                    sources.push(PathBuf::from(value));
                } else if baseline.replace(PathBuf::from(value)).is_some() {
                    return argument_error("Specify --baseline only once.", usage);
                }
            }
            "--help" | "-h" => {
                println!("{usage}");
                return 0;
            }
            option if option.starts_with('-') => {
                return argument_error("Unknown interview option.", usage);
            }
            _ => files.push(PathBuf::from(&args[index])),
        }
        index += 1;
    }
    if files.is_empty() {
        return argument_error("At least one input file is required.", usage);
    }
    if (!sources.is_empty() || baseline.is_some()) && !suggest {
        return argument_error("Source evidence options require --suggest.", usage);
    }
    if baseline.is_some() && !sources.is_empty() {
        return argument_error("--source and --baseline are mutually exclusive.", usage);
    }
    if let Err(error) = caac::checker::check(&files) {
        eprintln!("ERROR: {}", caac::suggest::safe_json(&error));
        return 2;
    }
    let mut result = 0;
    for file in &files {
        if suggest {
            let context = match caac::source_intake::load_evidence(&sources, baseline.as_deref()) {
                Ok(context) => context,
                Err(error) => {
                    eprintln!("ERROR: {}", caac::suggest::safe_json(&error));
                    return 2;
                }
            };
            let diagnostics = match caac::checker::check(std::slice::from_ref(file)) {
                Ok(diagnostics) => diagnostics,
                Err(error) => {
                    eprintln!("ERROR: {}", caac::suggest::safe_json(&error));
                    return 2;
                }
            };
            let decisions: Vec<_> = diagnostics
                .iter()
                .filter(|d| d.code == "open-decision")
                .collect();
            if decisions.is_empty() {
                continue;
            }
            if let Err(error) =
                caac::suggest::emit_batch(&context, decisions.iter().map(|d| d.message.as_str()))
            {
                eprintln!("ERROR: {}", caac::suggest::safe_json(&error));
                return 2;
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
    let mut baseline = None;
    let mut max_iterations = 10u32;
    let mut files = Vec::new();
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--" {
            files.extend(args[index + 1..].iter().map(PathBuf::from));
            break;
        }
        if args[index] == "--source" || args[index] == "--baseline" {
            let option = args[index].clone();
            index += 1;
            let Some(src) = args.get(index).filter(|v| !v.starts_with('-')) else {
                return argument_error("Source options require a file path.", "caac run --help");
            };
            if option == "--source" {
                sources.push(src.clone());
            } else if baseline.replace(PathBuf::from(src)).is_some() {
                return argument_error("Specify --baseline only once.", "caac run --help");
            }
        } else if args[index] == "--max-iterations" {
            index += 1;
            if let Some(n) = args.get(index) {
                max_iterations = n.parse().unwrap_or(10);
            }
        } else if matches!(args[index].as_str(), "--help" | "-h") {
            println!(
                "Run syntax and case-reference prechecks; semantic review remains pending.\n\nUsage: caac run [--source FILE | --baseline FILE] [--max-iterations N] FILE..."
            );
            return 0;
        } else if args[index].starts_with('-') {
            return argument_error(
                &format!("Unknown option: {}", args[index]),
                "Usage: caac run [--source FILE | --baseline FILE] [--max-iterations N] FILE...",
            );
        } else {
            files.push(PathBuf::from(&args[index]));
        }
        index += 1;
    }
    if files.is_empty() {
        return argument_error(
            "At least one input file is required.",
            "Usage: caac run [--source FILE | --baseline FILE] [--max-iterations N] FILE...",
        );
    }
    caac::harness::run_loop_with_baseline(&files, &sources, baseline.as_deref(), max_iterations)
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

fn run_check(command: &str, args: &[String]) -> i32 {
    let (usage, summary, field) = if command == "lint" {
        (
            "Usage: caac lint [--json] FILE...",
            "declared syntax and case bindings",
            "lint",
        )
    } else {
        (
            "Usage: caac check [--json] FILE...",
            "declared syntax only",
            "syntax",
        )
    };
    let mut json_output = false;
    let mut positional_only = false;
    let mut files = Vec::new();
    for argument in args {
        if positional_only {
            files.push(PathBuf::from(argument));
        } else if argument == "--" {
            positional_only = true;
        } else if matches!(argument.as_str(), "--help" | "-h") {
            println!("Check {summary}. Semantic review remains pending.\n\n{usage}");
            println!("\n  --json   Print machine-readable diagnostics.");
            return 0;
        } else if argument == "--json" {
            json_output = true;
        } else if argument.starts_with('-') {
            return argument_error(&format!("Unknown option: {argument}"), usage);
        } else {
            files.push(PathBuf::from(argument));
        }
    }

    if files.is_empty() {
        return argument_error("At least one input file is required.", usage);
    }

    let result = if command == "lint" {
        checker::lint(&files)
    } else {
        checker::check(&files)
    };
    match result {
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
                    (field): if diagnostics.is_empty() { "passed" } else { "failed" },
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
                    "{}: {summary}. Semantic review pending.",
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
        "Usage: caac scratch OUTPUT --title TITLE [--size XS|S|M]".to_owned()
    } else {
        format!("Usage: caac {mode} OUTPUT --title TITLE --source FILE... [--size XS|S|M]")
    }
}

fn print_root_help() {
    println!(
        "Contract authoring, declared syntax checks, repair watch, and interview.\n\n{ROOT_USAGE}"
    );
    println!("  caac compare-observations --before FILE --after FILE");
}

fn print_authoring_help(mode: &str) {
    println!(
        "Create an isolated draft contract.\n\n{}",
        authoring_usage(mode)
    );
    println!("  --title TITLE   Set the selected contract scope.");
    println!("  --size XS|S|M   Record a proposed scope size in the Markdown scaffold.");
    if mode != "scratch" {
        println!("  --source FILE   Select a source file. Repeat for more files.");
    }
}

fn run_baseline(args: &[String]) -> i32 {
    let usage = "Usage: caac baseline check BASELINE";
    if args.iter().any(|a| matches!(a.as_str(), "--help" | "-h")) {
        println!(
            "{usage}\nChecks selected source identities and syntax evidence, not semantic completeness or acceptance."
        );
        return 0;
    }
    if args.len() != 2 || args[0] != "check" {
        return argument_error("Expected baseline check and one file", usage);
    }
    caac::source_intake::run_check(Path::new(&args[1]))
}

fn run_candidate(args: &[String]) -> i32 {
    let usage = "Usage: caac candidate schema\n       caac candidate apply --base DRAFT --answers ANSWERS --output NEW_DIRECTORY [--review-receipt RECEIPT --findings FILE --collection FILE --packet FILE ...]\n       caac candidate interview --base DRAFT --answers JOURNAL --output NEW_DIRECTORY [--resume]\n       caac candidate verify --candidate FILE --provenance FILE --review FILE [--trace ID]\n       caac candidate compare-review --candidate FILE --provenance FILE --review ORIGINAL --edited EDITED --output NEW_DIRECTORY\n       caac candidate revise --candidate FILE --provenance FILE --review ORIGINAL (--proposal PROPOSAL | --presentation REQUEST) --output NEW_DIRECTORY";
    if args.first().map(String::as_str) == Some("apply")
        && args[1..].iter().step_by(2).any(|a| {
            matches!(
                a.as_str(),
                "--review-receipt"
                    | "--findings"
                    | "--collection"
                    | "--packet"
                    | "--contract"
                    | "--run-input"
                    | "--crawk-modules"
                    | "--crawk-deps"
                    | "--crawk-snapshot"
                    | "--crawk-source-root"
                    | "--root"
                    | "--root-assessment"
                    | "--baseline"
            )
        })
    {
        return run_evidence_apply(&args[1..]);
    }
    if args.first().map(String::as_str) == Some("schema") {
        if args.len() == 2 && matches!(args[1].as_str(), "--help" | "-h") {
            println!(
                "Usage: caac candidate schema\nExports structural JSON Schema for one Entity answer; runtime semantic validation still applies."
            );
            return 0;
        }
        if args.len() != 1 {
            return argument_error(
                "Schema export takes no input or output options",
                "Usage: caac candidate schema",
            );
        }
        return match serde_json::to_string_pretty(&caac::candidate::entity_schema()) {
            Ok(schema) => {
                println!("{schema}");
                0
            }
            Err(error) => {
                eprintln!("{}", caac::suggest::safe_json(&error.to_string()));
                2
            }
        };
    }

    if (args.len() == 1 && matches!(args[0].as_str(), "--help" | "-h"))
        || (args.len() == 2 && matches!(args[1].as_str(), "--help" | "-h"))
    {
        println!("{usage}");
        return 0;
    }
    if args.first().is_some_and(|a| a == "revise") {
        if args.len() != 11 {
            return argument_error("Expected five explicit comparison paths", usage);
        }
        let mut paths = std::collections::BTreeMap::new();
        for pair in args[1..].as_chunks::<2>().0 {
            if !matches!(
                pair[0].as_str(),
                "--candidate"
                    | "--provenance"
                    | "--review"
                    | "--proposal"
                    | "--presentation"
                    | "--output"
            ) || pair[1].is_empty()
                || paths.insert(pair[0].as_str(), pair[1].as_str()).is_some()
            {
                return argument_error("Unknown, duplicate or empty comparison option", usage);
            }
        }
        if !["--candidate", "--provenance", "--review", "--output"]
            .iter()
            .all(|key| paths.contains_key(key))
            || paths.contains_key("--proposal") == paths.contains_key("--presentation")
        {
            return argument_error(
                "Provide exactly one of --proposal or --presentation with source triple and output",
                usage,
            );
        }
        let result = if let Some(request) = paths.get("--presentation") {
            caac::candidate::revise_presentation(
                Path::new(paths["--candidate"]),
                Path::new(paths["--provenance"]),
                Path::new(paths["--review"]),
                Path::new(request),
                Path::new(paths["--output"]),
            )
        } else {
            caac::candidate::revise(
                Path::new(paths["--candidate"]),
                Path::new(paths["--provenance"]),
                Path::new(paths["--review"]),
                Path::new(paths["--proposal"]),
                Path::new(paths["--output"]),
            )
        };
        return match result {
            Ok(()) => {
                println!(
                    "Created a new pending revision. Source unchanged; no authority or evidence inherited."
                );
                0
            }
            Err(e) => {
                eprintln!("ERROR: {}", caac::suggest::safe_json(&e));
                2
            }
        };
    }
    if args.first().is_some_and(|a| a == "compare-review") {
        if args.len() != 11 {
            return argument_error("Expected five explicit comparison paths", usage);
        }
        let mut paths = std::collections::BTreeMap::new();
        for pair in args[1..].as_chunks::<2>().0 {
            if !matches!(
                pair[0].as_str(),
                "--candidate" | "--provenance" | "--review" | "--edited" | "--output"
            ) || pair[1].is_empty()
                || paths.insert(pair[0].as_str(), pair[1].as_str()).is_some()
            {
                return argument_error("Unknown, duplicate or empty comparison option", usage);
            }
        }
        return match caac::candidate::compare_review(
            Path::new(paths["--candidate"]),
            Path::new(paths["--provenance"]),
            Path::new(paths["--review"]),
            Path::new(paths["--edited"]),
            Path::new(paths["--output"]),
        ) {
            Ok(status) => {
                println!(
                    "Review comparison recorded: {status}. No changes applied; authority pending."
                );
                if status == "conflict" { 3 } else { 0 }
            }
            Err(e) => {
                eprintln!("ERROR: {}", caac::suggest::safe_json(&e));
                2
            }
        };
    }
    if args.first().is_some_and(|a| a == "verify") {
        if !matches!(args.len(), 7 | 9) {
            return argument_error("Expected candidate, provenance and review paths", usage);
        }
        let mut paths = std::collections::BTreeMap::new();
        for pair in args[1..].as_chunks::<2>().0 {
            if !matches!(
                pair[0].as_str(),
                "--candidate" | "--provenance" | "--review" | "--trace"
            ) || pair[1].is_empty()
                || paths.insert(pair[0].as_str(), pair[1].as_str()).is_some()
            {
                return argument_error(
                    "Unknown, duplicate or empty revision verification option",
                    usage,
                );
            }
        }
        if !["--candidate", "--provenance", "--review"]
            .iter()
            .all(|key| paths.contains_key(key))
            || (args.len() == 9 && !paths.contains_key("--trace"))
        {
            return argument_error(
                "Verification requires source triple and optional exact --trace ID",
                usage,
            );
        }
        if let Some(id) = paths.get("--trace") {
            return match caac::candidate::verify_trace(
                Path::new(paths["--candidate"]),
                Path::new(paths["--provenance"]),
                Path::new(paths["--review"]),
                id,
            ) {
                Ok(record) => {
                    println!("{record}");
                    0
                }
                Err(e) => {
                    eprintln!("ERROR: {}", caac::suggest::safe_json(&e));
                    2
                }
            };
        }
        return match caac::candidate::verify(
            Path::new(paths["--candidate"]),
            Path::new(paths["--provenance"]),
            Path::new(paths["--review"]),
        ) {
            Ok(()) => {
                println!(
                    "Verified unsigned candidate, revision bindings and derived review consistency. Original input provenance, authorship, acceptance and executed evidence are not verified."
                );
                0
            }
            Err(e) => {
                eprintln!("ERROR: {}", caac::suggest::safe_json(&e));
                2
            }
        };
    }
    let resume = args.last().is_some_and(|a| a == "--resume");
    let args = if resume {
        &args[..args.len() - 1]
    } else {
        args
    };
    if args.len() != 7
        || !matches!(args[0].as_str(), "apply" | "interview")
        || (resume && args[0] != "interview")
    {
        return argument_error(
            "Expected candidate apply/interview and three explicit paths",
            usage,
        );
    }
    let mut values = std::collections::BTreeMap::new();
    for pair in args[1..].as_chunks::<2>().0 {
        if !matches!(pair[0].as_str(), "--base" | "--answers" | "--output")
            || pair[1].is_empty()
            || values.insert(pair[0].as_str(), pair[1].as_str()).is_some()
        {
            return argument_error("Unknown, duplicate, or empty candidate option", usage);
        }
    }
    let result = if args[0] == "interview" {
        caac::candidate::interview(
            Path::new(values["--base"]),
            Path::new(values["--answers"]),
            Path::new(values["--output"]),
            resume,
            &mut std::io::stdin().lock(),
            &mut std::io::stdout().lock(),
        )
    } else {
        caac::candidate::apply(
            Path::new(values["--base"]),
            Path::new(values["--answers"]),
            Path::new(values["--output"]),
        )
    };
    match result {
        Ok(()) => {
            println!(
                "Created experimental candidate and review. Owner acceptance pending; contract completion not established."
            );
            0
        }
        Err(e) => {
            eprintln!("ERROR: {}", caac::suggest::safe_json(&e));
            2
        }
    }
}

fn run_collect(args: &[String]) -> i32 {
    let usage = "Usage: caac collect [--contract FILE]... [--run-input FILE]... [--root FILE | --root-assessment FILE] [--baseline FILE] [--crawk-modules FILE --crawk-deps FILE --crawk-snapshot FILE --crawk-source-root DIR] --output NEW_FILE";
    if args.len() == 1 && matches!(args[0].as_str(), "--help" | "-h") {
        println!("{usage}\nSelected intake only; no recursive discovery or semantic comparison.");
        return 0;
    }
    let mut request = caac::contract_collection::Request::default();
    let mut output = None;
    let mut index = 0;
    while index < args.len() {
        let Some(value) = args.get(index + 1) else {
            return argument_error("Expected a path after each option", usage);
        };
        match args[index].as_str() {
            "--contract" => request.contracts.push(PathBuf::from(value)),
            "--run-input" => request.run_inputs.push(PathBuf::from(value)),
            flag if is_crawk_flag(flag) => {
                if let Err(e) = set_crawk_path(&mut request, flag, value) {
                    return argument_error(&e, usage);
                }
            }
            "--root" if request.root.is_none() => request.root = Some(PathBuf::from(value)),
            "--root-assessment" if request.root_assessment.is_none() => {
                request.root_assessment = Some(PathBuf::from(value));
            }
            "--baseline" if request.baseline.is_none() => {
                request.baseline = Some(PathBuf::from(value))
            }
            "--output" if output.is_none() => output = Some(PathBuf::from(value)),
            _ => return argument_error("Unknown or duplicate collection option", usage),
        }
        index += 2;
    }
    let Some(output) = output else {
        return argument_error("Output file is required", usage);
    };
    match caac::contract_collection::collect(&request, &output) {
        Ok(collection) => {
            if collection.root.document_index.is_none() {
                println!("missing root contract ({})", collection.root.reason);
            }
            println!(
                "Captured selected intake; root and semantic authority remain unverified. No comparison or acceptance performed."
            );
            0
        }
        Err(error) => {
            eprintln!("{}", caac::suggest::safe_json(&error));
            2
        }
    }
}

fn run_discover(args: &[String]) -> i32 {
    let usage = "Usage: caac discover --directory DIR [--directory DIR...] [--depth 0..4]";
    if args.len() == 1 && matches!(args[0].as_str(), "--help" | "-h") {
        println!(
            "{usage}\nMetadata-only suggestions, nonrecursive by default; explicit collect selection remains required."
        );
        return 0;
    }
    if args.is_empty() || !args.len().is_multiple_of(2) {
        return argument_error("Explicit directory paths are required", usage);
    }
    let mut paths = Vec::new();
    let mut depth = None;
    for pair in args.as_chunks::<2>().0 {
        match pair[0].as_str() {
            "--directory" => paths.push(PathBuf::from(&pair[1])),
            "--depth" if depth.is_none() => {
                if !matches!(pair[1].as_str(), "0" | "1" | "2" | "3" | "4") {
                    return argument_error("Depth must be an integer from 0 to 4", usage);
                }
                depth = Some(pair[1].parse::<u8>().unwrap());
            }
            _ => return argument_error("Unknown or duplicate discovery option", usage),
        }
    }
    match caac::contract_discovery::discover_with_options(
        &paths,
        &caac::contract_discovery::Options {
            depth: depth.unwrap_or(0),
        },
    )
    .and_then(|r| caac::contract_discovery::terminal_json(&r))
    {
        Ok(record) => {
            println!("{record}");
            0
        }
        Err(error) => {
            eprintln!("{}", caac::suggest::safe_json(&error));
            2
        }
    }
}

fn run_review_packet(args: &[String]) -> i32 {
    let usage = "Usage: caac review-packet --collection FILE [--contract FILE]... [--run-input FILE]... [--root FILE | --root-assessment FILE] [--baseline FILE] [--crawk-modules FILE --crawk-deps FILE --crawk-snapshot FILE --crawk-source-root DIR] --output NEW_FILE";
    if args.len() == 1 && matches!(args[0].as_str(), "--help" | "-h") {
        println!(
            "{usage}\nReproduce exact collect selections; evidence index only, not semantic comparison."
        );
        return 0;
    }
    if !args.len().is_multiple_of(2) {
        return argument_error("Expected explicit option/path pairs", usage);
    }
    let mut request = caac::contract_collection::Request::default();
    let mut collection = None;
    let mut output = None;
    for pair in args.as_chunks::<2>().0 {
        match pair[0].as_str() {
            "--contract" => request.contracts.push(PathBuf::from(&pair[1])),
            "--run-input" => request.run_inputs.push(PathBuf::from(&pair[1])),
            flag if is_crawk_flag(flag) => {
                if let Err(e) = set_crawk_path(&mut request, flag, &pair[1]) {
                    return argument_error(&e, usage);
                }
            }
            "--root" if request.root.is_none() => request.root = Some(PathBuf::from(&pair[1])),
            "--root-assessment" if request.root_assessment.is_none() => {
                request.root_assessment = Some(PathBuf::from(&pair[1]));
            }
            "--baseline" if request.baseline.is_none() => {
                request.baseline = Some(PathBuf::from(&pair[1]))
            }
            "--collection" if collection.is_none() => collection = Some(PathBuf::from(&pair[1])),
            "--output" if output.is_none() => output = Some(PathBuf::from(&pair[1])),
            _ => return argument_error("Unknown or duplicate review-packet option", usage),
        }
    }
    let (Some(collection), Some(output)) = (collection, output) else {
        return argument_error("Collection and output paths are required", usage);
    };
    match caac::review_packet::prepare_review(&collection, &request, &output) {
        Ok(_) => {
            println!(
                "Created exact-collection evidence index; questions unresolved, comparison not performed, acceptance pending."
            );
            0
        }
        Err(error) => {
            eprintln!("{}", caac::suggest::safe_json(&error));
            2
        }
    }
}

fn run_review_findings(args: &[String]) -> i32 {
    let usage = "Usage: caac review-findings --collection FILE --packet FILE --findings FILE --base FILE [--contract FILE]... [--run-input FILE]... [--root FILE | --root-assessment FILE] [--baseline FILE] [--crawk-modules FILE --crawk-deps FILE --crawk-snapshot FILE --crawk-source-root DIR] --output NEW_DIRECTORY";
    if args.len() == 1 && matches!(args[0].as_str(), "--help" | "-h") {
        println!(
            "{usage}\nChecked citations and pending decision answers; semantic claim truth is not assessed."
        );
        return 0;
    }
    if !args.len().is_multiple_of(2) {
        return argument_error("Expected option/path pairs", usage);
    }
    let mut request = caac::contract_collection::Request::default();
    let mut paths = std::collections::BTreeMap::new();
    for pair in args.as_chunks::<2>().0 {
        match pair[0].as_str() {
            "--contract" => request.contracts.push(PathBuf::from(&pair[1])),
            "--run-input" => request.run_inputs.push(PathBuf::from(&pair[1])),
            flag if is_crawk_flag(flag) => {
                if let Err(e) = set_crawk_path(&mut request, flag, &pair[1]) {
                    return argument_error(&e, usage);
                }
            }
            "--root" if request.root.is_none() => request.root = Some(PathBuf::from(&pair[1])),
            "--root-assessment" if request.root_assessment.is_none() => {
                request.root_assessment = Some(PathBuf::from(&pair[1]));
            }
            "--baseline" if request.baseline.is_none() => {
                request.baseline = Some(PathBuf::from(&pair[1]))
            }
            "--collection" | "--packet" | "--findings" | "--base" | "--output"
                if !paths.contains_key(pair[0].as_str()) =>
            {
                paths.insert(pair[0].as_str(), PathBuf::from(&pair[1]));
            }
            _ => return argument_error("Unknown or duplicate review-findings option", usage),
        }
    }
    if ![
        "--collection",
        "--packet",
        "--findings",
        "--base",
        "--output",
    ]
    .iter()
    .all(|k| paths.contains_key(k))
    {
        return argument_error(
            "Collection, packet, findings, base and output are required",
            usage,
        );
    }
    match caac::review_findings::handoff(
        &paths["--collection"],
        &paths["--packet"],
        &paths["--findings"],
        &paths["--base"],
        &request,
        &paths["--output"],
    ) {
        Ok(()) => {
            println!(
                "Created pending decision answers and citation receipt; candidate apply remains explicit, claim truth not assessed."
            );
            0
        }
        Err(e) => {
            eprintln!("{}", caac::suggest::safe_json(&e));
            2
        }
    }
}

fn run_evidence_apply(args: &[String]) -> i32 {
    let usage = "Usage: caac candidate apply --base BASE --answers ANSWERS --output NEW_DIRECTORY --review-receipt RECEIPT --findings FILE --collection FILE --packet FILE [--contract FILE]... [--run-input FILE]... [--root FILE | --root-assessment FILE] [--baseline FILE] [--crawk-modules FILE --crawk-deps FILE --crawk-snapshot FILE --crawk-source-root DIR]";
    if args.len() == 1 && matches!(args[0].as_str(), "--help" | "-h") {
        println!(
            "{usage}\nRechecks exact citations at this invocation; standalone candidate verify does not repeat evidence checks or establish claim truth."
        );
        return 0;
    }
    if !args.len().is_multiple_of(2) {
        return argument_error("Expected explicit option/path pairs", usage);
    }
    let mut paths = std::collections::BTreeMap::new();
    let mut request = caac::contract_collection::Request::default();
    for pair in args.as_chunks::<2>().0 {
        match pair[0].as_str() {
            "--contract" => request.contracts.push(PathBuf::from(&pair[1])),
            "--run-input" => request.run_inputs.push(PathBuf::from(&pair[1])),
            flag if is_crawk_flag(flag) => {
                if let Err(e) = set_crawk_path(&mut request, flag, &pair[1]) {
                    return argument_error(&e, usage);
                }
            }
            "--root" if request.root.is_none() => request.root = Some(PathBuf::from(&pair[1])),
            "--root-assessment" if request.root_assessment.is_none() => {
                request.root_assessment = Some(PathBuf::from(&pair[1]));
            }
            "--baseline" if request.baseline.is_none() => {
                request.baseline = Some(PathBuf::from(&pair[1]))
            }
            "--base" | "--answers" | "--output" | "--review-receipt" | "--findings"
            | "--collection" | "--packet"
                if !paths.contains_key(pair[0].as_str()) =>
            {
                paths.insert(pair[0].as_str(), PathBuf::from(&pair[1]));
            }
            _ => return argument_error("Unknown or duplicate evidence apply option", usage),
        }
    }
    if ![
        "--base",
        "--answers",
        "--output",
        "--review-receipt",
        "--findings",
        "--collection",
        "--packet",
    ]
    .iter()
    .all(|k| paths.contains_key(k))
    {
        return argument_error(
            "Evidence apply requires base, answers, output, receipt, findings, collection and packet",
            usage,
        );
    }
    let evidence = caac::review_findings::ApplyEvidence {
        collection: &paths["--collection"],
        packet: &paths["--packet"],
        findings: &paths["--findings"],
        receipt: &paths["--review-receipt"],
        selections: &request,
    };
    match caac::review_findings::apply_bound(
        &paths["--base"],
        &paths["--answers"],
        &paths["--output"],
        &evidence,
    ) {
        Ok(()) => {
            println!(
                "Applied exact handoff answers after invocation-time citation revalidation. Claim truth and acceptance remain unassessed; standalone candidate verify does not recheck evidence."
            );
            0
        }
        Err(error) => {
            eprintln!("{}", caac::suggest::safe_json(&error));
            2
        }
    }
}

fn is_crawk_flag(flag: &str) -> bool {
    matches!(
        flag,
        "--crawk-modules" | "--crawk-deps" | "--crawk-snapshot" | "--crawk-source-root"
    )
}
fn set_crawk_path(
    request: &mut caac::contract_collection::Request,
    flag: &str,
    value: &str,
) -> Result<(), String> {
    let selection = request.crawk.get_or_insert_with(Default::default);
    let target = match flag {
        "--crawk-modules" => &mut selection.modules,
        "--crawk-deps" => &mut selection.deps,
        "--crawk-snapshot" => &mut selection.snapshot,
        "--crawk-source-root" => &mut selection.source_root,
        _ => return Err("Unknown analyzer selection".into()),
    };
    if !target.as_os_str().is_empty() || value.is_empty() {
        return Err("Duplicate or empty analyzer selection".into());
    }
    *target = PathBuf::from(value);
    Ok(())
}
