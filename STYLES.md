# Workspace Code Style

This document describes common patterns in the workspace. Follow the surrounding file when a local pattern differs. Explicit task instructions take precedence over these conventions.

## Formatting and Spacing

- Follow `rustfmt.toml`: Rust 2024, a 130-character maximum width, and field initialization shorthand. Use four-space indentation, LF line endings, and a final newline, as specified in `.editorconfig`.
- Separate imports, items, methods, and logical phases of a function with one blank line. Keep closely related declarations or assignments together without blank lines between each statement.
- Separate a control-flow block from the preceding and following statements with a blank line, including between consecutive `if` or `if let` blocks. This includes guards, loops, and `match` expressions used as statements. Do not insert blank lines inside a short block solely to pad it.
- Keep a final expression directly after the preceding statement when it completes the same operation; separate it when it begins a distinct phase. Use early returns for validation and exceptional cases.
- Let rustfmt lay out long signatures, method chains, match arms, and struct literals. Put attributes immediately above the item or field they apply to; adjacent attributed fields are often separated by a blank line.

## Names and Imports

- Use `snake_case` for files, modules, functions, variables, and fields; `UpperCamelCase` for types, traits, and enum variants; and `SCREAMING_SNAKE_CASE` for constants and statics.
- Name values for their role in the local operation: examples include `ctx`, `options`, `input`, `model`, `row`, `value`, `out`, and `id`. Use domain-specific names when they distinguish values. Short names such as `v` or `s` occur in small closures and serialization implementations.
- Group related imports with braces when useful. Common groups are standard library, external crates, and `crate` or `super` imports, separated by blank lines; follow the file's existing order and grouping rather than imposing a new order.
- Prefer established crate paths and local module imports. Binary crates commonly access workspace facilities through the `nim` facade; library crates use their direct dependencies.

## APIs and Implementations

- Avoid indirection unless it serves a purpose; put behavior directly in its implementation instead of adding functions that only delegate or are only called by a wrapper.
- Keep implementation details private or `pub(crate)` and expose the types, modules, constructors, and methods needed by callers. Follow the existing visibility of the owning module rather than adding public wrappers by default.
- Group related methods in `impl` blocks. Constructors are commonly named `new`; state service operations use names such as `create`, `update`, `delete`, and `find_by_id`.
- Use the workspace's `nim_error::Result` and diagnostics at application and library boundaries where surrounding code does so; storage methods commonly return their underlying `sqlx::Error`. Propagate errors with `?` and return the final expression directly where appropriate.
- Preserve the surrounding async API shape. Where traits return boxed futures, use the existing boxed-future pattern and explicit lifetimes; use `.await` at asynchronous call sites.
- Match existing derives and serde attributes for neighboring types. State IDs are UUID-backed newtypes with prefixed string representations and conversions to and from UUIDs.

## Files and Modules

- After imports, place constants and statics first. Order the remaining items by visibility first (`pub`, then `pub(crate)` and other restricted visibility, then private), and by item kind within each visibility group, with types before other items.
- Workspace libraries live under `crates/`; executable services live under `bins/`. Each crate has its own `Cargo.toml` and `src` tree and normally inherits workspace package settings and lints.
- Use lowercase `snake_case.rs` filenames. A module may be a single file or a directory with `mod.rs`; follow the existing subsystem's layout.
- State service directories commonly separate operations into files such as `create.rs`, `update.rs`, and `find_by_id.rs`, with `mod.rs` declaring modules and exposing their public items. Shared domain types live under `crates/nim-state/src/types/`.
- AI model implementations are grouped under `crates/nim-ai/src/models/<model>/`; pipelines are grouped by media and operation. Runtime pipelines mirror those media and operation groupings.
- Binary entry points typically have `main.rs`, `config.rs`, and a `context` module; service handlers live under `routes/`, with route modules composing their subroutes. The gateway also has templates under `bins/nim-gateway/templates/`.
- Keep comments and tests consistent with the task's explicit instructions; do not add them solely to enforce a style preference.
