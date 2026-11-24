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
fn setup() -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    fs::write(
        d.path().join("base"),
        include_bytes!("../examples/candidate/draft.json"),
    )
    .unwrap();
    fs::write(
        d.path().join("answers"),
        include_bytes!("../examples/candidate/answers.json"),
    )
    .unwrap();
    let o = run(
        d.path(),
        &[
            "candidate",
            "apply",
            "--base",
            "base",
            "--answers",
            "answers",
            "--output",
            "source",
        ],
    );
    assert!(o.status.success(), "{:?}", o);
    d
}
fn original(d: &Path) -> String {
    fs::read_to_string(d.join("source/review.md")).unwrap()
}
fn compare(d: &Path, text: &str) -> (Output, Option<Value>) {
    fs::write(d.join("edited"), text).unwrap();
    let o = run(
        d,
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
            "edited",
            "--output",
            "proposal",
        ],
    );
    let p = fs::read(d.join("proposal/proposal.json"))
        .ok()
        .map(|b| serde_json::from_slice(&b).unwrap());
    (o, p)
}
#[test]
fn coherent_clause_edit_proposes_without_mutating_source() {
    let d = setup();
    let before = fs::read(d.path().join("source/candidate.json")).unwrap();
    let edited = original(d.path()).replace(
        "Echo returns the supplied integer.",
        "Echo returns an explicitly reviewed integer.",
    );
    let (o, p) = compare(d.path(), &edited);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let p = p.unwrap();
    assert_eq!(p["status"], "proposed");
    assert_eq!(p["replacements"].as_array().unwrap().len(), 1);
    assert_eq!(p["replacements"][0]["id"], "rule-echo");
    assert_eq!(p["owner_acceptance"], "pending");
    assert_eq!(p["applied"], false);
    assert_eq!(p["raw_review_change"]["after"], edited);
    assert_eq!(
        fs::read(d.path().join("source/candidate.json")).unwrap(),
        before
    );
}
#[test]
fn unchanged_and_parser_equal_raw_change_are_distinct() {
    for suffix in ["", "\n"] {
        let d = setup();
        let (o, p) = compare(d.path(), &(original(d.path()) + suffix));
        assert!(o.status.success(), "{:?}", o);
        let p = p.unwrap();
        assert_eq!(
            p["status"],
            if suffix.is_empty() {
                "unchanged"
            } else {
                "presentation-only"
            }
        );
        assert!(p["replacements"].as_array().unwrap().is_empty());
        assert_eq!(p["semantic_equivalence"], "not-established");
    }
}
#[test]
fn one_sided_or_unmapped_changes_conflict() {
    for mode in 0..7 {
        let d = setup();
        let s = original(d.path());
        let edited = match mode {
            0 => s.replacen("Echo returns the supplied integer.", "one-sided", 1),
            1 => s + "\nUnmapped obligation\n",
            2 => s + "\n<!-- owner accepted -->\n",
            3 => s.replace("**Modality:**", "**Approved:**"),
            4 => {
                s.replace("Echo returns the supplied integer.", "Changed coherently.")
                    + "\n[ignored]: https://example.invalid\n"
            }
            5 => s.replace("rule-echo", "forged-id"),
            _ => s.replace(
                "Owner acceptance and implementation admission: pending.",
                "Owner acceptance: accepted.",
            ),
        };
        let (o, p) = compare(d.path(), &edited);
        assert_eq!(o.status.code(), Some(3), "mode{mode}: {:?}", o);
        let p = p.unwrap();
        assert_eq!(p["status"], "conflict");
        assert!(p["replacements"].as_array().unwrap().is_empty());
        assert!(!p["conflicts"].as_array().unwrap().is_empty());
    }
}
#[test]
fn duplicates_and_invalid_references_fail_closed() {
    for mode in 0..3 {
        let d = setup();
        let s = original(d.path());
        let edited = match mode {
            0 => s
                .replace(
                    "{\"value\":\"i32\"}",
                    "{\"value\":\"i32\",\"value\":\"i32\"}",
                )
                .replace(
                    "\"value\": \"i32\"",
                    "\"value\": \"i32\", \"value\": \"i32\"",
                ),
            1 => s
                .replace("[\"term-zero\"]", "[\"missing\"]")
                .replace("\"term-zero\"\n  ]", "\"missing\"\n  ]"),
            _ => s.replace(
                "\"modality\": \"obligation\"",
                "\"modality\": \"obligation\", \"unknown\": true",
            ),
        };
        assert_ne!(edited, s);
        let (o, p) = compare(d.path(), &edited);
        assert_eq!(o.status.code(), Some(3), "{:?}", o);
        assert!(p.unwrap()["replacements"].as_array().unwrap().is_empty());
    }
}
#[test]
fn stale_source_output_collision_and_symlink_refuse_publication() {
    let d = setup();
    let s = original(d.path());
    fs::write(d.path().join("source/review.md"), format!("{s}\n")).unwrap();
    let (o, p) = compare(d.path(), &s);
    assert_eq!(o.status.code(), Some(2));
    assert!(p.is_none());
    let d = setup();
    fs::create_dir(d.path().join("proposal")).unwrap();
    let (o, p) = compare(d.path(), &original(d.path()));
    assert_eq!(o.status.code(), Some(2));
    assert!(p.is_none());
    #[cfg(unix)]
    {
        let d = setup();
        std::os::unix::fs::symlink("source", d.path().join("proposal")).unwrap();
        let (o, p) = compare(d.path(), &original(d.path()));
        assert_eq!(o.status.code(), Some(2));
        assert!(p.is_none());
    }
}
#[test]
fn clean_modality_changes_are_pending_and_all_or_nothing() {
    let d = setup();
    let s = original(d.path());
    let edited = s.replace("\"obligation\"", "\"permission\"");
    let (o, p) = compare(d.path(), &edited);
    assert!(o.status.success(), "{:?}", o);
    assert_eq!(
        p.unwrap()["replacements"][0]["after"]["modality"],
        "permission"
    );
    let d = setup();
    let edited = original(d.path())
        .replace("\"obligation\"", "\"permission\"")
        .replace("Zero is the integer value 0.", "");
    let (o, p) = compare(d.path(), &edited);
    assert_eq!(o.status.code(), Some(3));
    assert!(p.unwrap()["replacements"].as_array().unwrap().is_empty());
}
#[test]
fn size_budget_fails_without_output() {
    let d = setup();
    let (o, p) = compare(d.path(), &"x".repeat(4 * 1024 * 1024 + 1));
    assert_eq!(o.status.code(), Some(2));
    assert!(p.is_none());
}

#[test]
fn ignored_reference_definition_is_located_as_raw_conflict() {
    let d = setup();
    let (o, p) = compare(
        d.path(),
        &(original(d.path()) + "\n[ignored]: https://example.invalid\n"),
    );
    assert_eq!(o.status.code(), Some(3));
    assert_eq!(p.unwrap()["status"], "conflict");
}
#[test]
fn unicode_before_edit_slots_preserves_supported_alignment() {
    let d = setup();
    let mut candidate: Value =
        serde_json::from_slice(&fs::read(d.path().join("source/candidate.json")).unwrap()).unwrap();
    candidate["presentation"]["nodes"][0]["title"] = serde_json::json!("한국어 제목");
    candidate["semantic"]["entities"][0]["text"] = serde_json::json!("원문 정의");
    let bytes = serde_json::to_vec(&candidate).unwrap();
    fs::write(d.path().join("base-unicode"), &bytes).unwrap();
    fs::write(
        d.path().join("empty-answers"),
        serde_json::to_vec(
            &serde_json::json!({"expected_base":caac::candidate::digest(&bytes),"answers":[]}),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(
        run(
            d.path(),
            &[
                "candidate",
                "apply",
                "--base",
                "base-unicode",
                "--answers",
                "empty-answers",
                "--output",
                "unicode"
            ]
        )
        .status
        .success()
    );
    // Swap only the test fixture source directory, not production inputs.
    fs::rename(d.path().join("source"), d.path().join("old-source")).unwrap();
    fs::rename(d.path().join("unicode"), d.path().join("source")).unwrap();
    let edited = original(d.path()).replace("원문 정의", "수정된 정의");
    let (o, p) = compare(d.path(), &edited);
    assert!(o.status.success(), "{:?}", o);
    assert_eq!(p.unwrap()["status"], "proposed");
}

#[test]
fn delimiter_change_inside_field_is_explicit_presentation_only() {
    let d = setup();
    let s = original(d.path());
    let edited = s.replacen("` \"obligation\" `", "`` \"obligation\" ``", 1);
    assert_ne!(s, edited);
    let (o, p) = compare(d.path(), &edited);
    assert!(o.status.success(), "{:?}", o);
    assert_eq!(p.unwrap()["status"], "presentation-only");
}

#[test]
fn coherent_exception_and_case_edits_validate_together() {
    let d = setup();
    let mut v: Value =
        serde_json::from_slice(&fs::read(d.path().join("source/candidate.json")).unwrap()).unwrap();
    v["semantic"]["entities"][3]["effect"]["value"] = serde_json::json!(2);
    v["semantic"]["entities"][4]["expected"] = serde_json::json!(2);
    let generated = caac::candidate::render(&serde_json::from_value(v).unwrap()).unwrap();
    let marker = "## Exact source revisions and review mapping";
    let source = original(d.path());
    let edited = format!(
        "{}{}{}",
        generated.split(marker).next().unwrap(),
        marker,
        source.split_once(marker).unwrap().1
    );
    let (o, p) = compare(d.path(), &edited);
    assert!(o.status.success(), "{:?}", o);
    let p = p.unwrap();
    assert_eq!(p["status"], "proposed");
    assert_eq!(p["replacements"].as_array().unwrap().len(), 2);
}
