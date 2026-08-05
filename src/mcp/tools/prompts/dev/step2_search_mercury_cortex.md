## Step 2 — Search Mercury Cortex

Search Mercury Cortex to find existing implementations before writing new
code. Follow a progressive search strategy and stop once you've exhausted
the defined search space or found a suitable candidate.

### 1. Use the Search Terms from Step 1

The search terms were built during Step 1 (Analyze the Request). Use
those terms directly for the search queries below. If the request
contains multiple independent concerns, perform one focused search per
concern instead of one broad search with mixed terms.

### 2. Check Project Dependencies

If the task involves an external dependency, package, or plugin, first
read the project's dependency management file (`Cargo.toml`,
`pubspec.yaml`, `package.json`, or similar) to determine whether it is
already used in the project.

- If the dependency **is already used**, search the current project for
  relevant source files (Layer 1 below). The implementations likely live
  alongside the existing usage.
- If the dependency **is not yet used**, broaden the search to other
  projects (Layers 2 and 3) to find reference implementations, patterns,
  or integrations that can be reused.

This single check avoids unnecessary searching and directly informs
which search layer to start with.

### 3. Search Strategy (Progressive & Mandatory Multi-Layer)

Search in layers. Start narrow and broaden only when the current layer
yields no usable results. You MUST evaluate all three layers sequentially
before concluding that 0 candidates exist.

**Layer 1 — Current project.** Start here. Search the active project for
existing implementations using specific technical terms. This is where
project-specific patterns live.

**Layer 2 — Same language/framework.** If Layer 1 produces nothing
usable, broaden to all projects with the same language and framework.

**Layer 3 — All projects (Global fallback).** If Layer 2 produces 0 results,
DO NOT stop. Immediately execute Layer 3 by setting `search_all_projects: true`
in your search query. This searches across every indexed project in the
knowledge base regardless of framework or language, surfacing generic UI
components, reference implementations, and utility patterns.

### 4. Automatic Fallback Queries

Within any search layer, if a detailed multi-word query (e.g. `query: "theme toggle"`) returns 0 results:
- **Do not immediately assume nothing exists.**
- Retry the layer with broader, single-word intent terms (e.g. `query: "theme"`, `query: "theming"`, or `query: "ThemeMode"`).
- Broader queries will catch files where keywords exist in purpose or summary even if specific feature tags are missing.

### 5. When to Stop Searching

Move to Step 3 (Decide: Reuse, Extend, or Create) when one of these
conditions is met:

- A suitable implementation is found that fully or partially satisfies
  the request.
- All three search layers (including Layer 3 with `search_all_projects: true`)
  and fallback queries have been exhausted with no usable results.
- You have determined that further searching will not produce better
  candidates.

### 6. Evaluate Search Results

Search results are the AI-generated metadata for indexed files. Use the
`summary`, `features`, `purpose`, and `exported_functions` fields to
decide whether to read the source. When a result does not provide enough
detail, read the source file from disk.

### 7. Read the Best Matches

Read only the highest-confidence matches that are likely to contribute
to the current implementation. Each search result includes `project_root`
(absolute path to the source project) and `path` (relative file path).
Read source code from disk at `<project_root>/<path>`.

Do not read every returned file — focus on the few most relevant results
to avoid unnecessary context usage.

### 8. Synthesize Findings

Assemble what you've learned before moving to Step 3:

- Which files already exist and what do they do?
- What APIs, types, and patterns do they expose?
- Which can be reused directly, extended, or composed?
- What is missing and must be created from scratch?

### Never

- **Never** skip reading promising files when the metadata provides
  insufficient detail
- **Never** generate new implementations before evaluating reusable code
- **Never** search beyond the defined progression — if all three layers
  produce nothing, move to Step 3
- **Never** skip Step 3 — always evaluate candidates before implementing

### Progress Report

Upon successful completion of Step 2, emit a concise progress update before moving to Step 3:

**Step 2 Complete — Search Mercury Cortex**
- **Layers Used:** `<Layer 1 / Layer 2 / Layer 3>`
- **Candidates Found:** `<count>` relevant files

*Proceeding to Step 3 (Decide: Reuse, Extend, or Create)…*
