## Step 1 — Validate Project

### Never Modify Project Identity

Project identity is owned by the CLI and the `project/register` MCP tool. The
init workflow is read-only with respect to project identity:

- **Never** generate or replace `project_id`.
- **Never** rewrite `.mercury-cortex/config.json`.
- **Never** create or register a new project programmatically.
- **Never** modify project identity in any way beyond the recovery case below.

If you cannot resolve identity issues through the recovery path below, stop
and instruct the user to run:

> `mercury-cortex project`

### Validation Steps

1. **Verify MCP connectivity.** Call `cortex/info`. If unreachable, report the
   error and stop.

2. **Identify the project root.** The project root is your current working
   directory — the directory where the user started the session. Use the
   absolute path to this directory as the project root throughout this
   workflow.

3. **Verify config exists.** Read `.mercury-cortex/config.json` from the
   project root. If the file is missing, not valid JSON, or does not contain a
   `project_id` field, stop. Instruct the user:

   > Run `mercury-cortex project` in this directory first.

4. **Verify the project exists in the database.** Read `project_id` from
   `config.json`. Call `project/open` with the `project_id` and the absolute
   project root path.

5. **Handle `project/open` errors.**

   - **`project not found`:** The database record for this project_id is
     missing (likely because the database was reset). Recovery is available:

     1. Call `project/register` with the project root path. This will detect
        the missing record and automatically re-register the project in the
        database, preserving its existing `config.json` and identity.
     2. Call `project/open` again with the original `project_id` and root
        path. If it succeeds, continue to step 6.
     3. If `project/register` or the retry of `project/open` fails, report
        the specific error and stop. Instruct the user:

        > Run `mercury-cortex project` in this directory.

   - **Other errors** (root path mismatch, identity conflict, or any other
     failure): Report the specific issue and stop. Do not retry, repair, or
     modify project identity.

6. **Verify `.mcignore` exists.** Check that `.mercury-cortex/.mcignore` is
   present in the project root. If missing, instruct the user to run
   `mercury-cortex project`.

### Expected Outcome

By the end of this step, the active session has a verified, open project with:
- A valid `config.json` on disk.
- A `project_id` that matches an existing record in the database.
- A root path that matches the registered project.
- No modifications to any project identity or configuration files beyond the
  `project/register` recovery path.

If any check fails and recovery does not resolve it, stop. Do not continue to
project analysis, `.mcignore` refinement, metadata generation, or any other
step until the user resolves the identity issue.

### Progress Report

Upon successful completion of Step 1, emit a concise progress update before moving to Step 2:

**Step 1 Complete — Project Validated**
- **Project ID:** `<project_id>`
- **Root Path:** `<project_root>`

*Proceeding to Step 2 (Project Analysis)…*
