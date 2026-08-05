## Step 3 — Decide: Reuse, Extend, or Create

Evaluate the search results and determine the most appropriate
implementation strategy. Prefer reusing existing code whenever it
provides a correct, maintainable, and high-quality solution. This step
produces a concrete implementation plan for Step 4.

### 1. Evaluate Every Candidate

Assess each search result against these criteria:

- **Functional similarity** — how closely does it match the request?
- **Code quality** — is it well-structured, tested, and maintained?
- **Compatibility** — does it fit the current request's requirements?
- **Language and framework** — does it use the same stack?
- **Architecture consistency** — does it follow the project's patterns?
- **Reusability** — can it be used directly or with minimal changes?
- **Maintainability** — will reusing it reduce or increase tech debt?

### 2. Understand Each Decision

**Reuse** — an existing implementation matches the request and can be
adapted. Reuse means understanding the implementation's full context:
its dependencies, related files, connected components, API surface, and
surrounding project patterns. You take that understanding and integrate
the relevant parts into the current solution. Reuse is not copying a
file — it is applying the same design, patterns, or logic in the right
places.

**Extend** — an existing implementation provides a strong foundation but
needs additional functionality. Modify or augment without duplicating
existing logic.

**Compose** — combine multiple reusable implementations into a new
solution. Prefer composition over duplication.

**Create** — no suitable implementation exists. Implement a new solution
that follows the project's architecture and coding standards.

| Situation | Action |
|---|---|
| Existing code fully satisfies the request | Reuse |
| Existing code partially satisfies the need | Extend or compose |
| Multiple partial matches exist | Compose |
| No relevant code found | Create |
| User is asking a question | Answer from search + source reading |

### 3. Locate Source Files in Other Projects

Each search result includes `project_root` (the absolute path to the
source project's root directory) and `path` (the relative file path
within that project). To read a source file found in a cross-project
search (Layer 2 or Layer 3):

```
<project_root>/<path>
```

For example, if a search result has `project_root: "/Users/dev/my-app"`
and `path: "src/utils/format.rs"`, read the file at
`/Users/dev/my-app/src/utils/format.rs`.

When reusing code from another project, read the full source file plus
any closely related files in the same module to understand the complete
implementation context before adapting it.

### 4. Compare Multiple Candidates

If several implementations were found, compare them before choosing:

- Which is most complete?
- Which best matches the current architecture?
- Which requires the fewest modifications?
- Which provides the highest long-term maintainability?

### 5. Principles

**Understand the full context before reusing.** Reuse is not copying a
file. Read the implementation, its dependencies, and related files.
Understand how it connects to the rest of the project — data flow,
interfaces, error handling, configuration — before deciding to reuse it.

**Prefer reuse, not blind reuse.** Reuse code only when it improves
consistency and maintainability. Do not force reuse if it results in
unnecessary complexity or an incorrect implementation.

**Avoid duplication.** Never create new code that duplicates existing
reusable functionality without a valid technical reason.

**Keep the user's intent first.** If reuse would change the requested
behavior, prioritize the user's requirements over reuse. Correctness is
always more important than maximizing reuse.

### Never

- **Never** reuse code without reading it and understanding its context
- **Never** copy an entire file as reuse — understand what it does and
  apply the relevant parts
- **Never** duplicate existing functionality
- **Never** choose the first search result without evaluation
- **Never** modify unrelated implementations to force reuse
- **Never** ignore better candidates found later in the search process

### 6. Produce an Implementation Plan

Before writing code, produce a clear plan covering:

- Which existing files will be reused (and how)
- Which files will be extended (and what will change)
- Which new files (if any) need to be created
- Dependencies and related files that must be understood or adapted
- Why this approach was selected

This plan becomes the input for the implementation step.

### Progress Report

Upon successful completion of Step 3, emit a concise progress update before moving to Step 4:

**Step 3 Complete — Strategy & Decision**
- **Strategy:** Reusing `<N>`, Extending `<N>`, Creating `<N>`

*Proceeding to Step 4 (Implement Changes)…*

