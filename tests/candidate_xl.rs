use serde_json::{Value, json};
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

fn read(d: &Path, path: &str) -> Value {
    serde_json::from_slice(&fs::read(d.join(path)).unwrap()).unwrap()
}

fn write(d: &Path, path: &str, value: &Value) {
    fs::write(d.join(path), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn xl_fixture() -> Value {
    let mut candidate: Value = serde_json::from_slice(include_bytes!(
        "../examples/candidate/shared-retry-draft.json"
    ))
    .unwrap();
    let chapters = candidate["presentation"]["nodes"].as_array().unwrap();
    let intake = chapters[0]["articles"].clone();
    let replay = chapters[1]["articles"].clone();
    candidate["presentation"] = json!({
        "size": "XL",
        "nodes": [
            {"kind":"module", "id":"module-intake", "title":"Intake",
             "file":"intake.md", "articles":intake, "decisions":["decision-policy-owner"]},
            {"kind":"module", "id":"module-replay", "title":"Replay",
             "file":"replay.md", "articles":replay, "decisions":[]}
        ]
    });
    candidate
}

fn apply(d: &Path, candidate: &Value, output: &str) -> Output {
    let base = serde_json::to_vec(candidate).unwrap();
    fs::write(d.join("base.json"), &base).unwrap();
    write(
        d,
        "answers.json",
        &json!({"expected_base":caac::candidate::digest(&base),"answers":[]}),
    );
    run(
        d,
        &[
            "candidate",
            "apply",
            "--base",
            "base.json",
            "--answers",
            "answers.json",
            "--output",
            output,
        ],
    )
}

fn setup() -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    let output = apply(d.path(), &xl_fixture(), "source");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    d
}

fn verify(d: &Path, package: &str, trace: Option<&str>) -> Output {
    let candidate = format!("{package}/candidate.json");
    let provenance = format!("{package}/provenance.json");
    let review = format!("{package}/review.md");
    let mut args = vec![
        "candidate",
        "verify",
        "--candidate",
        &candidate,
        "--provenance",
        &provenance,
        "--review",
        &review,
    ];
    if let Some(id) = trace {
        args.extend(["--trace", id]);
    }
    run(d, &args)
}

fn report(d: &Path, package: &str, id: &str) -> Value {
    let output = verify(d, package, Some(id));
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty());
    serde_json::from_slice(&output.stdout).unwrap()
}

fn assert_location(trace: &Value, file: &str, article: &str, number: usize) {
    let locations = trace["selected"]["mapping"].as_array().unwrap();
    assert_eq!(locations.len(), 1);
    assert_eq!(locations[0]["file"], file);
    assert_eq!(locations[0]["location"]["kind"], "article");
    assert_eq!(locations[0]["location"]["node_id"], article);
    assert_eq!(locations[0]["location"]["number"], number);
}

fn prepare_move(d: &Path) {
    let candidate = read(d, "source/candidate.json");
    let provenance = read(d, "source/provenance.json");
    let mut presentation = candidate["presentation"].clone();
    let policy = presentation["nodes"][0]["articles"]
        .as_array_mut()
        .unwrap()
        .remove(0);
    presentation["nodes"][1]["articles"]
        .as_array_mut()
        .unwrap()
        .insert(0, policy);
    write(
        d,
        "request.json",
        &json!({
            "format":"caac-presentation-request-v1",
            "expected_candidate_sha256":provenance["candidate_sha256"],
            "expected_package_revision":provenance["revision_manifest"]["package_revision"],
            "presentation":presentation
        }),
    );
}

fn revise(d: &Path) -> Output {
    revise_from(d, "source", "revised")
}

fn revise_from(d: &Path, package: &str, output: &str) -> Output {
    let candidate = format!("{package}/candidate.json");
    let provenance = format!("{package}/provenance.json");
    let review = format!("{package}/review.md");
    run(
        d,
        &[
            "candidate",
            "revise",
            "--candidate",
            &candidate,
            "--provenance",
            &provenance,
            "--review",
            &review,
            "--presentation",
            "request.json",
            "--output",
            output,
        ],
    )
}

#[test]
fn xl_split_publishes_an_index_and_traces_shared_objects() {
    let d = setup();
    let path = d.path();
    for file in [
        "candidate.json",
        "provenance.json",
        "review.md",
        "intake.md",
        "replay.md",
    ] {
        assert!(path.join("source").join(file).is_file(), "{file}");
    }
    let output = verify(path, "source", None);
    assert_eq!(output.status.code(), Some(0), "{output:?}");

    let index = fs::read_to_string(path.join("source/review.md")).unwrap();
    let intake = fs::read_to_string(path.join("source/intake.md")).unwrap();
    let replay = fs::read_to_string(path.join("source/replay.md")).unwrap();
    assert!(index.contains("intake.md") && index.contains("replay.md"));
    for text in [
        "A priority job has the priority input set to true.",
        "The retry policy returns a retry limit of 3.",
        "Who can approve changes to the shared retry policy?",
    ] {
        assert!(!index.contains(text), "index contains {text}");
        assert!(intake.contains(text), "intake omits {text}");
        assert!(!replay.contains(text), "replay contains {text}");
    }
    assert!(intake.contains("Article 1") && intake.contains("Article 2"));
    assert!(replay.contains("Article 3"));

    let rule = report(path, "source", "rule-retry-limit");
    let definition = report(path, "source", "term-priority");
    let exception = report(path, "source", "except-priority");
    let intake_operation = report(path, "source", "operation-intake");
    let replay_operation = report(path, "source", "operation-replay");
    let replay_case = report(path, "source", "test-replay-priority");
    for trace in [&rule, &definition, &exception] {
        assert_location(trace, "intake.md", "article-policy", 1);
    }
    assert_location(&intake_operation, "intake.md", "article-intake", 2);
    assert_location(&replay_operation, "replay.md", "article-replay", 3);
    assert_location(&replay_case, "replay.md", "article-replay", 3);
    let decision = report(path, "source", "decision-policy-owner");
    assert_eq!(decision["selected"]["mapping"].as_array().unwrap().len(), 1);
    assert_eq!(decision["selected"]["mapping"][0]["file"], "intake.md");
    assert_eq!(
        decision["selected"]["mapping"][0]["location"]["kind"],
        "unresolved-decision"
    );
    assert_eq!(decision["selected"]["body"]["required"], true);

    let address = &rule["selected"]["reference"];
    for operation in [&intake_operation, &replay_operation] {
        assert_eq!(operation["direct_outgoing"][0]["field"], "clause_refs");
        assert_eq!(&operation["direct_outgoing"][0]["target"], address);
    }
    assert_eq!(
        rule["direct_outgoing"][0]["target"],
        definition["selected"]["reference"]
    );
    assert_eq!(&exception["direct_outgoing"][0]["target"], address);
    assert!(
        replay_case["direct_outgoing"]
            .as_array()
            .unwrap()
            .iter()
            .any(|edge| {
                edge["field"] == "exception_refs"
                    && edge["target"] == exception["selected"]["reference"]
            })
    );
    let cases = rule["reachable_declared_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 2);
    assert_eq!(cases[0]["body"]["expected"], 3);
    assert_eq!(cases[1]["body"]["expected"], 5);
    assert_eq!(
        rule["evidence_scope"]["source_behavior_evidence"],
        "not-run"
    );
}

#[test]
fn xl_presentation_move_preserves_objects_and_updates_locations() {
    let d = setup();
    let path = d.path();
    let before_candidate = read(path, "source/candidate.json");
    let before = read(path, "source/provenance.json");
    let before_rule = report(path, "source", "rule-retry-limit");
    prepare_move(path);
    let output = revise(path);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let output = verify(path, "revised", None);
    assert_eq!(output.status.code(), Some(0), "{output:?}");

    let after_candidate = read(path, "revised/candidate.json");
    let after = read(path, "revised/provenance.json");
    assert_eq!(before_candidate["semantic"], after_candidate["semantic"]);
    assert_eq!(before_candidate["authority"], after_candidate["authority"]);
    assert_eq!(
        before["revision_manifest"]["semantic_revision"],
        after["revision_manifest"]["semantic_revision"]
    );
    assert_eq!(
        before["revision_manifest"]["objects"],
        after["revision_manifest"]["objects"]
    );
    assert_ne!(
        before["revision_manifest"]["presentation_revision"],
        after["revision_manifest"]["presentation_revision"]
    );
    assert_ne!(
        before["revision_manifest"]["package_revision"],
        after["revision_manifest"]["package_revision"]
    );

    for id in ["term-priority", "rule-retry-limit", "except-priority"] {
        let old = report(path, "source", id);
        let new = report(path, "revised", id);
        assert_location(&old, "intake.md", "article-policy", 1);
        assert_location(&new, "replay.md", "article-policy", 2);
        assert_eq!(
            old["selected"]["reference"]["id"],
            new["selected"]["reference"]["id"]
        );
        assert_eq!(
            old["selected"]["reference"]["entity_revision"],
            new["selected"]["reference"]["entity_revision"]
        );
        assert_eq!(
            new["selected"]["reference"]["package_revision"],
            after["revision_manifest"]["package_revision"]
        );
    }
    assert_location(
        &report(path, "revised", "operation-intake"),
        "intake.md",
        "article-intake",
        1,
    );
    assert_location(
        &report(path, "revised", "operation-replay"),
        "replay.md",
        "article-replay",
        3,
    );
    let decision = report(path, "revised", "decision-policy-owner");
    assert_eq!(decision["selected"]["mapping"][0]["file"], "intake.md");
    let rule = report(path, "revised", "rule-retry-limit");
    assert_ne!(
        before_rule["selected"]["reference"],
        rule["selected"]["reference"]
    );
    for id in ["operation-intake", "operation-replay"] {
        let operation = report(path, "revised", id);
        assert_eq!(
            operation["direct_outgoing"][0]["target"],
            rule["selected"]["reference"]
        );
    }
    let exception = report(path, "revised", "except-priority");
    assert_eq!(
        exception["direct_outgoing"][0]["target"],
        rule["selected"]["reference"]
    );
    let replay_case = report(path, "revised", "test-replay-priority");
    assert!(
        replay_case["direct_outgoing"]
            .as_array()
            .unwrap()
            .iter()
            .any(|edge| {
                edge["field"] == "exception_refs"
                    && edge["target"] == exception["selected"]["reference"]
            })
    );
    let intake = fs::read_to_string(path.join("revised/intake.md")).unwrap();
    let replay = fs::read_to_string(path.join("revised/replay.md")).unwrap();
    assert!(intake.contains("Article 1"));
    assert!(replay.contains("Article 2") && replay.contains("Article 3"));
    assert!(intake.contains("Who can approve changes to the shared retry policy?"));
}

#[test]
fn m_to_xl_to_m_preserves_semantics_and_restores_the_original_view() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path();
    let m: Value = serde_json::from_slice(include_bytes!(
        "../examples/candidate/shared-retry-draft.json"
    ))
    .unwrap();
    let output = apply(path, &m, "source");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let output = verify(path, "source", None);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let before = read(path, "source/provenance.json");
    let original = read(path, "source/candidate.json");
    let original_candidate_bytes = fs::read(path.join("source/candidate.json")).unwrap();
    let original_review_bytes = fs::read(path.join("source/review.md")).unwrap();
    assert!(!path.join("source/intake.md").exists());
    assert!(!path.join("source/replay.md").exists());
    write(
        path,
        "request.json",
        &json!({
            "format":"caac-presentation-request-v1",
            "expected_candidate_sha256":before["candidate_sha256"],
            "expected_package_revision":before["revision_manifest"]["package_revision"],
            "presentation":xl_fixture()["presentation"]
        }),
    );
    let output = revise(path);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let output = verify(path, "revised", None);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let revised = read(path, "revised/candidate.json");
    let after = read(path, "revised/provenance.json");
    assert_eq!(original["semantic"], revised["semantic"]);
    assert_eq!(original["authority"], revised["authority"]);
    assert_eq!(
        before["revision_manifest"]["objects"],
        after["revision_manifest"]["objects"]
    );
    assert_eq!(
        before["revision_manifest"]["semantic_revision"],
        after["revision_manifest"]["semantic_revision"]
    );
    assert_ne!(
        before["revision_manifest"]["package_revision"],
        after["revision_manifest"]["package_revision"]
    );
    assert!(path.join("revised/intake.md").is_file());
    assert!(path.join("revised/replay.md").is_file());
    assert_location(
        &report(path, "revised", "rule-retry-limit"),
        "intake.md",
        "article-policy",
        1,
    );
    assert_location(
        &report(path, "revised", "operation-replay"),
        "replay.md",
        "article-replay",
        3,
    );

    write(
        path,
        "request.json",
        &json!({
            "format":"caac-presentation-request-v1",
            "expected_candidate_sha256":after["candidate_sha256"],
            "expected_package_revision":after["revision_manifest"]["package_revision"],
            "presentation":original["presentation"]
        }),
    );
    let output = revise_from(path, "revised", "restored");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let output = verify(path, "restored", None);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(
        fs::read(path.join("restored/candidate.json")).unwrap(),
        original_candidate_bytes
    );
    assert_eq!(
        fs::read(path.join("restored/review.md")).unwrap(),
        original_review_bytes
    );
    assert_eq!(
        read(path, "restored/provenance.json")["revision_manifest"],
        before["revision_manifest"]
    );
    assert!(!path.join("restored/intake.md").exists());
    assert!(!path.join("restored/replay.md").exists());
}

#[test]
fn xl_invalid_references_placements_and_files_never_publish() {
    for mode in 0..15 {
        let d = tempfile::tempdir().unwrap();
        let mut candidate = xl_fixture();
        match mode {
            0 => candidate["semantic"]["entities"][4]["clause_refs"][0] = json!("rule-missing"),
            1 => candidate["presentation"]["nodes"][0]["articles"][0]["entities"]
                .as_array_mut()
                .unwrap()
                .push(json!("term-priority")),
            2 => {
                candidate["presentation"]["nodes"][0]["articles"][0]["entities"]
                    .as_array_mut()
                    .unwrap()
                    .pop();
            }
            3 => {
                candidate["presentation"]["nodes"][0]["articles"][0]["entities"][0] =
                    json!("term-missing")
            }
            4 => {
                candidate["presentation"]["nodes"][1]["decisions"] =
                    json!(["decision-policy-owner"])
            }
            5 => candidate["presentation"]["nodes"][0]["decisions"] = json!([]),
            6 => candidate["presentation"]["nodes"][0]["decisions"] = json!(["decision-missing"]),
            7 => candidate["presentation"]["nodes"][1]["file"] = json!("intake.md"),
            8 => candidate["presentation"]["nodes"][1]["file"] = json!("review.md"),
            9 => candidate["presentation"]["nodes"][1]["file"] = json!("../escape.md"),
            10 => candidate["presentation"]["nodes"][1]["file"] = json!("/tmp/escape.md"),
            11 => candidate["presentation"]["nodes"][1]["file"] = json!("nested/replay.md"),
            12 => candidate["presentation"]["nodes"][1]["id"] = json!("module-intake"),
            13 => candidate["presentation"]["nodes"][1]["articles"] = json!([]),
            _ => {
                let second = candidate["presentation"]["nodes"]
                    .as_array_mut()
                    .unwrap()
                    .pop()
                    .unwrap();
                candidate["presentation"]["nodes"][0]["articles"]
                    .as_array_mut()
                    .unwrap()
                    .extend(second["articles"].as_array().unwrap().iter().cloned());
            }
        }
        let output = apply(d.path(), &candidate, "invalid");
        assert_eq!(output.status.code(), Some(2), "mode {mode}: {output:?}");
        assert!(output.stdout.is_empty(), "mode {mode}: {output:?}");
        assert!(!output.stderr.is_empty(), "mode {mode}: {output:?}");
        assert!(!d.path().join("invalid").exists(), "mode {mode}");
        if mode == 0 {
            assert!(String::from_utf8_lossy(&output.stderr).contains("rule-missing"));
        }
    }
}

#[test]
fn xl_missing_or_modified_module_blocks_verify_trace_and_revise() {
    for (file, trace_id) in [
        ("intake.md", "operation-replay"),
        ("replay.md", "rule-retry-limit"),
    ] {
        for missing in [false, true] {
            for package in ["source", "revised"] {
                let d = setup();
                let path = d.path();
                prepare_move(path);
                if package == "revised" {
                    let output = revise(path);
                    assert_eq!(output.status.code(), Some(0), "{output:?}");
                }
                let module = path.join(package).join(file);
                if missing {
                    fs::remove_file(&module).unwrap();
                } else {
                    let mut contents = fs::read(&module).unwrap();
                    contents[0] ^= 1;
                    fs::write(&module, contents).unwrap();
                }
                for output in [
                    verify(path, package, None),
                    verify(path, package, Some(trace_id)),
                ] {
                    assert_eq!(
                        output.status.code(),
                        Some(2),
                        "{package}/{file}, missing={missing}: {output:?}"
                    );
                    assert!(output.stdout.is_empty(), "{output:?}");
                }
                if package == "source" {
                    let output = revise(path);
                    assert_eq!(output.status.code(), Some(2), "{output:?}");
                    assert!(output.stdout.is_empty(), "{output:?}");
                    assert!(!path.join("revised").exists());
                }
            }
        }
    }
}

#[test]
fn xl_compare_review_rejects_index_and_module_edits_before_proposal() {
    let d = setup();
    let path = d.path();
    for file in ["review.md", "intake.md"] {
        let mut edited = fs::read(path.join("source").join(file)).unwrap();
        edited.extend_from_slice(b"\nProposed edit.\n");
        fs::write(path.join("edited.md"), edited).unwrap();
        let output = run(
            path,
            &[
                "candidate",
                "compare-review",
                "--candidate",
                "source/candidate.json",
                "--provenance",
                "source/provenance.json",
                "--review",
                "source/review.md",
                "--edited",
                "edited.md",
                "--output",
                "proposal",
            ],
        );
        assert_eq!(output.status.code(), Some(2), "{file}: {output:?}");
        assert!(output.stdout.is_empty(), "{file}: {output:?}");
        assert!(!path.join("proposal").exists());
    }
}

#[cfg(unix)]
#[test]
fn xl_module_symlink_is_rejected_even_when_its_bytes_match() {
    use std::os::unix::fs::symlink;

    let d = setup();
    let path = d.path();
    prepare_move(path);
    fs::rename(path.join("source/intake.md"), path.join("detached.md")).unwrap();
    symlink("../detached.md", path.join("source/intake.md")).unwrap();
    for output in [
        verify(path, "source", None),
        verify(path, "source", Some("operation-replay")),
        revise(path),
    ] {
        assert_eq!(output.status.code(), Some(2), "{output:?}");
        assert!(output.stdout.is_empty(), "{output:?}");
    }
    assert!(!path.join("revised").exists());
}
