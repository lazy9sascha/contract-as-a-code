use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_caac"))
        .current_dir(root)
        .args(args)
        .output()
        .unwrap()
}

fn read(root: &Path, path: &str) -> Value {
    serde_json::from_slice(&fs::read(root.join(path)).unwrap()).unwrap()
}

fn write(root: &Path, path: &str, value: &Value) {
    fs::write(root.join(path), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn shared_retry() -> Value {
    serde_json::from_slice(include_bytes!(
        "../examples/candidate/shared-retry-draft.json"
    ))
    .unwrap()
}

fn article(id: &str, title: &str, entities: &[&str]) -> Value {
    json!({"kind":"article","id":id,"title":title,"entities":entities})
}

fn intake_article() -> Value {
    json!({
        "kind":"article",
        "id":"article-intake",
        "title":"New jobs",
        "paragraphs":[{
            "id":"paragraph-intake",
            "items":[{"id":"item-intake","entities":["operation-intake","test-intake-standard"]}]
        }]
    })
}

fn shared_retry_hierarchy() -> Value {
    json!({
        "kind":"part","id":"part-operations","title":"Operations","children":[
            {"kind":"chapter","id":"chapter-intake","title":"Intake","children":[
                {"kind":"section","id":"section-policy","title":"Retry policy","children":[
                    {"kind":"subsection","id":"subsection-retry","title":"Limit and exception","children":[
                        article("article-policy","Shared retry policy", &["term-priority","rule-retry-limit","except-priority"])
                    ]}
                ]},
                intake_article()
            ]},
            {"kind":"chapter","id":"chapter-replay","title":"Replay","children":[
                {"kind":"section","id":"section-replay","title":"Replayed jobs","children":[
                    article("article-replay","Replayed jobs", &["operation-replay","test-replay-priority"])
                ]}
            ]}
        ]
    })
}

fn l_presentation() -> Value {
    json!({
        "size":"L",
        "profile":"caac-item-presentation-v1",
        "nodes":[shared_retry_hierarchy()]
    })
}

fn l_candidate() -> Value {
    let mut candidate = shared_retry();
    candidate["presentation"] = l_presentation();
    candidate
}

fn xl_candidate() -> Value {
    let mut candidate = shared_retry();
    let mut intake_tree = shared_retry_hierarchy();
    let replay_chapter = intake_tree["children"].as_array_mut().unwrap().remove(1);
    let replay_article = replay_chapter["children"][0]["children"][0].clone();
    candidate["presentation"] = json!({
        "size":"XL",
        "profile":"caac-item-presentation-v1",
        "nodes":[
            {"kind":"module","id":"module-intake","title":"Intake","file":"intake.md","decisions":["decision-policy-owner"],"children":[
                intake_tree
            ]},
            {"kind":"module","id":"module-replay","title":"Replay","file":"replay.md","decisions":[],"articles":[
                replay_article
            ]}
        ]
    });
    candidate
}

fn s_candidate() -> Value {
    let mut candidate = shared_retry();
    let articles = candidate["presentation"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|chapter| chapter["articles"].as_array().unwrap().iter().cloned())
        .collect::<Vec<_>>();
    candidate["presentation"] = json!({"size":"S","nodes":articles});
    candidate
}

fn m_extended_candidate() -> Value {
    let mut candidate = shared_retry();
    let chapters = candidate["presentation"]["nodes"].as_array().unwrap();
    let policy = chapters[0]["articles"][0].clone();
    let intake = intake_article();
    let replay = chapters[1]["articles"][0].clone();
    candidate["presentation"] = json!({
        "size":"M",
        "nodes":[
            intake,
            {"kind":"chapter","id":"chapter-intake","title":"Intake","children":[
                {"kind":"section","id":"section-policy","title":"Retry policy","children":[
                    {"kind":"subsection","id":"subsection-retry","title":"Limit and exception","children":[policy]}
                ]}
            ]},
            {"kind":"chapter","id":"chapter-replay","title":"Replay","children":[replay]}
        ]
    });
    candidate
}

fn subitem_candidate(size: &str) -> Value {
    let mut candidate = match size {
        "S" => s_candidate(),
        "M" => m_extended_candidate(),
        "L" => l_candidate(),
        "XL" => xl_candidate(),
        _ => panic!("unsupported presentation size: {size}"),
    };
    candidate["presentation"]["profile"] = json!("caac-subitem-presentation-v1");
    let policy = node_mut(&mut candidate, "article-policy");
    policy.as_object_mut().unwrap().remove("entities");
    policy["paragraphs"] = json!([{
        "id":"paragraph-policy",
        "items":[{
            "id":"item-policy",
            "subitems":[
                {"id":"subitem-priority","entities":["term-priority"]},
                {"id":"subitem-rule","entities":["rule-retry-limit","except-priority"]}
            ]
        }]
    }]);
    candidate
}

fn apply(root: &Path, candidate: &Value, output: &str) -> Output {
    let bytes = serde_json::to_vec(candidate).unwrap();
    fs::write(root.join("base.json"), &bytes).unwrap();
    write(
        root,
        "answers.json",
        &json!({"expected_base":caac::candidate::digest(&bytes),"answers":[]}),
    );
    apply_inputs(root, output)
}

fn apply_inputs(root: &Path, output: &str) -> Output {
    run(
        root,
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

fn verify(root: &Path, package: &str, trace_id: Option<&str>) -> Output {
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
    if let Some(id) = trace_id {
        args.extend(["--trace", id]);
    }
    run(root, &args)
}

fn trace(root: &Path, package: &str, id: &str) -> Value {
    let output = verify(root, package, Some(id));
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    serde_json::from_slice(&output.stdout).unwrap()
}

fn node_mut<'a>(value: &'a mut Value, id: &str) -> &'a mut Value {
    fn find<'a>(nodes: &'a mut [Value], id: &str) -> Option<&'a mut Value> {
        for node in nodes.iter_mut() {
            if node["id"] == id {
                return Some(node);
            }
            let field = if node.get("children").and_then(Value::as_array).is_some() {
                "children"
            } else {
                "articles"
            };
            if let Some(children) = node.get_mut(field).and_then(Value::as_array_mut)
                && let Some(found) = find(children, id)
            {
                return Some(found);
            }
        }
        None
    }
    find(value["presentation"]["nodes"].as_array_mut().unwrap(), id).unwrap()
}

fn revise_presentation(root: &Path, request: &Value) -> Output {
    revise_from(root, "source", request, "revised")
}

fn revise_from(root: &Path, source: &str, request: &Value, output: &str) -> Output {
    write(root, "request.json", request);
    run(
        root,
        &[
            "candidate",
            "revise",
            "--candidate",
            &format!("{source}/candidate.json"),
            "--provenance",
            &format!("{source}/provenance.json"),
            "--review",
            &format!("{source}/review.md"),
            "--presentation",
            "request.json",
            "--output",
            output,
        ],
    )
}

fn ancestors(trace: &Value) -> &Value {
    &trace["selected"]["mapping"][0]["ancestors"]
}

#[test]
fn l_publishes_one_hierarchical_review_with_global_article_numbers_and_ancestry() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let output = apply(root, &l_candidate(), "source");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(verify(root, "source", None).status.code(), Some(0));

    for file in ["candidate.json", "provenance.json", "review.md"] {
        assert!(root.join("source").join(file).is_file(), "{file}");
    }
    for file in ["intake.md", "replay.md"] {
        assert!(!root.join("source").join(file).exists(), "{file}");
    }
    let manifest = read(root, "source/provenance.json");
    assert_eq!(
        manifest["renderer_profile"],
        "caac-candidate-legal-review-v6"
    );
    assert_eq!(
        manifest["revision_manifest"]["format"],
        "caac-candidate-revisions-v6"
    );
    let review = fs::read_to_string(root.join("source/review.md")).unwrap();
    for label in [
        "Part 1",
        "Chapter 1",
        "Chapter 2",
        "Section 1",
        "Subsection 1",
        "Article 1",
        "Article 2",
        "Article 3",
        "Operations",
        "Retry policy",
        "New jobs",
        "Replayed jobs",
    ] {
        assert!(review.contains(label), "review omits {label}");
    }
    assert!(
        review
            .lines()
            .any(|line| line.starts_with("###### Clause ·")),
        "deep clause title must remain a valid H6 heading"
    );
    assert!(
        !review.lines().any(|line| line.starts_with("#######")),
        "renderer emitted an invalid H7 heading"
    );

    let rule = trace(root, "source", "rule-retry-limit");
    assert_eq!(
        rule["selected"]["mapping"][0]["location"]["node_id"],
        "article-policy"
    );
    assert_eq!(rule["selected"]["mapping"][0]["location"]["number"], 1);
    assert_eq!(
        ancestors(&rule),
        &json!([
            {"kind":"part","node_id":"part-operations","number":1},
            {"kind":"chapter","node_id":"chapter-intake","number":1},
            {"kind":"section","node_id":"section-policy","number":1},
            {"kind":"subsection","node_id":"subsection-retry","number":1}
        ])
    );

    let intake = trace(root, "source", "operation-intake");
    assert_eq!(intake["selected"]["mapping"][0]["location"]["kind"], "item");
    assert_eq!(
        intake["selected"]["mapping"][0]["location"]["article_number"],
        2
    );
    assert_eq!(
        intake["selected"]["mapping"][0]["location"]["node_id"],
        "item-intake"
    );
    assert_eq!(
        ancestors(&intake),
        &json!([
            {"kind":"part","node_id":"part-operations","number":1},
            {"kind":"chapter","node_id":"chapter-intake","number":1}
        ])
    );
    let replay = trace(root, "source", "operation-replay");
    assert_eq!(
        replay["selected"]["mapping"][0]["location"]["node_id"],
        "article-replay"
    );
    assert_eq!(replay["selected"]["mapping"][0]["location"]["number"], 3);
    assert_eq!(
        ancestors(&replay),
        &json!([
            {"kind":"part","node_id":"part-operations","number":1},
            {"kind":"chapter","node_id":"chapter-replay","number":2},
            {"kind":"section","node_id":"section-replay","number":1}
        ])
    );
}

#[test]
fn l_part_can_contain_a_section_without_a_chapter() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let mut candidate = l_candidate();
    let part_children = candidate["presentation"]["nodes"][0]["children"]
        .as_array_mut()
        .unwrap();
    let chapter = part_children.pop().unwrap();
    part_children.push(chapter["children"][0].clone());

    let output = apply(root, &candidate, "source");
    assert_eq!(output.status.code(), Some(0), "{output:?}");

    let replay = trace(root, "source", "operation-replay");
    let location = &replay["selected"]["mapping"][0]["location"];
    assert_eq!(location["node_id"], "article-replay");
    assert_eq!(location["number"], 3);
    assert_eq!(
        ancestors(&replay),
        &json!([
            {"kind":"part","node_id":"part-operations","number":1},
            {"kind":"section","node_id":"section-replay","number":1}
        ])
    );
}

#[test]
fn xl_renders_mixed_flat_and_hierarchical_modules_with_global_article_numbers() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let output = apply(root, &xl_candidate(), "source");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(verify(root, "source", None).status.code(), Some(0));

    for file in [
        "candidate.json",
        "provenance.json",
        "review.md",
        "intake.md",
        "replay.md",
    ] {
        assert!(root.join("source").join(file).is_file(), "{file}");
    }
    let index = fs::read_to_string(root.join("source/review.md")).unwrap();
    let intake = fs::read_to_string(root.join("source/intake.md")).unwrap();
    let replay = fs::read_to_string(root.join("source/replay.md")).unwrap();
    assert!(index.contains("intake.md") && index.contains("replay.md"));
    assert!(!index.contains("The retry policy returns a retry limit of 3."));
    assert!(intake.contains("Article 1") && intake.contains("Article 2"));
    assert!(replay.contains("Article 3"));
    assert!(intake.contains("The retry policy returns a retry limit of 3."));
    assert!(!replay.contains("The retry policy returns a retry limit of 3."));
    let provenance = read(root, "source/provenance.json");
    assert_eq!(
        provenance["renderer_profile"],
        "caac-candidate-legal-review-v6"
    );
    assert_eq!(
        provenance["revision_manifest"]["format"],
        "caac-candidate-revisions-v6"
    );

    let rule = trace(root, "source", "rule-retry-limit");
    assert_eq!(rule["selected"]["mapping"][0]["file"], "intake.md");
    assert_eq!(rule["selected"]["mapping"][0]["location"]["number"], 1);
    assert_eq!(
        ancestors(&rule),
        &json!([
            {"kind":"part","node_id":"part-operations","number":1},
            {"kind":"chapter","node_id":"chapter-intake","number":1},
            {"kind":"section","node_id":"section-policy","number":1},
            {"kind":"subsection","node_id":"subsection-retry","number":1}
        ])
    );
    let replay = trace(root, "source", "operation-replay");
    assert_eq!(replay["selected"]["mapping"][0]["file"], "replay.md");
    assert_eq!(replay["selected"]["mapping"][0]["location"]["number"], 3);
    assert!(ancestors(&replay).is_null());
}

#[test]
fn moving_a_clause_to_another_parent_preserves_its_object_and_semantic_revision() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    assert_eq!(apply(root, &l_candidate(), "source").status.code(), Some(0));
    let before_candidate = read(root, "source/candidate.json");
    let before = read(root, "source/provenance.json");
    let before_trace = trace(root, "source", "rule-retry-limit");

    let mut presentation = before_candidate["presentation"].clone();
    let source =
        &mut presentation["nodes"][0]["children"][0]["children"][0]["children"][0]["children"][0];
    source["entities"]
        .as_array_mut()
        .unwrap()
        .retain(|id| id != "rule-retry-limit");
    let destination = &mut presentation["nodes"][0]["children"][1]["children"][0]["children"][0];
    destination["entities"]
        .as_array_mut()
        .unwrap()
        .push(json!("rule-retry-limit"));

    let output = revise_presentation(
        root,
        &json!({
            "format":"caac-presentation-request-v1",
            "expected_candidate_sha256":before["candidate_sha256"],
            "expected_package_revision":before["revision_manifest"]["package_revision"],
            "presentation":presentation
        }),
    );
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(verify(root, "revised", None).status.code(), Some(0));

    let after_candidate = read(root, "revised/candidate.json");
    let after = read(root, "revised/provenance.json");
    assert_eq!(before_candidate["semantic"], after_candidate["semantic"]);
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
    let after_trace = trace(root, "revised", "rule-retry-limit");
    assert_eq!(
        before_trace["selected"]["reference"]["entity_revision"],
        after_trace["selected"]["reference"]["entity_revision"]
    );
    assert_eq!(
        after_trace["selected"]["mapping"][0]["location"]["node_id"],
        "article-replay"
    );
    assert_eq!(
        after_trace["selected"]["mapping"][0]["location"]["number"],
        3
    );
    assert_eq!(
        ancestors(&after_trace),
        &json!([
            {"kind":"part","node_id":"part-operations","number":1},
            {"kind":"chapter","node_id":"chapter-replay","number":2},
            {"kind":"section","node_id":"section-replay","number":1}
        ])
    );
}

#[test]
fn deep_clause_edit_survives_compare_review_revise_and_verify() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    assert_eq!(apply(root, &l_candidate(), "source").status.code(), Some(0));
    let original = fs::read_to_string(root.join("source/review.md")).unwrap();
    let old_text = "The retry policy returns a retry limit of 3.";
    let new_text = "The retry policy returns a retry limit of 4.";
    assert_eq!(original.matches(old_text).count(), 2);
    fs::write(root.join("edited.md"), original.replace(old_text, new_text)).unwrap();
    let proposal = run(
        root,
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
    assert_eq!(proposal.status.code(), Some(0), "{proposal:?}");
    assert_eq!(read(root, "proposal/proposal.json")["status"], "proposed");
    let revised = run(
        root,
        &[
            "candidate",
            "revise",
            "--candidate",
            "source/candidate.json",
            "--provenance",
            "source/provenance.json",
            "--review",
            "source/review.md",
            "--proposal",
            "proposal/proposal.json",
            "--output",
            "revised",
        ],
    );
    assert_eq!(revised.status.code(), Some(0), "{revised:?}");
    assert_eq!(verify(root, "revised", None).status.code(), Some(0));
    let review = fs::read_to_string(root.join("revised/review.md")).unwrap();
    assert!(review.contains(new_text));
    assert!(!review.lines().any(|line| line.starts_with("#######")));
    let trace = trace(root, "revised", "rule-retry-limit");
    assert_eq!(trace["selected"]["body"]["text"], new_text);
    assert_eq!(
        ancestors(&trace),
        &json!([
            {"kind":"part","node_id":"part-operations","number":1},
            {"kind":"chapter","node_id":"chapter-intake","number":1},
            {"kind":"section","node_id":"section-policy","number":1},
            {"kind":"subsection","node_id":"subsection-retry","number":1}
        ])
    );
}

#[test]
fn verification_rejects_a_tampered_hierarchy_ancestor_binding() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    assert_eq!(apply(root, &l_candidate(), "source").status.code(), Some(0));
    let mut provenance = read(root, "source/provenance.json");
    let entry = provenance["revision_manifest"]["presentation"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entry| entry["target"]["id"] == "rule-retry-limit")
        .unwrap();
    entry["ancestors"][0]["number"] = json!(9);
    write(root, "source/provenance.json", &provenance);
    let output = verify(root, "source", None);
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("Revision manifest does not match the exact typed candidate snapshot"),
        "{output:?}"
    );
}

fn rejects(root: &Path, candidate: &Value, name: &str) {
    let output = apply(root, candidate, name);
    assert_eq!(output.status.code(), Some(2), "{name}: {output:?}");
    assert!(output.stdout.is_empty(), "{name}: {output:?}");
    assert!(!root.join(name).exists(), "{name} published output");
}

#[test]
fn malformed_hierarchy_and_reference_shapes_publish_no_package() {
    let mut cases: Vec<(&str, Value)> = Vec::new();

    let mut inverted = l_candidate();
    node_mut(&mut inverted, "chapter-intake")["kind"] = json!("section");
    node_mut(&mut inverted, "section-policy")["kind"] = json!("chapter");
    cases.push(("inverted-level", inverted));

    let mut repeated = l_candidate();
    node_mut(&mut repeated, "section-policy")["kind"] = json!("chapter");
    cases.push(("repeated-level", repeated));

    let mut empty = l_candidate();
    node_mut(&mut empty, "part-operations")["children"] = json!([]);
    cases.push(("empty-group", empty));

    let mut nested_module = l_candidate();
    node_mut(&mut nested_module, "chapter-intake")["children"][0] = json!({
        "kind":"module","id":"nested-module","title":"Nested","file":"nested.md",
        "decisions":[],"children":[article("article-policy","Shared retry policy", &["term-priority","rule-retry-limit","except-priority"])]
    });
    cases.push(("module-nested-in-tree", nested_module));

    let mut misplaced = shared_retry();
    let original_article = misplaced["presentation"]["nodes"][0]["articles"][0].clone();
    misplaced["presentation"]["nodes"][0]["articles"][0] = json!({
        "kind":"section","id":"misplaced-section","title":"Misplaced group","children":[original_article]
    });
    cases.push(("group-in-legacy-articles", misplaced));

    let mut dropped = l_candidate();
    node_mut(&mut dropped, "article-policy")["entities"]
        .as_array_mut()
        .unwrap()
        .retain(|id| id != "term-priority");
    cases.push(("dropped-entity-reference", dropped));

    let mut unknown = l_candidate();
    node_mut(&mut unknown, "article-policy")["entities"][1] = json!("missing-entity");
    cases.push(("unknown-entity-reference", unknown));

    let mut duplicate_entity = l_candidate();
    node_mut(&mut duplicate_entity, "article-intake")["paragraphs"][0]["items"][0]["entities"]
        .as_array_mut()
        .unwrap()
        .push(json!("rule-retry-limit"));
    cases.push(("duplicate-entity-placement", duplicate_entity));

    let mut duplicate_node = l_candidate();
    node_mut(&mut duplicate_node, "section-replay")["id"] = json!("section-policy");
    cases.push(("duplicate-node-id-across-tree", duplicate_node));

    let mut ambiguous_chapter = l_candidate();
    node_mut(&mut ambiguous_chapter, "chapter-intake")["articles"] = json!([]);
    cases.push(("chapter-both-articles-and-children", ambiguous_chapter));

    let mut absent_chapter = l_candidate();
    node_mut(&mut absent_chapter, "chapter-intake")
        .as_object_mut()
        .unwrap()
        .remove("children");
    cases.push(("chapter-neither-articles-nor-children", absent_chapter));

    let mut null_articles = l_candidate();
    node_mut(&mut null_articles, "chapter-intake")["articles"] = Value::Null;
    cases.push(("chapter-null-articles", null_articles));

    let mut null_children = shared_retry();
    null_children["presentation"]["nodes"][0]["children"] = Value::Null;
    cases.push(("chapter-null-children", null_children));

    let mut m_part = shared_retry();
    let chapters = m_part["presentation"]["nodes"].clone();
    m_part["presentation"]["nodes"] = json!([{
        "kind":"part","id":"part-unsupported","title":"Unsupported","children":chapters
    }]);
    cases.push(("m-rejects-part-level", m_part));

    let mut both_module_fields = xl_candidate();
    node_mut(&mut both_module_fields, "module-intake")["articles"] = json!([]);
    cases.push(("module-both-articles-and-children", both_module_fields));

    let mut neither_module_field = xl_candidate();
    node_mut(&mut neither_module_field, "module-replay")
        .as_object_mut()
        .unwrap()
        .remove("articles");
    cases.push(("module-neither-articles-nor-children", neither_module_field));

    let mut null_module_articles = xl_candidate();
    node_mut(&mut null_module_articles, "module-intake")["articles"] = Value::Null;
    cases.push(("module-null-articles", null_module_articles));

    let mut null_module_children = xl_candidate();
    node_mut(&mut null_module_children, "module-replay")["children"] = Value::Null;
    cases.push(("module-null-children", null_module_children));

    let mut nested_xl_module = xl_candidate();
    node_mut(&mut nested_xl_module, "part-operations")["children"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "kind":"module","id":"nested-module","title":"Nested","file":"nested.md",
            "decisions":[],"children":[article("article-extra","Extra", &["operation-replay"])]
        }));
    cases.push(("module-nested-in-xl-hierarchy", nested_xl_module));

    let directory = tempfile::tempdir().unwrap();
    for (name, candidate) in cases {
        rejects(directory.path(), &candidate, name);
    }
}

#[test]
fn legacy_s_and_m_keep_renderer_and_revision_profiles() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    for (size, candidate) in [("S", s_candidate()), ("M", shared_retry())] {
        let package = format!("source-{size}");
        let output = apply(root, &candidate, &package);
        assert_eq!(output.status.code(), Some(0), "{size}: {output:?}");
        assert_eq!(
            verify(root, &package, None).status.code(),
            Some(0),
            "{size}"
        );
        let saved = read(root, &format!("{package}/candidate.json"));
        let provenance = read(root, &format!("{package}/provenance.json"));
        assert_eq!(saved["presentation"]["size"], size);
        assert_eq!(
            provenance["renderer_profile"], "caac-candidate-legal-review-v1",
            "legacy {size} renderer profile changed"
        );
        assert_eq!(
            provenance["revision_manifest"]["format"], "caac-candidate-revisions-v1",
            "legacy {size} revision profile changed"
        );
        if size == "M" {
            assert!(saved["presentation"]["nodes"][0]["articles"].is_array());
            assert!(saved["presentation"]["nodes"][0].get("children").is_none());
        }
    }
}

#[test]
fn subitems_render_and_trace_across_s_to_xl_with_m_ancestor_paths() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    for size in ["S", "M", "L", "XL"] {
        let package = size.to_lowercase();
        let output = apply(root, &subitem_candidate(size), &package);
        assert_eq!(output.status.code(), Some(0), "{size}: {output:?}");
        assert_eq!(
            verify(root, &package, None).status.code(),
            Some(0),
            "{size}"
        );

        let provenance = read(root, &format!("{package}/provenance.json"));
        assert_eq!(
            provenance["renderer_profile"], "caac-candidate-legal-review-v7",
            "{size} renderer profile"
        );
        assert_eq!(
            provenance["revision_manifest"]["format"], "caac-candidate-revisions-v7",
            "{size} revision profile"
        );
        let review_path = if size == "XL" {
            format!("{package}/intake.md")
        } else {
            format!("{package}/review.md")
        };
        let review = fs::read_to_string(root.join(review_path)).unwrap();
        assert!(review.contains("Subitem profile: `caac-subitem-presentation-v1`"));
        assert!(review.contains("Subitem 1"));
    }

    let review = fs::read_to_string(root.join("m/review.md")).unwrap();
    assert!(review.contains("Chapter 1") && review.contains("Section 1"));
    assert!(review.contains("Subsection 1") && review.contains("Article 1"));
    assert!(
        review
            .lines()
            .any(|line| line.starts_with("###### Subitem 2"))
    );
    assert!(
        !review.lines().any(|line| line.starts_with("#######")),
        "renderer emitted a heading deeper than H6"
    );

    let trace = trace(root, "m", "rule-retry-limit");
    assert_eq!(
        trace["selected"]["mapping"][0]["location"],
        json!({
            "kind":"subitem",
            "node_id":"subitem-rule",
            "item_id":"item-policy",
            "paragraph_id":"paragraph-policy",
            "article_id":"article-policy",
            "chapter_id":"chapter-intake",
            "article_number":2,
            "paragraph_number":1,
            "item_number":1,
            "number":2
        })
    );
    assert_eq!(
        ancestors(&trace),
        &json!([
            {"kind":"chapter","node_id":"chapter-intake","number":1},
            {"kind":"section","node_id":"section-policy","number":1},
            {"kind":"subsection","node_id":"subsection-retry","number":1}
        ])
    );
}

#[test]
fn moving_a_subitem_to_another_parent_updates_its_trace_without_semantic_revision() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    assert_eq!(
        apply(root, &subitem_candidate("M"), "source").status.code(),
        Some(0)
    );
    let before_candidate = read(root, "source/candidate.json");
    let before = read(root, "source/provenance.json");
    let before_trace = trace(root, "source", "rule-retry-limit");

    let mut presentation = before_candidate["presentation"].clone();
    let moved_subitem = presentation["nodes"][1]["children"][0]["children"][0]
        ["children"][0]["paragraphs"][0]["items"][0]["subitems"]
        .as_array_mut()
        .unwrap()
        .remove(1);
    let target_item = &mut presentation["nodes"][0]["paragraphs"][0]["items"][0];
    target_item.as_object_mut().unwrap().remove("entities");
    target_item["subitems"] = json!([
        {"id":"subitem-intake","entities":["operation-intake","test-intake-standard"]},
        moved_subitem
    ]);

    let output = revise_presentation(
        root,
        &json!({
            "format":"caac-presentation-request-v1",
            "expected_candidate_sha256":before["candidate_sha256"],
            "expected_package_revision":before["revision_manifest"]["package_revision"],
            "presentation":presentation
        }),
    );
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(verify(root, "revised", None).status.code(), Some(0));

    let after_candidate = read(root, "revised/candidate.json");
    let after = read(root, "revised/provenance.json");
    assert_eq!(before_candidate["semantic"], after_candidate["semantic"]);
    assert_eq!(
        before["revision_manifest"]["semantic_revision"],
        after["revision_manifest"]["semantic_revision"]
    );
    assert_ne!(
        before["revision_manifest"]["presentation_revision"],
        after["revision_manifest"]["presentation_revision"]
    );

    let after_trace = trace(root, "revised", "rule-retry-limit");
    assert_eq!(
        before_trace["selected"]["reference"]["entity_revision"],
        after_trace["selected"]["reference"]["entity_revision"]
    );
    assert_eq!(
        after_trace["selected"]["mapping"][0]["location"],
        json!({
            "kind":"subitem",
            "node_id":"subitem-rule",
            "item_id":"item-intake",
            "paragraph_id":"paragraph-intake",
            "article_id":"article-intake",
            "chapter_id":null,
            "article_number":1,
            "paragraph_number":1,
            "item_number":1,
            "number":2
        })
    );
    assert!(ancestors(&after_trace).is_null());
}

#[test]
fn malformed_subitem_item_shapes_publish_no_package() {
    let mut cases = Vec::new();

    let mut both = subitem_candidate("M");
    node_mut(&mut both, "article-policy")["paragraphs"][0]["items"][0]["entities"] =
        json!(["term-priority"]);
    cases.push(("item-both-entities-and-subitems", both));

    let mut neither = subitem_candidate("M");
    node_mut(&mut neither, "article-policy")["paragraphs"][0]["items"][0]
        .as_object_mut()
        .unwrap()
        .remove("subitems");
    cases.push(("item-without-entities-or-subitems", neither));

    let mut empty_subitems = subitem_candidate("M");
    node_mut(&mut empty_subitems, "article-policy")["paragraphs"][0]["items"][0]["subitems"] =
        json!([]);
    cases.push(("item-empty-subitems", empty_subitems));

    let mut empty_subitem_entities = subitem_candidate("M");
    node_mut(&mut empty_subitem_entities, "article-policy")["paragraphs"][0]["items"][0]["subitems"]
        [0]["entities"] = json!([]);
    cases.push(("subitem-empty-entities", empty_subitem_entities));

    let directory = tempfile::tempdir().unwrap();
    for (name, candidate) in cases {
        rejects(directory.path(), &candidate, name);
    }
}

const ECHO_POLICY: &[&str] = &["term-zero", "rule-echo", "except-zero"];
const ECHO_CASES: &[&str] = &["case-zero", "case-negative"];
const ECHO_ATTACHMENTS: &[&str] = &["annex-echo-input", "form-echo-request"];
const ECHO_RULE: &str = "Echo returns the supplied integer.";
const ECHO_QUESTION: &str = "Which integer operations are in scope?";

fn echo_entity_mut<'a>(entities: &'a mut [Value], id: &str) -> &'a mut Value {
    entities
        .iter_mut()
        .find(|entity| entity["id"] == id)
        .unwrap()
}

fn echo_with_attachments(original: &Value) -> Value {
    let mut base = original.clone();
    let entities = base["semantic"]["entities"].as_array_mut().unwrap();
    let rule = echo_entity_mut(entities, "rule-echo");
    rule["annex_refs"] = json!([{"id":"annex-echo-input", "usage":"illustrative"}]);
    rule["form_refs"] = json!([{"id":"form-echo-request", "usage":"illustrative"}]);

    entities.extend([
        json!({"id":"annex-echo-input", "kind":"annex", "status":"illustrative",
            "title":"Declared Echo parameter example", "columns":["parameter","declared type"],
            "rows":[["value","i32"]]}),
        json!({"id":"form-echo-request", "kind":"form", "status":"illustrative",
        "title":"Echo invocation field example", "fields":[
            {"id":"value","label":"Value parameter","value_type":"i32","required":true}
        ]}),
    ]);
    let mut main = ECHO_POLICY.to_vec();
    main.extend(["operation-echo", "case-zero", "case-negative"]);
    base["presentation"] = json!({
        "size":"XS",
        "nodes":[article("article-main", "Echo behavior", &main)],
        "attachments":ECHO_ATTACHMENTS,
    });
    base
}

fn echo_presentations() -> [(&'static str, Value); 5] {
    let policy = json!({"kind":"article", "id":"article-main", "title":"Echo behavior",
    "paragraphs":[{"id":"paragraph-policy","items":[{"id":"item-policy","subitems":[
        {"id":"subitem-policy","entities":ECHO_POLICY}
    ]}]}]});
    let operation = article("article-operation", "Invocation", &["operation-echo"]);
    let cases = article("article-cases", "Examples", ECHO_CASES);
    let section = json!({"kind":"section","id":"section-policy","title":"Policy",
        "children":[policy]});
    let part = json!({"kind":"part","id":"part-echo","title":"Echo",
        "children":[section,cases]});
    let profile = "caac-subitem-presentation-v1";
    let xl = json!({
        "size":"XL", "profile":profile,
        "nodes":[
            {"kind":"module","id":"module-policy","title":"Policy","file":"policy.md",
             "articles":[policy,operation],"attachments":ECHO_ATTACHMENTS,"decisions":["decision-scope"]},
            {"kind":"module","id":"module-usage","title":"Usage","file":"usage.md",
             "articles":[cases],"decisions":[]}
        ]
    });
    let mut moved = xl.clone();
    let policy_article = moved["nodes"][0]["articles"]
        .as_array_mut()
        .unwrap()
        .remove(0);
    moved["nodes"][1]["articles"]
        .as_array_mut()
        .unwrap()
        .push(policy_article);
    [
        (
            "s",
            json!({"size":"S", "profile":profile, "nodes":[policy,operation,cases],
            "attachments":ECHO_ATTACHMENTS}),
        ),
        (
            "m",
            json!({"size":"M", "profile":profile, "nodes":[operation,section,cases],
            "attachments":ECHO_ATTACHMENTS}),
        ),
        (
            "l",
            json!({"size":"L", "profile":profile, "nodes":[operation,part],
            "attachments":ECHO_ATTACHMENTS}),
        ),
        ("xl", xl),
        ("xl-moved", moved),
    ]
}

fn artifact_hash(root: &Path, path: &str) {
    let bytes = fs::read(root.join(path)).unwrap();
    println!("{}  {path}", caac::candidate::digest(&bytes));
}

fn echo_markdown_block<'a>(review: &'a str, heading: &str) -> &'a str {
    let (_, body) = review
        .split_once(heading)
        .unwrap_or_else(|| panic!("missing {heading}"));
    body.split("\n#").next().unwrap()
}

fn check_echo_markdown(rule_file: &str, decision_file: &str) {
    let rule = echo_markdown_block(rule_file, "Clause · ` \"rule-echo\" `\n");
    assert!(
        rule.lines()
            .any(|line| line == format!("- **Authored text:** ` \"{ECHO_RULE}\" `"))
    );
    assert!(!rule.contains(ECHO_QUESTION));

    let decision = echo_markdown_block(decision_file, "### Decision ` \"decision-scope\" `\n");
    assert!(
        decision
            .lines()
            .any(|line| line == format!("- **Open question:** ` \"{ECHO_QUESTION}\" `"))
    );
    assert!(
        decision
            .lines()
            .any(|line| line == "- **Required decision:** ` true `")
    );
    assert!(!decision.contains(ECHO_RULE));
}

fn show_xl_markdown(path: &str, review: &str) {
    println!("Markdown in {path}:");
    for line in review.lines().filter(|line| {
        line.starts_with("# Candidate")
            || line.starts_with("- [`")
            || (line.starts_with('#')
                && [
                    "Article ",
                    "Clause ·",
                    "Exception ·",
                    "Operation ·",
                    "Case ·",
                    "Attachment ",
                    "Decision ",
                ]
                .iter()
                .any(|label| line.contains(label)))
            || line.contains("**Authored text:**") && line.contains(ECHO_RULE)
            || line.contains("**Open question:**")
    }) {
        println!("{line}");
    }
}

fn check_echo_mapping(
    root: &Path,
    stage: &str,
    id: &str,
    file: &str,
    location: &Value,
    ancestors: &Value,
) -> Value {
    let report = trace(root, stage, id);
    let mappings = report["selected"]["mapping"].as_array().unwrap();
    assert_eq!(mappings.len(), 1, "{stage} {id}");
    let mapping = &mappings[0];
    assert_eq!(mapping["file"], file, "{stage} {id}");
    assert_eq!(&mapping["location"], location, "{stage} {id}");
    assert_eq!(mapping["target"], report["selected"]["reference"]);
    assert_eq!(
        mapping["target"]["package_revision"],
        report["source"]["package_revision"]
    );
    let observed = mapping
        .get("ancestors")
        .cloned()
        .unwrap_or_else(|| json!([]));
    assert_eq!(&observed, ancestors, "{stage} {id}");
    report
}

fn reject_echo(root: &Path, result: Output, path: &str, diagnostic: &str) {
    assert_eq!(result.status.code(), Some(2), "{path}: {result:?}");
    assert!(result.stdout.is_empty(), "{path}: {result:?}");
    let stderr = String::from_utf8(result.stderr).unwrap();
    assert!(stderr.contains(diagnostic), "{path}: {stderr}");
    assert!(!root.join(path).exists(), "{path}");
    println!("{path}: exit 2, empty stdout, {}", stderr.trim());
}

#[test]
fn one_echo_package_preserves_bindings_across_all_sizes_and_an_xl_move() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    fs::write(
        root.join("base.json"),
        include_bytes!("../examples/candidate/draft.json"),
    )
    .unwrap();
    fs::write(
        root.join("answers.json"),
        include_bytes!("../examples/candidate/answers.json"),
    )
    .unwrap();
    let applied = apply_inputs(root, "original");
    assert_eq!(applied.status.code(), Some(0), "{applied:?}");
    assert_eq!(verify(root, "original", None).status.code(), Some(0));
    let original = read(root, "original/candidate.json");
    let original_review = fs::read_to_string(root.join("original/review.md")).unwrap();
    check_echo_markdown(&original_review, &original_review);
    for file in [
        "original/candidate.json",
        "original/provenance.json",
        "original/review.md",
    ] {
        artifact_hash(root, file);
    }

    let base = echo_with_attachments(&original);
    let applied = apply(root, &base, "xs");
    assert_eq!(applied.status.code(), Some(0), "{applied:?}");
    artifact_hash(root, "base.json");
    artifact_hash(root, "answers.json");

    let initial = read(root, "xs/candidate.json");
    let mut authored = initial["semantic"].clone();
    let authored_entities = authored["entities"].as_array_mut().unwrap();
    authored_entities.retain(|entity| !matches!(entity["kind"].as_str(), Some("annex" | "form")));
    let rule = echo_entity_mut(authored_entities, "rule-echo");
    assert_eq!(rule["text"], ECHO_RULE);
    let rule = rule.as_object_mut().unwrap();
    rule.remove("annex_refs").unwrap();
    rule.remove("form_refs").unwrap();
    let operation = echo_entity_mut(authored_entities, "operation-echo");
    assert_eq!(operation["inputs"], json!({"value":"i32"}));
    assert_eq!(operation["output"], "i32");
    assert_eq!(
        authored["unresolved"],
        json!([{
            "id":"decision-scope", "question":ECHO_QUESTION, "required":true
        }])
    );
    assert_eq!(authored, original["semantic"]);

    let first = read(root, "xs/provenance.json");
    let first_manifest = &first["revision_manifest"];
    let edges = first_manifest["references"].as_array().unwrap();
    for (source, field, target, usage) in [
        ("operation-echo", "clause_refs", "rule-echo", None),
        ("rule-echo", "definition_refs", "term-zero", None),
        (
            "rule-echo",
            "annex_refs",
            "annex-echo-input",
            Some("illustrative"),
        ),
        (
            "rule-echo",
            "form_refs",
            "form-echo-request",
            Some("illustrative"),
        ),
        ("except-zero", "target", "rule-echo", None),
        ("case-zero", "exception_refs", "except-zero", None),
    ] {
        assert!(
            edges.iter().any(|edge| {
                edge["source"]["id"] == source
                    && edge["field"] == field
                    && edge["target"]["id"] == target
                    && edge["attachment_usage"].as_str() == usage
            }),
            "{source} {field} {target}"
        );
    }
    let mut source = "xs";
    for (destination, presentation) in echo_presentations()
        .into_iter()
        .chain([("xs-restored", base["presentation"].clone())])
    {
        let parent = read(root, &format!("{source}/provenance.json"));
        let request = json!({
            "format":"caac-presentation-request-v1",
            "expected_candidate_sha256":parent["candidate_sha256"],
            "expected_package_revision":parent["revision_manifest"]["package_revision"],
            "presentation":presentation,
        });
        let result = revise_from(root, source, &request, destination);
        assert_eq!(result.status.code(), Some(0), "{destination}: {result:?}");
        println!("{source} -> {destination}");
        artifact_hash(root, "request.json");

        let provenance = read(root, &format!("{destination}/provenance.json"));
        let manifest = &provenance["revision_manifest"];
        for field in ["semantic_revision", "objects"] {
            assert_eq!(
                manifest[field], first_manifest[field],
                "{destination} {field}"
            );
        }
        for field in ["presentation_revision", "package_revision"] {
            assert_ne!(
                manifest[field], parent["revision_manifest"][field],
                "{destination} {field}"
            );
        }
        assert_eq!(provenance["semantic_sha256"], first["semantic_sha256"]);
        assert_eq!(
            provenance["revision_transition"]["evidence_policy"],
            "fresh-review-required-for-whole-package"
        );

        let mut rebound = first_manifest["references"].clone();
        for edge in rebound.as_array_mut().unwrap() {
            for end in ["source", "target"] {
                edge[end]["package_revision"] = manifest["package_revision"].clone();
            }
        }
        assert_eq!(manifest["references"], rebound, "{destination}");
        source = destination;
    }

    for (stage, policy_number, operation_number, case_number) in [
        ("xs", 0, 0, 0),
        ("s", 1, 2, 3),
        ("m", 2, 1, 3),
        ("l", 2, 1, 3),
        ("xl", 1, 2, 3),
        ("xl-moved", 3, 1, 2),
    ] {
        let verified = verify(root, stage, None);
        assert_eq!(verified.status.code(), Some(0), "{stage}: {verified:?}");
        let candidate = read(root, &format!("{stage}/candidate.json"));
        assert_eq!(candidate["semantic"], initial["semantic"], "{stage}");
        assert_eq!(
            candidate["authority"],
            json!({
                "owner_acceptance":"pending", "implementation_admission":"pending"
            })
        );
        let provenance = read(root, &format!("{stage}/provenance.json"));
        assert_eq!(provenance["behavior_evidence"], "not-run");
        assert_eq!(provenance["contract_complete"], false);
        assert_eq!(provenance["required_unresolved"], 1);
        assert_eq!(
            provenance["renderer_profile"],
            "caac-candidate-legal-review-v8"
        );

        let xl = stage.starts_with("xl");
        let operation_file = if xl { "policy.md" } else { "review.md" };
        let case_file = if xl { "usage.md" } else { "review.md" };
        let policy_file = if stage == "xl-moved" {
            case_file
        } else {
            operation_file
        };
        let review = fs::read_to_string(root.join(stage).join(policy_file)).unwrap();
        let heading = if policy_number == 0 {
            "## ` \"Echo behavior\" `\n".to_owned()
        } else {
            format!("Article {policy_number} — ` \"Echo behavior\" `")
        };
        assert!(review.contains(&heading), "{stage}: missing {heading}");
        let exception = echo_markdown_block(&review, "Exception · ` \"except-zero\" `\n");
        assert!(exception.contains("- **Target clause:** ` \"rule-echo\" `"));
        assert!(!exception.contains("**Sentence reference:**"));
        for label in ["Paragraph 1", "Item 1", "Subitem 1"] {
            assert_eq!(
                review.contains(label),
                policy_number > 0,
                "{stage}: {label}"
            );
        }
        for (label, present) in [
            ("Part 1", stage == "l"),
            ("Chapter 1", false),
            ("Section 1", matches!(stage, "m" | "l")),
            ("Subsection 1", false),
        ] {
            assert_eq!(review.contains(label), present, "{stage}: {label}");
        }

        let attachments = fs::read_to_string(root.join(stage).join(operation_file)).unwrap();
        for text in [
            "## Attachments",
            "Attachment 2",
            "Value parameter",
            "Declared Echo parameter example",
            "Echo invocation field example",
        ] {
            assert!(attachments.contains(text), "{stage}: missing {text}");
        }
        check_echo_markdown(&review, &attachments);
        let mut files = vec!["candidate.json", "provenance.json", "review.md"];
        if xl {
            files.extend(["policy.md", "usage.md"]);
            let index = fs::read_to_string(root.join(stage).join("review.md")).unwrap();
            assert!(index.contains("policy.md") && index.contains("usage.md"));
            for text in [ECHO_RULE, ECHO_QUESTION, "except-zero"] {
                assert!(!index.contains(text), "{stage}: index contains {text}");
            }
            let other_file = if policy_file == "policy.md" {
                case_file
            } else {
                operation_file
            };
            let other_review = fs::read_to_string(root.join(stage).join(other_file)).unwrap();
            for text in [ECHO_RULE, "Exception · ` \"except-zero\" `", &heading] {
                assert!(
                    !other_review.contains(text),
                    "{stage}: {other_file} contains {text}"
                );
            }
            let usage_review = if stage == "xl-moved" {
                &review
            } else {
                &other_review
            };
            assert!(!usage_review.contains(ECHO_QUESTION));
            assert!(!usage_review.contains("## Attachments"));
            show_xl_markdown(&format!("{stage}/review.md"), &index);
            show_xl_markdown(&format!("{stage}/policy.md"), &attachments);
            show_xl_markdown(&format!("{stage}/usage.md"), usage_review);
        }
        assert_eq!(fs::read_dir(root.join(stage)).unwrap().count(), files.len());
        for file in files {
            artifact_hash(root, &format!("{stage}/{file}"));
        }

        let part = json!({"kind":"part","node_id":"part-echo","number":1});
        let section = json!({"kind":"section","node_id":"section-policy","number":1});
        let policy_ancestors = match stage {
            "m" => json!([section]),
            "l" => json!([part, section]),
            _ => json!([]),
        };
        let case_ancestors = if stage == "l" {
            json!([part])
        } else {
            json!([])
        };
        let unnumbered = json!({"kind":"unnumbered-article","node_id":"article-main"});
        let policy_location = if policy_number == 0 {
            unnumbered.clone()
        } else {
            json!({"kind":"subitem","node_id":"subitem-policy","item_id":"item-policy",
                "paragraph_id":"paragraph-policy","article_id":"article-main","chapter_id":null,
                "article_number":policy_number,"paragraph_number":1,"item_number":1,"number":1})
        };
        let article_location = |node, number| {
            if number == 0 {
                unnumbered.clone()
            } else {
                json!({"kind":"article","node_id":node,"chapter_id":null,"number":number})
            }
        };
        for &id in ECHO_POLICY {
            check_echo_mapping(
                root,
                stage,
                id,
                policy_file,
                &policy_location,
                &policy_ancestors,
            );
        }
        for &id in ECHO_CASES {
            check_echo_mapping(
                root,
                stage,
                id,
                case_file,
                &article_location("article-cases", case_number),
                &case_ancestors,
            );
        }
        check_echo_mapping(
            root,
            stage,
            "operation-echo",
            operation_file,
            &article_location("article-operation", operation_number),
            &json!([]),
        );
        for (index, &id) in ECHO_ATTACHMENTS.iter().enumerate() {
            check_echo_mapping(
                root,
                stage,
                id,
                operation_file,
                &json!({"kind":"attachment","node_id":id,"number":index + 1}),
                &json!([]),
            );
        }
        let decision = check_echo_mapping(
            root,
            stage,
            "decision-scope",
            operation_file,
            &json!({"kind":"unresolved-decision","index":0}),
            &json!([]),
        );
        assert_eq!(decision["selected"]["body"]["required"], true);
        assert_eq!(decision["selected"]["body"]["question"], ECHO_QUESTION);
        assert_eq!(decision["unresolved_decisions"][0]["status"], "unresolved");

        let rule = trace(root, stage, "rule-echo");
        assert_eq!(rule["selected"]["body"]["text"], ECHO_RULE);
        let cases = rule["reachable_declared_cases"].as_array().unwrap();
        assert_eq!(cases.len(), 2);
        for (case, input, expected, exceptions) in [
            (&cases[0], 0, 1, json!(["except-zero"])),
            (&cases[1], -7, -7, json!([])),
        ] {
            assert_eq!(case["body"]["input"], json!({"value":input}));
            assert_eq!(case["body"]["expected"], expected);
            assert_eq!(case["body"]["operation"], "operation-echo");
            assert_eq!(case["body"]["clause_refs"], json!(["rule-echo"]));
            assert_eq!(case["body"]["exception_refs"], exceptions);
            assert_eq!(
                case["execution_evidence"],
                "not-recorded-in-supplied-artifacts"
            );
        }
        assert_eq!(rule["evidence_scope"]["completion"], "not-assessed");
    }

    assert_eq!(verify(root, "xs-restored", None).status.code(), Some(0));
    artifact_hash(root, "xs-restored/provenance.json");
    for file in ["candidate.json", "review.md"] {
        assert_eq!(
            fs::read(root.join("xs").join(file)).unwrap(),
            fs::read(root.join("xs-restored").join(file)).unwrap()
        );
    }
    assert_eq!(
        read(root, "xs-restored/provenance.json")["revision_manifest"],
        *first_manifest
    );
    assert_eq!(
        fs::read(root.join("xl/review.md")).unwrap(),
        fs::read(root.join("xl-moved/review.md")).unwrap()
    );

    let mut lossy = base.clone();
    lossy["presentation"]["nodes"][0]["entities"]
        .as_array_mut()
        .unwrap()
        .retain(|id| id != "except-zero");
    let request = json!({
        "format":"caac-presentation-request-v1",
        "expected_candidate_sha256":first["candidate_sha256"],
        "expected_package_revision":first_manifest["package_revision"],
        "presentation":lossy["presentation"],
    });
    let rejected = revise_from(root, "xs", &request, "lossy");
    artifact_hash(root, "request.json");
    reject_echo(
        root,
        rejected,
        "lossy",
        "Presentation must place every semantic entity exactly once",
    );

    lossy["semantic"]["entities"]
        .as_array_mut()
        .unwrap()
        .retain(|entity| entity["id"] != "except-zero");
    let rejected = apply(root, &lossy, "deleted-exception");
    artifact_hash(root, "base.json");
    artifact_hash(root, "answers.json");
    reject_echo(
        root,
        rejected,
        "deleted-exception",
        "Reference except-zero must resolve to exception",
    );

    let mut broken = base.clone();
    broken["semantic"]["entities"][1]["annex_refs"][0]["id"] = json!("annex-missing");
    let rejected = apply(root, &broken, "broken-annex");
    artifact_hash(root, "base.json");
    artifact_hash(root, "answers.json");
    reject_echo(
        root,
        rejected,
        "broken-annex",
        "Reference annex-missing must resolve to annex",
    );

    let mut grouped = base.clone();
    grouped["presentation"]["profile"] = json!("caac-paragraph-presentation-v1");
    let main = &mut grouped["presentation"]["nodes"][0];
    let entities = main.as_object_mut().unwrap().remove("entities").unwrap();
    main["paragraphs"] = json!([{"id":"paragraph-all","entities":entities}]);
    grouped["presentation"]["size"] = json!("S");
    let control = apply(root, &grouped, "grouped-s");
    assert_eq!(control.status.code(), Some(0), "{control:?}");
    assert_eq!(verify(root, "grouped-s", None).status.code(), Some(0));
    assert_eq!(
        read(root, "grouped-s/candidate.json")["semantic"],
        initial["semantic"]
    );
    println!("grouped-s: apply and verify exit 0");
    for file in [
        "base.json",
        "answers.json",
        "grouped-s/candidate.json",
        "grouped-s/provenance.json",
        "grouped-s/review.md",
    ] {
        artifact_hash(root, file);
    }
    grouped["presentation"]["size"] = json!("XS");
    let rejected = apply(root, &grouped, "grouped-xs");
    artifact_hash(root, "base.json");
    artifact_hash(root, "answers.json");
    reject_echo(
        root,
        rejected,
        "grouped-xs",
        "The XS function profile requires one flat article",
    );

    let mut whole_clause_part = base.clone();
    let entities = whole_clause_part["semantic"]["entities"]
        .as_array_mut()
        .unwrap();
    echo_entity_mut(entities, "except-zero")["sentence"] = json!("sentence-whole-clause");
    entities.push(json!({
        "id":"sentence-whole-clause", "kind":"sentence-part", "clause":"rule-echo",
        "portion":"first", "text":ECHO_RULE
    }));
    whole_clause_part["presentation"]["nodes"][0]["entities"]
        .as_array_mut()
        .unwrap()
        .push(json!("sentence-whole-clause"));
    let rejected = apply(root, &whole_clause_part, "whole-clause-part");
    artifact_hash(root, "base.json");
    artifact_hash(root, "answers.json");
    reject_echo(
        root,
        rejected,
        "whole-clause-part",
        "Sentence part must match a proper prefix or suffix of its clause",
    );

    let mut revised_semantics = base;
    revised_semantics["semantic"]["entities"][0]["text"] =
        json!("Zero is the integer denoted by 0.");
    let changed = apply(root, &revised_semantics, "changed");
    assert_eq!(changed.status.code(), Some(0), "{changed:?}");
    assert_eq!(verify(root, "changed", None).status.code(), Some(0));
    let provenance = read(root, "changed/provenance.json");
    assert_ne!(
        provenance["revision_manifest"]["semantic_revision"],
        first_manifest["semantic_revision"]
    );
    assert_eq!(provenance["behavior_evidence"], "not-run");
    for file in [
        "base.json",
        "answers.json",
        "changed/candidate.json",
        "changed/provenance.json",
        "changed/review.md",
    ] {
        artifact_hash(root, file);
    }
    let rejected = run(
        root,
        &[
            "candidate",
            "verify",
            "--candidate",
            "changed/candidate.json",
            "--provenance",
            "xs/provenance.json",
            "--review",
            "changed/review.md",
        ],
    );
    reject_echo(
        root,
        rejected,
        "stale-provenance",
        "Revision manifest does not match the exact typed candidate snapshot",
    );
}
