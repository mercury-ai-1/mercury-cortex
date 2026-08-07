## Step 2: Project Analysis

1. Analyze the project using any relevant signals, including configuration
   files, project structure, dependency declarations, build configuration,
   and other patterns, to determine the project's language and framework.

2. Call `project/update` with the detected information. Only include
   fields you can determine with reasonable confidence. Mercury Cortex
   stores only the fields you explicitly provide; omitted fields are
   not overwritten:

   ```json
   {
     "project_id": "<from config.json>",
     "metadata": {
       "language": "Rust",
       "framework": "Axum"
     }
   }
   ```

   - Always include `language` when detected.
   - Include `framework` only if you can determine one with reasonable
     confidence.
   - Never send empty strings; omit the field entirely instead.
     Mercury Cortex uses your explicit fields without overwriting
     existing values for omitted fields.
   - **CRITICAL**: The `metadata` value must be a raw JSON object, NOT a
     JSON-encoded string.  Send `"metadata": {"language": "Rust"}`,
     NOT `"metadata": "{\"language\": \"Rust\"}"`.  Double-encoded
     strings will be rejected by the server and cause the tool call to
     fail.

3. **Wait for the response.** If `project/update` returns an error, stop
   and report the failure. Do not proceed to Step 3 until the update is
   confirmed successful.

### Progress Report

Upon successful completion of Step 2, emit a concise progress update before moving to Step 3:

**Step 2 Complete: Project Analysis**
- **Language:** `<language>`
- **Framework:** `<framework or "None">`

*Proceeding to Step 3 (.mcignore Refinement)…*

