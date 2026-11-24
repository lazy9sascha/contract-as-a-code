use caac::{
    candidate::digest,
    contract_collection::{Request, collect},
};
use serde_json::Value;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
fn run(d: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_caac"))
        .current_dir(d)
        .args(args)
        .output()
        .unwrap()
}
fn read(d: &Path, p: &str) -> Value {
    serde_json::from_slice(&fs::read(d.join(p)).unwrap()).unwrap()
}
fn baseline(d: &Path) {
    fs::write(
        d.join("lib.rs"),
        "pub fn echo(value: i32) -> i32 { value }\n",
    )
    .unwrap();
    assert!(
        run(
            d,
            &[
                "plugin",
                "intake",
                "--title",
                "Synthetic",
                "--source",
                "lib.rs"
            ]
        )
        .status
        .success()
    );
}
#[test]
fn exact_scattered_documents_are_not_promoted_or_followed() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let text = "# 계약\r\n\r\n[other](private.md)\r\n```\r\nnot a requirement\r\n```\r\n<!-- hidden retained -->\r\n";
    fs::write(p.join("selected.md"), text).unwrap();
    fs::write(p.join("private.md"), "DO NOT COLLECT").unwrap();
    let out = run(
        p,
        &[
            "collect",
            "--contract",
            "selected.md",
            "--output",
            "collection.json",
        ],
    );
    assert!(out.status.success(), "{:?}", out);
    let v = read(p, "collection.json");
    assert_eq!(v["documents"].as_array().unwrap().len(), 1);
    assert_eq!(v["documents"][0]["text"], text);
    assert_eq!(v["documents"][0]["sha256"], digest(text.as_bytes()));
    assert_eq!(v["documents"][0]["byte_length"], text.len());
    assert_eq!(v["root"]["finding"], "missing-root-contract");
    assert_eq!(v["root"]["reason"], "not-selected-in-declared-input-scope");
    assert_eq!(v["coverage"], "unverified");
    assert_eq!(v["mismatch"], "unverified");
    assert_eq!(v["comparison"], "not-performed");
    assert_eq!(v["owner_acceptance"], "pending");
    assert!(
        !fs::read_to_string(p.join("collection.json"))
            .unwrap()
            .contains("DO NOT COLLECT")
    );
}
#[test]
fn root_alias_is_one_document_and_remains_unverified() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    fs::write(p.join("README.md"), "# Index only").unwrap();
    let v = collect(
        &Request {
            contracts: vec![p.join("README.md")],
            root: Some(p.join("./README.md")),
            ..Request::default()
        },
        &p.join("out"),
    )
    .unwrap();
    let v = serde_json::to_value(v).unwrap();
    assert_eq!(v, read(p, "out"));
    assert_eq!(v["documents"].as_array().unwrap().len(), 1);
    assert_eq!(v["root"]["document_index"], 0);
    assert_eq!(v["root"]["finding"], "root-contract-unverified");
    assert_eq!(
        v["documents"][0]["role"],
        "caller-designated-root-unverified"
    );
}

#[test]
fn explicit_missing_root_assessments_bind_to_pending_packets() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    baseline(p);
    fs::write(p.join("README.md"), "# Project introduction").unwrap();
    fs::write(p.join("index.md"), "# Contracts\n\n[Module](module.md)").unwrap();
    fs::write(p.join("module.md"), "# Distributed module contract").unwrap();
    fs::write(p.join("task"), "Review the selected evidence.").unwrap();
    let selected = [
        "--baseline",
        "intake/baseline.json",
        "--contract",
        "README.md",
        "--contract",
        "index.md",
        "--contract",
        "module.md",
        "--run-input",
        "task",
    ];
    for (status, reporter) in [
        ("not-found-in-search-scope", "Scope reviewer"),
        ("owner-confirmed-absent", "Project owner"),
    ] {
        let report = serde_json::json!({
            "status": status,
            "scope": "Project introduction, contract index, and module contract",
            "reported_by": reporter,
        });
        let report_text = serde_json::to_string_pretty(&report).unwrap();
        let report_path = format!("{status}.json");
        let collection_path = format!("{status}-collection.json");
        let packet_path = format!("{status}-packet.json");
        fs::write(p.join(&report_path), &report_text).unwrap();
        let assessment = ["--root-assessment", &report_path];
        let output = run(
            p,
            &[
                &["collect", "--output", &collection_path][..],
                &selected,
                &assessment,
            ]
            .concat(),
        );
        assert!(output.status.success(), "{output:?}");
        assert!(
            String::from_utf8_lossy(&output.stdout)
                .contains(&format!("missing root contract ({status})"))
        );
        let collection = read(p, &collection_path);
        assert_eq!(collection["format"], "caac-contract-collection-v4");
        assert_eq!(collection["root"]["finding"], "missing-root-contract");
        assert_eq!(collection["root"]["reason"], status);
        assert_eq!(collection["root"]["document_index"], Value::Null);
        assert_eq!(collection["owner_acceptance"], "pending");
        let documents = collection["documents"].as_array().unwrap();
        assert_eq!(documents.len(), 3);
        assert!(
            documents
                .iter()
                .all(|doc| doc["role"] == "selected-contract-unverified")
        );
        let run_inputs = collection["run_inputs"].as_array().unwrap();
        assert_eq!(run_inputs.len(), 2);
        let assessment_index = collection["root"]["assessment_input_index"]
            .as_u64()
            .unwrap() as usize;
        assert_eq!(assessment_index, 0);
        assert_eq!(run_inputs[assessment_index]["text"], report_text);
        assert_eq!(
            run_inputs[assessment_index]["sha256"],
            digest(report_text.as_bytes())
        );
        assert_eq!(
            run_inputs[assessment_index]["authority"],
            "non-normative-caller-context"
        );

        let packet_args = [
            &["review-packet", "--collection", &collection_path][..],
            &selected,
        ]
        .concat();
        let packet = run(
            p,
            &[&packet_args[..], &assessment, &["--output", &packet_path]].concat(),
        );
        assert!(packet.status.success(), "{packet:?}");
        let packet = read(p, &packet_path);
        assert_eq!(packet["collection_profile"], "exact-canonical-collect-v4");
        assert_eq!(packet["root"], collection["root"]);
        assert_eq!(packet["owner_acceptance"], "pending");
        assert!(
            packet["questions"]
                .as_array()
                .unwrap()
                .iter()
                .any(|q| { q["topic"] == "root" && q["status"] == "unresolved" })
        );
        assert!(
            packet["evidence"]
                .as_array()
                .unwrap()
                .iter()
                .all(|e| { e["file_sha256"] != run_inputs[assessment_index]["sha256"] })
        );

        let bad_output = [&packet_args[..], &["--output", "bad-packet"]].concat();
        assert_eq!(run(p, &bad_output).status.code(), Some(2));
        assert!(!p.join("bad-packet").exists());
        fs::write(p.join(&report_path), format!("{report_text}\n")).unwrap();
        assert_eq!(
            run(p, &[&bad_output[..], &assessment].concat())
                .status
                .code(),
            Some(2)
        );
        assert!(!p.join("bad-packet").exists());
    }
}

#[test]
fn invalid_or_conflicting_root_assessments_publish_nothing() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    fs::write(p.join("doc"), "# Document").unwrap();
    let valid = serde_json::json!({
        "status": "owner-confirmed-absent",
        "scope": "Selected project",
        "reported_by": "Owner",
    });
    let collect = ["collect", "--root-assessment", "report", "--output", "out"];
    fs::write(p.join("report"), valid.to_string()).unwrap();
    for other in [
        &[][..],
        &["--contract", "doc", "--root", "doc"],
        &["--contract", "doc", "--root-assessment", "report"],
    ] {
        assert_eq!(
            run(p, &[&collect[..], other].concat()).status.code(),
            Some(2)
        );
        assert!(!p.join("out").exists());
    }
    for (field, value) in [
        ("status", "established"),
        ("scope", " \t\n"),
        ("reported_by", " \t\n"),
        ("unknown", "unexpected field"),
    ] {
        let mut invalid = valid.clone();
        invalid[field] = serde_json::json!(value);
        fs::write(p.join("report"), invalid.to_string()).unwrap();
        let output = run(p, &[&collect[..], &["--contract", "doc"]].concat());
        assert_eq!(output.status.code(), Some(2), "{field}: {output:?}");
        assert!(!p.join("out").exists());
    }
    for malformed in [
        r#"["owner-confirmed-absent","project","owner"]"#,
        r#"{"status":"owner-confirmed-absent","scope":"project"}"#,
        r#"{"status":"owner-confirmed-absent","status":"not-found-in-search-scope","scope":"project","reported_by":"owner"}"#,
    ] {
        fs::write(p.join("report"), malformed).unwrap();
        let output = run(p, &[&collect[..], &["--contract", "doc"]].concat());
        assert_eq!(output.status.code(), Some(2), "{output:?}");
        assert!(!p.join("out").exists());
    }
}
#[test]
fn baseline_only_links_actual_receipt_and_stale_source_is_refused() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    baseline(p);
    assert!(
        run(
            p,
            &[
                "collect",
                "--baseline",
                "intake/baseline.json",
                "--output",
                "out"
            ]
        )
        .status
        .success()
    );
    let v = read(p, "out");
    let hash = digest(&fs::read(p.join("intake/baseline.json")).unwrap());
    assert_eq!(v["baseline"]["sha256"], hash);
    assert_eq!(
        v["baseline"]["evidence"]["baseline_receipt"]["baseline_sha256"],
        hash
    );
    assert_eq!(v["baseline"]["evidence"]["authority"], "observation");
    assert_eq!(v["root"]["finding"], "missing-root-contract");
    assert_eq!(v["documents"], serde_json::json!([]));
    fs::write(p.join("lib.rs"), "pub fn changed() {}\n").unwrap();
    assert_eq!(
        run(
            p,
            &[
                "collect",
                "--baseline",
                "intake/baseline.json",
                "--output",
                "stale"
            ]
        )
        .status
        .code(),
        Some(2)
    );
    assert!(!p.join("stale").exists());
}
#[test]
fn no_clobber_invalid_inputs_and_duplicate_options_create_nothing() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    fs::write(p.join("doc"), "unchanged").unwrap();
    for args in [
        vec!["collect", "--contract", "doc", "--output", "doc"],
        vec![
            "collect", "--root", "doc", "--root", "doc", "--output", "out",
        ],
        vec!["collect", "--output", "out"],
    ] {
        assert_eq!(run(p, &args).status.code(), Some(2));
        assert!(!p.join("out").exists());
    }
    assert_eq!(fs::read(p.join("doc")).unwrap(), b"unchanged");
    fs::write(p.join("invalid"), [255]).unwrap();
    fs::write(p.join("large"), vec![b'x'; 1024 * 1024 + 1]).unwrap();
    for input in ["invalid", "large"] {
        assert_eq!(
            run(p, &["collect", "--contract", input, "--output", "out"])
                .status
                .code(),
            Some(2)
        );
        assert!(!p.join("out").exists());
    }
}
#[cfg(unix)]
#[test]
fn symlinks_and_fifo_are_refused_without_opening_special_inputs() {
    use std::os::unix::fs::symlink;
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    fs::write(p.join("doc"), "original").unwrap();
    symlink(p.join("doc"), p.join("link")).unwrap();
    assert!(
        Command::new("mkfifo")
            .arg(p.join("fifo"))
            .status()
            .unwrap()
            .success()
    );
    for input in ["link", "fifo"] {
        assert_eq!(
            run(p, &["collect", "--contract", input, "--output", "out"])
                .status
                .code(),
            Some(2)
        );
        assert!(!p.join("out").exists());
    }
    symlink(p.join("absent"), p.join("out")).unwrap();
    assert_eq!(
        run(p, &["collect", "--contract", "doc", "--output", "out"])
            .status
            .code(),
        Some(2)
    );
    assert!(
        fs::symlink_metadata(p.join("out"))
            .unwrap()
            .file_type()
            .is_symlink()
    );
}
#[test]
fn encoded_expansion_limit_is_checked_before_publication() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let contracts = (0..8)
        .map(|i| {
            let path = p.join(format!("{i}.txt"));
            fs::write(&path, vec![0; 1024 * 1024]).unwrap();
            path
        })
        .collect();
    let error = match collect(
        &Request {
            contracts,
            ..Request::default()
        },
        &p.join("out"),
    ) {
        Ok(_) => panic!("expanded artifact admitted"),
        Err(e) => e,
    };
    assert!(error.contains("32 MiB"));
    assert!(!p.join("out").exists());
}

#[test]
fn altered_baseline_observations_cannot_supply_a_current_context() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    baseline(p);
    let mut forged = read(p, "intake/baseline.json");
    forged["files"][0]["analysis"]["observations"][0]["excerpt"] =
        serde_json::json!("invented approved requirement");
    fs::write(
        p.join("intake/baseline.json"),
        serde_json::to_vec_pretty(&forged).unwrap(),
    )
    .unwrap();
    let output = run(
        p,
        &[
            "collect",
            "--baseline",
            "intake/baseline.json",
            "--output",
            "out",
        ],
    );
    assert_eq!(output.status.code(), Some(2));
    assert!(!p.join("out").exists());
}

#[test]
fn explicit_run_context_is_separate_bound_and_never_followed() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    fs::write(p.join("doc"), "Contract").unwrap();
    fs::write(
        p.join("task"),
        "Prefer S. 계약\r\nRead /nonexistent/private; model=claimed",
    )
    .unwrap();
    let mut r = Request {
        contracts: vec![p.join("doc")],
        ..Request::default()
    };
    collect(&r, &p.join("v1")).unwrap();
    collect(&r, &p.join("v1copy")).unwrap();
    assert_eq!(
        fs::read(p.join("v1")).unwrap(),
        fs::read(p.join("v1copy")).unwrap()
    );
    assert!(read(p, "v1").get("run_inputs").is_none());
    r.run_inputs = vec![p.join("task"), p.join("./task")];
    collect(&r, &p.join("v2")).unwrap();
    let v = read(p, "v2");
    assert_eq!(v["format"], "caac-contract-collection-v2");
    assert_eq!(v["run_inputs"].as_array().unwrap().len(), 1);
    assert_eq!(
        v["run_inputs"][0]["text"],
        fs::read_to_string(p.join("task")).unwrap()
    );
    assert_eq!(v["run_inputs"][0]["role"], "caller-supplied-run-input");
    assert_eq!(v["documents"].as_array().unwrap().len(), 1);
    caac::review_packet::prepare_review(&p.join("v2"), &r, &p.join("packet")).unwrap();
    let packet = read(p, "packet");
    assert_eq!(packet["collection_profile"], "exact-canonical-collect-v2");
    assert_eq!(packet["evidence"].as_array().unwrap().len(), 1);
    fs::write(p.join("task"), "Assess alternatives neutrally.").unwrap();
    collect(&r, &p.join("neutral")).unwrap();
    assert_ne!(
        digest(&fs::read(p.join("v2")).unwrap()),
        digest(&fs::read(p.join("neutral")).unwrap())
    );
    assert!(caac::review_packet::prepare_review(&p.join("v2"), &r, &p.join("stale")).is_err());
    assert!(!p.join("stale").exists());
}
#[test]
fn run_context_cli_and_shared_limits_fail_closed() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    fs::write(p.join("doc"), "Contract").unwrap();
    fs::write(p.join("task"), "neutral").unwrap();
    assert!(
        run(
            p,
            &[
                "collect",
                "--contract",
                "doc",
                "--run-input",
                "task",
                "--output",
                "out"
            ]
        )
        .status
        .success()
    );
    let r = Request {
        contracts: vec![p.join("doc")],
        run_inputs: vec![p.join("task"); 128],
        ..Request::default()
    };
    assert!(collect(&r, &p.join("too-many")).is_err());
    fs::write(p.join("large"), vec![b'a'; 1024 * 1024 + 1]).unwrap();
    assert_eq!(
        run(
            p,
            &[
                "collect",
                "--contract",
                "doc",
                "--run-input",
                "large",
                "--output",
                "bad"
            ]
        )
        .status
        .code(),
        Some(2)
    );
    assert!(!p.join("bad").exists());
    let mut combined = Request {
        contracts: vec![p.join("doc")],
        ..Request::default()
    };
    for i in 0..8 {
        let path = p.join(format!("context-{i}"));
        fs::write(&path, vec![b'a'; 1024 * 1024]).unwrap();
        combined.run_inputs.push(path);
    }
    assert!(
        collect(&combined, &p.join("over-total"))
            .err()
            .unwrap()
            .contains("8 MiB")
    );
    assert!(!p.join("over-total").exists());
    assert_eq!(
        run(
            p,
            &["collect", "--run-input", "task", "--output", "context-only"]
        )
        .status
        .code(),
        Some(2)
    );
    assert!(!p.join("context-only").exists());
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(p.join("task"), p.join("link")).unwrap();
        assert!(
            Command::new("mkfifo")
                .arg(p.join("fifo"))
                .status()
                .unwrap()
                .success()
        );
        for file in ["link", "fifo"] {
            assert_eq!(
                run(
                    p,
                    &[
                        "collect",
                        "--contract",
                        "doc",
                        "--run-input",
                        file,
                        "--output",
                        "bad"
                    ]
                )
                .status
                .code(),
                Some(2)
            );
            assert!(!p.join("bad").exists());
        }
    }
}
