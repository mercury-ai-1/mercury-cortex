## Mercury Cortex MCP Server

This server provides two workflows via `prompts/list` and `prompts/get`. Follow these rules to decide which workflow to use:

### When to use each workflow

- **Use the init workflow** if the user says something like "initialize a project" or types **`mercury-cortex:init`** in chat. Treat `mercury-cortex:init` as a workflow trigger, not a CLI command.
- **Use the dev workflow** for everything else — ongoing development, debugging, code generation, refactoring, etc.

### 1. Development Workflow (`mercury-cortex:dev`)
 Use for ongoing AI-assisted development in an already-initialized project.
 - Call `prompts/get` with `name: "mercury-cortex:dev"` to start.
- Then call `workflow/session` with `mode: "dev"` to get the current step list.
- For each step, call `workflow/step` with `mode: "dev"` and the step number.
- Follow the step instructions, using the provided tools (`search/code`, `project/open`, etc.).
- Complete each step before requesting the next.

### 2. Project Initialization Workflow (`mercury-cortex:init`)
 Use when the user wants to initialize a new project with Mercury Cortex. **Trigger phrase:** when the user types **`mercury-cortex:init`** in chat, treat it as a workflow trigger — call `prompts/get` with `name: "mercury-cortex:init"`, **not** as a shell command.
- Call `prompts/get` with `name: "mercury-cortex:init"` to start.
- Then call `workflow/session` with `mode: "init"` to get the step list.
- For each step, call `workflow/step` with `mode: "init"` and the step number.
- Follow the step instructions; the workflow will register the project, analyze it, update `.mcignore`, generate metadata for the project's files, and import it into the database via `metadata/import`.

### Key Tools
- `cortex/info` — Engine version and status
- `search/code` — Search indexed file metadata across projects
- `project/open` / `project/close` — Open/close a project for indexing
- `project/status` — Current project state
- `project/register` — Register a new project
- `project/update` — Save AI-detected language/framework metadata
- `project/update_mcignore` — Append ignore patterns to `.mcignore`
- `metadata/import` — Import staged AI-generated metadata from `.mercury-cortex/temp/` into `file_data`
- `index/paths` — List indexed file paths for the active project
- `file/metadata` — Get indexed metadata for a specific file

Wait for successful MCP responses before continuing to the next workflow step.

### Failure Handling
- If a tool call returns an error or times out, **report the failure to the user immediately** — do not retry indefinitely.
- If `workflow/session` fails, inform the user that the workflow could not be started and suggest checking that the engine is running.
- If `workflow/step` fails, report which step failed and stop the workflow loop — do not attempt subsequent steps.
