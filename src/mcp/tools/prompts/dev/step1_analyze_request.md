## Step 1 — Analyze the Request

Analyze the user's request to understand the intended outcome before
searching Mercury Cortex. Identify the technical concepts, required
functionality, and implementation scope. This step performs analysis
only — do not implement changes or generate code.

### 1. Identify the Request Type

Determine the type of work requested:

- **Bug fix** — what is the incorrect behavior and what should it do?
- **New feature** — what functionality is being added?
- **Refactor** — what structure is changing and why?
- **Performance improvement** — what is being optimized and under what
  conditions?
- **UI/UX change** — what visual or interaction change is needed?
- **Documentation** — what needs to be documented and for whom?
- **Configuration** — what settings or environment is changing?
- **Testing** — what behavior needs test coverage?
- **Migration** — what is moving from one system/version to another?
- **Other** — describe the nature of the request

This helps guide later search decisions and implementation approach.

### 2. Extract Technical Concepts

Identify specific technical terminology from the request, not just
conversational wording:

- **Features** — "theme toggle", "auth middleware", "dark mode"
- **Components** — "UserRepository", "AuthProvider", "NavBar"
- **Services** — "EmailService", "PaymentGateway"
- **APIs / endpoints** — "POST /api/login", "graphql mutations"
- **Design patterns** — "observer pattern", "dependency injection",
  "repository pattern"
- **Architecture concepts** — "layered architecture", "event-driven",
  "CQRS"
- **Framework-specific terminology** — "middleware", "hook", "decorator",
  "provider", "guard"
- **Library names** — "serde", "axum", "react-router", "sqlx"
- **Important function/class names** — "validate_token", "UserService",
  "handle_request"

Use technical terminology from the codebase's domain.

### 3. Decompose Complex Requests

If the request contains multiple independent concerns, split them into
separate sub-problems before searching. Each sub-problem will get its
own targeted search in Step 2.

For example, "add dark mode with JWT authentication" decomposes into:

- **Authentication** — JWT token handling, login flow, protected routes
- **Theme** — dark mode toggle, theme persistence, CSS variables

This prepares Step 2 for multiple focused searches instead of one
broad search with mixed terms.

### 4. Decide What Needs Searching

Determine what must be searched in Mercury Cortex:

- **Existing implementation** — find how a feature is already built
- **Similar feature** — find analogous patterns from other parts of the
  codebase
- **Utility functions** — find reusable helpers that might apply
- **Shared components** — find UI or library code shared across modules
- **Architecture examples** — find structural patterns to follow
- **Nothing** — the change is simple enough that search is unnecessary
  (e.g., trivial config change, typo fix, literal rename)

### 5. Build Search Terms

Generate concise, intent-based technical search terms (1–2 words per term) from the extracted concepts.
Never use long conversational phrases (e.g. avoid `"Create a single-page application with support for Light, Dark, and System theme modes"` or `"ThemeMode theme toggle dark light"`).
Instead, break concepts into short, focused terms that target specific capabilities, components, or UI elements.

| Conversational / Prompt Request | Intent-Based Technical Search Terms |
|---|---|
| "add dark mode with toggle" | `theme`, `theme-toggle`, `theme-mode`, `ThemeMode` |
| "JWT auth login flow" | `jwt-auth`, `authentication`, `AuthGuard` |
| "save user settings" | `UserSettings`, `preferences` |
| "email notification service" | `EmailService`, `send_email` |

### 6. Prefer Reuse

Before planning new code, always consider whether reusable
implementations may already exist in the codebase. The goal of Mercury
Cortex is to maximize code reuse before generating new implementations.

Note any files, patterns, or types you already know about from the
session context — these will inform the search strategy.

### Progress Report

Upon successful completion of Step 1, emit a concise progress update before moving to Step 2:

**Step 1 Complete — Request Analysis**
- **Request Type:** `<type>`
- **Search Terms:** `<terms>`

*Proceeding to Step 2 (Search Mercury Cortex)…*

