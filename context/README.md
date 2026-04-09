# Context System

This folder is a lightweight shared memory for humans and agents working on `ssw-rs`.

## Files

- `current.md`: current truth for active work, updated at the start and end of a run
- `decisions.md`: append-only log of durable technical or product decisions
- `sessions/`: optional session notes for substantial work

## Rules

- Keep entries short and concrete.
- Timestamp entries with local date and time when useful.
- Separate `fact`, `decision`, `risk`, and `todo`.
- Link evidence when possible, such as files, commits, or commands.
- Do not store secrets.
