use caac::{
    candidate,
    contract_collection::{self, Request},
    review_findings, review_packet,
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
fn read(p: &Path, n: &str) -> Value {
    serde_json::from_slice(&fs::read(p.join(n)).unwrap()).unwrap()
}
fn write(p: &Path, n: &str, v: &Value) {
    fs::write(p.join(n), serde_json::to_vec_pretty(v).unwrap()).unwrap();
}
fn setup(p: &Path) -> (Request, Value) {
    setup_with_root_assessment(p, None)
}
fn setup_with_root_assessment(p: &Path, root_assessment: Option<&str>) -> (Request, Value) {
    fs::write(p.join("doc"), "Echo returns a value. 계약\r\n").unwrap();
    fs::write(
        p.join("lib.rs"),
        "pub fn echo(value: i32) -> i32 { value }\n",
    )
    .unwrap();
    assert!(
        run(
            p,
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
    let request = Request {
        contracts: vec![p.join("doc")],
        root_assessment: root_assessment.map(|file| p.join(file)),
        baseline: Some(p.join("intake/baseline.json")),
        ..Request::default()
    };
    contract_collection::collect(&request, &p.join("collection")).unwrap();
    review_packet::prepare_review(&p.join("collection"), &request, &p.join("packet")).unwrap();
    fs::write(
        p.join("draft"),
        include_bytes!("../examples/candidate/draft.json"),
    )
    .unwrap();
    fs::write(
        p.join("answers"),
        include_bytes!("../examples/candidate/answers.json"),
    )
    .unwrap();
    candidate::apply(&p.join("draft"), &p.join("answers"), &p.join("initial")).unwrap();
    fs::copy(p.join("initial/candidate.json"), p.join("base")).unwrap();
    let hash = |n: &str| candidate::digest(&fs::read(p.join(n)).unwrap());
    let packet = read(p, "packet");
    let refs = packet["evidence"].as_array().unwrap();
    let doc = refs.iter().find(|e| e["kind"] == "document").unwrap();
    let src = refs.iter().find(|e| e["kind"] == "source-file").unwrap();
    let f = json!({"format":"caac-review-findings-v1","expected_collection_sha256":hash("collection"),"expected_packet_sha256":hash("packet"),"expected_base_sha256":hash("base"),"findings":[{"id":"review-echo","claim_kind":"aligned","claim":"The selected text and syntax provide limited static support.","inference_limits":"Not behavioral conformance or owner acceptance.","citations":[{"evidence_id":doc["id"],"start_byte":0,"end_byte":doc["end_byte"],"excerpt":fs::read_to_string(p.join("doc")).unwrap()},{"evidence_id":src["id"],"start_byte":0,"end_byte":src["end_byte"],"excerpt":fs::read_to_string(p.join("lib.rs")).unwrap()}],"handoff":{"decision_id":"decision-scope","proposal_text":"Review the limited static evidence before resolving scope.","entity_refs":["operation-echo"]}}]});
    write(p, "findings", &f);
    (request, f)
}
fn handoff(p: &Path, r: &Request) -> Result<(), String> {
    review_findings::handoff(
        &p.join("collection"),
        &p.join("packet"),
        &p.join("findings"),
        &p.join("base"),
        r,
        &p.join("handoff"),
    )
}
#[test]
fn checked_citations_propose_existing_decision_then_explicit_apply_stays_pending() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let (_r, _) = setup(p);
    let before = fs::read(p.join("base")).unwrap();
    let output = run(
        p,
        &[
            "review-findings",
            "--collection",
            "collection",
            "--packet",
            "packet",
            "--findings",
            "findings",
            "--base",
            "base",
            "--contract",
            p.join("doc").to_str().unwrap(),
            "--baseline",
            p.join("intake/baseline.json").to_str().unwrap(),
            "--output",
            "handoff",
        ],
    );
    assert!(output.status.success(), "{:?}", output);
    let receipt = read(p, "handoff/receipt.json");
    assert_eq!(receipt["claim_truth"], "not-assessed");
    assert_eq!(receipt["candidate_application"], "not-performed");
    assert_eq!(fs::read(p.join("base")).unwrap(), before);
    candidate::apply(
        &p.join("base"),
        &p.join("handoff/answers.json"),
        &p.join("result"),
    )
    .unwrap();
    candidate::verify(
        &p.join("result/candidate.json"),
        &p.join("result/provenance.json"),
        &p.join("result/review.md"),
    )
    .unwrap();
    let base = read(p, "base");
    let result = read(p, "result/candidate.json");
    assert_eq!(base["semantic"]["entities"], result["semantic"]["entities"]);
    assert_eq!(base["authority"], result["authority"]);
    for field in ["id", "question", "required"] {
        assert_eq!(
            base["semantic"]["unresolved"][0][field],
            result["semantic"]["unresolved"][0][field]
        );
    }
    assert!(result["semantic"]["unresolved"][0]["proposal"].is_object());
    assert_eq!(
        read(p, "result/provenance.json")["answers_sha256"],
        receipt["answers_sha256"]
    );
}
#[test]
fn invalid_claim_citation_and_handoff_shapes_fail_before_publication() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let (r, original) = setup(p);
    let edits = [
        ("/findings/0/citations/0/excerpt", json!("forged")),
        ("/findings/0/citations/0/end_byte", json!(0)),
        ("/findings/0/citations/0/start_byte", json!(23)),
        ("/findings/0/citations/0/evidence_id", json!("unknown")),
        ("/findings/0/claim_kind", json!("approved")),
        ("/findings/0/handoff/decision_id", json!("unknown")),
        ("/findings/0/handoff/entity_refs", json!(["missing"])),
        ("/expected_packet_sha256", json!("stale")),
    ];
    for (path, value) in edits {
        let mut f = original.clone();
        *f.pointer_mut(path).unwrap() = value;
        write(p, "findings", &f);
        assert!(handoff(p, &r).is_err(), "{path}");
        assert!(!p.join("handoff").exists());
    }
    for mode in 0..4 {
        let mut f = original.clone();
        match mode {
            0 => {
                let item = f["findings"][0].clone();
                f["findings"].as_array_mut().unwrap().push(item);
            }
            1 => {
                let mut item = f["findings"][0].clone();
                item["id"] = json!("other");
                f["findings"].as_array_mut().unwrap().push(item);
            }
            2 => {
                f["findings"][0]["authority"] = json!("accepted");
            }
            _ => {
                f["findings"][0]["citations"].as_array_mut().unwrap().pop();
            }
        }
        write(p, "findings", &f);
        assert!(handoff(p, &r).is_err());
        assert!(!p.join("handoff").exists());
    }
    let text = serde_json::to_string_pretty(&original).unwrap().replacen(
        "\"claim_kind\":",
        "\"claim_kind\":\"aligned\",\"claim_kind\":",
        1,
    );
    fs::write(p.join("findings"), text).unwrap();
    assert!(handoff(p, &r).is_err());
}
#[test]
fn source_file_body_cannot_be_claimed_as_narrow_observation() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let (r, mut f) = setup(p);
    let packet = read(p, "packet");
    let e = packet["evidence"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| {
            e["kind"] == "source-observation"
                && e["end_byte"].as_u64().unwrap()
                    < fs::read(p.join("lib.rs")).unwrap().len() as u64
        })
        .unwrap();
    f["findings"][0]["citations"][1]["evidence_id"] = e["id"].clone();
    write(p, "findings", &f);
    assert!(handoff(p, &r).is_err());
    assert!(!p.join("handoff").exists());
}
#[test]
fn stale_inputs_no_clobber_and_existing_proposal_are_rejected() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let (r, mut f) = setup(p);
    let source = fs::read(p.join("lib.rs")).unwrap();
    fs::write(p.join("lib.rs"), "changed").unwrap();
    assert!(handoff(p, &r).is_err());
    fs::write(p.join("lib.rs"), source).unwrap();
    handoff(p, &r).unwrap();
    let receipt = fs::read(p.join("handoff/receipt.json")).unwrap();
    assert!(handoff(p, &r).is_err());
    assert_eq!(fs::read(p.join("handoff/receipt.json")).unwrap(), receipt);
    let mut base = read(p, "base");
    base["semantic"]["unresolved"][0]["proposal"] =
        json!({"text":"Existing proposal","entity_refs":["operation-echo"]});
    write(p, "base", &base);
    f["expected_base_sha256"] = json!(candidate::digest(&fs::read(p.join("base")).unwrap()));
    write(p, "findings", &f);
    assert!(
        review_findings::handoff(
            &p.join("collection"),
            &p.join("packet"),
            &p.join("findings"),
            &p.join("base"),
            &r,
            &p.join("replacement")
        )
        .is_err()
    );
    assert!(!p.join("replacement").exists());
}

#[test]
fn unverified_single_observation_subrange_remains_an_authored_proposal() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let (request, mut findings) = setup(p);
    let packet = read(p, "packet");
    let observation = packet["evidence"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| {
            e["kind"] == "source-observation"
                && e["end_byte"].as_u64().unwrap() > e["start_byte"].as_u64().unwrap()
        })
        .unwrap();
    let start = observation["start_byte"].as_u64().unwrap() as usize;
    let source = fs::read(p.join("lib.rs")).unwrap();
    // One ASCII byte strictly within a valid observation is enough for identity
    // checking, not sufficient evidence for the truth of the authored claim.
    assert!(source[start].is_ascii());
    findings["findings"][0]["claim_kind"] = json!("unverified");
    findings["findings"][0]["citations"] = json!([{
        "evidence_id": observation["id"], "start_byte": start, "end_byte": start + 1,
        "excerpt": std::str::from_utf8(&source[start..start + 1]).unwrap()
    }]);
    write(p, "findings", &findings);
    handoff(p, &request).unwrap();
    let receipt = read(p, "handoff/receipt.json");
    assert_eq!(receipt["findings"][0]["claim_kind"], "unverified");
    assert_eq!(
        receipt["findings"][0]["citations"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(receipt["claim_truth"], "not-assessed");
    assert_eq!(receipt["candidate_application"], "not-performed");
}

fn bound_apply(p: &Path, request: &Request, output: &str) -> Result<(), String> {
    review_findings::apply_bound(
        &p.join("base"),
        &p.join("handoff/answers.json"),
        &p.join(output),
        &review_findings::ApplyEvidence {
            collection: &p.join("collection"),
            packet: &p.join("packet"),
            findings: &p.join("findings"),
            receipt: &p.join("handoff/receipt.json"),
            selections: request,
        },
    )
}
#[test]
fn explicit_evidence_apply_matches_ordinary_output_but_revalidates_inputs() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let report = json!({
        "status":"owner-confirmed-absent",
        "scope":"Selected synthetic project",
        "reported_by":"Fixture owner"
    });
    write(p, "root-report", &report);
    let (request, _) = setup_with_root_assessment(p, Some("root-report"));
    let selection = [
        "--contract",
        request.contracts[0].to_str().unwrap(),
        "--baseline",
        request.baseline.as_deref().unwrap().to_str().unwrap(),
    ];
    let assessment = [
        "--root-assessment",
        request
            .root_assessment
            .as_deref()
            .unwrap()
            .to_str()
            .unwrap(),
    ];
    let review = [
        "review-findings",
        "--collection",
        "collection",
        "--packet",
        "packet",
        "--findings",
        "findings",
        "--base",
        "base",
        "--output",
        "handoff",
    ];
    let output = run(p, &[&review[..], &selection].concat());
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(!p.join("handoff").exists());
    let output = run(p, &[&review[..], &selection, &assessment].concat());
    assert!(output.status.success(), "{output:?}");
    let receipt = fs::read(p.join("handoff/receipt.json")).unwrap();
    let apply = [
        "candidate",
        "apply",
        "--base",
        "base",
        "--answers",
        "handoff/answers.json",
        "--output",
        "bound",
        "--review-receipt",
        "handoff/receipt.json",
        "--findings",
        "findings",
        "--collection",
        "collection",
        "--packet",
        "packet",
    ];
    let output = run(p, &[&apply[..], &selection].concat());
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(!p.join("bound").exists());
    let mut changed_report = report.clone();
    changed_report["status"] = json!("not-found-in-search-scope");
    write(p, "root-report", &changed_report);
    let output = run(p, &[&apply[..], &selection, &assessment].concat());
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(!p.join("bound").exists());
    write(p, "root-report", &report);
    let result = run(p, &[&apply[..], &selection, &assessment].concat());
    assert!(result.status.success(), "{:?}", result);
    assert!(
        String::from_utf8(result.stdout)
            .unwrap()
            .contains("standalone candidate verify does not recheck evidence")
    );
    candidate::apply(
        &p.join("base"),
        &p.join("handoff/answers.json"),
        &p.join("ordinary"),
    )
    .unwrap();
    for file in ["candidate.json", "review.md", "provenance.json"] {
        assert_eq!(
            fs::read(p.join("bound").join(file)).unwrap(),
            fs::read(p.join("ordinary").join(file)).unwrap()
        );
    }
    assert_eq!(fs::read(p.join("handoff/receipt.json")).unwrap(), receipt);
    let bound = read(p, "bound/candidate.json");
    assert_eq!(bound["authority"]["owner_acceptance"], "pending");
    assert_eq!(bound["authority"]["implementation_admission"], "pending");
    candidate::verify(
        &p.join("bound/candidate.json"),
        &p.join("bound/provenance.json"),
        &p.join("bound/review.md"),
    )
    .unwrap();
    assert!(bound_apply(p, &request, "bound").is_err());
}
#[test]
fn bound_apply_rejects_tampered_receipt_answers_and_every_stale_snapshot() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let (request, _) = setup(p);
    handoff(p, &request).unwrap();
    for file in [
        "handoff/receipt.json",
        "handoff/answers.json",
        "collection",
        "packet",
        "findings",
        "base",
        "doc",
        "lib.rs",
    ] {
        let original = fs::read(p.join(file)).unwrap();
        let mut changed = original.clone();
        changed.push(b' ');
        fs::write(p.join(file), changed).unwrap();
        assert!(bound_apply(p, &request, "out").is_err(), "{file}");
        assert!(!p.join("out").exists());
        fs::write(p.join(file), original).unwrap();
    }
    let receipt = fs::read_to_string(p.join("handoff/receipt.json")).unwrap();
    for injected in ["\"claim_truth\":\"approved\",", "\"unknown\":true,"] {
        fs::write(
            p.join("handoff/receipt.json"),
            receipt.replacen('{', &format!("{{{injected}"), 1),
        )
        .unwrap();
        assert!(bound_apply(p, &request, "out").is_err());
        assert!(!p.join("out").exists());
    }
    fs::write(p.join("handoff/receipt.json"), receipt).unwrap();
    let mut answers = read(p, "handoff/answers.json");
    answers["answers"][0]["value"]["text"] = json!("Changed but structurally valid proposal");
    write(p, "handoff/answers.json", &answers);
    assert!(bound_apply(p, &request, "out").is_err());
    assert!(!p.join("out").exists());
    // Legacy apply intentionally has no receipt/citation verification contract.
    candidate::apply(
        &p.join("base"),
        &p.join("handoff/answers.json"),
        &p.join("ordinary"),
    )
    .unwrap();
}
#[test]
fn evidence_mode_requires_complete_options_and_regular_receipt() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let (request, _) = setup(p);
    handoff(p, &request).unwrap();
    for flag in [
        "--review-receipt",
        "--findings",
        "--collection",
        "--packet",
        "--contract",
    ] {
        let o = run(
            p,
            &[
                "candidate",
                "apply",
                "--base",
                "base",
                "--answers",
                "handoff/answers.json",
                "--output",
                "out",
                flag,
                "packet",
            ],
        );
        assert_eq!(o.status.code(), Some(2));
        assert!(o.stdout.is_empty());
        assert!(!p.join("out").exists());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        fs::rename(p.join("handoff/receipt.json"), p.join("saved-receipt")).unwrap();
        symlink(p.join("saved-receipt"), p.join("handoff/receipt.json")).unwrap();
        assert!(bound_apply(p, &request, "out").is_err());
        fs::remove_file(p.join("handoff/receipt.json")).unwrap();
        assert!(
            Command::new("mkfifo")
                .arg(p.join("handoff/receipt.json"))
                .status()
                .unwrap()
                .success()
        );
        assert!(bound_apply(p, &request, "out").is_err());
        assert!(!p.join("out").exists());
    }
}

#[test]
fn flag_looking_path_values_do_not_select_modes_or_help() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let (request, _) = setup(p);
    handoff(p, &request).unwrap();
    fs::copy(p.join("handoff/answers.json"), p.join("--packet")).unwrap();
    let ordinary = run(
        p,
        &[
            "candidate",
            "apply",
            "--base",
            "base",
            "--answers",
            "--packet",
            "--output",
            "ordinary-flag-path",
        ],
    );
    assert!(ordinary.status.success(), "{:?}", ordinary);
    fs::copy(p.join("handoff/answers.json"), p.join("--help")).unwrap();
    let ordinary_help_path = run(
        p,
        &[
            "candidate",
            "apply",
            "--base",
            "base",
            "--answers",
            "--help",
            "--output",
            "ordinary-help-path",
        ],
    );
    assert!(ordinary_help_path.status.success());
    assert!(p.join("ordinary-help-path/provenance.json").exists());
    let bound = run(
        p,
        &[
            "candidate",
            "apply",
            "--base",
            "base",
            "--answers",
            "--help",
            "--output",
            "bound-flag-path",
            "--review-receipt",
            "handoff/receipt.json",
            "--findings",
            "findings",
            "--collection",
            "collection",
            "--packet",
            "packet",
            "--contract",
            p.join("doc").to_str().unwrap(),
            "--baseline",
            p.join("intake/baseline.json").to_str().unwrap(),
        ],
    );
    assert!(bound.status.success(), "{:?}", bound);
    assert!(p.join("bound-flag-path/provenance.json").exists());
    let missing = run(
        p,
        &[
            "candidate",
            "apply",
            "--base",
            "base",
            "--answers",
            "--help",
            "--output",
            "missing-flags",
            "--review-receipt",
            "handoff/receipt.json",
        ],
    );
    assert_eq!(missing.status.code(), Some(2));
    assert!(missing.stdout.is_empty());
    assert!(!p.join("missing-flags").exists());
}

#[test]
fn run_context_binds_handoff_and_apply_but_is_not_citation_evidence() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let (mut r, mut f) = setup(p);
    fs::write(p.join("task"), "Prefer S").unwrap();
    r.run_inputs.push(p.join("task"));
    fs::remove_file(p.join("collection")).unwrap();
    fs::remove_file(p.join("packet")).unwrap();
    contract_collection::collect(&r, &p.join("collection")).unwrap();
    review_packet::prepare_review(&p.join("collection"), &r, &p.join("packet")).unwrap();
    let old = f["expected_collection_sha256"]
        .as_str()
        .unwrap()
        .to_string();
    let hash = candidate::digest(&fs::read(p.join("collection")).unwrap());
    f = serde_json::from_str(&serde_json::to_string(&f).unwrap().replace(&old, &hash)).unwrap();
    f["expected_packet_sha256"] = json!(candidate::digest(&fs::read(p.join("packet")).unwrap()));
    write(p, "findings", &f);
    handoff(p, &r).unwrap();
    bound_apply(p, &r, "bound-context").unwrap();
    fs::write(p.join("task"), "Consider all sizes").unwrap();
    assert!(bound_apply(p, &r, "stale-context").is_err());
    assert!(!p.join("stale-context").exists());
    assert!(
        review_findings::handoff(
            &p.join("collection"),
            &p.join("packet"),
            &p.join("findings"),
            &p.join("base"),
            &r,
            &p.join("stale-handoff")
        )
        .is_err()
    );
    fs::write(p.join("task"), "Prefer S").unwrap();
    f["findings"][0]["citations"][0]["evidence_id"] = json!(format!("{hash}:run-input:0"));
    write(p, "findings", &f);
    assert!(
        review_findings::handoff(
            &p.join("collection"),
            &p.join("packet"),
            &p.join("findings"),
            &p.join("base"),
            &r,
            &p.join("fake-citation")
        )
        .is_err()
    );
    assert!(!p.join("fake-citation").exists());
}

#[test]
fn analyzer_output_quotes_never_substitute_for_source_citations() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let (mut r, mut f) = setup(p);
    fs::write(p.join("modules"), "[lib] lib\n").unwrap();
    fs::write(p.join("deps"), "lib -> missing [unknown]\n").unwrap();
    write(
        p,
        "snapshot",
        &json!({"revision":"caller-claim","files":[{"path":"lib.rs","sha256":candidate::digest(&fs::read(p.join("lib.rs")).unwrap())}]}),
    );
    r.crawk = Some(caac::crawk_evidence::Selection {
        modules: p.join("modules"),
        deps: p.join("deps"),
        snapshot: p.join("snapshot"),
        source_root: p.into(),
    });
    fs::remove_file(p.join("collection")).unwrap();
    fs::remove_file(p.join("packet")).unwrap();
    contract_collection::collect(&r, &p.join("collection")).unwrap();
    review_packet::prepare_review(&p.join("collection"), &r, &p.join("packet")).unwrap();
    let old = f["expected_collection_sha256"]
        .as_str()
        .unwrap()
        .to_string();
    let hash = candidate::digest(&fs::read(p.join("collection")).unwrap());
    f = serde_json::from_str(&serde_json::to_string(&f).unwrap().replace(&old, &hash)).unwrap();
    f["expected_packet_sha256"] = json!(candidate::digest(&fs::read(p.join("packet")).unwrap()));
    let source = f["findings"][0]["citations"][1].clone();
    f["findings"][0]["citations"][1] = json!({"evidence_id":format!("{hash}:analyzer:1"),"start_byte":0,"end_byte":fs::read(p.join("deps")).unwrap().len(),"excerpt":fs::read_to_string(p.join("deps")).unwrap()});
    write(p, "findings", &f);
    assert!(handoff(p, &r).is_err());
    assert!(!p.join("handoff").exists());
    // An analyzer quote supplements independently checked document and source quotes.
    f["findings"][0]["citations"]
        .as_array_mut()
        .unwrap()
        .push(source);
    write(p, "findings", &f);
    handoff(p, &r).unwrap();
    bound_apply(p, &r, "analyzer-bound").unwrap();
    fs::write(p.join("deps"), "lib -> another [unknown]\n").unwrap();
    assert!(bound_apply(p, &r, "analyzer-stale").is_err());
    assert!(!p.join("analyzer-stale").exists());
    fs::write(p.join("deps"), "lib -> missing [unknown]\n").unwrap();
    // Analyzer-only claims can be unverified, never a certified source claim.
    let only = f["findings"][0]["citations"][1].clone();
    f["findings"][0]["citations"] = json!([only]);
    f["findings"][0]["claim_kind"] = json!("unverified");
    write(p, "findings", &f);
    review_findings::handoff(
        &p.join("collection"),
        &p.join("packet"),
        &p.join("findings"),
        &p.join("base"),
        &r,
        &p.join("analyzer-unverified"),
    )
    .unwrap();
    assert_eq!(
        read(p, "analyzer-unverified/receipt.json")["claim_truth"],
        "not-assessed"
    );
}
