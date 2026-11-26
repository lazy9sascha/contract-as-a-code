use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

const BIN: &str = env!("CARGO_BIN_EXE_caac");
const MAIN: &str = "BOX-001 | Queue must retain the request.\n\n## Cases\n\n| Case | Input | Expected result | Clauses |\n| --- | --- | --- | --- |\n| Save | Request | Retained | @BOX-001 |\n";

struct Session {
    child: Child,
    input: ChildStdin,
    output: Receiver<Value>,
    id: u64,
}

impl Session {
    fn open() -> Self {
        let mut child = Command::new(BIN)
            .arg("mcp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let input = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let (sender, output) = channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let message =
                    serde_json::from_str(&line.unwrap()).expect("MCP stdout was not JSON");
                if sender.send(message).is_err() {
                    break;
                }
            }
        });
        let mut session = Self {
            child,
            input,
            output,
            id: 0,
        };
        let welcome = session.request(
            "initialize",
            json!({
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": {"name": "caac-tests", "version": "1"}
            }),
        );
        assert_eq!(
            welcome["result"]["protocolVersion"], "2025-11-25",
            "{welcome}"
        );
        assert!(
            welcome["result"]["capabilities"]["tools"].is_object(),
            "{welcome}"
        );
        session.send(json!({"jsonrpc": "2.0", "method": "notifications/initialized"}));
        session
    }

    fn send(&mut self, message: Value) {
        writeln!(self.input, "{message}").unwrap();
        self.input.flush().unwrap();
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        self.id += 1;
        self.send(json!({"jsonrpc": "2.0", "id": self.id, "method": method, "params": params}));
        let reply = self
            .output
            .recv_timeout(Duration::from_secs(20))
            .expect("MCP response");
        assert_eq!(reply["id"], self.id, "{reply}");
        reply
    }

    fn lint(&mut self, arguments: Value) -> Value {
        let reply = self.request(
            "tools/call",
            json!({"name": "caac_lint", "arguments": arguments}),
        );
        assert!(reply.get("error").is_none(), "{reply}");
        reply["result"].clone()
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn payload(result: &Value) -> &Value {
    assert_eq!(result["isError"], false, "{result}");
    let payload = &result["structuredContent"];
    let text = result["content"][0]["text"].as_str().unwrap();
    assert_eq!(serde_json::from_str::<Value>(text).unwrap(), *payload);
    payload
}

#[test]
fn mcp_lint_matches_cli_and_preserves_the_declared_scope() {
    let work = tempfile::tempdir().unwrap();
    let mut session = Session::open();
    let tools = session.request("tools/list", json!({}));
    let tools = tools["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0]["name"], "caac_lint");
    assert!(tools[0]["inputSchema"]["properties"]["scope"].is_object());
    assert_eq!(
        tools[0]["inputSchema"]["properties"]["documents"]["type"],
        "array"
    );
    assert!(tools[0]["outputSchema"]["properties"]["diagnostics"].is_object());
    let other = MAIN.replace("BOX-001", "ACK-001");
    for (scope, reference, expected) in [("complete", "ACK-001", 0), ("partial", "BOX-999", 1)] {
        let first = MAIN.replace("@BOX-001", &format!("@{reference}"));
        let mut docs = Vec::new();
        let mut hashes = Vec::new();
        for (name, text) in [("queue.md", first.as_str()), ("ack.md", other.as_str())] {
            std::fs::write(work.path().join(name), text).unwrap();
            docs.push(json!({"name": name, "markdown": text}));
            let sha256: String = Sha256::digest(text)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect();
            hashes.push(json!({"name": name, "sha256": sha256}));
        }
        let cli = Command::new(BIN)
            .args(["lint", "--json", "queue.md", "ack.md"])
            .current_dir(work.path())
            .output()
            .unwrap();
        assert_eq!(cli.status.code(), Some(expected));
        let mut cli: Value = serde_json::from_slice(&cli.stdout).unwrap();
        cli["scope"] = json!(scope);
        cli["documents"] = json!(hashes);
        assert_eq!(cli["semantic_review"], "pending");
        let result = session.lint(json!({"scope": scope, "documents": docs}));
        assert_eq!(payload(&result), &cli);
    }
    let missing = session.request(
        "tools/call",
        json!({"name": "missing_tool", "arguments": {}}),
    );
    assert!(missing.get("error").is_some(), "{missing}");
}

#[test]
fn mcp_lint_enforces_limits_and_recovers_after_rejected_invocations() {
    let mut session = Session::open();
    let oversized = "é".repeat(128 * 1024 + 1);
    let doc = json!({"name": "a", "markdown": MAIN});
    for arguments in [
        json!({}),
        json!({"scope": "all", "documents": [doc]}),
        json!({"scope": "partial", "documents": []}),
        json!({"scope": "partial", "documents": [doc, doc]}),
        json!({"scope": "partial", "documents": [{"name": " ", "markdown": MAIN}]}),
        json!({"scope": "partial", "documents": [{"name": "字".repeat(129), "markdown": MAIN}]}),
        json!({"scope": "partial", "documents": [{"name": "a", "markdown": oversized}]}),
        json!({"scope": "partial", "documents": [{"name": "a", "markdown": MAIN, "path": "unused"}]}),
    ] {
        let rejected = session.lint(arguments);
        assert_eq!(rejected["isError"], true, "{rejected}");
        assert!(rejected["structuredContent"].is_null(), "{rejected}");
    }
    let bad = MAIN.replace("@BOX-001", "@BOX-999");
    let result = session.lint(
        json!({"scope": "unknown", "documents": [{"name": "../opaque.md", "markdown": bad}]}),
    );
    let result = payload(&result);
    assert_eq!(result["scope"], "unknown");
    assert_eq!(result["lint"], "failed");
    assert_eq!(result["diagnostics"][0]["path"], "../opaque.md");
    assert_eq!(session.request("ping", json!({}))["result"], json!({}));
}

#[test]
fn mcp_lint_accepts_the_maximum_escaped_selection() {
    let mut session = Session::open();
    let docs: Vec<_> = (1..=8)
        .map(|index| {
            let prefix = MAIN.replace("BOX-001", &format!("BOX-{index:03}")) + "\n<!--";
            let suffix = "-->\n";
            let markdown =
                prefix.clone() + &"\u{1}".repeat(256 * 1024 - prefix.len() - suffix.len()) + suffix;
            json!({"name": format!("{index}.md"), "markdown": markdown})
        })
        .collect();
    let result = session.lint(json!({"scope": "partial", "documents": docs}));
    let result = payload(&result);
    assert_eq!(result["lint"], "passed", "{result}");
    assert_eq!(result["documents"].as_array().unwrap().len(), 8);
}
