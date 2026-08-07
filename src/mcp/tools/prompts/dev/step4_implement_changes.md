## Step 4: Implement Changes

Implement the approved solution by reusing, extending, or creating code
as determined in the previous step. Follow the project's architecture,
coding standards, and Mercury Cortex policies.

### 1. Follow the Implementation Plan

Execute the decisions made in Step 3 without reconsidering the strategy
unless new information is discovered:

- **Reuse**: use identified files as-is. When reusing from another
  project (Layer 2 or Layer 3), read the source file at
  `<project_root>/<path>` using the `project_root` from the search result.
  Read closely related files in the same module to understand the full
  context before adapting.
- **Extend**: modify existing code to add functionality
- **Compose**: combine multiple existing implementations
- **Create**: write new files only when necessary

### 2. Preserve Project Consistency

Ensure all new and modified code follows existing conventions:

- **Architecture**: match the project's module and layer structure
- **Folder structure**: place files where similar files live
- **Naming conventions**: use the same casing, prefixes, and suffixes
- **Coding style**: mimic surrounding code (formatting, idioms)
- **Dependency usage**: use existing libraries, avoid new ones unless
  necessary
- **Error handling**: follow established patterns (error types,
  propagation, logging)
- **Logging**: use the project's logging framework and levels
- **Documentation**: follow docs conventions from policy.md
- **Comments**: include comments only where policy.md requires them

### 3. Minimize Changes

Make the smallest set of changes required to satisfy the user's request.
Do not perform unrelated refactoring, reformatting, or cleanup unless
explicitly requested.

### 4. Reuse Before Creating

Before writing new code, verify that existing reusable code cannot
satisfy the requirement through reuse, extension, or composition. When
evaluating cross-project candidates, read the source file using
`<project_root>/<path>` from the search result to confirm suitability.

### 5. Respect Existing Behavior

When modifying existing implementations:

- Preserve backward compatibility whenever possible
- Avoid breaking unrelated functionality
- Keep public APIs stable unless the request explicitly requires changes

### 6. Validate During Implementation

Continuously verify that:

- The implementation satisfies the request
- No duplicated logic has been introduced
- New code integrates naturally with the project
- Existing reusable code remains reusable

### Never

- **Never** rewrite working implementations without a valid reason
- **Never** introduce duplicate functionality
- **Never** ignore the project's architecture
- **Never** add unnecessary dependencies
- **Never** modify unrelated files
- **Never** implement features beyond the user's request

### 7. Completion Criteria

Before moving to the next step, confirm:

- The requested functionality is implemented
- All required files have been updated
- The project remains consistent
- The solution is ready for metadata generation

Prioritize correctness, maintainability, consistency, and code reuse
over implementation speed or the amount of new code written.

### Progress Report

Upon successful completion of Step 4, emit a concise progress update before moving to Step 5:

**Step 4 Complete: Implement Changes**
- **Created:** `<count>` files
- **Modified:** `<count>` files

*Proceeding to Step 5 (Generate & Submit Metadata)…*

