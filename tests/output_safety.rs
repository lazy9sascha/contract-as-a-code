use std::fs;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_caac"))
}
const SCHEMA: &str =
    r#"{"name":"Example","operations":[{"name":"value","params":"","returns":"i64"}]}"#;

#[test]
fn generators_never_overwrite_existing_outputs_or_input() {
    let dir = tempfile::tempdir().unwrap();
    let schema = dir.path().join("schema.json");
    fs::write(&schema, SCHEMA).unwrap();
    for command in ["gen", "md"] {
        let output = cli()
            .arg(command)
            .arg(&schema)
            .arg("--output")
            .arg(&schema)
            .output()
            .unwrap();
        assert!(!output.status.success(), "{command} overwrote its input");
        assert_eq!(fs::read_to_string(&schema).unwrap(), SCHEMA);
    }
    for (generator, consumer, name) in [
        ("md", "parse", "contract.md"),
        ("gen", "extract", "trait.rs"),
    ] {
        let input = dir.path().join(name);
        assert!(
            cli()
                .arg(generator)
                .arg(&schema)
                .arg("--output")
                .arg(&input)
                .output()
                .unwrap()
                .status
                .success()
        );
        let output = cli()
            .arg(consumer)
            .arg(&input)
            .arg("--output")
            .arg(&schema)
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(2),
            "{consumer}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(fs::read_to_string(&schema).unwrap(), SCHEMA);
        let original_input = fs::read(&input).unwrap();
        assert_eq!(
            cli()
                .arg(consumer)
                .arg(&input)
                .arg("--output")
                .arg(&input)
                .output()
                .unwrap()
                .status
                .code(),
            Some(2)
        );
        assert_eq!(fs::read(&input).unwrap(), original_input);
    }
    let output = cli()
        .arg("sync")
        .arg(&schema)
        .arg("--code")
        .arg(&schema)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(fs::read_to_string(&schema).unwrap(), SCHEMA);
    let output = cli()
        .args([
            "pattern",
            "gen",
            "ABS-INHERIT",
            "--slots",
            r#"{"base_trait":"Base","extended_trait":"Child","method_name":"f"}"#,
            "--output",
        ])
        .arg(&schema)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(fs::read_to_string(&schema).unwrap(), SCHEMA);
}

#[test]
fn sync_preflights_all_outputs_and_requires_destination() {
    let dir = tempfile::tempdir().unwrap();
    let schema = dir.path().join("schema.json");
    fs::write(&schema, SCHEMA).unwrap();
    let code = dir.path().join("code.rs");
    let docs = dir.path().join("docs.md");
    fs::write(&docs, "keep me").unwrap();
    let output = cli()
        .arg("sync")
        .arg(&schema)
        .arg("--code")
        .arg(&code)
        .arg("--docs")
        .arg(&docs)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(!code.exists());
    assert_eq!(fs::read_to_string(&docs).unwrap(), "keep me");
    let output = cli()
        .arg("sync")
        .arg(&schema)
        .arg("--code")
        .arg(&code)
        .arg("--docs")
        .arg(&code)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(!code.exists());
    assert_eq!(
        cli()
            .arg("sync")
            .arg(&schema)
            .arg("--code")
            .arg(&code)
            .arg("--docs")
            .arg(code.join("nested.md"))
            .output()
            .unwrap()
            .status
            .code(),
        Some(2)
    );
    assert!(!code.exists());
    assert_eq!(
        cli()
            .arg("sync")
            .arg(&schema)
            .output()
            .unwrap()
            .status
            .code(),
        Some(2)
    );
    for interval in ["0", "bad"] {
        assert_eq!(
            cli()
                .arg("sync")
                .arg(&schema)
                .arg("--code")
                .arg(&code)
                .args(["--interval", interval])
                .output()
                .unwrap()
                .status
                .code(),
            Some(2)
        );
    }
}

#[cfg(unix)]
#[test]
fn output_symlinks_and_aliased_parent_paths_are_rejected() {
    use std::os::unix::fs::symlink;
    let dir = tempfile::tempdir().unwrap();
    let schema = dir.path().join("schema.json");
    fs::write(&schema, SCHEMA).unwrap();
    let link = dir.path().join("output.rs");
    symlink(&schema, &link).unwrap();
    assert!(caac::output::write_new(&link, b"bad").is_err());
    assert_eq!(fs::read_to_string(&schema).unwrap(), SCHEMA);
    let dangling = dir.path().join("dangling");
    symlink(dir.path().join("missing"), &dangling).unwrap();
    assert!(caac::output::write_new(&dangling, b"bad").is_err());
    let alias = dir.path().join("alias");
    symlink(dir.path(), &alias).unwrap();
    let code = dir.path().join("fresh.rs");
    assert_eq!(
        cli()
            .arg("sync")
            .arg(&schema)
            .arg("--code")
            .arg(&code)
            .arg("--docs")
            .arg(alias.join("fresh.rs"))
            .output()
            .unwrap()
            .status
            .code(),
        Some(2)
    );
    assert!(!code.exists());
}

#[test]
fn concurrent_create_publishes_one_complete_file() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("artifact");
    let handles: Vec<_> = (0..8)
        .map(|index| {
            let output = output.clone();
            std::thread::spawn(move || {
                caac::output::write_new(&output, &vec![index; 10000]).is_ok()
            })
        })
        .collect();
    let winners = handles
        .into_iter()
        .filter_map(|h| h.join().ok())
        .filter(|result| *result)
        .count();
    assert_eq!(winners, 1);
    let bytes = fs::read(output).unwrap();
    assert_eq!(bytes.len(), 10000);
    assert!(bytes.iter().all(|b| *b == bytes[0]));
}

struct Running(Child);
impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn wait_for(mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(6);
    while !condition() {
        assert!(
            Instant::now() < deadline,
            "timed out waiting for test process"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn sync_watch_updates_own_outputs_but_rejects_external_edits() {
    let dir = tempfile::tempdir().unwrap();
    let schema = dir.path().join("schema.json");
    let output = dir.path().join("output.rs");
    fs::write(&schema, SCHEMA).unwrap();
    let mut process = Running(
        cli()
            .arg("sync")
            .arg(&schema)
            .arg("--code")
            .arg(&output)
            .args(["--watch", "--interval", "1"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    wait_for(|| output.exists());
    fs::write(&schema, SCHEMA.replace("value", "updated")).unwrap();
    wait_for(|| fs::read_to_string(&output).is_ok_and(|s| s.contains("fn updated(")));
    fs::write(&output, "user edit").unwrap();
    fs::write(&schema, SCHEMA.replace("value", "again")).unwrap();
    wait_for(|| process.0.try_wait().unwrap().is_some());
    assert_eq!(process.0.wait().unwrap().code(), Some(2));
    assert_eq!(fs::read_to_string(&output).unwrap(), "user edit");
}
