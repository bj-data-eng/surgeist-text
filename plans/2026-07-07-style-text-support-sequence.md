# Style Text Support Sequencing Plan

Date: 2026-07-07

## Purpose

Sequence the implementation plans needed for `surgeist-text` to consume the
text-owned slice of root/style CSS integration work.

This is not an implementation plan. It authorizes the next crate-local planning
artifacts and defines their order, scope, dependencies, and review gates.

## Source Inputs

Primary directive:

```text
/Users/codex/Development/surgeist-text/plans/2026-07-07-style-text-support-directive.md
```

Root/style inventory inputs:

```text
/Users/codex/Development/surgeist/plans/2026-07-04-css-integration-support-inventory.md
/Users/codex/Development/surgeist-style/plans/2026-07-07-style-root-handoff-notes.md
/Users/codex/Development/surgeist-style/plans/2026-07-07-style-css-api-artifact.md
/Users/codex/Development/surgeist-style/plans/2026-07-05-css-property-coverage-ledger.md
```

Local crate surfaces to review in each implementation plan:

```text
/Users/codex/Development/surgeist-text/src/style.rs
/Users/codex/Development/surgeist-text/src/options.rs
/Users/codex/Development/surgeist-text/src/system.rs
/Users/codex/Development/surgeist-text/src/layout.rs
/Users/codex/Development/surgeist-text/src/source.rs
/Users/codex/Development/surgeist-text/src/cache.rs
/Users/codex/Development/surgeist-text/src/tests.rs
```

## Boundary

`surgeist-text` consumes text-ready values. It must not parse CSS, depend on
`surgeist-css`, lower CSS syntax into style values, or reach into root/private
style internals.

Root owns CSS-to-style lowering and root-to-text adapter code. Style owns
resolved style values, cascade, variables, selectors, style buckets, and style
diagnostics. Layout owns layout algorithms and layout-specific input contracts.
Render owns final paint realization.

Text implementation plans should expose crate-owned text contracts that root can
target once root integration work is ready.

## Current Support Snapshot

Already supported in `surgeist-text`:

- font family list strings with validation through Parley font-family parsing
- named font weight values
- three font stretch/width buckets: condensed, normal, expanded
- normal, italic, and oblique slant with finite angle validation
- OpenType font feature and variation settings as CSS setting-list strings
- font size, line height, letter spacing, word spacing, text brush, locale
- word break, wrap/no-wrap, overflow wrap
- paragraph alignment through `Options::alignment`
- first-line/each-line/hanging text indent where Parley can express it
- underline and strikethrough with offset, thickness, and brush override
- glyph run, line, baseline, selection, inline-box, decoration, and render
  projection facts after layout

Partial support:

- font policy defaults to generic sans-serif when no family list is provided,
  but no font resource or `@font-face` intake policy exists
- white-space has an enum but only `Preserve` is supported
- text alignment has no final-line alignment model
- text decoration does not model overline, decoration stroke style, skip policy,
  or symbolic color resolution
- inline boxes exist, but vertical-align is not modeled as text-owned input
- metrics are post-layout facts, not a standalone root/layout-facing inline
  metric contract

Not yet supported:

- numeric/intermediate font weights beyond the existing named buckets
- richer font stretch values beyond the three current buckets
- `font-variant`
- `text-align-last`
- `text-overflow`
- `text-transform`
- selection color projection
- symbolic color/currentColor/system-color intake for text or decoration paint
- root-facing unsupported-value reporting contract for style values that text
  cannot consume yet

## Sequence Overview

The directive should become five implementation plans. Each plan should be
reviewed cleanly before the next one is written in detail, because later plans
depend on the contracts established by earlier ones.

1. Style intake contract and rejection matrix
2. Font policy and font value parity
3. Text flow behavior parity
4. Decoration and selection projection
5. Inline alignment and layout-ready metric facts

The first plan is the foundation. It defines the public text-domain intake
shape and the exact unsupported-value behavior that root can rely on while the
remaining plans add support incrementally.

## Plan 1: Style Intake Contract And Rejection Matrix

Proposed file:

```text
/Users/codex/Development/surgeist-text/plans/2026-07-07-style-text-intake-contract.md
```

Goal:

Define the crate-owned contract for consuming root/style-normalized text inputs,
including what maps directly to existing `Style`/`Options`, what needs new
text-domain types, and what root must reject initially.

Required coverage:

- map each directive item to current `surgeist-text` API support
- define the crate-owned names and shapes for any new text-domain adapter types
- keep CSS and style crate dependencies out of this crate
- define explicit unsupported diagnostics for values text cannot consume
- define cache-key implications for any new style/options fields
- define focused tests for accepted, rejected, and cache-distinct inputs

Expected implementation scope:

- likely adds a text-owned intake or normalization module only if direct
  `Style`/`Options` expansion is not sufficient
- may add new error detail variants if the existing unsupported-feature detail
  is too coarse for root coordination
- should not implement the large behavior gaps yet

Dependencies:

- none beyond the directive and local source review

Completion gate:

- implementation plan reviewed cleanly
- plan explicitly states the initial root rejection matrix
- plan keeps all code changes crate-local and CSS-free

## Plan 2: Font Policy And Font Value Parity

Proposed file:

```text
/Users/codex/Development/surgeist-text/plans/2026-07-07-style-text-font-policy.md
```

Goal:

Expand or explicitly constrain text's font-facing model so root can pass
style-normalized font data without guessing.

Required coverage:

- font family list policy, including generic/symbolic family handling
- font resource policy boundaries for `@font-face` and loaded font data
- numeric or intermediate font weight policy
- font stretch parity beyond condensed/normal/expanded, or explicit rejection
- `font-variant` intake policy
- oblique angle mapping and validation
- font feature settings and variation settings validation boundaries
- tests proving root/style-normalized font values either shape or reject
  predictably

Expected implementation scope:

- `src/style.rs` font enums/types and validation
- `src/system.rs` Parley projection
- `src/cache.rs` hashing for new font fields
- `src/tests.rs` font validation and shaping tests

Dependencies:

- Plan 1 must define accepted/rejected font value categories

Completion gate:

- root can know which font values can be lowered into text immediately
- all unsupported font values have explicit rejection behavior
- no font file loading or host font discovery is introduced

## Plan 3: Text Flow Behavior Parity

Proposed file:

```text
/Users/codex/Development/surgeist-text/plans/2026-07-07-style-text-flow-behavior.md
```

Goal:

Close the behavior gaps for text-owned line-flow controls that are needed by
style integration.

Required coverage:

- white-space handling beyond preserve, or an explicit staged subset
- text wrapping parity with style values
- word-break and overflow-wrap parity checks
- `text-overflow` policy and projection facts
- `text-align-last`
- text transform policy and source-range implications
- text indentation parity and remaining unsupported combinations
- tests for source ranges, clusters, selection, cursor movement, and cache keys
  where transformations or whitespace processing can alter shaped text

Expected implementation scope:

- `src/style.rs` and `src/options.rs` for new fields/types
- `src/system.rs` for Parley projection or pre-shaping normalization
- `src/layout.rs` if new projection facts are needed
- `src/cache.rs` for cache-key changes
- `src/tests.rs` for flow behavior and range preservation

Dependencies:

- Plan 1 must define unsupported-value reporting
- Plan 2 should settle font-size and line-height semantics used by flow tests

Completion gate:

- root can pass or reject style text-flow values deterministically
- no generated content, pseudo-element materialization, or layout algorithm work
  enters this crate

## Plan 4: Decoration And Selection Projection

Proposed file:

```text
/Users/codex/Development/surgeist-text/plans/2026-07-07-style-text-decoration-selection.md
```

Goal:

Represent text-owned decoration and selection paint facts with enough fidelity
for root/render coordination while keeping final paint realization outside
text.

Required coverage:

- decoration line support: underline, strikethrough, overline if accepted
- decoration stroke style policy: solid now or modeled variants with explicit
  render coordination
- decoration thickness policy: auto, from-font, absolute, or staged subset
- decoration color policy, including currentColor/symbolic-color boundaries
- selection color as text selection projection data
- render adapter impact for `text-render`
- tests for decoration projection, render scene encoding, and selection facts

Expected implementation scope:

- `src/style.rs` decoration and selection style fields
- `src/layout.rs` decoration and selection projection structs
- optional `text-render` adapter updates for newly projected paint facts
- `src/cache.rs` for new style fields
- `src/tests.rs` including `text-render` feature tests where applicable

Dependencies:

- Plan 1 rejection contract
- Plan 3 source/range behavior if selection projection depends on transformed
  or collapsed text

Completion gate:

- text exposes selection and decoration facts without claiming final render
  realization
- render/root coordination notes identify any paint features text cannot encode

## Plan 5: Inline Alignment And Layout-Ready Metric Facts

Proposed file:

```text
/Users/codex/Development/surgeist-text/plans/2026-07-07-style-text-inline-metrics.md
```

Goal:

Expose text-owned inline metric facts and vertical-align inputs that root/layout
can consume without making text depend on `surgeist-layout`.

Required coverage:

- vertical-align values that affect inline text metrics or inline box alignment
- inline box baseline/alignment data needed by root/layout
- font/text metric facts that can be computed by text before or after shaping
- relationship between `Metrics`, `Line`, `RunMetrics`, `PositionedInlineBox`,
  and any new metric contract
- line-height and font policy assumptions required for reliable metrics
- tests for inline boxes, baselines, vertical alignment, and layout-facing
  metric stability

Expected implementation scope:

- `src/source.rs` inline box inputs if new alignment metadata is needed
- `src/layout.rs` metric projection structs and accessors
- `src/style.rs` if vertical-align is style-owned text input
- `src/cache.rs` if inputs affect layout cache
- `src/tests.rs` for metric and inline-box behavior

Dependencies:

- Plan 2 font policy
- Plan 3 line-height/text-flow behavior
- Plan 4 only if selection/decoration metrics affect line boxes

Completion gate:

- root/layout can consume text-owned inline metric facts through public
  `surgeist-text` APIs
- no dependency on `surgeist-layout`
- layout coordination notes distinguish text-owned metrics from layout-owned
  algorithms

## Cross-Plan Rules

- Each implementation plan must fit in this repo's normal plan size range. Aim
  for under 1,000 lines; split further before reaching roughly 2,500 lines.
- Each implementation plan must be executable through the local `AGENTS.md`
  worker/reviewer workflow.
- Each plan must include exact file ownership, task-scoped commits, focused
  tests, and final crate checks.
- Backwards compatibility shims are not required at this development phase.
- Do not create root adapters or edit sibling crates from this repo.
- Do not update root submodule pointers from this repo.
- Do not add a dependency on `surgeist-style`, `surgeist-css`,
  `surgeist-layout`, root, retained, runtime, or render except the existing
  optional `surgeist-render` feature.
- Any root/style/layout/render requirement discovered during implementation
  planning should be reported as coordination notes, not implemented here.

## Review Order

For each implementation plan:

1. Coordinator drafts the plan in `plans/`.
2. Separate reviewer checks scope against this sequencing plan and the directive.
3. Coordinator reconciles findings.
4. The implementation plan is committed only after review is clean.
5. The next implementation plan is drafted using any coordination notes from
   the previous plan.

Do not start implementation of a later slice until the earlier slice's plan is
clean, unless the user explicitly changes the sequence.

## Final Outcome

The sequence is complete when all five implementation plans exist, each has a
clean review cycle, and the final plan set gives root a clear map of:

- which style text/font values text consumes
- which values root must reject initially
- which text APIs expose layout-ready facts
- which render/layout/root coordination items remain outside this crate
