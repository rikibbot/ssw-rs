# ssw-rs

This repository hosts `ssw-rs`, an HTML-first Rust framework for durable server-rendered web apps.

Start from the architecture in [`ARCHITECTURE.md`](./ARCHITECTURE.md). Keep the implementation aligned with the product thesis: server-side rendering by default, server-owned state, optional JavaScript enhancement, minimal dependencies, and clear crate boundaries.

## Working Principles

- Work autonomously when the path is clear.
- Ask for clarification when requirements are ambiguous or when a foundational design choice is uncertain.
- Be explicit about tradeoffs and call out risks directly.
- Prefer simpler designs over clever abstractions.
- Surface meaningful refactoring opportunities when they materially improve clarity, ergonomics, or long-term maintenance.
- Treat dangerous or irreversible actions as opt-in and confirm before proceeding.

## Context Recovery

- Read `context/current.md` at the start of a run.
- Update `context/current.md` at the end of a run.
- Record durable decisions in `context/decisions.md` as append-only entries.
- Use `context/sessions/YYYY-MM-DD.md` only for substantial work.
- Do not store secrets in `context/`.

## Architecture Guardrails

- Server-side rendering is the default model.
- Client-side routing is out of scope unless the architecture is intentionally revised.
- JavaScript enhancement must be optional and additive.
- `ssw-core` must remain independent from backend-specific concerns.
- Backend integrations should feel native to their framework instead of forcing a lowest-common-denominator abstraction.
- `ssw-components` is optional and must not define the core framework architecture.
- Prefer small, well-understood dependencies; justify new ones.

## Style and Structure

- Prefer precise names and small public APIs.
- Keep comments minimal and decision-focused; do not restate the code.
- Update nearby docs when implementation changes would otherwise make them stale.
- Prefer ASCII punctuation in prose.
- Keep crate boundaries intentional; avoid dependency cycles and accidental cross-layer leakage.

## Testing

- Add tests alongside behavior that is stable enough to commit to.
- Prefer high-signal tests that exercise user-visible behavior.
- If a test exposes likely incorrect behavior, flag the bug instead of codifying it.

## Commits and PRs

Use conventional commits: `type(scope): description`.

- Types: `feat`, `fix`, `refactor`, `test`, `chore`, `perf`, `docs`
- Scope: crate name (`core`, `actix`, `html`, `components`) or omit for repo-wide changes
- Keep each commit focused on one logical change
- Call out design decisions explicitly when they affect crate boundaries or rendering semantics

## Key References

- [`ARCHITECTURE.md`](./ARCHITECTURE.md)
- [`README.md`](./README.md)
- [`context/current.md`](./context/current.md)
- [`context/decisions.md`](./context/decisions.md)
