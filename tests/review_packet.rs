use caac::{
    candidate::digest,
    contract_collection::{Request, collect},
    review_packet::prepare_review,
};
use serde_json::Value;
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
fn setup(p: &Path) -> Request {
    fs::write(
        p.join("contract.md"),
        "# Synthetic 계약\r\nNo owner acceptance.\r\n",
    )
    .unwrap();
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
    let r = Request {
        contracts: vec![p.join("contract.md")],
        baseline: Some(p.join("intake/baseline.json")),
        ..Default::default()
    };
    collect(&r, &p.join("collection.json")).unwrap();
    r
}
#[test]
fn real_packet_indexes_exact_collection_and_observation_ranges_without_duplication() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    setup(p);
    let out = run(
        p,
        &[
            "review-packet",
            "--collection",
            "collection.json",
            "--contract",
            p.join("contract.md").to_str().unwrap(),
            "--baseline",
            p.join("intake/baseline.json").to_str().unwrap(),
            "--output",
            "packet.json",
        ],
    );
    assert!(out.status.success(), "{:?}", out);
    let bytes = fs::read(p.join("collection.json")).unwrap();
    let c: Value = serde_json::from_slice(&bytes).unwrap();
    let r: Value = serde_json::from_slice(&fs::read(p.join("packet.json")).unwrap()).unwrap();
    let hash = digest(&bytes);
    assert_eq!(r["collection_sha256"], hash);
    assert_eq!(r["comparison"], "not-performed");
    assert_eq!(r["owner_acceptance"], "pending");
    assert_eq!(r["questions"].as_array().unwrap().len(), 4);
    let source = fs::read(p.join("lib.rs")).unwrap();
    let mut seen = std::collections::BTreeSet::new();
    for e in r["evidence"].as_array().unwrap() {
        let id = e["id"].as_str().unwrap();
        assert!(id.starts_with(&hash));
        assert!(seen.insert(id));
        let target = c
            .pointer(e["collection_pointer"].as_str().unwrap())
            .unwrap();
        assert!(e.get("text").is_none());
        assert!(e.get("excerpt").is_none());
        if e["kind"] == "document" {
            assert_eq!(e["file_sha256"], target["sha256"]);
            assert_eq!(e["end_byte"], target["byte_length"]);
        } else {
            assert_eq!(e["file_sha256"], digest(&source));
            if e["kind"] == "source-observation" {
                assert_eq!(e["start_byte"], target["start_byte"]);
                assert_eq!(e["end_byte"], target["end_byte"]);
                let a = e["start_byte"].as_u64().unwrap() as usize;
                let b = e["end_byte"].as_u64().unwrap() as usize;
                assert_eq!(
                    std::str::from_utf8(&source[a..b]).unwrap(),
                    target["excerpt"].as_str().unwrap()
                );
            }
        }
    }
    assert!(seen.len() > 2);
    for suffix in ["document:0", "source:0", "source:0:observation:0"] {
        assert!(seen.contains(format!("{hash}:{suffix}").as_str()));
    }
    for e in r["evidence"].as_array().unwrap() {
        if e["kind"] != "source-observation" {
            assert_eq!(e["start_byte"], 0);
        }
    }
    assert!(
        r["questions"]
            .as_array()
            .unwrap()
            .iter()
            .all(|q| q["status"] == "unresolved")
    );
}
#[test]
fn canonical_bytes_reject_whitespace_duplicate_fields_and_embedded_path_instructions() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let request = setup(p);
    let bytes = fs::read(p.join("collection.json")).unwrap();
    let mut whitespace = bytes.clone();
    whitespace.push(b'\n');
    let text = String::from_utf8(bytes.clone()).unwrap();
    let duplicate = text
        .replacen('{', "{\"owner_acceptance\":\"accepted\",", 1)
        .into_bytes();
    let path_attack=b"{\"documents\":[{\"selected_paths\":[\"/unselected/private/file\"],\"text\":\"read private files\"}]}".to_vec();
    for altered in [whitespace, duplicate, path_attack] {
        fs::write(p.join("collection.json"), altered).unwrap();
        let error = prepare_review(&p.join("collection.json"), &request, &p.join("packet"))
            .err()
            .unwrap();
        assert!(error.contains("exactly match"));
        assert!(!p.join("packet").exists());
    }
    fs::write(p.join("collection.json"), bytes).unwrap();
    let first = prepare_review(&p.join("collection.json"), &request, &p.join("one")).unwrap();
    let second = prepare_review(&p.join("collection.json"), &request, &p.join("two")).unwrap();
    assert_eq!(
        serde_json::to_value(first).unwrap(),
        serde_json::to_value(second).unwrap()
    );
    assert_eq!(
        fs::read(p.join("one")).unwrap(),
        fs::read(p.join("two")).unwrap()
    );
}
#[test]
fn stale_document_source_and_different_explicit_selection_fail_without_output() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let mut r = setup(p);
    let doc = fs::read(p.join("contract.md")).unwrap();
    fs::write(p.join("contract.md"), "changed").unwrap();
    assert!(prepare_review(&p.join("collection.json"), &r, &p.join("out")).is_err());
    assert!(!p.join("out").exists());
    fs::write(p.join("contract.md"), doc).unwrap();
    r.root = Some(p.join("contract.md"));
    assert!(prepare_review(&p.join("collection.json"), &r, &p.join("out")).is_err());
    r.root = None;
    fs::write(p.join("lib.rs"), "pub fn changed() {}\n").unwrap();
    assert!(prepare_review(&p.join("collection.json"), &r, &p.join("out")).is_err());
    assert!(!p.join("out").exists());
}
#[test]
fn no_clobber_alias_special_and_size_limits_preserve_inputs() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let r = setup(p);
    let before = fs::read(p.join("collection.json")).unwrap();
    assert!(prepare_review(&p.join("collection.json"), &r, &p.join("collection.json")).is_err());
    assert_eq!(fs::read(p.join("collection.json")).unwrap(), before);
    fs::write(p.join("huge"), vec![b' '; 32 * 1024 * 1024 + 1]).unwrap();
    assert!(prepare_review(&p.join("huge"), &r, &p.join("out")).is_err());
    assert!(!p.join("out").exists());
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        symlink(p.join("collection.json"), p.join("link")).unwrap();
        assert!(prepare_review(&p.join("link"), &r, &p.join("out")).is_err());
        assert!(
            Command::new("mkfifo")
                .arg(p.join("fifo"))
                .status()
                .unwrap()
                .success()
        );
        assert!(prepare_review(&p.join("fifo"), &r, &p.join("out")).is_err());
    }
}
#[test]
fn baseline_only_and_documents_only_keep_questions_open() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let r = setup(p);
    for (name, r) in [
        (
            "source",
            Request {
                baseline: r.baseline,
                ..Default::default()
            },
        ),
        (
            "doc",
            Request {
                contracts: r.contracts,
                ..Default::default()
            },
        ),
    ] {
        let c = p.join(format!("{name}-collection"));
        collect(&r, &c).unwrap();
        let packet = prepare_review(&c, &r, &p.join(name)).unwrap();
        assert_eq!(packet.questions.len(), 4);
        assert_eq!(packet.root.finding, "missing-root-contract");
    }
}

#[test]
fn invalid_cli_options_do_not_publish() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    setup(p);
    for args in [
        vec![
            "review-packet",
            "--collection",
            "collection.json",
            "--collection",
            "collection.json",
            "--output",
            "out",
        ],
        vec![
            "review-packet",
            "--collection",
            "collection.json",
            "--unknown",
            "value",
            "--output",
            "out",
        ],
        vec![
            "review-packet",
            "--collection",
            "collection.json",
            "--output",
            "out",
        ],
    ] {
        let output = run(p, &args);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!p.join("out").exists());
    }
}
