use std::collections::HashSet;
use std::sync::Arc;

use async_trait::async_trait;
use caac::candidate::digest;
use caac::checker::{Diagnostic, lint_documents};
use rust_mcp_sdk::mcp_server::{McpServerOptions, ServerHandlerCore, server_runtime};
use rust_mcp_sdk::schema::{
    CallToolRequestParams, CallToolResult, Implementation, InitializeResult,
    NotificationFromClient, ProtocolVersion, RequestFromClient, Result as RpcResult,
    ResultFromServer, RpcError, ServerCapabilities, ServerCapabilitiesTools, TextContent,
};
use rust_mcp_sdk::{McpServer, StdioTransport, ToMcpServerHandlerCore, TransportOptions};
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const TOOL_NAME: &str = "caac_lint";
const MAX_DOCUMENTS: usize = 8;
const MAX_LABEL_CHARS: usize = 128;
const MAX_MARKDOWN_BYTES: usize = 256 * 1024;
const MAX_FRAME_BYTES: usize = 16 * 1024 * 1024;

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum Scope {
    Partial,
    Complete,
    Unknown,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Document {
    #[schemars(length(min = 1, max = "MAX_LABEL_CHARS"))]
    name: String,
    #[schemars(description = "Inline Markdown with a maximum of 256 KiB when encoded as UTF-8.")]
    markdown: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct LintRequest {
    #[schemars(description = "Caller-declared scope of the selected documents.")]
    scope: Scope,
    #[schemars(length(min = 1, max = "MAX_DOCUMENTS"))]
    documents: Vec<Document>,
}

#[derive(Serialize, JsonSchema)]
struct DocumentHash {
    name: String,
    #[schemars(regex(pattern = "^[0-9a-f]{64}$"))]
    sha256: String,
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum LintVerdict {
    Passed,
    Failed,
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum SemanticReview {
    Pending,
}

#[derive(Serialize, JsonSchema)]
struct LintResponse {
    scope: Scope,
    documents: Vec<DocumentHash>,
    lint: LintVerdict,
    diagnostics: Vec<Diagnostic>,
    semantic_review: SemanticReview,
}

impl LintRequest {
    fn lint(self) -> Result<LintResponse, &'static str> {
        if !(1..=MAX_DOCUMENTS).contains(&self.documents.len()) {
            return Err("Provide between one and eight documents.");
        }
        let mut names = HashSet::new();
        for document in &self.documents {
            if document.name.trim().is_empty() || document.name.chars().count() > MAX_LABEL_CHARS {
                return Err("Use nonblank document names of at most 128 Unicode characters.");
            }
            if !names.insert(document.name.as_str()) {
                return Err("Document names must be unique.");
            }
            if document.markdown.len() > MAX_MARKDOWN_BYTES {
                return Err("Each Markdown document must use at most 256 KiB in UTF-8.");
            }
        }
        let diagnostics = lint_documents(
            self.documents
                .iter()
                .map(|doc| (doc.name.as_str(), doc.markdown.as_str())),
        );
        let lint = if diagnostics.is_empty() {
            LintVerdict::Passed
        } else {
            LintVerdict::Failed
        };
        let documents = self
            .documents
            .into_iter()
            .map(|doc| DocumentHash {
                name: doc.name,
                sha256: digest(doc.markdown.as_bytes()),
            })
            .collect();
        Ok(LintResponse {
            scope: self.scope,
            documents,
            lint,
            diagnostics,
            semantic_review: SemanticReview::Pending,
        })
    }
}

struct LintService {
    tool_catalog: RpcResult,
}

impl LintService {
    fn new() -> Self {
        let tool = json!({
            "name": TOOL_NAME,
            "description": "Lint syntax and clause references in selected Markdown. Scope is caller-declared.",
            "inputSchema": schema_for!(LintRequest),
            "outputSchema": schema_for!(LintResponse),
            "annotations": {"readOnlyHint": true, "openWorldHint": false},
        });
        Self {
            tool_catalog: RpcResult {
                meta: None,
                extra: Some(
                    [("tools".to_owned(), Value::Array(vec![tool]))]
                        .into_iter()
                        .collect(),
                ),
            },
        }
    }
}

#[async_trait]
impl ServerHandlerCore for LintService {
    async fn handle_request(
        &self,
        request: RequestFromClient,
        runtime: Arc<dyn McpServer>,
    ) -> Result<ResultFromServer, RpcError> {
        match request {
            RequestFromClient::InitializeRequest(params) => {
                let mut details = runtime.server_info().clone();
                if params.protocol_version == ProtocolVersion::V2025_06_18.to_string() {
                    details.protocol_version = params.protocol_version;
                }
                Ok(details.into())
            }
            RequestFromClient::ListToolsRequest(_) => Ok(self.tool_catalog.clone().into()),
            RequestFromClient::CallToolRequest(params) => call_tool(params).map(Into::into),
            RequestFromClient::PingRequest(_) => Ok(RpcResult {
                meta: None,
                extra: None,
            }
            .into()),
            _ => Err(RpcError::method_not_found()),
        }
    }

    async fn handle_notification(
        &self,
        _: NotificationFromClient,
        _: Arc<dyn McpServer>,
    ) -> Result<(), RpcError> {
        Ok(())
    }

    async fn handle_error(&self, _: &RpcError, _: Arc<dyn McpServer>) -> Result<(), RpcError> {
        Ok(())
    }
}

fn call_tool(params: CallToolRequestParams) -> Result<CallToolResult, RpcError> {
    if params.name != TOOL_NAME {
        return Err(RpcError::invalid_params().with_message("Unknown tool.".to_owned()));
    }
    let args =
        serde_json::from_value::<LintRequest>(Value::Object(params.arguments.unwrap_or_default()));
    let report = args
        .map_err(|_| "Provide a valid scope and a list of named Markdown documents.")
        .and_then(LintRequest::lint);
    let structured = match report {
        Ok(report) => match serde_json::to_value(report) {
            Ok(Value::Object(object)) => object,
            _ => {
                return Err(RpcError::internal_error()
                    .with_message("Could not encode lint result.".to_owned()));
            }
        },
        Err(message) => {
            return Ok(CallToolResult {
                content: vec![TextContent::new(message.to_owned(), None, None).into()],
                is_error: Some(true),
                meta: None,
                structured_content: None,
            });
        }
    };
    let text = serde_json::to_string(&structured).map_err(|_| {
        RpcError::internal_error().with_message("Could not encode lint result.".to_owned())
    })?;
    Ok(CallToolResult {
        content: vec![TextContent::new(text, None, None).into()],
        is_error: Some(false),
        meta: None,
        structured_content: Some(structured),
    })
}

pub fn serve_stdio() -> Result<(), String> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| error.to_string())?
        .block_on(async {
            let transport = StdioTransport::new(TransportOptions {
                max_line_length: MAX_FRAME_BYTES,
                channel_capacity: 1,
                ..Default::default()
            })
            .map_err(|error| error.to_string())?;
            let server = server_runtime::create_server(McpServerOptions {
                transport,
                handler: LintService::new().to_mcp_server_handler(),
                server_details: InitializeResult {
                    capabilities: ServerCapabilities {
                        tools: Some(ServerCapabilitiesTools { list_changed: None }),
                        ..Default::default()
                    },
                    instructions: None,
                    meta: None,
                    protocol_version: ProtocolVersion::V2025_11_25.into(),
                    server_info: Implementation {
                        name: "caac".to_owned(),
                        version: env!("CARGO_PKG_VERSION").to_owned(),
                        title: None,
                        description: None,
                        icons: Vec::new(),
                        website_url: None,
                    },
                },
                task_store: None,
                client_task_store: None,
                message_observer: None,
            });
            server.start().await.map_err(|error| error.to_string())
        })
}
