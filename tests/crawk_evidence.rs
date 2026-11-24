use caac::{
    candidate::digest,
    contract_collection::{self, Request},
    crawk_evidence::Selection,
    review_packet,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
fn run(p: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_caac"))
        .current_dir(p)
        .args(args)
        .output()
        .unwrap()
}
fn read(p: &Path, name: &str) -> Value {
    serde_json::from_slice(&fs::read(p.join(name)).unwrap()).unwrap()
}
fn write(p: &Path, name: &str, value: &Value) {
    fs::write(p.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}
fn setup(p: &Path) -> Request {
    fs::create_dir(p.join("src")).unwrap();
    fs::write(p.join("src/lib.rs"), "pub fn observed() {}\n").unwrap();
    fs::write(
        p.join("contract"),
        "Shared rules are proposed, not accepted.\n",
    )
    .unwrap();
    assert!(
        run(
            p,
            &[
                "plugin",
                "intake",
                "--title",
                "Synthetic graph",
                "--source",
                "src/lib.rs"
            ]
        )
        .status
        .success()
    );
    fs::write(p.join("modules"),"[lib] lib\n[lib] deferred_terminal\n[lib] runtime_admission\n[lib] mcp_sdk\n[bin:pigeonmq] mcp_sdk\n[bin:pigeonmq-agent] pigeonmq-agent\n").unwrap();
    fs::write(
        p.join("deps"),
        "deferred_terminal -> runtime_admission [validate_terminal]\nmcp_sdk -> absent [unknown]\n",
    )
    .unwrap();
    write(
        p,
        "snapshot",
        &json!({"revision":"unverified-revision","files":[{"path":"src/lib.rs","sha256":digest(&fs::read(p.join("src/lib.rs")).unwrap())},{"path":"private-not-selected/never-read","sha256":"0".repeat(64)}]}),
    );
    Request {
        contracts: vec![p.join("contract")],
        baseline: Some(p.join("intake/baseline.json")),
        crawk: Some(Selection {
            modules: p.join("modules"),
            deps: p.join("deps"),
            snapshot: p.join("snapshot"),
            source_root: p.into(),
        }),
        ..Default::default()
    }
}
#[test]
fn inventory_claims_bind_selected_files_not_whole_graph_or_targets() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let r = setup(p);
    contract_collection::collect(&r, &p.join("collection")).unwrap();
    let c = read(p, "collection");
    assert_eq!(c["format"], "caac-contract-collection-v3");
    let a = &c["analyzer_evidence"];
    assert_eq!(a["profile"], "crawk-0.7.1-plain-v1");
    assert_eq!(a["selected_files_match"].as_array().unwrap().len(), 1);
    assert_eq!(
        a["unverified_manifest_files"],
        json!(["private-not-selected/never-read"])
    );
    assert_eq!(a["analyzer_producer"], "not-observed");
    assert_eq!(a["analyzer_options"], "not-observed");
    assert_eq!(
        a["dependencies"][0]["source"]["status"],
        "inventory-unambiguous"
    );
    assert_eq!(a["dependencies"][1]["source"]["status"], "ambiguous");
    assert_eq!(
        a["dependencies"][1]["source"]["inventory_matches"],
        json!([3, 4])
    );
    assert_eq!(a["dependencies"][1]["target"]["status"], "unresolved");
    assert_eq!(a["dependencies"].as_array().unwrap().len(), 2); // No inferred missing/shared-state edges.
    for edge in a["dependencies"].as_array().unwrap() {
        let bytes = fs::read(p.join("deps")).unwrap();
        let start = edge["start_byte"].as_u64().unwrap() as usize;
        let end = edge["end_byte"].as_u64().unwrap() as usize;
        assert!(
            std::str::from_utf8(&bytes[start..end])
                .unwrap()
                .contains(" -> ")
        );
    }
    review_packet::prepare_review(&p.join("collection"), &r, &p.join("packet")).unwrap();
    let packet = read(p, "packet");
    assert_eq!(packet["collection_profile"], "exact-canonical-collect-v3");
    assert_eq!(
        packet["evidence"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["kind"] == "analyzer-output")
            .count(),
        3
    );
    for name in ["modules", "deps", "snapshot", "src/lib.rs"] {
        let old = fs::read(p.join(name)).unwrap();
        let mut changed = old.clone();
        changed.push(b' ');
        fs::write(p.join(name), changed).unwrap();
        assert!(
            review_packet::prepare_review(&p.join("collection"), &r, &p.join("stale")).is_err(),
            "{name}"
        );
        assert!(!p.join("stale").exists());
        fs::write(p.join(name), old).unwrap();
    }
}
#[test]
fn strict_snapshot_output_and_selected_root_reject_without_publication() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let mut r = setup(p);
    let original = read(p, "snapshot");
    let mut cases = Vec::new();
    for path in [
        "../escape",
        "/absolute",
        "./src/lib.rs",
        "src//lib.rs",
        "src\\lib.rs",
    ] {
        let mut v = original.clone();
        v["files"][0]["path"] = json!(path);
        cases.push(v);
    }
    let mut v = original.clone();
    v["files"][0]["sha256"] = json!("0".repeat(64));
    cases.push(v);
    let mut v = original.clone();
    let row = v["files"][0].clone();
    v["files"].as_array_mut().unwrap().push(row);
    cases.push(v);
    let mut v = original.clone();
    v["unknown"] = json!(true);
    cases.push(v);
    for v in cases {
        write(p, "snapshot", &v);
        assert!(contract_collection::collect(&r, &p.join("bad")).is_err());
        assert!(!p.join("bad").exists());
    }
    let raw = serde_json::to_string(&original).unwrap();
    fs::write(
        p.join("snapshot"),
        raw.replacen('{', "{\"revision\":\"duplicate\",", 1),
    )
    .unwrap();
    assert!(contract_collection::collect(&r, &p.join("bad")).is_err());
    write(p, "snapshot", &original);
    let saved = fs::read(p.join("modules")).unwrap();
    for text in [
        "mcp_sdk\n",
        "[lib] lib\n[lib] lib\n",
        "[lib] lib\n\n",
        "[lib] lib\u{1b}[2J\n",
    ] {
        fs::write(p.join("modules"), text).unwrap();
        assert!(contract_collection::collect(&r, &p.join("bad")).is_err());
    }
    fs::write(p.join("modules"), saved).unwrap();
    r.crawk.as_mut().unwrap().source_root = p.join("intake");
    assert!(contract_collection::collect(&r, &p.join("bad")).is_err());
    r.crawk.as_mut().unwrap().source_root = p.into();
    r.baseline = None;
    assert!(contract_collection::collect(&r, &p.join("bad")).is_err());
    assert!(!p.join("bad").exists());
}
#[test]
fn cli_requires_complete_selection_and_bounded_regular_files() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let r = setup(p);
    let args = [
        "collect",
        "--contract",
        "contract",
        "--baseline",
        "intake/baseline.json",
        "--crawk-modules",
        "modules",
        "--crawk-deps",
        "deps",
        "--crawk-snapshot",
        "snapshot",
        "--crawk-source-root",
        ".",
        "--output",
        "collection",
    ];
    let result = run(p, &args);
    assert!(result.status.success(), "{:?}", result);
    assert_eq!(
        run(
            p,
            &[
                "collect",
                "--contract",
                "contract",
                "--crawk-modules",
                "modules",
                "--output",
                "bad"
            ]
        )
        .status
        .code(),
        Some(2)
    );
    assert!(!p.join("bad").exists());
    fs::write(p.join("deps"), vec![b'a'; 1024 * 1024 + 1]).unwrap();
    assert!(contract_collection::collect(&r, &p.join("bad")).is_err());
    #[cfg(unix)]
    {
        fs::remove_file(p.join("deps")).unwrap();
        std::os::unix::fs::symlink(p.join("modules"), p.join("deps")).unwrap();
        assert!(contract_collection::collect(&r, &p.join("bad")).is_err());
        fs::remove_file(p.join("deps")).unwrap();
        assert!(
            Command::new("mkfifo")
                .arg(p.join("deps"))
                .status()
                .unwrap()
                .success()
        );
        assert!(contract_collection::collect(&r, &p.join("bad")).is_err());
    }
    assert!(!p.join("bad").exists());
}

#[test]
fn record_caps_and_combined_text_budget_are_fail_closed() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let mut r = setup(p);
    let modules = fs::read(p.join("modules")).unwrap();
    fs::write(
        p.join("modules"),
        (0..4097)
            .map(|i| format!("[lib] module_{i}\n"))
            .collect::<String>(),
    )
    .unwrap();
    assert!(contract_collection::collect(&r, &p.join("bad")).is_err());
    fs::write(p.join("modules"), modules).unwrap();
    let deps = fs::read(p.join("deps")).unwrap();
    fs::write(
        p.join("deps"),
        (0..8193)
            .map(|i| format!("lib -> target_{i}\n"))
            .collect::<String>(),
    )
    .unwrap();
    assert!(contract_collection::collect(&r, &p.join("bad")).is_err());
    fs::write(p.join("deps"), deps).unwrap();
    let snapshot = read(p, "snapshot");
    let mut too_many = snapshot.clone();
    too_many["files"] = json!(
        (0..4097)
            .map(|i| json!({"path":format!("file_{i}"),"sha256":"0".repeat(64)}))
            .collect::<Vec<_>>()
    );
    write(p, "snapshot", &too_many);
    assert!(contract_collection::collect(&r, &p.join("bad")).is_err());
    write(p, "snapshot", &snapshot);
    for i in 0..7 {
        let name = p.join(format!("large-doc-{i}"));
        fs::write(&name, vec![b'x'; 1024 * 1024]).unwrap();
        r.contracts.push(name);
    }
    fs::write(
        p.join("context"),
        vec![b'x'; 1024 * 1024 - fs::read(p.join("contract")).unwrap().len()],
    )
    .unwrap();
    r.run_inputs.push(p.join("context"));
    assert!(
        contract_collection::collect(&r, &p.join("bad"))
            .err()
            .unwrap()
            .contains("8 MiB")
    );
    assert!(!p.join("bad").exists());
}

#[test]
fn repeated_ambiguous_endpoints_fail_before_unbounded_expansion() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let r = setup(p);
    let modules: String = (0..4096)
        .map(|i| format!("[bin:target{i}] shared\n"))
        .collect();
    let deps: String = (0..8192)
        .map(|i| format!("shared -> unknown{i}\n"))
        .collect();
    assert!(modules.len() < 1024 * 1024 && deps.len() < 1024 * 1024);
    fs::write(p.join("modules"), modules).unwrap();
    fs::write(p.join("deps"), deps).unwrap();
    let error = contract_collection::collect(&r, &p.join("collection"))
        .err()
        .unwrap();
    assert!(
        error.contains("Aggregate endpoint inventory-match resource limit"),
        "{error}"
    );
    assert!(!p.join("collection").exists());
    // The exact resource boundary preserves every ambiguous match; no truncation.
    fs::write(
        p.join("deps"),
        (0..16)
            .map(|i| format!("shared -> unknown{i}\n"))
            .collect::<String>(),
    )
    .unwrap();
    contract_collection::collect(&r, &p.join("at-limit")).unwrap();
    let c = read(p, "at-limit");
    for edge in c["analyzer_evidence"]["dependencies"].as_array().unwrap() {
        assert_eq!(
            edge["source"]["inventory_matches"]
                .as_array()
                .unwrap()
                .len(),
            4096
        );
        assert_eq!(edge["source"]["status"], "ambiguous");
    }
}
