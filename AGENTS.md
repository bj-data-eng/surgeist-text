# surgeist-text Repository Guide

Use `$pisct:coordination` for standalone delivery and the smallest focused
`$pisct:<skill>` for focused work. Preserve a workflow explicitly selected by the
user or higher instructions. Use `$pisct:plane-coordination` only when plane
coordination is explicitly selected. This guide supplies repository facts; it
does not grant mutation, installation, commit, publication, or cross-repository
authority.

## Authority Split

This file is the repository's committed discovery entry point. It owns the
mapping from mutable repository facts to authoritative sources, the intended
crate boundary, configured command inventory, and repository workflow selection.
PISCT skills supply reusable coordination and engineering guidance, including
the safety constraints in `$pisct:invariants`. Higher-priority user and system
instructions still apply.

Use the sources below for current product facts and the task-appropriate PISCT
skill for the work being performed. An investigation does not authorize a fix,
and a documentation task does not authorize changing the implementation.

## Repository Identity And Ownership

`surgeist-text` is an independent Rust library repository. It owns text source
composition and edits, font-facing inputs, shaping and measurement, line and
text layout, and text geometry. Its manifest, public front door, implementation,
focused tests, and documentation belong here. Placement in a parent workspace,
project, task, branch, or worktree does not transfer ownership.

The root `surgeist` repository owns the facade and public composition surface,
cross-crate adapters, root integration tests and tools, this leaf's gitlink, and
the API generator and generated audit artifacts. Inspect root authorities only
when root integration is in scope; do not copy its mutable inventory here.

## Discover The Current Structure

Read these sources instead of relying on cached descriptions.

| Fact | Authoritative source |
| --- | --- |
| Package identity, version, edition, dependencies, features, and targets | `Cargo.toml` |
| Public front door | `src/lib.rs` and its reexports |
| Human entry point and documentation navigation | `README.md` |
| Source composition, ranges, validation, and edits | `src/source.rs`, `src/composer.rs`, `src/range.rs`, `src/edit.rs` |
| Text style inputs, supported features, options, and diagnostics | `src/style.rs`, `src/style_support.rs`, `src/options.rs`, `src/error.rs` |
| Layout construction and cache identity | `src/system.rs`, `src/cache.rs` |
| Layout outputs, geometry, movement, and optional projections | `src/layout.rs`, `src/geometry.rs` |
| Focused verification | `src/tests.rs`, loaded by `src/lib.rs` |
| Configured commands and feature gates | `Cargo.toml`, this command inventory, and tracked task-runner or CI configuration when present |
| Project license | `LICENSE` |
| Direct-dependency attribution scope and included legal material | `NOTICE.md`, `licenses/`, and the exact dependencies in `Cargo.toml` |
| Integration MSRV, authoritative leaf URL, and compatible pin when root integration is in scope | Root `Cargo.toml`, root `.gitmodules`, and root's committed gitlink |

When sources disagree, report exact paths and revisions. Do not guess, silently
rewrite another authority, or widen the task to reconcile them. The leaf
manifest does not declare a `rust-version`; its edition is not an integration
MSRV policy.

## Product Boundary

The library accepts text with resolved style spans and inline boxes. It
validates inputs, shapes and lays out text through Parley, and exposes layout
facts, cursor and selection geometry, low-level movement, and source edits.
It does not own style resolution, rendering engines, widgets, document identity,
or application commands. Source IDs and revisions support text and cache
identity; they do not make this crate the document owner.

Cross-Surgeist lowering and adapters are assigned to root by the committed
boundary. An existing discrepancy is present at source revision
`9109087d7f82d28b10c85e312b32a7a006cb0605`: `Cargo.toml` declares the optional
`surgeist-render` sibling dependency and `text-render` feature, while
`src/layout.rs` implements `Layout::push_render_text` and render conversions.
Record this implementation when assessing affected behavior; it does not
resolve or broaden the assigned boundary. Reconciliation requires a separately
scoped decision. The `text-accessibility` feature also exposes AccessKit
projection through the public layout API.

For work involving another repository, resolve each repository's ownership from
its current committed policy and source. Inspecting a sibling or root does not
grant write authority there.

## Generated Artifacts

Source in this repository is authoritative. Root `surgeist` owns the API
generator and generated API audit artifacts; this leaf carries no copies.
Resolve refresh and check commands at that owner when API audit work is in
scope. Do not hand-edit generated artifacts or introduce a leaf generator.

## Command Inventory

These commands describe local verification capability. The assigned scope and
PISCT skill guidance select the required checks. Run noninteractive checks
through `$pisct:process` with caller-authorized, already-present tooling.

```sh
cargo check -p surgeist-text
cargo test -p surgeist-text
cargo clippy -p surgeist-text --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
```

Default features are empty. For affected optional surfaces, use Cargo's
`--features text-accessibility`, `--features text-render`, or
`--features text-accessibility,text-render` on the applicable check, test, and
Clippy commands, before Clippy's `--` separator. Feature definitions in
`Cargo.toml` and gated tests in `src/tests.rs` own current coverage.

Cargo must be able to resolve the declared sibling path dependencies. Render
tests include headless renderer initialization and need a usable graphics
backend. Prefer offline checks; use `--locked` when a current local lockfile is
present. `Cargo.lock` is ignored by this library repository. Missing tooling or
cached dependencies does not authorize acquisition.

Discovery is complete when ownership, product boundary, public entry points,
dependency and feature facts, generated-artifact ownership, verification sources,
and the applicable command inventory are established from current source.
