## Step 5: Verification, Cleanup, and Summary

### 1. Verify the Complete Workflow

Before reporting success, confirm that every prior step completed:

- Project validation passed (config exists, DB project exists, root path
  matches).
- Project analysis completed (language, framework detected and saved).
- `.mcignore` updated (new patterns submitted, user rules preserved).
- Metadata generation completed (key source files indexed).
- `metadata/import` completed, with the engine importing every staged metadata
  file.

If any step was skipped or failed, report the specific issue in the summary.

### 2. Verify Engine State

After import, confirm that Mercury Cortex reflects the expected state:

- The project is registered and accessible.
- The `indexed_files` count from `metadata/import` is visible and nonzero.
- The `results` array from `metadata/import` shows no failures (every
  `success` is `true`).

### 3. Handle Failures Explicitly

- **MCP errors:** If the `metadata/import` MCP call itself returned an error,
  report it and do not display a success summary.
- **Import failures:** Check the `results` array in the `metadata/import`
  response for entries with `success: false`. Fix the reported issues and call
  `metadata/import` again.
- **Any other failures:** Report the specific step that failed and what the
  user should do next.

### 4. Display a Structured Summary

Present a concise completion report covering the final state of each step.
Adapt the sections based on what actually happened, omitting sections with no
meaningful data. Do not repeat details already shown during earlier steps.

The `indexed_files` count comes from the `metadata/import` response.

**Success (✅):**

```markdown
## ✅ Mercury Cortex Initialization Complete

### ✅ Project

| Field | Value |
|-------|-------|
| ID | `<project_id>` |
| Language | `<language>` |
| Framework | `<framework>` |

### ✅ .mcignore

| Metric | Count |
|--------|-------|
| Patterns Added | N |
| Existing Patterns | N |

### ✅ Indexed Files

| Metric | Count |
|--------|-------|
| Total | `<indexed_files>` |
| Imported | `<count>` |

> **Outcome:** Initialization completed successfully. The project is now
> fully indexed and ready for AI-assisted search and code reuse.
```

**Partial failure (⚠️):**

If specific steps failed but others succeeded, show only the failed
sections with ❌ and include an action items list:

```markdown
## ⚠️ Mercury Cortex Initialization: Partial

### ✅ Project

| Field | Value |
|-------|-------|
| ID | `<project_id>` |
| Language | `<language>` |
| Framework | `<framework>` |

### ❌ Metadata Import

The import reported failures. Fix the reported issues and retry `metadata/import`.

> **Outcome:** Initialization completed with warnings. Fix the reported
> issues and retry `metadata/import`.
```

**Full failure (❌):**

If a critical step failed (e.g. project validation, MCP connectivity),
report the failure and what the user should do next:

```markdown
## ❌ Mercury Cortex Initialization Failed

<description of what failed and why>

> **Outcome:** Initialization failed. <specific next step for the user>.
```

### 5. Final Message

End with the appropriate outcome blockquote:

- ✅ `> **Outcome:** Initialization completed successfully.`
- ⚠️ `> **Outcome:** Initialization completed with warnings.`
- ❌ `> **Outcome:** Initialization failed. <reason>`
