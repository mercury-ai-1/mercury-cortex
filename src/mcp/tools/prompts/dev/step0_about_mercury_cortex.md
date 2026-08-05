## About Mercury Cortex

Mercury Cortex is a context-aware code intelligence engine that indexes
project metadata and makes it searchable. It enables AI tools to discover
existing implementations, understand their purpose and API surface, and
find relevant code across the entire workspace — the current project and
every other project Mercury Cortex knows about.

### How It Works

- The **engine** imports AI-generated metadata from `.mercury-cortex/temp/`
  and persists it as the permanent project index. The index is exactly the
  set of files the AI generated metadata for.
- The **AI** (this workflow) searches indexed metadata to find reusable
  code, implements changes, then generates and imports metadata to keep
  the index current.
- The **importer** persists AI-generated metadata as the source of truth
  for the index.

The engine owns the metadata lifecycle; the AI owns the content. Each
side has a single, well-defined responsibility.

### Core Principles

**Search before you build.** Mercury Cortex exists to maximize code
reuse. Always search for existing implementations before writing new
code. The engine makes cross-project discovery fast and reliable.

**The engine tracks state; the AI produces content.** The engine imports
and persists AI-generated metadata as the project index. The AI generates
accurate metadata and submits it. Neither side crosses this boundary.

**Metadata quality determines search quality.** Accurate, consistent
metadata with well-chosen feature tags and clear summaries is what makes
Mercury Cortex valuable. Invest in metadata quality for every file you
change.

### Workflow Overview

1. **Analyze** the request and build search terms.
2. **Search** Mercury Cortex for existing implementations.
3. **Decide** what to reuse, extend, compose, or create.
4. **Implement** the changes.
5. **Generate and submit** metadata for changed files.
6. **Report** the result.

Follow each step in order. Do not skip steps unless the request is
trivial (e.g., a typo fix or literal rename) — and even then, consider
whether metadata should be updated.

### Open the Current Project First

Mercury Cortex searches, imports metadata, and lists indexed files only
for the **active project**. Before searching, ensure the current project
is open:

1. Call `project/status`. If it returns a `project_id`, the project is
   already open — proceed directly to Step 1.

2. If `project/status` returns `{"status": "no_project_open"}`, read
   `.mercury-cortex/config.json` from the project root. Use its
   `project_id` field and the absolute project root path to call
   `project/open`.

3. If `.mercury-cortex/config.json` does not exist, the project is not
   yet initialized. Stop and tell the user the project must be
   initialized first (run `mercury-cortex:init`, or `mercury-cortex
   project` from the project directory). Do **not** guess or fabricate a
   `project_id`.

4. If `project/open` reports the project is not found in the database
   (e.g., after a database reset), call `project/register` with the
   project root path, then retry `project/open` with the same
   `project_id`. If that still fails, stop and ask the user to run
   `mercury-cortex project`.
