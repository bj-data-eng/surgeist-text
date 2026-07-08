# Style Text Intake Contract Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a crate-owned, CSS-free text style support contract so root can map style-normalized text values into `surgeist-text` or reject unsupported values with typed reasons.

**Architecture:** Introduce a small public `style_support` model that names text-owned style features and their current support status. Wire current unsupported paths through structured `ErrorDetail::UnsupportedTextStyle` values while leaving larger behavior gaps for later plans in the sequence.

**Tech Stack:** Rust 2024, Parley 0.9, current `surgeist-text` front-door APIs, `guidance/surgeist-rust-modeling-guide.md`.

---

## Scope

This plan implements Plan 1 from:

```text
plans/2026-07-07-style-text-support-sequence.md
```

It covers only the style intake contract and rejection matrix. It does not add
new text shaping behavior, CSS parsing, root adapters, style crate dependencies,
font resource loading, render paint realization, layout algorithms, or
backwards compatibility shims.

## Modeling Direction

The Rust model should satisfy these guide points:

- Name ownership and phase: the support model is text-owned normalized intake
  metadata, not authored CSS and not root/style data.
- Keep symbolic values symbolic: text should not flatten `currentColor`,
  system colors, font resources, or layout-dependent units prematurely.
- Prefer semantic errors: unsupported style values should have typed feature and
  reason data rather than prose-only `feature` and `reason` strings.
- Keep conversion boundaries narrow: root remains responsible for mapping
  resolved style values into text-owned `Style`, `Options`, or future intake
  types.
- Public APIs need front doors: export the support contract through `lib.rs`
  rather than expecting root to inspect private modules or error strings.

## Target File Responsibilities

- `src/style_support.rs`: New public support matrix for text-owned style-facing
  features. It contains `TextStyleFeature`, `TextStyleSupport`, and
  `UnsupportedTextStyleReason`.
- `src/error.rs`: Add a structured `ErrorDetail::UnsupportedTextStyle` variant
  that carries the new support contract types.
- `src/style.rs`: Replace current stringly unsupported direction and
  white-space details with typed unsupported text-style details.
- `src/options.rs`: Replace current stringly unsupported indent detail with a
  typed unsupported text-style detail.
- `src/lib.rs`: Export the new support contract types.
- `src/tests.rs`: Add focused tests for the support matrix and typed error
  details.

## Scoped Worker And Commit Groups

Use the local `AGENTS.md` coordinator workflow for each scoped group:

1. Coordinator checks `git status --short --branch` before assigning the group.
2. Worker implements the scoped group and reports tests plus status.
3. Separate reviewer inspects only the scoped group diff.
4. Coordinator reconciles findings.
5. Coordinator runs the focused checks for that group.
6. Coordinator commits the clean scoped group.

Commit groups:

- Group A: Task 1 only. Commit message:
  `refactor: add text style support matrix`.
- Group B: Tasks 2, 3, and 4 together. Task 2 introduces the typed error
  variant and a still-failing test; Task 3 wires current emitters; Task 4
  verifies the front door and cache implications. Commit message:
  `refactor: type unsupported text style errors`.
- Group C: Task 5 final holistic review and checks. Commit only if review fixes
  are required after Group B.

## Support Matrix

The first implementation should publish this initial matrix through
`TextStyleFeature::support()` and `TextStyleFeature::ALL`.

Supported now:

- `FontFamilyList`
- `NamedFontWeight`
- `BasicFontStretch`
- `FontStyle`
- `ObliqueSlant`
- `FontFeatureSettings`
- `FontVariationSettings`
- `FontSize`
- `LineHeight`
- `LetterSpacing`
- `WordSpacing`
- `ConcreteTextColor`
- `Locale`
- `WhiteSpacePreserve`
- `WordBreak`
- `TextWrap`
- `OverflowWrap`
- `TextAlignment`
- `TextIndent`
- `Underline`
- `Strikethrough`
- `DecorationOffset`
- `DecorationThickness`
- `ConcreteDecorationColor`

Unsupported initially:

- `ExplicitTextDirection`: requires Parley public base-direction controls.
- `WhiteSpaceCollapse`: requires source-range-preserving collapse policy.
- `NumericFontWeight`: deferred to the font policy plan.
- `ExpandedFontStretch`: deferred to the font policy plan.
- `FontVariant`: deferred to the font policy plan.
- `TextAlignLast`: deferred to the text flow behavior plan.
- `TextOverflow`: deferred to the text flow behavior plan.
- `TextTransform`: deferred to the text flow behavior plan.
- `Overline`: deferred to the decoration/selection plan.
- `DecorationStyle`: deferred to the decoration/selection plan.
- `SymbolicTextColor`: requires root/render color realization policy.
- `SymbolicDecorationColor`: requires root/render color realization policy.
- `SelectionColor`: deferred to the decoration/selection plan.
- `VerticalAlign`: deferred to the inline metrics plan.

Value shapes rejected under otherwise supported features:

- `TextIndent` with each-line indentation but without first-line indentation:
  current Parley projection cannot express this shape. Errors should report
  `feature: TextStyleFeature::TextIndent` and
  `reason: UnsupportedTextStyleReason::IndentShapeNotExpressibleByCurrentBackend`.

Root may use the public matrix as documentation and as a stable set of feature
names for integration diagnostics. Root still owns the actual style-to-text
lowering and rejection timing.

## Task 1: Add The Public Support Contract

**Files:**
- Create: `src/style_support.rs`
- Modify: `src/lib.rs`
- Test: `src/tests.rs`

- [ ] **Step 0: Check starting status**

Run:

```sh
git status --short --branch
```

Expected: clean apart from any already-approved planning changes the
coordinator explicitly reports.

- [ ] **Step 1: Write failing support matrix tests**

Add these tests near the existing style validation tests in `src/tests.rs`:

```rust
#[test]
fn text_style_support_matrix_reports_current_support() {
    assert_eq!(
        TextStyleFeature::FontFamilyList.support(),
        TextStyleSupport::Supported
    );
    assert_eq!(
        TextStyleFeature::FontSize.support(),
        TextStyleSupport::Supported
    );
    assert_eq!(
        TextStyleFeature::WhiteSpacePreserve.support(),
        TextStyleSupport::Supported
    );
    assert_eq!(
        TextStyleFeature::WhiteSpaceCollapse.support(),
        TextStyleSupport::Unsupported(
            UnsupportedTextStyleReason::RequiresSourceRangePreservation
        )
    );
    assert_eq!(
        TextStyleFeature::TextOverflow.support(),
        TextStyleSupport::Unsupported(
            UnsupportedTextStyleReason::RequiresTextFlowPolicy
        )
    );
    assert!(
        TextStyleFeature::ALL.contains(&TextStyleFeature::SelectionColor),
        "root should be able to enumerate selection color support"
    );
}
```

- [ ] **Step 2: Run the focused test and confirm it fails**

Run:

```sh
cargo test -p surgeist-text text_style_support_matrix_reports_current_support
```

Expected: FAIL because `TextStyleFeature`, `TextStyleSupport`, and
`UnsupportedTextStyleReason` do not exist yet.

- [ ] **Step 3: Add `src/style_support.rs`**

Create `src/style_support.rs` with the support contract. Keep fields private by
using enums and `const fn` methods rather than public mutable data.

```rust
/// Text-owned style-facing feature names for root/style integration.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TextStyleFeature {
    FontFamilyList,
    NamedFontWeight,
    NumericFontWeight,
    BasicFontStretch,
    ExpandedFontStretch,
    FontStyle,
    ObliqueSlant,
    FontVariant,
    FontFeatureSettings,
    FontVariationSettings,
    FontSize,
    LineHeight,
    LetterSpacing,
    WordSpacing,
    ConcreteTextColor,
    SymbolicTextColor,
    Locale,
    ExplicitTextDirection,
    WhiteSpacePreserve,
    WhiteSpaceCollapse,
    WordBreak,
    TextWrap,
    OverflowWrap,
    TextOverflow,
    TextAlignment,
    TextAlignLast,
    TextIndent,
    VerticalAlign,
    Underline,
    Strikethrough,
    Overline,
    DecorationOffset,
    DecorationThickness,
    DecorationStyle,
    ConcreteDecorationColor,
    SymbolicDecorationColor,
    TextTransform,
    SelectionColor,
}

impl TextStyleFeature {
    pub const ALL: &'static [Self] = &[
        Self::FontFamilyList,
        Self::NamedFontWeight,
        Self::NumericFontWeight,
        Self::BasicFontStretch,
        Self::ExpandedFontStretch,
        Self::FontStyle,
        Self::ObliqueSlant,
        Self::FontVariant,
        Self::FontFeatureSettings,
        Self::FontVariationSettings,
        Self::FontSize,
        Self::LineHeight,
        Self::LetterSpacing,
        Self::WordSpacing,
        Self::ConcreteTextColor,
        Self::SymbolicTextColor,
        Self::Locale,
        Self::ExplicitTextDirection,
        Self::WhiteSpacePreserve,
        Self::WhiteSpaceCollapse,
        Self::WordBreak,
        Self::TextWrap,
        Self::OverflowWrap,
        Self::TextOverflow,
        Self::TextAlignment,
        Self::TextAlignLast,
        Self::TextIndent,
        Self::VerticalAlign,
        Self::Underline,
        Self::Strikethrough,
        Self::Overline,
        Self::DecorationOffset,
        Self::DecorationThickness,
        Self::DecorationStyle,
        Self::ConcreteDecorationColor,
        Self::SymbolicDecorationColor,
        Self::TextTransform,
        Self::SelectionColor,
    ];

    #[must_use]
    pub const fn support(self) -> TextStyleSupport {
        match self {
            Self::FontFamilyList
            | Self::NamedFontWeight
            | Self::BasicFontStretch
            | Self::FontStyle
            | Self::ObliqueSlant
            | Self::FontFeatureSettings
            | Self::FontVariationSettings
            | Self::FontSize
            | Self::LineHeight
            | Self::LetterSpacing
            | Self::WordSpacing
            | Self::ConcreteTextColor
            | Self::Locale
            | Self::WhiteSpacePreserve
            | Self::WordBreak
            | Self::TextWrap
            | Self::OverflowWrap
            | Self::TextAlignment
            | Self::TextIndent
            | Self::Underline
            | Self::Strikethrough
            | Self::DecorationOffset
            | Self::DecorationThickness
            | Self::ConcreteDecorationColor => TextStyleSupport::Supported,
            Self::ExplicitTextDirection => TextStyleSupport::Unsupported(
                UnsupportedTextStyleReason::RequiresParleyBaseDirection,
            ),
            Self::WhiteSpaceCollapse => TextStyleSupport::Unsupported(
                UnsupportedTextStyleReason::RequiresSourceRangePreservation,
            ),
            Self::NumericFontWeight
            | Self::ExpandedFontStretch
            | Self::FontVariant => {
                TextStyleSupport::Unsupported(UnsupportedTextStyleReason::RequiresFontPolicy)
            }
            Self::TextAlignLast
            | Self::TextOverflow
            | Self::TextTransform => {
                TextStyleSupport::Unsupported(UnsupportedTextStyleReason::RequiresTextFlowPolicy)
            }
            Self::Overline
            | Self::DecorationStyle
            | Self::SelectionColor => TextStyleSupport::Unsupported(
                UnsupportedTextStyleReason::RequiresDecorationSelectionPolicy,
            ),
            Self::SymbolicTextColor
            | Self::SymbolicDecorationColor => {
                TextStyleSupport::Unsupported(UnsupportedTextStyleReason::RequiresColorResolution)
            }
            Self::VerticalAlign => TextStyleSupport::Unsupported(
                UnsupportedTextStyleReason::RequiresInlineMetricContract,
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TextStyleSupport {
    Supported,
    Unsupported(UnsupportedTextStyleReason),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UnsupportedTextStyleReason {
    RequiresParleyBaseDirection,
    RequiresSourceRangePreservation,
    RequiresFontPolicy,
    RequiresTextFlowPolicy,
    RequiresDecorationSelectionPolicy,
    RequiresColorResolution,
    RequiresInlineMetricContract,
    IndentShapeNotExpressibleByCurrentBackend,
}
```

- [ ] **Step 4: Export the support contract**

In `src/lib.rs`, add the module:

```rust
mod style_support;
```

Then export the public types:

```rust
pub use style_support::{TextStyleFeature, TextStyleSupport, UnsupportedTextStyleReason};
```

- [ ] **Step 5: Run the focused test and confirm it passes**

Run:

```sh
cargo test -p surgeist-text text_style_support_matrix_reports_current_support
```

Expected: PASS.

- [ ] **Step 6: Run Group A checks, review, and commit**

Run:

```sh
cargo test -p surgeist-text text_style_support_matrix_reports_current_support
cargo fmt --check
git diff --check
git status --short --branch
git diff --stat
git diff -- src/style_support.rs src/lib.rs src/tests.rs
```

Expected: focused test, formatting, and diff check pass. The diff contains only
the support matrix module, `lib.rs` export, and support matrix test.

After the separate reviewer approves Group A, commit:

```sh
git add src/style_support.rs src/lib.rs src/tests.rs
git commit -m "refactor: add text style support matrix"
```

## Task 2: Add Typed Unsupported Text Style Errors

**Files:**
- Modify: `src/error.rs`
- Test: `src/tests.rs`

- [ ] **Step 0: Check starting status for Group B**

Run:

```sh
git status --short --branch
```

Expected: clean after the Group A commit.

- [ ] **Step 1: Write failing tests for typed unsupported details**

Add this test near the existing whitespace and indent unsupported tests:

```rust
#[test]
fn unsupported_text_style_errors_are_typed() {
    let style = Style {
        direction: Direction::LeftToRight,
        ..Style::default()
    };

    let error = ValidatedStyle::try_from(style)
        .expect_err("explicit direction should remain unsupported");

    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::UnsupportedTextStyle {
            feature: TextStyleFeature::ExplicitTextDirection,
            reason: UnsupportedTextStyleReason::RequiresParleyBaseDirection,
        })
    );
}
```

- [ ] **Step 2: Run the focused test and confirm it fails**

Run:

```sh
cargo test -p surgeist-text unsupported_text_style_errors_are_typed
```

Expected: FAIL because `ErrorDetail::UnsupportedTextStyle` does not exist yet.

- [ ] **Step 3: Add `UnsupportedTextStyle` to `ErrorDetail`**

In `src/error.rs`, import the support types:

```rust
use super::{TextStyleFeature, UnsupportedTextStyleReason};
```

Add the structured variant:

```rust
UnsupportedTextStyle {
    feature: TextStyleFeature,
    reason: UnsupportedTextStyleReason,
},
```

Keep `UnsupportedCombination` for non-style or transitional unsupported
features until later plans remove any remaining uses. Do not replace unrelated
errors in this task.

- [ ] **Step 4: Run the focused test and confirm the new variant exists**

Run:

```sh
cargo test -p surgeist-text unsupported_text_style_errors_are_typed
```

Expected: still FAIL because `src/style.rs` has not yet emitted the new detail.
Do not commit after this step; Task 3 completes the tightly coupled Group B
behavior.

## Task 3: Wire Current Unsupported Style Paths Through The Contract

**Files:**
- Modify: `src/style.rs`
- Modify: `src/options.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Update unsupported direction detail**

In `src/style.rs`, import the support types if needed:

```rust
use super::{
    Error, ErrorCode, ErrorDetail, NumericRequirement, Result, TextStyleFeature,
    UnsupportedTextStyleReason,
};
```

Replace the explicit direction error detail with:

```rust
.with_detail(ErrorDetail::UnsupportedTextStyle {
    feature: TextStyleFeature::ExplicitTextDirection,
    reason: UnsupportedTextStyleReason::RequiresParleyBaseDirection,
})
```

- [ ] **Step 2: Update unsupported white-space collapse detail**

In `src/style.rs`, replace the white-space collapse detail with:

```rust
.with_detail(ErrorDetail::UnsupportedTextStyle {
    feature: TextStyleFeature::WhiteSpaceCollapse,
    reason: UnsupportedTextStyleReason::RequiresSourceRangePreservation,
})
```

- [ ] **Step 3: Update unsupported indent shape detail**

In `src/options.rs`, import the support types if needed:

```rust
use super::{
    Error, ErrorCode, ErrorDetail, NumericRequirement, Result, TextStyleFeature,
    UnsupportedTextStyleReason,
};
```

Replace the each-line-without-first-line indent detail with:

```rust
.with_detail(ErrorDetail::UnsupportedTextStyle {
    feature: TextStyleFeature::TextIndent,
    reason: UnsupportedTextStyleReason::IndentShapeNotExpressibleByCurrentBackend,
})
```

- [ ] **Step 4: Strengthen existing unsupported tests**

Update `whitespace_collapse_reports_explicit_error` in `src/tests.rs` to assert
the typed detail:

```rust
assert_eq!(
    error.detail(),
    Some(&ErrorDetail::UnsupportedTextStyle {
        feature: TextStyleFeature::WhiteSpaceCollapse,
        reason: UnsupportedTextStyleReason::RequiresSourceRangePreservation,
    })
);
```

Update `rejects_each_line_indent_without_first_line_scope` or
`validated_options_reject_unsupported_indent_shape` to assert:

```rust
assert_eq!(
    error.detail(),
    Some(&ErrorDetail::UnsupportedTextStyle {
        feature: TextStyleFeature::TextIndent,
        reason: UnsupportedTextStyleReason::IndentShapeNotExpressibleByCurrentBackend,
    })
);
```

- [ ] **Step 5: Run focused tests**

Run:

```sh
cargo test -p surgeist-text unsupported_text_style_errors_are_typed
cargo test -p surgeist-text whitespace_collapse_reports_explicit_error
cargo test -p surgeist-text rejects_each_line_indent_without_first_line_scope
cargo test -p surgeist-text validated_options_reject_unsupported_indent_shape
```

Expected: all pass.

## Task 4: Verify The Contract As A Root-Facing Front Door

**Files:**
- Modify: `src/tests.rs`
- Review: `src/lib.rs`

- [ ] **Step 1: Add a front-door export test**

Add this test to `src/tests.rs`:

```rust
#[test]
fn public_text_style_contract_is_enumerable() {
    let unsupported: Vec<_> = TextStyleFeature::ALL
        .iter()
        .copied()
        .filter(|feature| matches!(feature.support(), TextStyleSupport::Unsupported(_)))
        .collect();

    assert!(unsupported.contains(&TextStyleFeature::FontVariant));
    assert!(unsupported.contains(&TextStyleFeature::TextOverflow));
    assert!(unsupported.contains(&TextStyleFeature::VerticalAlign));
    assert!(unsupported.contains(&TextStyleFeature::SelectionColor));
}
```

- [ ] **Step 2: Run the focused test**

Run:

```sh
cargo test -p surgeist-text public_text_style_contract_is_enumerable
```

Expected: PASS.

- [ ] **Step 3: Check the public API boundary manually**

Review `src/lib.rs` and confirm all new public types are exported from the crate
front door and no private module path is required by callers:

```rust
pub use style_support::{TextStyleFeature, TextStyleSupport, UnsupportedTextStyleReason};
```

Confirm no dependency was added to `Cargo.toml`.

- [ ] **Step 4: Record cache-key implications**

Confirm in the worker report that this plan adds no `Style`, `Options`, source,
or inline-box fields. Because the implementation only adds support metadata and
typed error details, `src/cache.rs` does not change and no cache-distinct tests
are required for this plan.

- [ ] **Step 5: Run Group B checks, review, and commit**

Run:

```sh
cargo test -p surgeist-text unsupported_text_style_errors_are_typed
cargo test -p surgeist-text whitespace_collapse_reports_explicit_error
cargo test -p surgeist-text rejects_each_line_indent_without_first_line_scope
cargo test -p surgeist-text validated_options_reject_unsupported_indent_shape
cargo test -p surgeist-text public_text_style_contract_is_enumerable
cargo fmt --check
git diff --check
git status --short --branch
git diff --stat
git diff -- src/error.rs src/style.rs src/options.rs src/tests.rs
```

Expected: all focused tests, formatting, and diff check pass. The diff contains
only typed unsupported text-style error modeling, current unsupported emitter
wiring, and front-door tests.

After the separate reviewer approves Group B, commit:

```sh
git add src/error.rs src/style.rs src/options.rs src/tests.rs
git commit -m "refactor: type unsupported text style errors"
```

## Task 5: Final Holistic Checks

**Files:**
- Review: `src/style_support.rs`
- Review: `src/error.rs`
- Review: `src/style.rs`
- Review: `src/options.rs`
- Review: `src/lib.rs`
- Review: `src/tests.rs`

- [ ] **Step 1: Check status after scoped commits**

Run:

```sh
git status --short --branch
```

Expected: clean after Group A and Group B commits.

- [ ] **Step 2: Run the crate checks**

Run:

```sh
cargo test -p surgeist-text
cargo clippy -p surgeist-text --all-targets -- -D warnings
cargo fmt --check
git diff --check
```

Expected: all pass.

- [ ] **Step 3: Review the final branch diff**

Run:

```sh
git diff --stat HEAD~2..HEAD
git diff HEAD~2..HEAD -- src/style_support.rs src/error.rs src/style.rs src/options.rs src/lib.rs src/tests.rs
```

Expected:

- one new model module
- public exports in `lib.rs`
- typed unsupported-style detail in `error.rs`
- existing unsupported style paths use the typed detail
- tests cover support matrix, current unsupported errors, and front-door exports
- no CSS/style/root/layout dependency additions

- [ ] **Step 4: Run final holistic review**

Assign a separate holistic reviewer to inspect both scoped commits against this
plan, the sequence plan, `AGENTS.md`, and
`guidance/surgeist-rust-modeling-guide.md`.

Expected: no Critical, Important, or Minor findings. If review fixes are
required, apply them as a new scoped fix with worker/reviewer approval, run the
relevant focused checks, and commit the fix before declaring the plan complete.

## Coordination Notes For Later Plans

- Root should use `TextStyleFeature` and `TextStyleSupport` as a diagnostic and
  rejection vocabulary, not as a substitute for root-owned style lowering.
- Later font, flow, decoration/selection, and inline-metric plans should update
  the support matrix as they convert unsupported features to supported features.
- If a later plan introduces more granular values, prefer adding a new semantic
  feature name instead of overloading one broad feature with hidden meanings.
- If a later plan removes the last use of `UnsupportedCombination` for style
  input, that plan may decide whether the older variant still belongs in
  `ErrorDetail` for non-style unsupported cases.

## Reviewer Checklist

The clean-context reviewer must check this plan against
`guidance/surgeist-rust-modeling-guide.md` and verify:

- the new model names the text-owned normalized intake phase
- the support matrix is CSS-free and style-crate-free
- unsupported values use semantic typed errors, not only strings
- invalid future states are not represented as public mutable structs
- the conversion boundary remains root-owned and narrow
- the implementation tasks do not solve later sequence work early
- tests prove the public front door and typed current rejection behavior
