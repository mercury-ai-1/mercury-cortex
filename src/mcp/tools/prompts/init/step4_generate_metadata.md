## Step 4: Metadata Generation and Import

Generate high-quality AI-derived metadata that becomes the project index. The
engine imports the staged metadata from `.mercury-cortex/temp/` via
`metadata/import`; this step adds semantic metadata for the files where it
adds the most search value.

Do not include database-managed fields such as `id`, `project_id`, timestamps,
hashes, or any other system-managed values.

### Indexing Pipeline

The indexing pipeline must follow this strict order. Every step is mandatory:

1. **Traverse** the project, applying `.mcignore` filtering first and
   exclusively. Ignored paths are completely outside the pipeline.
2. **Identify** the files where metadata adds real search value: core
   modules, services, handlers, providers, schemas, models, and any file with
   reusable functionality.
3. **Generate** metadata for each identified file.
4. **Write** metadata JSON files into `.mercury-cortex/temp/`.
5. **Import**: call `metadata/import` once. The engine reads every JSON file
   in `.mercury-cortex/temp/`, applies `.mcignore`, and upserts the metadata
   into `file_data`.

`.mercury-cortex/temp/` is the **only** source of truth for imported metadata.
Never import source files directly or generate metadata outside the temp
directory.

### .mcignore Enforcement (Hard Rule)

`.mcignore` must be treated **exactly like `.gitignore`**. If a path matches
`.mcignore`, it is completely outside the indexing pipeline. Period.

An ignored path must never:

- Be discovered during project traversal
- Be scanned or read
- Be analyzed
- Have metadata generated for it
- Produce a JSON file under `.mercury-cortex/temp/`
- Be imported into `file_data`

The `.mcignore` file is the definitive exclusion list. Do not second-guess
it. If a path matches `.mcignore`, skip it unconditionally. Do not check
whether the file "looks important" or "might have reusable code"; it is
ignored, full stop.

### What the Index Contains

The index is exactly the set of files you stage metadata for; there is no
separate full-file inventory. Metadata is the index:

- Generate metadata for files where it adds genuine search value: core
  modules, services, feature implementations, and reusable utilities.
- Staging metadata for a subset is acceptable, since the index reflects exactly
  the files you staged. Prioritize the most reused and most important files.
- Dependency manifests are **mandatory** (see below); they make the whole
  project searchable by dependency name.

### Never

- Invent functionality not present in the file.
- Guess exported APIs.
- Copy metadata from unrelated files.
- Use placeholder descriptions for implemented files.
- Generate metadata without analyzing the source.
- Include duplicate values in any list field.
- Express the same concept using different synonyms unless they represent
  distinct functionality.
- Generate metadata for paths matching `.mcignore`.
- Stage metadata for paths matching `.mcignore`.

### Analyze Project Dependencies

Before analyzing individual source files, read the project's dependency
management file (`Cargo.toml`, `pubspec.yaml`, `package.json`, `mix.exs`,
`build.gradle`, `go.mod`, `Gemfile`, `requirements.txt`, or similar).
This single file tells you what external dependencies, packages, and
plugins the project uses.

Use this information to:

- Understand the project's technical stack and ecosystem before you begin
  file-by-file analysis.
- Include relevant dependency names in `features` and `tags` for files
  that use them (e.g., `serde` for a serialization module, `axum` for a
  web handler, `riverpod` for state management).
- When a task involves an external dependency, check this file first to
  determine whether it is already used in the project. Based on that,
  decide whether to search for relevant source files within the project
  or across other indexed projects.

### Build a Dependency Inventory

After reading the dependency manifest and before starting file-by-file
analysis, build a structured inventory of all direct dependencies, plugins,
and packages declared by the project.

For each dependency:
- Record its **name** (package name only; never include version numbers).
- Determine its **primary functional category**; this is the role it plays in the
  project. Use one or more of the following, or coin a clear equivalent:
  `state-management`, `networking`, `database`, `auth`, `ui`, `serialization`,
  `testing`, `build-tool`, `logging`, `routing`, `validation`, `file-storage`,
  `payments`, `analytics`, `push-notifications`, `image-processing`,
  `platform-integration`, `code-generation`, `observability`.

Keep this inventory in context for the entire analysis phase. When generating
metadata for each source file:
- If the file imports or uses a dependency, include the dependency's **name**
  in `features` (e.g., `"axum"`, `"riverpod"`, `"serde"`).
- Include the dependency's **functional category** in `tags`
  (e.g., `"networking"`, `"state-management"`, `"serialization"`).
- Only tag a file with a dependency if the file **actually imports or uses
  it**. Do not propagate a dependency tag across all files in a module just
  because one file uses it.
- Do not include version numbers anywhere in metadata. Version information
  is always available in the dependency manifest; storing it in metadata
  would make searches noisy as versions change.

### Index Dependency Manifests

Each dependency manifest file discovered during the dependency analysis must
also be indexed. Generate metadata for every manifest file using this
guidance:

- **`type`**: `"config"`
- **`purpose`**: Describe what the manifest declares (e.g., "Declares the
  project's Rust crate dependencies and build configuration").
- **`summary`**: List the primary dependency categories (e.g., "Web
  framework, serialization, async runtime, and database access").
- **`features`**: Include **every** dependency, package, and plugin name
  declared in the manifest. Use the package name only; never include
  version numbers. This makes the manifest searchable by dependency name
  across all projects (e.g., searching "serde" finds every project that
  declares it).
- **`tags`**: Include the ecosystem and language (e.g., `"rust"`,
  `"cargo"`, `"web"`, `"serialization"`).

Example metadata for a `Cargo.toml`:

```json
{
  "path": "Cargo.toml",
  "type": "config",
  "purpose": "Declares the project's Rust crate dependencies and workspace configuration",
  "summary": "Web framework, serialization, async runtime, and database access dependencies",
  "features": ["axum", "serde", "serde_json", "tokio", "surrealdb", "anyhow", "thiserror"],
  "tags": ["rust", "cargo", "web", "serialization", "database"]
}
```

Lock files (`Cargo.lock`, `package-lock.json`, `pubspec.lock`, `yarn.lock`,
`poetry.lock`, `go.sum`) must **not** be indexed; they are
machine-generated and change on every dependency update.

### How to Analyze a File

1. Read the entire file before generating metadata.
2. Determine the file's primary responsibility.
3. Identify its public API surface.
4. Note which broader domain or category it belongs to.
5. Keep descriptions concise, consistent, and technical.

### Metadata Field Guide

```json
{
  "path": "relative/path/to/file.rs",
  "type": "component",
  "purpose": "One-sentence description of why this file exists",
  "summary": "What the file implements and its primary responsibilities",
  "features": ["jwt-auth", "file-upload"],
  "tags": ["networking", "security"],
  "exported_functions": ["authenticate", "uploadFile"]
}
```

| Field | Guidance |
|-------|----------|
| `type` | One of: `main`, `module`, `component`, `utility`, `service`, `handler`, `middleware`, `route`, `schema`, `migration`, `repository`, `provider`, `hook`, `command`, `config`, `test`, `type`, `error` |
| `purpose` | Why the file exists, in one clear sentence. Include key domain functionality terms (e.g., mention light/dark mode, system theme, auth guard). |
| `summary` | What the file implements, its primary responsibilities, and which important modules or services it interacts with. Avoid line-by-line descriptions. |
| `features` | Specific, reusable technical capabilities this file contributes. This is the **most important search field**. Include both specific compound terms and common search keywords (e.g., for theme controls include `["theme", "theme-mode", "theme-picker", "theme-toggle", "dark-mode", "light-mode"]`). Avoid overly sparse or single generic terms like `ui` or `logic`. |
| `tags` | Broader domain categories and key concepts. Include both broad domains and specific topic tags (e.g., `tags: ["ui", "theming", "theme", "state-management"]`). |
| `exported_functions` | Public functions, methods, classes, components, traits, interfaces, hooks, or providers. Ignore private or internal helpers. |

### Consistency Rules

If multiple files implement the same concept, use consistent terminology for
`purpose`, `summary`, `features`, and `tags` across the entire project. Ensure
that core feature keywords (such as `theme`, `auth`, `storage`) are present on all
related files so they are easily discoverable.

Every metadata value should be unique, meaningful, and contribute additional
search value. Remove duplicate entries from `features`, `tags`, and
`exported_functions`.

### What to Skip

Skip generated, machine-produced, or non-source files that are not useful as
reusable implementation references:

- **Generated code** (`.g.dart`, `.pb.rs`, `.gen.ts`)
- **Documentation-only files** unless they contain architectural information
- **Binary or minified files**
- **Lock files** (`package-lock.json`, `pubspec.lock`, `Cargo.lock`,
  `yarn.lock`, `poetry.lock`, `go.sum`)

### Strict JSON Output

All metadata must be written as valid **`.json` files only**. Do not use YAML,
TOML, Markdown, or any other format. Each file must contain a single valid JSON
object that matches the metadata schema. Invalid JSON files will be rejected by
the importer.

### Temp File Naming

Write each file's metadata as a separate `.json` file into
`.mercury-cortex/temp/`. Preserve the project's relative directory structure:

```
src/auth/login.rs → .mercury-cortex/temp/src/auth/login.rs.json
```

Create missing intermediate directories automatically. Overwrite any existing
temporary JSON file for the same source path.

### Before Writing Metadata

Validate that each generated metadata entry conforms to the required schema:
all required fields present, no invalid types, no empty arrays where values
are expected, no database-managed fields included.

### Import Process

1. Walk the project tree, applying `.mcignore` at every step, to identify the
   files worth indexing.
2. Analyze each candidate and generate its metadata.
3. Write the metadata JSON files into `.mercury-cortex/temp/`, preserving the
   project's relative directory structure.
4. **Import**: call `metadata/import` once. The engine reads every JSON file in
   `.mercury-cortex/temp/`, applies `.mcignore`, upserts the metadata into
   `file_data`, and removes the staged files after a successful import. The
   staged JSON files are the source of the index; a file is indexed if and only
   if you generated metadata for it (except paths matching `.mcignore`, which
   are skipped).
5. Wait for the `metadata/import` response. It reports `indexed_files` (the
   total `file_data` rows for the project) and a `results` array with one entry
   per staged JSON file (`path`, `success`, `error`).

If any entry in `results` has `success: false`, fix the reported issues and
call `metadata/import` again.

If the MCP call itself returns an error, stop and report the failure. Do not
proceed to Step 5 until the import has been verified.

### Re-run

`metadata/import` is idempotent and safe to call again. Generate metadata for
any new files that warrant it, stage them, and call `metadata/import` again.

### Progress Report

Upon successful completion of Step 4, emit a concise progress update before moving to Step 5:

**Step 4 Complete: Metadata Generation & Import**
- **Indexed:** `<indexed_files>` files
- **Imported:** `<count>` metadata files

*Proceeding to Step 5 (Verification & Summary)…*
