# Style Text Support Directive

Date: 2026-07-07

## Purpose

Prepare `surgeist-text` to review and plan only the text-owned work implied by
the root CSS/style integration inventory.

This is a directive for a future crate-local implementation plan. It is not an
implementation plan and does not authorize broad CSS, style, layout, render, or
root adapter work inside `surgeist-text`.

## Source Inventory

Primary root inventory:

```text
/Users/codex/Development/surgeist/plans/2026-07-04-css-integration-support-inventory.md
```

Relevant style handoff package:

```text
/Users/codex/Development/surgeist-style/plans/2026-07-07-style-root-handoff-notes.md
/Users/codex/Development/surgeist-style/plans/2026-07-07-style-css-api-artifact.md
/Users/codex/Development/surgeist-style/plans/2026-07-05-css-property-coverage-ledger.md
```

## Boundary

`surgeist-text` consumes text-ready style data. It must not consume raw
`surgeist-css` syntax or own CSS-to-style lowering.

Root owns adapters and integration. Style owns resolved style values, authored
cascade, variables, custom properties, selectors, and style diagnostics. Layout
owns layout-ready inline metric contracts and layout algorithms. Render owns
paint realization.

Text should expose text-domain contracts that root can use without forcing text
to depend on layout, CSS, retained, or root.

## Required Text-Owned Review Items

The text coordinator should create a plan that reviews support for these
style-facing inputs only:

- font family lists and symbolic font family policy
- font weight
- font style and oblique slant
- font stretch
- font variant
- font feature settings
- font size as text-facing input
- line height as text-facing input
- letter spacing
- whitespace handling
- text wrapping
- word breaking
- overflow wrapping
- text overflow
- text alignment and final-line alignment where text owns behavior
- text indentation where text owns line shaping behavior
- vertical-align only where it affects inline text metrics or inline box
  alignment data exposed to root/layout
- text decoration line, style, thickness, and color as text shaping or
  projection inputs
- text transform policy
- selection color only as text selection projection data
- deriving layout-ready inline metric facts from text/font policy without
  depending on `surgeist-layout`

## Explicitly Out Of Scope

Do not plan or implement:

- CSS parsing
- CSS-to-style lowering
- root adapters
- layout algorithms
- layout input contracts owned by `surgeist-layout`
- render paint realization for backgrounds, borders, shadows, masks, filters,
  transforms, or final color spaces
- font file loading, font discovery, or host font fallback unless the plan
  explicitly keeps those behind text-owned abstractions and reports root
  resource-policy needs
- animation scheduling or keyframe sampling
- pseudo-element materialization
- generated content tree materialization

## Questions The Plan Must Answer

- Which current `surgeist-text` APIs already consume the relevant style-facing
  concepts?
- Which style values map directly to existing text types?
- Which style values need new text-domain types or adapters?
- Which values should root reject initially because text cannot consume them
  yet?
- What text-owned metric facts can be produced for layout without taking a
  dependency on `surgeist-layout`?
- What font policy is required before text can derive reliable metrics?
- Which tests should prove text can consume root/style-normalized inputs?

## Deliverable

Write a crate-local implementation plan in:

```text
/Users/codex/Development/surgeist-text/plans/
```

The plan should be task-scoped for the normal `AGENTS.md` worker/reviewer
workflow. Completion for the planning goal is a clean reviewer cycle on that
plan, not code implementation.

## Coordination Output

The text plan should report required upstream/downstream coordination as notes
only:

- root adapter requirements
- style value gaps, if any
- layout inline-metric contract requirements
- render requirements for text decoration or selection projection
- test harness requirements
