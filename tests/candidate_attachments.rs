use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn run(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_caac"))
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap()
}

fn read(dir: &Path, path: &str) -> Value {
    serde_json::from_slice(&fs::read(dir.join(path)).unwrap()).unwrap()
}

fn write(dir: &Path, path: &str, value: &Value) {
    fs::write(dir.join(path), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn attachment_fixture() -> Value {
    let mut candidate: Value = serde_json::from_slice(include_bytes!(
        "../examples/candidate/shared-retry-draft.json"
    ))
    .unwrap();
    let entities = candidate["semantic"]["entities"].as_array_mut().unwrap();
    let retry_rule = entities
        .iter_mut()
        .find(|entity| entity["id"] == "rule-retry-limit")
        .unwrap();
    retry_rule["text"] = json!(
        "The retry policy returns a retry limit of 3. It applies the priority exception only to priority jobs."
    );
    retry_rule["annex_refs"] = json!([
        {"id":"annex-priority-rules", "usage":"binding"},
        {"id":"annex-priority-example", "usage":"illustrative"}
    ]);
    retry_rule["form_refs"] = json!([
        {"id":"form-job-request", "usage":"binding"}
    ]);

    let exception = entities
        .iter_mut()
        .find(|entity| entity["id"] == "except-priority")
        .unwrap();
    exception["sentence"] = json!("sentence-priority-first");

    entities.extend([
        json!({
            "kind":"annex", "id":"annex-priority-rules", "title":"Priority retry limits",
            "status":"binding", "columns":["priority", "retry_limit"],
            "rows":[[true, 3], [false, 1]]
        }),
        json!({
            "kind":"annex", "id":"annex-priority-example", "title":"Priority job example",
            "status":"illustrative", "columns":["scenario"],
            "rows":[["standard job"]]
        }),
        json!({
            "kind":"form", "id":"form-job-request", "title":"Job request",
            "status":"binding", "fields":[
                {"id":"job_id", "label":"Job ID", "value_type":"string", "required":true},
                {"id":"priority", "label":"Priority job", "value_type":"bool", "required":true}
            ]
        }),
        json!({
            "kind":"sentence-part", "id":"sentence-priority-first", "clause":"rule-retry-limit",
            "portion":"first", "text":"The retry policy returns a retry limit of 3."
        }),
        json!({
            "kind":"sentence-part", "id":"sentence-priority-second", "clause":"rule-retry-limit",
            "portion":"second", "text":"It applies the priority exception only to priority jobs."
        }),
        json!({
            "kind":"clause", "id":"rule-queue-retention", "actor":"Queue",
            "modality":"obligation", "text":"The queue stores every valid request. It preserves the saved order.",
            "definition_refs":[], "annex_refs":[], "form_refs":[]
        }),
        json!({
            "kind":"sentence-part", "id":"sentence-queue-first", "clause":"rule-queue-retention",
            "portion":"first", "text":"The queue stores every valid request."
        }),
    ]);

    candidate["presentation"]["nodes"][0]["articles"][0]["entities"]
        .as_array_mut()
        .unwrap()
        .extend([
            json!("sentence-priority-first"),
            json!("sentence-priority-second"),
            json!("rule-queue-retention"),
            json!("sentence-queue-first"),
        ]);
    candidate["presentation"]["attachments"] = json!([
        "annex-priority-rules",
        "form-job-request",
        "annex-priority-example"
    ]);
    candidate
}

fn apply(dir: &Path, candidate: &Value, output: &str) -> Output {
    let bytes = serde_json::to_vec(candidate).unwrap();
    fs::write(dir.join("base.json"), &bytes).unwrap();
    write(
        dir,
        "answers.json",
        &json!({"expected_base":caac::candidate::digest(&bytes),"answers":[]}),
    );
    run(
        dir,
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

fn verify(dir: &Path, package: &str, trace: Option<&str>) -> Output {
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
    run(dir, &args)
}

fn report(dir: &Path, package: &str, id: &str) -> Value {
    let output = verify(dir, package, Some(id));
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    serde_json::from_slice(&output.stdout).unwrap()
}

fn assert_edge(trace: &Value, field: &str, target_id: &str) {
    assert!(
        trace["direct_outgoing"]
            .as_array()
            .unwrap()
            .iter()
            .any(|edge| { edge["field"] == field && edge["target"]["id"] == target_id }),
        "missing {field} edge to {target_id}: {trace}"
    );
}

fn assert_attachment_edge(trace: &Value, field: &str, target_id: &str, usage: &str) {
    assert!(
        trace["direct_outgoing"]
            .as_array()
            .unwrap()
            .iter()
            .any(|edge| {
                edge["field"] == field
                    && edge["target"]["id"] == target_id
                    && edge["attachment_usage"] == usage
            }),
        "missing {field} edge to {target_id} with {usage} usage: {trace}"
    );
}

fn assert_attachment_location(trace: &Value, file: &str, node_id: &str, number: usize) {
    let mapping = trace["selected"]["mapping"].as_array().unwrap();
    assert_eq!(mapping.len(), 1);
    assert_eq!(mapping[0]["file"], file);
    assert_eq!(mapping[0]["location"]["kind"], "attachment");
    assert_eq!(mapping[0]["location"]["node_id"], node_id);
    assert_eq!(mapping[0]["location"]["number"], number);
}

fn xl_presentation(candidate: &Value) -> Value {
    let intake_articles = candidate["presentation"]["nodes"][0]["articles"].clone();
    let replay_articles = candidate["presentation"]["nodes"][1]["articles"].clone();
    json!({
        "size":"XL",
        "nodes":[
            {
                "kind":"module", "id":"module-intake", "title":"Intake", "file":"intake.md",
                "articles":intake_articles, "decisions":["decision-policy-owner"],
                "attachments":["annex-priority-rules", "form-job-request"]
            },
            {
                "kind":"module", "id":"module-replay", "title":"Replay", "file":"replay.md",
                "articles":replay_articles, "decisions":[],
                "attachments":["annex-priority-example"]
            }
        ]
    })
}

fn revise(dir: &Path, package: &str, presentation: Value, output: &str) -> Output {
    let candidate = format!("{package}/candidate.json");
    let provenance = format!("{package}/provenance.json");
    let review = format!("{package}/review.md");
    let source = read(dir, &provenance);
    write(
        dir,
        "presentation-request.json",
        &json!({
            "format":"caac-presentation-request-v1",
            "expected_candidate_sha256":source["candidate_sha256"],
            "expected_package_revision":source["revision_manifest"]["package_revision"],
            "presentation":presentation
        }),
    );
    run(
        dir,
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
            "presentation-request.json",
            "--output",
            output,
        ],
    )
}

#[test]
fn v8_attachments_trace_and_move_from_single_file_to_xl_modules() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    let output = apply(dir, &attachment_fixture(), "source");
    assert_eq!(output.status.code(), Some(0), "{output:?}");

    let candidate = read(dir, "source/candidate.json");
    let provenance = read(dir, "source/provenance.json");
    let review = fs::read_to_string(dir.join("source/review.md")).unwrap();
    assert_eq!(
        provenance["revision_manifest"]["format"],
        "caac-candidate-revisions-v8"
    );
    assert!(review.contains("Renderer profile: `caac-candidate-legal-review-v8`"));
    assert!(review.contains("## Attachments"));
    assert!(review.contains("**Table** (same source as the exact entity)"));
    assert!(review.contains("| ` true ` | ` 3 ` |"));
    assert!(review.contains("**Form fields** (same source as the exact entity)"));
    assert!(review.contains("Job ID"));
    assert!(review.contains("Priority job"));
    assert_eq!(
        candidate["presentation"]["attachments"],
        json!([
            "annex-priority-rules",
            "form-job-request",
            "annex-priority-example"
        ])
    );
    let article_entities = candidate["presentation"]["nodes"][0]["articles"][0]["entities"]
        .as_array()
        .unwrap();
    for id in ["sentence-priority-first", "sentence-priority-second"] {
        assert!(article_entities.iter().any(|placed| placed == id));
    }
    for id in [
        "annex-priority-rules",
        "annex-priority-example",
        "form-job-request",
    ] {
        assert!(!article_entities.iter().any(|placed| placed == id));
    }

    for (id, number) in [
        ("annex-priority-rules", 1),
        ("form-job-request", 2),
        ("annex-priority-example", 3),
    ] {
        let trace = report(dir, "source", id);
        assert_attachment_location(&trace, "review.md", id, number);
    }

    let clause = report(dir, "source", "rule-retry-limit");
    assert_eq!(
        clause["selected"]["body"]["annex_refs"],
        json!([
            {"id":"annex-priority-rules", "usage":"binding"},
            {"id":"annex-priority-example", "usage":"illustrative"}
        ])
    );
    assert_eq!(
        clause["selected"]["body"]["form_refs"],
        json!([{"id":"form-job-request", "usage":"binding"}])
    );
    assert_attachment_edge(&clause, "annex_refs", "annex-priority-rules", "binding");
    assert_attachment_edge(
        &clause,
        "annex_refs",
        "annex-priority-example",
        "illustrative",
    );
    assert_attachment_edge(&clause, "form_refs", "form-job-request", "binding");

    let sentence = report(dir, "source", "sentence-priority-first");
    assert_eq!(sentence["selected"]["body"]["clause"], "rule-retry-limit");
    assert_eq!(sentence["selected"]["body"]["portion"], "first");
    assert_edge(&sentence, "sentence_clause", "rule-retry-limit");
    let second_sentence = report(dir, "source", "sentence-priority-second");
    assert_eq!(
        second_sentence["selected"]["body"]["clause"],
        "rule-retry-limit"
    );
    assert_eq!(second_sentence["selected"]["body"]["portion"], "second");
    assert_edge(&second_sentence, "sentence_clause", "rule-retry-limit");
    let exception = report(dir, "source", "except-priority");
    assert_eq!(exception["selected"]["body"]["target"], "rule-retry-limit");
    assert_eq!(
        exception["selected"]["body"]["sentence"],
        "sentence-priority-first"
    );
    assert_edge(&exception, "target", "rule-retry-limit");
    assert_edge(&exception, "sentence", "sentence-priority-first");

    let output = revise(dir, "source", xl_presentation(&candidate), "revised");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let verify = verify(dir, "revised", None);
    assert_eq!(verify.status.code(), Some(0), "{verify:?}");

    let revised_candidate = read(dir, "revised/candidate.json");
    let revised_provenance = read(dir, "revised/provenance.json");
    assert_eq!(candidate["semantic"], revised_candidate["semantic"]);
    assert_eq!(
        provenance["revision_manifest"]["semantic_revision"],
        revised_provenance["revision_manifest"]["semantic_revision"]
    );
    assert_eq!(
        provenance["revision_manifest"]["objects"],
        revised_provenance["revision_manifest"]["objects"]
    );
    assert_ne!(
        provenance["revision_manifest"]["presentation_revision"],
        revised_provenance["revision_manifest"]["presentation_revision"]
    );
    for (id, file, number) in [
        ("annex-priority-rules", "intake.md", 1),
        ("form-job-request", "intake.md", 2),
        ("annex-priority-example", "replay.md", 1),
    ] {
        let trace = report(dir, "revised", id);
        assert_attachment_location(&trace, file, id, number);
    }
    let intake = fs::read_to_string(dir.join("revised/intake.md")).unwrap();
    let replay = fs::read_to_string(dir.join("revised/replay.md")).unwrap();
    assert!(intake.contains("## Attachments") && intake.contains("Job ID"));
    assert!(replay.contains("## Attachments") && replay.contains("standard job"));
}

#[test]
fn xl_article_move_preserves_attachment_and_sentence_exception_bindings() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    let output = apply(dir, &attachment_fixture(), "source");
    assert_eq!(output.status.code(), Some(0), "{output:?}");

    let source = read(dir, "source/candidate.json");
    let output = revise(dir, "source", xl_presentation(&source), "split");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let split = read(dir, "split/candidate.json");
    assert_eq!(split["presentation"]["size"], "XL");

    let mut presentation = split["presentation"].clone();
    let article = presentation["nodes"][0]["articles"]
        .as_array_mut()
        .unwrap()
        .remove(0);
    assert_eq!(article["id"], "article-policy");
    presentation["nodes"][1]["articles"]
        .as_array_mut()
        .unwrap()
        .insert(0, article);
    let output = revise(dir, "split", presentation, "moved");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let output = verify(dir, "moved", None);
    assert_eq!(output.status.code(), Some(0), "{output:?}");

    let mut moved = read(dir, "moved/candidate.json");
    assert_eq!(moved["presentation"]["size"], "XL");
    assert_eq!(split["semantic"], moved["semantic"]);
    assert_eq!(split["authority"], moved["authority"]);

    let before = &read(dir, "split/provenance.json")["revision_manifest"];
    let after = &read(dir, "moved/provenance.json")["revision_manifest"];
    for field in ["semantic_revision", "objects"] {
        assert_eq!(before[field], after[field], "{field}");
    }
    for field in ["presentation_revision", "package_revision"] {
        assert_ne!(before[field], after[field], "{field}");
    }

    for (package, file, number, revision) in [
        ("split", "intake.md", 1, &before["package_revision"]),
        ("moved", "replay.md", 2, &after["package_revision"]),
    ] {
        let clause = report(dir, package, "rule-retry-limit");
        let sentence = report(dir, package, "sentence-priority-first");
        let exception = report(dir, package, "except-priority");
        for trace in [&clause, &sentence, &exception] {
            let mapping = trace["selected"]["mapping"].as_array().unwrap();
            assert_eq!(mapping.len(), 1);
            assert_eq!(mapping[0]["file"], file);
            assert_eq!(mapping[0]["location"]["kind"], "article");
            assert_eq!(mapping[0]["location"]["node_id"], "article-policy");
            assert_eq!(mapping[0]["location"]["number"], number);
            assert_eq!(
                &trace["selected"]["reference"]["package_revision"],
                revision
            );
        }

        let annex = report(dir, package, "annex-priority-rules");
        assert_attachment_location(&annex, "intake.md", "annex-priority-rules", 1);
        assert_eq!(annex["selected"]["reference"]["id"], "annex-priority-rules");
        assert_eq!(
            &annex["selected"]["reference"]["package_revision"],
            revision
        );
        assert_eq!(annex["selected"]["body"]["status"], "binding");
        assert_eq!(sentence["selected"]["body"]["portion"], "first");
        assert_eq!(
            exception["selected"]["body"]["applicability"],
            json!({"kind":"input-equals", "field":"priority", "value":true})
        );
        assert_eq!(
            exception["selected"]["body"]["effect"],
            json!({"kind":"replace-result", "value":5})
        );
        for (source, field, target, usage) in [
            (&clause, "annex_refs", &annex, json!("binding")),
            (&sentence, "sentence_clause", &clause, Value::Null),
            (&exception, "target", &clause, Value::Null),
            (&exception, "sentence", &sentence, Value::Null),
        ] {
            assert!(
                source["direct_outgoing"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|edge| edge["field"] == field
                        && edge["target"] == target["selected"]["reference"]
                        && edge["attachment_usage"] == usage),
                "missing {field} binding in {package}: {source}"
            );
        }
    }

    let intake = fs::read_to_string(dir.join("moved/intake.md")).unwrap();
    let replay = fs::read_to_string(dir.join("moved/replay.md")).unwrap();
    assert!(replay.contains("Article 2 — ` \"Retry policy\" `"));
    for marker in [
        "Presentation ID: ` \"article-policy\" `",
        "Clause · ` \"rule-retry-limit\" `",
        "Exception · ` \"except-priority\" `",
        "Sentence-Part · ` \"sentence-priority-first\" `",
    ] {
        assert!(replay.contains(marker), "Replay omits {marker}");
        assert!(!intake.contains(marker), "Intake still includes {marker}");
    }
    let annex_heading = "Annex · ` \"annex-priority-rules\" `";
    assert!(intake.contains(annex_heading));
    assert!(!replay.contains(annex_heading));
    assert!(replay.contains("**Sentence reference:** ` \"sentence-priority-first\" `"));

    let exception = moved["semantic"]["entities"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entity| entity["id"] == "except-priority")
        .unwrap();
    exception["sentence"] = json!("sentence-queue-first");
    let output = apply(dir, &moved, "broken");
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("Exception sentence must belong to its target clause"),
        "{output:?}"
    );
    assert!(!dir.join("broken").exists());
}

#[test]
fn invalid_attachment_and_sentence_links_fail_before_output() {
    let expected_errors = [
        "Reference form-job-request must resolve to annex",
        "Attachment reference annex-priority-rules must match its status",
        "Exception sentence must belong to its target clause",
        "Sentence parts of the same clause overlap",
    ];
    for (mode, expected_error) in expected_errors.into_iter().enumerate() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path();
        let mut candidate = attachment_fixture();
        let entities = candidate["semantic"]["entities"].as_array_mut().unwrap();
        match mode {
            0 => {
                let clause = entities
                    .iter_mut()
                    .find(|entity| entity["id"] == "rule-retry-limit")
                    .unwrap();
                clause["annex_refs"][0]["id"] = json!("form-job-request");
            }
            1 => {
                let clause = entities
                    .iter_mut()
                    .find(|entity| entity["id"] == "rule-retry-limit")
                    .unwrap();
                clause["annex_refs"][0]["usage"] = json!("illustrative");
            }
            2 => {
                let exception = entities
                    .iter_mut()
                    .find(|entity| entity["id"] == "except-priority")
                    .unwrap();
                exception["sentence"] = json!("sentence-queue-first");
            }
            _ => {
                let first = entities
                    .iter_mut()
                    .find(|entity| entity["id"] == "sentence-priority-first")
                    .unwrap();
                first["text"] = json!(
                    "The retry policy returns a retry limit of 3. It applies the priority exception only to priority"
                );
            }
        }
        let output = apply(dir, &candidate, "invalid");
        assert_eq!(output.status.code(), Some(2), "mode {mode}: {output:?}");
        assert!(output.stdout.is_empty(), "mode {mode}: {output:?}");
        assert!(!output.stderr.is_empty(), "mode {mode}: {output:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(expected_error),
            "mode {mode}: {output:?}"
        );
        assert!(!dir.join("invalid").exists(), "mode {mode}");
    }
}
