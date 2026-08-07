use crate::mcp::context::LazyContext;
use crate::mcp::models::prompt::PromptContent;
use crate::mcp::models::{
    FileMetadataParams, ProjectOpenParams, ProjectRegisterParams, ProjectUpdateParams,
    SearchCodeParams, UpdateMcIgnoreParams, WorkflowSessionParams, WorkflowStepParams,
};
use crate::mcp::tools::prompts::registry;
use crate::mcp::tools::{cortex, file, index, metadata, project, search, workflow};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, ContentBlock, ErrorData, GetPromptRequestParams, GetPromptResponse,
    GetPromptResult, Implementation, ListPromptsResult, PaginatedRequestParams, Prompt,
    PromptArgument, PromptMessage, Role, ServerCapabilities, ServerInfo,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::{ServerHandler, tool, tool_handler, tool_router};

/// `McpHandler` ties the MCP protocol server to Mercury Cortex's engine.
///
/// Implements all MCP tools via `#[tool]` annotations and the `ServerHandler`
/// trait for prompts.  Instantiated once per process and served over stdio
/// via `rmcp::serve_server`.
#[derive(Clone)]
pub struct McpHandler {
    /// Shared access to the engine, config, and runtime context.
    ///
    /// Lazily initialized; the MCP server starts responding to protocol
    /// handshakes immediately, while the engine is initialized in the
    /// background. Tool callers block on [`LazyContext::get`] until ready.
    pub ctx: LazyContext,
}

#[tool_router]
impl McpHandler {
    #[tool(
        name = "cortex/info",
        description = "Get engine version and running status"
    )]
    async fn cortex_info(&self) -> Result<CallToolResult, ErrorData> {
        self.run_tool("cortex/info", cortex::handle_info).await
    }

    #[tool(
        name = "index/paths",
        description = "List indexed file paths for the active project"
    )]
    async fn index_paths(&self) -> Result<CallToolResult, ErrorData> {
        self.run_tool("index/paths", index::handle_index_paths)
            .await
    }

    #[tool(
        name = "project/close",
        description = "Close the currently active project"
    )]
    async fn project_close(&self) -> Result<CallToolResult, ErrorData> {
        self.run_tool("project/close", project::handle_close).await
    }

    #[tool(
        name = "project/status",
        description = "Get the status of the currently active project"
    )]
    async fn project_status(&self) -> Result<CallToolResult, ErrorData> {
        self.run_tool("project/status", project::handle_project_status)
            .await
    }

    #[tool(
        name = "metadata/import",
        description = "Import staged AI-generated metadata from .mercury-cortex/temp/ (requires an open project)"
    )]
    async fn metadata_import(&self) -> Result<CallToolResult, ErrorData> {
        self.run_tool("metadata/import", metadata::handle_import_metadata)
            .await
    }

    #[tool(name = "search/code", description = "Search indexed file metadata")]
    async fn search_code(
        &self,
        params: Parameters<SearchCodeParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.run_tool_with_params("search/code", params, search::handle_search)
            .await
    }

    #[tool(
        name = "project/open",
        description = "Open a project in the engine for indexing"
    )]
    async fn project_open(
        &self,
        params: Parameters<ProjectOpenParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.run_tool_with_params("project/open", params, project::handle_open)
            .await
    }

    #[tool(
        name = "project/register",
        description = "Register a new project with Mercury Cortex"
    )]
    async fn project_register(
        &self,
        params: Parameters<ProjectRegisterParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.run_tool_with_params("project/register", params, project::handle_register)
            .await
    }

    #[tool(
        name = "project/update",
        description = "Save AI-detected language/framework metadata"
    )]
    async fn project_update(
        &self,
        params: Parameters<ProjectUpdateParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.run_tool_with_params("project/update", params, project::handle_update)
            .await
    }

    #[tool(
        name = "project/update_mcignore",
        description = "Append AI-detected ignore patterns to .mcignore"
    )]
    async fn update_mcignore(
        &self,
        params: Parameters<UpdateMcIgnoreParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.run_tool_with_params(
            "project/update_mcignore",
            params,
            project::handle_update_mcignore,
        )
        .await
    }

    #[tool(
        name = "file/metadata",
        description = "Get indexed metadata for a specific file by path"
    )]
    async fn file_metadata(
        &self,
        params: Parameters<FileMetadataParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.run_tool_with_params("file/metadata", params, file::handle_get_file_metadata)
            .await
    }

    #[tool(
        name = "workflow/session",
        description = "Get session context, project state, and workflow step list"
    )]
    async fn workflow_session(
        &self,
        params: Parameters<WorkflowSessionParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.run_tool_with_params(
            "workflow/session",
            params,
            workflow::handle_workflow_session,
        )
        .await
    }

    #[tool(
        name = "workflow/step",
        description = "Get instructions for a specific workflow step"
    )]
    async fn workflow_step(
        &self,
        params: Parameters<WorkflowStepParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.run_tool_with_params("workflow/step", params, workflow::handle_workflow_step)
            .await
    }
}

#[tool_handler]
impl ServerHandler for McpHandler {
    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, ErrorData> {
        let prompts = registry::list_prompts();
        let rmcp_prompts: Vec<Prompt> = prompts
            .iter()
            .map(|p| {
                let args = p.arguments.as_ref().map(|args| {
                    args.iter()
                        .map(|a| {
                            let mut arg = PromptArgument::new(a.name.clone());
                            if let Some(ref desc) = a.description {
                                arg = arg.with_description(desc.clone());
                            }
                            arg.with_required(a.required.unwrap_or(false))
                        })
                        .collect()
                });
                Prompt::new(p.name.clone(), p.description.clone(), args)
            })
            .collect();
        Ok(ListPromptsResult::with_all_items(rmcp_prompts))
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResponse, ErrorData> {
        let args_json = request
            .arguments
            .as_ref()
            .map(|m| serde_json::Value::Object(m.clone()));
        let result = registry::get_prompt(&request.name, args_json.as_ref())
            .map_err(|e| ErrorData::invalid_params(e.to_string(), None))?;

        let messages: Vec<PromptMessage> = result
            .messages
            .into_iter()
            .map(|msg| {
                let role = match msg.role.as_str() {
                    "assistant" => Role::Assistant,
                    _ => Role::User,
                };
                let content = match msg.content {
                    PromptContent::Text { text } => ContentBlock::text(text),
                };
                PromptMessage::new(role, content)
            })
            .collect();

        let res = GetPromptResult::new(messages);
        let response: GetPromptResponse = if let Some(desc) = result.description {
            res.with_description(desc)
        } else {
            res
        }
        .into();
        Ok(response)
    }

    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .build(),
        )
        .with_server_info(Implementation::new(
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
        ))
        .with_instructions(include_str!("tools/prompts/instructions.md"))
    }
}
