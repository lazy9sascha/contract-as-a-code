use caac::candidate::{Candidate, render, revisions};
use comrak::{Arena, Options, nodes::NodeValue, parse_document};
use serde_json::{Value, json};
use std::{fs, process::Command};
fn fixture() -> Value {
    let mut base: Value =
        serde_json::from_str(include_str!("../examples/candidate/draft.json")).unwrap();
    let answers: Value =
        serde_json::from_str(include_str!("../examples/candidate/answers.json")).unwrap();
    base["semantic"]["entities"] = json!(
        answers["answers"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|a| a.get("value").cloned())
            .collect::<Vec<_>>()
    );
    base
}
fn typed(v: &Value) -> Candidate {
    serde_json::from_value(v.clone()).unwrap()
}
fn code_values(text: &str) -> (Vec<Value>, Vec<Value>) {
    let arena = Arena::new();
    let root = parse_document(&arena, text, &Options::default());
    let mut inline = vec![];
    let mut blocks = vec![];
    for node in root.descendants() {
        match &node.data.borrow().value {
            NodeValue::Code(c) => {
                if let Ok(v) = serde_json::from_str(&c.literal) {
                    inline.push(v);
                }
            }
            NodeValue::CodeBlock(c) if c.info == "json" => {
                blocks.push(serde_json::from_str(&c.literal).expect("exact JSON projection"))
            }
            NodeValue::Link(_)
            | NodeValue::Image(_)
            | NodeValue::HtmlInline(_)
            | NodeValue::HtmlBlock(_) => panic!("data escaped into active Markdown"),
            _ => {}
        }
    }
    (inline, blocks)
}
#[test]
fn every_body_field_visible_and_exact_typed_entities_preserved() {
    let mut v = fixture();
    v["semantic"]["unresolved"][0]["proposal"] =
        json!({"text":"Pending choice","entity_refs":["rule-echo"]});
    for modality in ["obligation", "prohibition", "permission"] {
        v["semantic"]["entities"][1]["modality"] = json!(modality);
        let text = render(&typed(&v)).unwrap();
        let (inline, blocks) = code_values(&text);
        for entity in v["semantic"]["entities"].as_array().unwrap() {
            assert_eq!(blocks.iter().filter(|b| *b == entity).count(), 1);
            for field in entity.as_object().unwrap().values() {
                assert!(inline.contains(field), "missing labeled field: {field}");
            }
        }
        for field in v["semantic"]["unresolved"][0].as_object().unwrap().values() {
            assert!(inline.contains(field));
        }
        for label in [
            "Actor",
            "Modality",
            "Authored text",
            "Applicability",
            "Expected result (not executed)",
            "pending",
        ] {
            assert!(text.contains(label));
        }
        assert!(!text.contains("must Echo"));
    }
}
#[test]
fn hostile_strings_remain_literal_under_actual_markdown_parser() {
    let mut v = fixture();
    let hostile = "한국어 ` `` ``` | <img src=x onerror=alert(1)> &copy; [link](https://evil) ![image](x) # forged \u{202e}owner accepted";
    v["semantic"]["entities"][0]["text"] = json!(hostile);
    v["presentation"]["nodes"][0]["title"] = json!(hostile);
    let text = render(&typed(&v)).unwrap();
    let (inline, blocks) = code_values(&text);
    assert!(inline.contains(&json!(hostile)));
    assert!(blocks.contains(&v["semantic"]["entities"][0]));
    assert!(text.contains("한국어"));
    assert!(!text.contains('\u{202e}'));
    assert!(text.contains("\\u202e"));
    for bad in ["line\n# forged", "\u{1b}[2Jforged"] {
        v["semantic"]["entities"][0]["text"] = json!(bad);
        assert!(render(&typed(&v)).is_err()); // unsupported authored control text fails closed
    }
}
#[test]
fn typed_literals_keep_false_zero_empty_string_and_spaces_distinct() {
    for (ty, values) in [
        ("i32", vec![json!(0)]),
        ("bool", vec![json!(false)]),
        (
            "string",
            vec![
                json!(""),
                json!("0"),
                json!(" "),
                json!("x\n# forged\u{1b}[2J\u{007f}\u{009b}\u{202e}```<b>"),
            ],
        ),
    ] {
        for value in values {
            let mut v = fixture();
            v["semantic"]["entities"][2]["inputs"]["value"] = json!(ty);
            v["semantic"]["entities"][2]["output"] = json!(ty);
            v["semantic"]["entities"][3]["applicability"]["value"] = value.clone();
            v["semantic"]["entities"][3]["effect"]["value"] = value.clone();
            for i in [4, 5] {
                v["semantic"]["entities"][i]["input"]["value"] = value.clone();
                v["semantic"]["entities"][i]["expected"] = value.clone();
            }
            let text = render(&typed(&v)).unwrap();
            let (inline, blocks) = code_values(&text);
            assert!(inline.contains(&value));
            assert!(blocks.contains(&v["semantic"]["entities"][4]));
            assert!(!text.contains('\u{1b}'));
            assert!(!text.contains('\u{007f}'));
            assert!(!text.contains('\u{009b}'));
            assert!(!text.contains('\u{202e}'));
        }
    }
}
#[test]
fn chapter_move_and_renumber_changes_view_not_semantics_or_entity_identity() {
    let v = fixture();
    let original = revisions(&typed(&v)).unwrap();
    let mut m = v.clone();
    m["presentation"]["size"] = json!("M");
    let article = m["presentation"]["nodes"][0].clone();
    m["presentation"]["nodes"] =
        json!([{"kind":"chapter","id":"ch-one","title":"First","articles":[article]}]);
    let r = revisions(&typed(&m)).unwrap();
    assert_eq!(r.semantic_revision, original.semantic_revision);
    assert_eq!(r.objects, original.objects);
    let text = render(&typed(&m)).unwrap();
    assert!(text.contains("## Chapter 1"));
    assert!(text.contains("### Article 1"));
    assert_eq!(text.matches("#### Clause ·").count(), 1);
    let all = m["presentation"]["nodes"][0]["articles"][0]["entities"].clone();
    m["presentation"]["nodes"] = json!([
      {"kind":"chapter","id":"ch-one","title":"First","articles":[{"kind":"article","id":"a-first","title":"Definition","entities":[all[0]]}]},
      {"kind":"chapter","id":"ch-two","title":"Second","articles":[{"kind":"article","id":"a-second","title":"Remainder","entities":all.as_array().unwrap()[1..]}]}
    ]);
    let text = render(&typed(&m)).unwrap();
    assert!(text.contains("## Chapter 2"));
    assert!(text.contains("### Article 2"));
    assert_eq!(
        revisions(&typed(&m)).unwrap().semantic_revision,
        original.semantic_revision
    );
}
#[test]
fn real_cli_profile_and_changed_labeled_prose_are_verified() {
    let d = tempfile::tempdir().unwrap();
    let base = serde_json::to_vec(&fixture()).unwrap();
    fs::write(d.path().join("base"), &base).unwrap();
    fs::write(
        d.path().join("answers"),
        serde_json::to_vec(&json!({"expected_base":caac::candidate::digest(&base),"answers":[]}))
            .unwrap(),
    )
    .unwrap();
    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_caac"))
            .current_dir(d.path())
            .args(args)
            .output()
            .unwrap()
    };
    assert!(
        run(&[
            "candidate",
            "apply",
            "--base",
            "base",
            "--answers",
            "answers",
            "--output",
            "out"
        ])
        .status
        .success()
    );
    let verify = || {
        run(&[
            "candidate",
            "verify",
            "--candidate",
            "out/candidate.json",
            "--provenance",
            "out/provenance.json",
            "--review",
            "out/review.md",
        ])
    };
    assert!(verify().status.success());
    let ppath = d.path().join("out/provenance.json");
    let original: Value = serde_json::from_slice(&fs::read(&ppath).unwrap()).unwrap();
    assert_eq!(
        original["renderer_profile"],
        "caac-candidate-legal-review-v1"
    );
    for (field, value) in [
        ("renderer_profile", "unknown"),
        ("format", "caac-candidate-provenance-v2"),
    ] {
        let mut p = original.clone();
        p[field] = json!(value);
        fs::write(&ppath, serde_json::to_vec(&p).unwrap()).unwrap();
        assert!(!verify().status.success());
    }
    let review = fs::read_to_string(d.path().join("out/review.md"))
        .unwrap()
        .replace("**Modality:**", "**Approved obligation:**");
    let mut p = original;
    p["review_sha256"] = json!(caac::candidate::digest(review.as_bytes()));
    fs::write(&ppath, serde_json::to_vec(&p).unwrap()).unwrap();
    fs::write(d.path().join("out/review.md"), review).unwrap();
    assert!(!verify().status.success());
}
