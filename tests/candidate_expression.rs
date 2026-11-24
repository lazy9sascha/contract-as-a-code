use caac::candidate::digest;
use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Output, Stdio},
};

const DRAFT: &str = include_str!("../examples/candidate/expression-draft.json");
const ANSWERS: &str = include_str!("../examples/candidate/expression-answers.json");
const EXPRESSION: &str = "expression-less-than";
const EXPRESSION_CASE: &str = "case-less-than";
const OPERATION: &str = "operation-compare-control";
const OPERATION_CASE: &str = "case-compare-control";
const CLAUSE: &str = "rule-less-than";

fn command(d: &Path, args: &[&str]) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_caac"));
    command.current_dir(d).arg("candidate").args(args);
    command
}

fn from_package(d: &Path, action: &str, package: &str, args: &[&str]) -> Output {
    command(d, &[action])
        .args([
            "--candidate",
            &format!("{package}/candidate.json"),
            "--provenance",
            &format!("{package}/provenance.json"),
            "--review",
            &format!("{package}/review.md"),
        ])
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

fn run_apply(d: &Path, output: &str) -> Output {
    command(
        d,
        &[
            "apply",
            "--base",
            "base.json",
            "--answers",
            "answers.json",
            "--output",
            output,
        ],
    )
    .output()
    .unwrap()
}

fn apply(d: &Path, base: &Value, output: &str) -> Output {
    write(d, "base.json", base);
    write(
        d,
        "answers.json",
        &json!({"expected_base":digest(&fs::read(d.join("base.json")).unwrap()),"answers":[]}),
    );
    run_apply(d, output)
}

fn success(result: Output) -> Output {
    assert!(result.status.success(), "{result:?}");
    result
}

fn rejection(d: &Path, result: Output, path: &str, message: &str) {
    assert_eq!(result.status.code(), Some(2), "{result:?}");
    assert!(
        String::from_utf8_lossy(&result.stderr).contains(message),
        "{path}: expected {message:?} in {result:?}"
    );
    assert!(!d.join(path).exists(), "published {path}");
}

fn verify(d: &Path, package: &str) {
    success(from_package(d, "verify", package, &[]));
}

fn trace(d: &Path, package: &str, id: &str) -> Value {
    let result = success(from_package(d, "verify", package, &["--trace", id]));
    serde_json::from_slice(&result.stdout).unwrap()
}

fn receipt(d: &Path, package: &str) -> Value {
    read(d, &format!("{package}/provenance.json"))
}

fn revise(d: &Path, source: &str, presentation: &Value, output: &str) -> Output {
    let parent = receipt(d, source);
    write(
        d,
        "request.json",
        &json!({
            "format": "caac-presentation-request-v1",
            "expected_candidate_sha256": parent["candidate_sha256"],
            "expected_package_revision": parent["revision_manifest"]["package_revision"],
            "presentation": presentation,
        }),
    );
    from_package(
        d,
        "revise",
        source,
        &["--presentation", "request.json", "--output", output],
    )
}

fn draft() -> Value {
    serde_json::from_str(DRAFT).unwrap()
}

fn place(base: &mut Value, entity: Value) {
    base["presentation"]["nodes"][0]["entities"]
        .as_array_mut()
        .unwrap()
        .push(entity["id"].clone());
    base["semantic"]["entities"]
        .as_array_mut()
        .unwrap()
        .push(entity);
}

fn mixed_s() -> Value {
    let mut base = draft();
    base["presentation"]["size"] = json!("S");
    place(
        &mut base,
        json!({
            "kind":"operation","id":OPERATION,"name":"compare_control",
            "inputs":{"left":"i32","right":"i32"},"output":"bool","clause_refs":[CLAUSE],"exclusions":[]
        }),
    );
    let mut case = base["semantic"]["entities"][2].clone();
    case.as_object_mut().unwrap().remove("expression");
    case["id"] = json!(OPERATION_CASE);
    case["operation"] = json!(OPERATION);
    place(&mut base, case);
    base
}

fn exact_address(manifest: &Value, id: &str) -> Value {
    let object = manifest["objects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|obj| obj["id"] == id)
        .unwrap();
    json!({
        "namespace": manifest["namespace"],
        "package_revision": manifest["package_revision"],
        "kind": object["kind"],
        "id": id,
        "entity_revision": object["revision"]
    })
}

#[test]
fn xs_expression_applies_verifies_and_round_trips_exact_bytes() {
    let temp = tempfile::tempdir().unwrap();
    let d = temp.path();
    fs::write(d.join("base.json"), DRAFT).unwrap();
    fs::write(d.join("answers.json"), ANSWERS).unwrap();
    success(run_apply(d, "xs"));
    let provenance = receipt(d, "xs");
    let manifest = &provenance["revision_manifest"];
    assert_eq!(provenance["format"], "caac-candidate-provenance-v3");
    assert_eq!(provenance["base_sha256"], digest(DRAFT.as_bytes()));
    assert_eq!(provenance["answers_sha256"], digest(ANSWERS.as_bytes()));
    assert_eq!(
        provenance["renderer_profile"],
        "caac-candidate-legal-review-v9"
    );
    assert_eq!(manifest["format"], "caac-candidate-revisions-v9");
    assert_eq!(provenance["behavior_evidence"], "not-run");
    assert_eq!(provenance["owner_acceptance"], "pending");
    assert_eq!(provenance["implementation_admission"], "pending");
    assert_eq!(provenance["contract_complete"], false);
    assert_eq!(read(d, "xs/candidate.json"), draft());

    let review = fs::read_to_string(d.join("xs/review.md")).unwrap();
    for line in [
        "## ` \"left < right\" `",
        "### Expression · ` \"expression-less-than\" `",
        "- **Authored text:** ` \"left < right\" `",
        "- **Declared input types:** ` {\"left\":\"i32\",\"right\":\"i32\"} `",
        "- **Declared result type:** ` \"bool\" `",
        "- **Clause references:** ` [\"rule-less-than\"] `",
        "### Case · ` \"case-less-than\" `",
        "- **Expected result (not executed):** ` true `",
        "Behavior cases: not run.",
    ] {
        assert!(review.contains(line), "{line}");
    }
    assert!(!review.contains("## Article"));
    let before = trace(d, "xs", EXPRESSION);
    assert_eq!(
        before["selected"]["reference"],
        exact_address(manifest, EXPRESSION)
    );
    assert_eq!(
        before["selected"]["mapping"][0]["location"],
        json!({
            "kind": "unnumbered-article", "node_id": "article-expression"
        })
    );
    assert_eq!(
        before["direct_incoming"],
        json!([{
            "source": exact_address(manifest, EXPRESSION_CASE), "field": "expression",
            "index": null, "target": exact_address(manifest, EXPRESSION)
        }])
    );
    assert_eq!(
        before["direct_outgoing"],
        json!([{
            "source": exact_address(manifest, EXPRESSION), "field": "clause_refs",
            "index": 0, "target": exact_address(manifest, CLAUSE)
        }])
    );
    assert_eq!(
        before["reachable_declared_cases"][0]["body"]["expected"],
        true
    );
    assert_eq!(
        before["evidence_scope"]["source_behavior_evidence"],
        "not-run"
    );
    assert_eq!(before["evidence_scope"]["owner_acceptance"], "pending");

    let mut flat = draft()["presentation"].clone();
    flat["size"] = json!("S");
    success(revise(d, "xs", &flat, "s"));
    let mid = receipt(d, "s");
    assert_eq!(mid["format"], "caac-candidate-provenance-v5");
    assert_eq!(mid["renderer_profile"], provenance["renderer_profile"]);
    assert_eq!(mid["revision_manifest"]["format"], manifest["format"]);
    assert_eq!(mid["semantic_sha256"], provenance["semantic_sha256"]);
    for field in ["semantic_revision", "objects"] {
        assert_eq!(mid["revision_manifest"][field], manifest[field]);
    }
    for field in ["presentation_revision", "package_revision"] {
        assert_ne!(mid["revision_manifest"][field], manifest[field]);
    }
    assert_eq!(
        mid["revision_transition"]["parent_provenance_sha256"],
        digest(&fs::read(d.join("xs/provenance.json")).unwrap())
    );
    assert_eq!(
        mid["revision_transition"]["input_presentation_sha256"],
        digest(&fs::read(d.join("request.json")).unwrap())
    );
    assert_eq!(
        mid["revision_transition"]["evidence_policy"],
        "fresh-review-required-for-whole-package"
    );
    assert_eq!(
        trace(d, "s", EXPRESSION)["selected"]["mapping"][0]["location"],
        json!({
            "kind": "article", "node_id": "article-expression", "chapter_id": null, "number": 1
        })
    );
    assert!(
        fs::read_to_string(d.join("s/review.md"))
            .unwrap()
            .contains("## Article 1")
    );

    success(revise(d, "s", &draft()["presentation"], "restored"));
    verify(d, "restored");
    for file in ["candidate.json", "review.md"] {
        assert_eq!(
            fs::read(d.join(format!("xs/{file}"))).unwrap(),
            fs::read(d.join(format!("restored/{file}"))).unwrap()
        );
    }
    let restored = receipt(d, "restored");
    assert_eq!(restored["revision_manifest"], *manifest);
    assert_eq!(
        restored["revision_transition"]["parent_package_revision"],
        mid["revision_manifest"]["package_revision"]
    );
    assert_eq!(restored["behavior_evidence"], "not-run");
    assert_ne!(restored, provenance);
}

#[test]
fn flat_s_cases_use_separate_edges_and_reject_ambiguous_or_wrong_kind_targets() {
    let temp = tempfile::tempdir().unwrap();
    let d = temp.path();
    let base = mixed_s();
    success(apply(d, &base, "mixed"));
    let manifest = receipt(d, "mixed")["revision_manifest"].clone();
    for (case, field, target) in [
        (EXPRESSION_CASE, "expression", EXPRESSION),
        (OPERATION_CASE, "operation", OPERATION),
    ] {
        let traced = trace(d, "mixed", case);
        let edges = traced["direct_outgoing"].as_array().unwrap();
        assert_eq!(edges.len(), 2);
        assert_eq!(
            edges[0],
            json!({
                "source": exact_address(&manifest, case), "field": field,
                "index": null, "target": exact_address(&manifest, target)
            })
        );
        assert_eq!(edges[1]["field"], "clause_refs");
    }

    let mut both = base.clone();
    both["semantic"]["entities"][2]["operation"] = json!(OPERATION);
    let mut neither = base.clone();
    neither["semantic"]["entities"][2]
        .as_object_mut()
        .unwrap()
        .remove("expression");
    for (path, invalid) in [("both", both), ("neither", neither)] {
        rejection(
            d,
            apply(d, &invalid, path),
            path,
            "Case requires exactly one of operation or expression",
        );
    }
    for (index, path, field, target) in [
        (2, "wrong-expression", "expression", OPERATION),
        (4, "wrong-operation", "operation", EXPRESSION),
    ] {
        let mut invalid = base.clone();
        invalid["semantic"]["entities"][index][field] = json!(target);
        rejection(
            d,
            apply(d, &invalid, path),
            path,
            &format!("Reference {target} must resolve to {field}"),
        );
        invalid["semantic"]["entities"][index][field] = Value::Null;
        let null_path = format!("null-{field}");
        rejection(
            d,
            apply(d, &invalid, &null_path),
            &null_path,
            "invalid type: null",
        );
    }
}

#[test]
fn xs_target_count_counts_entities_with_or_without_cases() {
    let temp = tempfile::tempdir().unwrap();
    let d = temp.path();
    let mut two = draft();
    two["presentation"]["size"] = json!("S");
    let mut second = two["semantic"]["entities"][1].clone();
    second["id"] = json!("expression-less-than-copy");
    second["text"] = json!("(left < right)");
    place(&mut two, second);
    let mut zero = draft();
    zero["presentation"]["size"] = json!("S");
    zero["semantic"]["entities"]
        .as_array_mut()
        .unwrap()
        .truncate(1);
    zero["presentation"]["nodes"][0]["entities"] = json!([CLAUSE]);

    for (label, mut base, message) in [
        (
            "zero",
            zero,
            "XS function profile requires exactly one declared operation",
        ),
        (
            "mixed",
            mixed_s(),
            "XS profile requires exactly one declared operation or expression",
        ),
        (
            "two",
            two,
            "XS profile requires exactly one declared operation or expression",
        ),
    ] {
        let source = format!("{label}-s");
        success(apply(d, &base, &source));
        verify(d, &source);
        base["presentation"]["size"] = json!("XS");
        let applied = format!("{label}-xs-apply");
        rejection(d, apply(d, &base, &applied), &applied, message);
        let revised = format!("{label}-xs-revise");
        rejection(
            d,
            revise(d, &source, &base["presentation"], &revised),
            &revised,
            message,
        );
        verify(d, &source);
    }
}

fn layout(base: &Value, profile: &str, target: &str, case: &str) -> Value {
    match profile {
        "grouped-s" => json!({
            "size":"S","profile":"caac-paragraph-presentation-v1",
            "nodes":[{"kind":"article","id":"article-expression","title":"left < right",
                "paragraphs":[{"id":"paragraph-expression","entities":[CLAUSE,target,case]}]}]
        }),
        "XL" => json!({
            "size":"XL","nodes":[
                {"kind":"module","id":"module-rule","title":"module-rule","file":"rule.md","decisions":[],
                 "articles":[{"kind":"article","id":"article-rule","title":"Rule","entities":[CLAUSE]}]},
                {"kind":"module","id":"module-expression","title":"module-expression","file":"expression.md","decisions":[],
                 "articles":[{"kind":"article","id":"article-expression","title":"left < right","entities":[target,case]}]}
            ]
        }),
        size => {
            let mut presentation = base["presentation"].clone();
            presentation["size"] = json!(size);
            presentation
        }
    }
}

#[test]
fn expression_profiles_reject_apply_and_revise_with_valid_operation_controls() {
    let temp = tempfile::tempdir().unwrap();
    let d = temp.path();
    let mut expression = draft();
    expression["presentation"]["size"] = json!("S");
    success(apply(d, &expression, "expression-s"));
    verify(d, "expression-s");
    let mut operation = mixed_s();
    operation["semantic"]["entities"]
        .as_array_mut()
        .unwrap()
        .drain(1..3);
    operation["presentation"]["nodes"][0]["entities"] = json!([CLAUSE, OPERATION, OPERATION_CASE]);
    success(apply(d, &operation, "operation-s"));
    verify(d, "operation-s");

    for profile in ["grouped-s", "M", "L", "XL"] {
        expression["presentation"] = layout(&draft(), profile, EXPRESSION, EXPRESSION_CASE);
        let message = "Expressions require XS or flat S presentation";
        let applied = format!("{profile}-expression-apply");
        rejection(d, apply(d, &expression, &applied), &applied, message);
        let revised = format!("{profile}-expression-revise");
        rejection(
            d,
            revise(d, "expression-s", &expression["presentation"], &revised),
            &revised,
            message,
        );
        let control_presentation = layout(
            &read(d, "operation-s/candidate.json"),
            profile,
            OPERATION,
            OPERATION_CASE,
        );
        operation["presentation"] = control_presentation;
        let applied = format!("{profile}-operation-apply");
        success(apply(d, &operation, &applied));
        verify(d, &applied);
        let revised = format!("{profile}-operation-revise");
        success(revise(
            d,
            "operation-s",
            &operation["presentation"],
            &revised,
        ));
        verify(d, &revised);
    }
    verify(d, "expression-s");
    verify(d, "operation-s");
}

#[test]
fn expression_case_validates_types_clause_membership_and_exception_refs() {
    let temp = tempfile::tempdir().unwrap();
    let d = temp.path();
    let mut base = mixed_s();
    let mut unrelated = base["semantic"]["entities"][0].clone();
    unrelated["id"] = json!("rule-unrelated");
    place(&mut base, unrelated);
    place(
        &mut base,
        json!({
            "kind":"exception","id":"except-control","target":CLAUSE,
            "applicability":{"kind":"input-equals","field":"left","value":2},
            "effect":{"kind":"replace-result","value":true}
        }),
    );
    base["semantic"]["entities"][4]["exception_refs"] = json!(["except-control"]);
    success(apply(d, &base, "valid"));
    verify(d, "valid");
    for (path, field, value, message) in [
        (
            "missing-input",
            "input",
            json!({"left":2}),
            "Case input/result types",
        ),
        (
            "extra-input",
            "input",
            json!({"left":2,"right":3,"other":4}),
            "Case input/result types",
        ),
        (
            "wrong-input-type",
            "input",
            json!({"left":true,"right":3}),
            "Case input/result types",
        ),
        (
            "wrong-result-type",
            "expected",
            json!(1),
            "Case input/result types",
        ),
        (
            "empty-clauses",
            "clause_refs",
            json!([]),
            "Case clauses must belong to its expression",
        ),
        (
            "unrelated-clause",
            "clause_refs",
            json!(["rule-unrelated"]),
            "Case clauses must belong to its expression",
        ),
        (
            "exception",
            "exception_refs",
            json!(["except-control"]),
            "Expression cases require empty exception_refs",
        ),
    ] {
        let mut invalid = base.clone();
        invalid["semantic"]["entities"][2][field] = value;
        rejection(d, apply(d, &invalid, path), path, message);
    }
    for (path, field, value, message) in [
        ("empty-text", "text", json!("  "), "Expected nonblank text"),
        (
            "no-expression-clause",
            "clause_refs",
            json!([]),
            "Expression requires at least one clause",
        ),
        (
            "missing-expression-clause",
            "clause_refs",
            json!(["rule-missing"]),
            "Reference rule-missing must resolve to clause",
        ),
    ] {
        let mut invalid = draft();
        invalid["semantic"]["entities"]
            .as_array_mut()
            .unwrap()
            .pop();
        invalid["presentation"]["nodes"][0]["entities"] = json!([CLAUSE, EXPRESSION]);
        invalid["semantic"]["entities"][1][field] = value;
        rejection(d, apply(d, &invalid, path), path, message);
    }
}

#[test]
fn interview_applies_expression_answers_and_compare_revises_authored_data() {
    let temp = tempfile::tempdir().unwrap();
    let d = temp.path();
    let mut base = draft();
    let proposed = base["semantic"]["entities"]
        .as_array_mut()
        .unwrap()
        .split_off(1);
    write(d, "base.json", &base);

    let mut child = command(
        d,
        &[
            "interview",
            "--base",
            "base.json",
            "--answers",
            "journal.json",
            "--output",
            "interview",
        ],
    )
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .unwrap();
    {
        let mut stdin = child.stdin.take().unwrap();
        for entity in proposed {
            writeln!(stdin, "{entity}").unwrap();
        }
    }
    success(child.wait_with_output().unwrap());
    let p = receipt(d, "interview");
    assert_eq!(
        p["answers_sha256"],
        digest(&fs::read(d.join("journal.json")).unwrap())
    );
    let fields = p["field_changes"].as_array().unwrap();
    assert!(
        fields
            .iter()
            .any(|f| f["entity"] == EXPRESSION_CASE && f["field"] == "expression")
    );
    assert_eq!(read(d, "interview/candidate.json"), draft());
    verify(d, "interview");

    let review = fs::read_to_string(d.join("interview/review.md")).unwrap();
    let edited = review
        .replace(
            "- **Authored text:** ` \"left < right\" `",
            "- **Authored text:** ` \"(left < right)\" `",
        )
        .replace("\"text\": \"left < right\"", "\"text\": \"(left < right)\"")
        .replace(
            "- **Expected result (not executed):** ` true `",
            "- **Expected result (not executed):** ` false `",
        )
        .replace("\"expected\": true", "\"expected\": false");
    fs::write(d.join("edited.md"), edited).unwrap();
    success(from_package(
        d,
        "compare-review",
        "interview",
        &["--edited", "edited.md", "--output", "proposal"],
    ));
    let proposal = read(d, "proposal/proposal.json");
    assert_eq!(proposal["status"], "proposed");
    assert_eq!(proposal["replacements"].as_array().unwrap().len(), 2);
    success(from_package(
        d,
        "revise",
        "interview",
        &[
            "--proposal",
            "proposal/proposal.json",
            "--output",
            "revised",
        ],
    ));
    let traced = trace(d, "revised", EXPRESSION);
    assert_eq!(traced["selected"]["body"]["text"], "(left < right)");
    assert_eq!(
        traced["reachable_declared_cases"][0]["body"]["expected"],
        false
    );
    assert_eq!(
        traced["evidence_scope"]["source_behavior_evidence"],
        "not-run"
    );
    assert_eq!(traced["evidence_scope"]["owner_acceptance"], "pending");
    assert_eq!(
        receipt(d, "revised")["format"],
        "caac-candidate-provenance-v4"
    );
}
