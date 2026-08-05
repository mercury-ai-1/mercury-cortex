## Step 5 — Generate and Submit Metadata

Generate accurate metadata for every file created or modified during the
current request, then submit it to the engine.

### Scope

Generate metadata only for files that were created or modified as part
of the current request. Never regenerate metadata for unchanged files.

### How to Analyze a File

1. Read the entire file before generating metadata.
2. Determine its primary responsibility.
3. Identify its public API surface.
4. Note which broader domain or category it belongs to.
5. Focus on reusable functionality using concise technical terminology.
6. Keep descriptions concise, consistent, and technical.

### Document Dependencies

When analyzing a file, identify which external packages, libraries, or
plugins the file imports or uses:

- Include each as a named entry in `features` using the package name only
  (e.g., `"serde"`, `"axios"`, `"riverpod"`). Never include version numbers.
- Include the dependency's functional category in `tags`
  (e.g., `"serialization"`, `"http"`, `"state-management"`).
- Only include a dependency if the file **actually imports or uses it**.
  Do not propagate dependencies from other files in the same module.
- If you are modifying a file that already has metadata, take this
  opportunity to update its dependency documentation if it is missing
  or outdated.

Version numbers must never appear in metadata. If version information
is needed, it is always available in the project's dependency manifest.

### Index Dependency Manifests

If the current request creates or modifies a dependency manifest file
(`Cargo.toml`, `package.json`, `pubspec.yaml`, `go.mod`, `Gemfile`,
`requirements.txt`, `build.gradle`, `mix.exs`, or similar), generate
metadata for it:

- **`type`**: `"config"`
- **`purpose`**: Describe what the manifest declares.
- **`summary`**: List the primary dependency categories.
- **`features`**: Include **every** dependency, package, and plugin name
  declared in the manifest — not just the ones added or changed in this
  request. Use the package name only, never version numbers.
- **`tags`**: Include the ecosystem and language.

This ensures the manifest remains searchable by dependency name after
the update. Lock files (`Cargo.lock`, `package-lock.json`, `pubspec.lock`,
etc.) must not be indexed.

### Metadata Field Guide

```json
{
  "path": "relative/path/to/file.rs",
  "type": "component",
  "purpose": "One-sentence description of why this file exists",
  "summary": "What the file implements and its primary responsibilities",
  "features": ["jwt-auth", "file-upload"],
  "tags": ["networking", "security"],
  "exported_functions": ["authenticate", "upload_file"]
}
```

| Field | Guidance |
|-------|----------|
| `type` | One of: `main`, `module`, `component`, `utility`, `service`, `handler`, `middleware`, `route`, `schema`, `migration`, `repository`, `provider`, `hook`, `command`, `config`, `test`, `type`, `error` |
| `purpose` | Why the file exists, in one clear sentence. Include domain keywords (e.g. mention light/dark mode, system theme, auth guard). |
| `summary` | What the file implements, its primary responsibilities, and which important modules or services it interacts with. Avoid line-by-line descriptions. |
| `features` | Specific, reusable technical capabilities this file contributes. **This is the most important search field.** Include both specific compound terms and common search keywords (e.g., for theme controls include `["theme", "theme-mode", "theme-picker", "theme-toggle", "dark-mode", "light-mode"]`). Avoid overly sparse or single generic terms like `ui` or `logic`. |
| `tags` | Broader domain categories and key concepts. Include both broad domains and specific topic tags (e.g., `tags: ["ui", "theming", "theme", "state-management"]`). |
| `exported_functions` | Public functions, methods, classes, components, traits, interfaces, hooks, or providers. Ignore private or internal helpers. |

### Consistency Rules

If multiple files implement the same concept, use consistent terminology
for `purpose`, `summary`, `features`, and `tags` across the project. Do
not refer to the same concept with different names in different files.

Every metadata value should be unique, meaningful, and contribute
additional search value. Remove duplicate entries from `features`,
`tags`, and `exported_functions`.

### Validate Before Writing

Before creating each metadata JSON file, verify:
- All required fields exist
- Arrays contain no duplicate values
- Empty arrays are omitted where appropriate
- Metadata accurately represents the source file
- No database-managed fields are included (`id`, `project_id`, hashes,
  timestamps, or any other system-managed values)

### Temp File Structure

Write each file's metadata as a separate `.json` file into
`.mercury-cortex/temp/`. Preserve the project's relative directory
structure:

```
src/auth/login.rs → .mercury-cortex/temp/src/auth/login.rs.json
```

Create missing intermediate directories automatically. Overwrite any
existing temporary JSON file for the same source path.

### Import Process

1. Generate all metadata JSON files into `.mercury-cortex/temp/`.
2. Validate — confirm every JSON file maps to a source path that is not
   `.mcignore`-d. The source file may not exist on disk; the staged metadata
   is still indexed.
3. If no metadata files were staged, skip the import.
4. Call `metadata/import` once. The engine reads the `.json` files in
   `temp/` and upserts them into `file_data`.
5. **Wait for the response.** Check the result before proceeding.

After a successful import the metadata is in the index. You do not need
to track freshness yourself.

### Handling Import Failures

Wait for the `metadata/import` response and verify the result. The response is
a `results` array with one entry per staged JSON file, each carrying:
- `path` — the source file the metadata was generated for
- `success` — whether the import succeeded
- `error` — the failure reason when `success` is `false`

Compute the outcome by inspecting the array (e.g. errors count = entries with
`success == false`). If you cannot reach the server or the MCP call returns an
error, stop and report the failure.

On partial failure:
- Note which files failed (the `path` of each failed entry)
- Fix the reported issues and call `metadata/import` again

### Never

- Invent functionality not present in the file
- Guess exported APIs
- Copy metadata from unrelated files
- Use placeholder descriptions
- Generate metadata without analyzing the source
- Include database-managed fields (`id`, `project_id`, timestamps,
  hashes, or any other system-managed values)
- Include duplicate values in any array field
- Express the same concept using different synonyms unless they
  represent distinct functionality
- Import metadata in individual file calls — call `metadata/import`
  once with all files

### Progress Report

Upon successful completion of Step 5, emit a concise progress update before moving to Step 6:

**Step 5 Complete — Generate & Submit Metadata**
- **Staged:** `<count>` metadata files
- **Import:** `<imported_count>` imported, `<failed_count>` failed

*Proceeding to Step 6 (Report)…*
