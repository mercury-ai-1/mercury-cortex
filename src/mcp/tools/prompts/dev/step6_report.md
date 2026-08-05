## Step 6 — Report

Summarize the completed work, verify the outcome, and report the final
status of the implementation and metadata import.

### 1. Verify the Implementation

Before displaying the summary, confirm:

- The requested functionality has been implemented
- The implementation follows the selected strategy (reuse, extend,
  compose, or create)
- All required source files have been updated

### 2. Verify Metadata

Report the metadata status:

- Files with generated metadata
- Successfully imported files
- Failed imports
- Skipped files (if any)

### 3. Report

Display a structured summary of the completed workflow. Adapt the
sections based on what actually happened — omit sections with no
meaningful data.

**With changes (✅ success):**

```markdown
## ✅ Development Workflow Complete

### Implementation

| Action | Count |
|--------|-------|
| Reused | 2 files |
| Created | 1 file |
| Modified | 3 files |

### Metadata

| Action | Count |
|--------|-------|
| Generated | 4 files |
| Imported | 4 files |
| Failed | 0 |

> **Outcome:** Request completed successfully.
```

**No changes (✅ success, no implementation needed):**

If no source files were created or modified, clearly state that no
implementation changes were required and confirm that metadata
generation and `metadata/import` were skipped.

```markdown
## ✅ Development Workflow Complete

### Implementation

No source changes were required.

### Metadata

Not generated — no files changed.

> **Outcome:** Request completed successfully. No implementation changes needed.
```

**Import failures (⚠️ partial success):**

If `metadata/import` reported failures, list the affected files and
explain how to retry them after fixing the reported issues.

```markdown
## ⚠️ Development Workflow Complete — Warnings

### Implementation

| Action | Count |
|--------|-------|
| Reused | 2 files |
| Created | 1 file |

### Metadata

| Action | Count |
|--------|-------|
| Generated | 4 files |
| Imported | 3 files |
| Failed | 1 file |

#### Failed Files

| File | Reason |
|------|--------|
| `src/auth/login.rs` | validation error |

> **Outcome:** Request completed with metadata import warnings. Fix the
> reported issues and retry `metadata/import`.
```

### Final Status

End with the appropriate outcome blockquote:

- ✅ `> **Outcome:** Request completed successfully.`
- ⚠️ `> **Outcome:** Request completed with metadata import warnings.`
- ❌ `> **Outcome:** Development workflow failed. <reason>`
