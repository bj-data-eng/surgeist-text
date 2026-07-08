# Style Text Flow Behavior Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make text-flow style intake deterministic for root by explicitly modeling default no-op flow policies, preserving current Parley-backed wrapping behavior, and keeping non-default text transform, overflow, align-last, and whitespace collapse behavior rejected through the support matrix.

**Architecture:** Add small text-owned flow policy enums at the `Style` and `Options` front doors for values that root must name even when only the default value is accepted. Keep direct Parley projection for existing `WordBreak`, `Wrap`, `OverflowWrap`, `Alignment`, and supported indent shapes, and keep source-changing or post-layout behavior out until a later source-map or layout policy exists.

**Tech Stack:** Rust 2024, Parley 0.9, current `surgeist-text` style/options/cache/layout APIs, `guidance/surgeist-rust-modeling-guide.md`.

---

## Scope

This plan implements Plan 3 from:

```text
plans/2026-07-07-style-text-support-sequence.md
```

It covers text-owned flow behavior intake for:

- explicit default `text-transform` modeling
- explicit default `text-overflow` modeling
- explicit default `text-align-last` modeling
- current `word-break`, `text-wrap`, `overflow-wrap`, alignment, and indent
  parity checks
- cache-key participation for newly modeled flow policy fields
- source-range preservation checks for the staged no-op path
- explicit rejection boundaries for non-default flow behavior root must still
  reject before constructing text values

It does not implement CSS parsing, root adapters, generated content,
pseudo-element materialization, whitespace collapse, transformed text storage,
ellipsis insertion, line clamping, clipping paint, final-line alignment
algorithms, or layout-owned APIs.

## Current Flow Support Snapshot

Already supported and projected to Parley:

- `WordBreak::{Normal, BreakAll, KeepAll}`
- `Wrap::{None, Word}`
- `OverflowWrap::{Normal, Anywhere, BreakWord}`
- `Alignment::{Start, End, Left, Right, Center, Justify}`
- `Indent` where current Parley `IndentOptions` can express the shape

Already rejected with typed unsupported detail:

- `WhiteSpace::Collapse`, because collapsing authored whitespace changes shaped
  text and requires a source-range mapping policy before selection, cursor, and
  cluster APIs can stay honest.
- explicit `Direction::{LeftToRight, RightToLeft}`, because current Parley
  public APIs do not expose a base-direction override.
- `Indent` with each-line indentation but without first-line indentation and
  without hanging indentation, because current Parley projection cannot express
  that shape.

Still policy-only after this plan:

- non-default `text-transform` values
- non-default `text-overflow` values
- non-default `text-align-last` values
- whitespace collapse behavior

## Modeling Direction

Use text-owned normalized values, not CSS syntax values:

- `TextTransform` is a text-domain transform policy. This plan only models
  `TextTransform::None`; root must reject uppercase, lowercase, capitalize,
  full-width, full-size-kana, and any future source-changing transform before
  constructing text values.
- `TextOverflow` is a text-domain overflow policy. This plan only models
  `TextOverflow::Clip`, which is the current text behavior. Root/render/layout
  must keep ellipsis and custom marker realization out of this crate until a
  later text/layout/render coordination plan defines projection facts.
- `TextAlignLast` is a text-domain final-line alignment policy. This plan only
  models `TextAlignLast::Auto`, preserving current Parley alignment behavior.
- `WhiteSpace` remains the existing preserve-or-collapse enum. Collapse stays
  rejected because source-changing normalization needs an authored-to-shaped
  source map before cursor, selection, cluster, accessibility, and render
  projections can remain stable.
- Existing `WordBreak`, `Wrap`, `OverflowWrap`, `Alignment`, and supported
  `Indent` values remain direct text-owned model types.

This approach follows the modeling guide by making the accepted default values
constructible in text, keeping invalid source-changing states hard to express
inside text, and keeping conversion boundaries narrow for root/style lowering.

## Target File Responsibilities

- `src/style.rs`: Add normal-only `TextTransform`, add `Style::text_transform`,
  default it to `TextTransform::None`, and expose the authored value through
  `ValidatedStyle`.
- `src/options.rs`: Add normal-only `TextOverflow` and `TextAlignLast`, add
  fields to `Options`, default them to `Clip` and `Auto`, and expose authored
  values through `ValidatedOptions`.
- `src/cache.rs`: Hash the new `Style` and `Options` flow policy fields.
- `src/lib.rs`: Export the new public flow policy types.
- `src/tests.rs`: Add focused tests for default no-op flow policies, support
  matrix boundaries, existing flow behavior parity, cache-key participation,
  and source-range preservation.

No `src/system.rs` projection is expected for the new fields because every new
modeled value is a no-op under current behavior. Existing `WordBreak`, `Wrap`,
and `OverflowWrap` projection remains in `push_style_properties`.

## Scoped Worker And Commit Groups

Use the local `AGENTS.md` coordinator workflow for each group:

1. Coordinator checks `git status --short --branch`.
2. Worker implements the scoped group and reports RED/GREEN tests plus status.
3. Separate reviewer inspects only the scoped group diff.
4. Coordinator reconciles findings.
5. Coordinator runs focused checks.
6. Coordinator commits the clean scoped group.

Commit groups:

Before assigning Group A, record the implementation base and keep that value for
the final holistic diff:

```sh
BASE=$(git rev-parse HEAD)
```

If review fixes add extra commits, do not recompute `BASE`; the final review
must cover every implementation commit made after this recorded point.

- Group A: Task 1. Commit message:
  `refactor: model default text flow policies`.
- Group B: Task 2. Commit message:
  `test: characterize text flow behavior`.
- Group C: Task 3 final holistic checks. Commit only if review fixes are
  required after Group B.

## Flow Policy Matrix

Supported after this plan:

- `WordBreak::Normal`
- `WordBreak::BreakAll`
- `WordBreak::KeepAll`
- `Wrap::None`
- `Wrap::Word`
- `OverflowWrap::Normal`
- `OverflowWrap::Anywhere`
- `OverflowWrap::BreakWord`
- `Alignment::{Start, End, Left, Right, Center, Justify}`
- `Indent` with current expressible Parley shapes
- `WhiteSpace::Preserve`
- `TextTransform::None`
- `TextOverflow::Clip`
- `TextAlignLast::Auto`

Rejected after this plan:

- `WhiteSpace::Collapse`, with
  `TextStyleFeature::WhiteSpaceCollapse` and
  `UnsupportedTextStyleReason::RequiresSourceRangePreservation`
- explicit text direction, with `TextStyleFeature::ExplicitTextDirection` and
  `UnsupportedTextStyleReason::RequiresParleyBaseDirection`
- unsupported indent shapes, with `TextStyleFeature::TextIndent` and
  `UnsupportedTextStyleReason::IndentShapeNotExpressibleByCurrentBackend`
- every non-default transform, overflow marker, and align-last value before text
  construction. Root should report these through the existing support matrix:
  `TextStyleFeature::{TextTransform, TextOverflow, TextAlignLast}` with
  `UnsupportedTextStyleReason::RequiresTextFlowPolicy`.

## Task 1: Default Text Flow Policy Models

**Files:**
- Modify: `src/style.rs`
- Modify: `src/options.rs`
- Modify: `src/cache.rs`
- Modify: `src/lib.rs`
- Test: `src/tests.rs`

- [ ] **Step 0: Check starting status**

Run:

```sh
git status --short --branch
```

Expected: clean except for previously committed sequence/plan work.

- [ ] **Step 1: Write failing default policy tests**

Add these tests near the existing style support and flow validation tests in
`src/tests.rs`:

```rust
#[test]
fn default_flow_policy_values_are_explicit_noops() {
    assert_eq!(Style::default().text_transform, TextTransform::None);
    assert_eq!(Options::default().text_overflow, TextOverflow::Clip);
    assert_eq!(Options::default().text_align_last, TextAlignLast::Auto);

    let mut system = System::default();
    let style = Style {
        text_transform: TextTransform::None,
        ..Style::default()
    };
    let options = Options {
        text_overflow: TextOverflow::Clip,
        text_align_last: TextAlignLast::Auto,
        ..Options::default()
    };
    let mut builder = system.builder("Flow 123");
    builder.default_style(style);
    builder.options(options);

    let layout = builder.build().expect("default flow policies should build");

    assert_eq!(layout.source().text(), "Flow 123");
}

#[test]
fn default_flow_policy_values_preserve_cache_identity() {
    let mut system = System::default();
    let first = system
        .builder("flow")
        .build()
        .expect("default flow policy should build");

    let mut second = system.builder("flow");
    second.default_style(Style {
        text_transform: TextTransform::None,
        ..Style::default()
    });
    second.options(Options {
        text_overflow: TextOverflow::Clip,
        text_align_last: TextAlignLast::Auto,
        ..Options::default()
    });
    let second = second
        .build()
        .expect("explicit default flow policy should build");

    assert_eq!(first.key(), second.key());
}
```

- [ ] **Step 2: Run focused tests and confirm RED**

Run:

```sh
cargo test -p surgeist-text default_flow_policy_values_are_explicit_noops
cargo test -p surgeist-text default_flow_policy_values_preserve_cache_identity
```

Expected: FAIL because `TextTransform`, `TextOverflow`, `TextAlignLast`, and
the new `Style`/`Options` fields do not exist yet.

- [ ] **Step 3: Add `TextTransform` to style input**

In `src/style.rs`, add this enum near the existing flow enums:

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TextTransform {
    None,
}
```

Add the field to `Style` after `overflow_wrap`:

```rust
pub text_transform: TextTransform,
```

Default it in `impl Default for Style`:

```rust
text_transform: TextTransform::None,
```

Add this accessor to `ValidatedStyle`:

```rust
#[must_use]
pub fn text_transform(&self) -> TextTransform {
    self.authored.text_transform
}
```

Do not add transform projection to `src/system.rs`; `TextTransform::None` is a
no-op and source-changing transforms remain root-rejected.

- [ ] **Step 4: Add `TextOverflow` and `TextAlignLast` to layout options**

In `src/options.rs`, add these enums near `Alignment`:

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TextOverflow {
    Clip,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TextAlignLast {
    Auto,
}
```

Add fields to `Options` after `alignment`:

```rust
pub text_overflow: TextOverflow,
pub text_align_last: TextAlignLast,
```

Default them in `impl Default for Options`:

```rust
text_overflow: TextOverflow::Clip,
text_align_last: TextAlignLast::Auto,
```

Add accessors to `ValidatedOptions`:

```rust
#[must_use]
pub const fn text_overflow(self) -> TextOverflow {
    self.authored.text_overflow
}

#[must_use]
pub const fn text_align_last(self) -> TextAlignLast {
    self.authored.text_align_last
}
```

Do not change `system.rs` alignment calls; `TextAlignLast::Auto` preserves
current behavior and `TextOverflow::Clip` has no text-owned projection fact yet.

- [ ] **Step 5: Hash and export the new flow policy fields**

In `src/cache.rs`, update `stable_hash_options`:

```rust
options.alignment.hash(&mut hasher);
options.text_overflow.hash(&mut hasher);
options.text_align_last.hash(&mut hasher);
hash_indent(options.indent, &mut hasher);
```

Update `hash_style`:

```rust
style.overflow_wrap.hash(hasher);
style.text_transform.hash(hasher);
```

In `src/lib.rs`, export the new public types beside the other style and option
types:

```rust
pub use options::{Alignment, Indent, Options, TextAlignLast, TextOverflow, ValidatedOptions};
pub use style::{
    Brush, Decoration, Direction, Font, FontVariant, FontWeightValue, FontWidthRatio, LineHeight,
    OverflowWrap, Slant, Style, TextTransform, ValidatedStyle, Weight, WhiteSpace, Width,
    WordBreak, Wrap,
};
```

- [ ] **Step 6: Run Group A checks, review, and commit**

Run:

```sh
cargo test -p surgeist-text default_flow_policy_values_are_explicit_noops
cargo test -p surgeist-text default_flow_policy_values_preserve_cache_identity
cargo fmt --check
git diff --check
git status --short --branch
git diff --stat
git diff -- src/style.rs src/options.rs src/cache.rs src/lib.rs src/tests.rs
```

Expected: focused tests, formatting, and diff check pass. The diff only adds
normal-only flow policy models, authored accessors, cache hashing, exports, and
tests.

After reviewer approval, commit:

```sh
git add src/style.rs src/options.rs src/cache.rs src/lib.rs src/tests.rs
git commit -m "refactor: model default text flow policies"
```

## Task 2: Flow Behavior Characterization Tests

**Files:**
- Test: `src/tests.rs`

This task adds characterization coverage for behavior that already exists. It
does not need production code unless a characterization exposes a real mismatch
between the support matrix and current behavior.

- [ ] **Step 0: Check starting status**

Run:

```sh
git status --short --branch
```

Expected: clean after Group A commit.

- [ ] **Step 1: Add supported flow parity tests**

Add these tests near the existing wrapping and movement tests in `src/tests.rs`:

```rust
#[test]
fn supported_flow_controls_have_distinct_cache_keys() {
    let mut system = System::default();
    let normal = system
        .builder("alpha beta gamma")
        .build()
        .expect("normal flow should build");

    let mut nowrap_builder = system.builder("alpha beta gamma");
    nowrap_builder.default_style(Style {
        wrap: Wrap::None,
        ..Style::default()
    });
    let nowrap = nowrap_builder
        .build()
        .expect("nowrap flow should build");

    let mut break_all_builder = system.builder("alpha beta gamma");
    break_all_builder.default_style(Style {
        word_break: WordBreak::BreakAll,
        ..Style::default()
    });
    let break_all = break_all_builder
        .build()
        .expect("break-all flow should build");

    let mut anywhere_builder = system.builder("alpha beta gamma");
    anywhere_builder.default_style(Style {
        overflow_wrap: OverflowWrap::Anywhere,
        ..Style::default()
    });
    let anywhere = anywhere_builder
        .build()
        .expect("anywhere flow should build");

    assert_ne!(normal.key(), nowrap.key());
    assert_ne!(normal.key(), break_all.key());
    assert_ne!(normal.key(), anywhere.key());
}

#[test]
fn preserve_whitespace_flow_keeps_source_ranges_stable() {
    let mut system = System::default();
    let mut builder = system.builder("a  b\nc");
    builder.default_style(Style {
        white_space: WhiteSpace::Preserve,
        ..Style::default()
    });

    let layout = builder.build().expect("preserved whitespace should build");
    let clusters = layout.clusters();

    assert_eq!(layout.source().text(), "a  b\nc");
    assert!(
        clusters.iter().any(|cluster| cluster.range() == Range::new(0, 1)),
        "cluster ranges should continue to point at authored source"
    );
    assert!(
        clusters.iter().all(|cluster| cluster.range().end <= layout.source().text().len()),
        "cluster ranges must stay within authored source text"
    );
}

#[test]
fn flow_policy_support_matrix_keeps_behavior_gaps_unsupported() {
    assert_eq!(
        TextStyleFeature::TextOverflow.support(),
        TextStyleSupport::Unsupported(UnsupportedTextStyleReason::RequiresTextFlowPolicy)
    );
    assert_eq!(
        TextStyleFeature::TextAlignLast.support(),
        TextStyleSupport::Unsupported(UnsupportedTextStyleReason::RequiresTextFlowPolicy)
    );
    assert_eq!(
        TextStyleFeature::TextTransform.support(),
        TextStyleSupport::Unsupported(UnsupportedTextStyleReason::RequiresTextFlowPolicy)
    );
    assert_eq!(
        TextStyleFeature::WhiteSpaceCollapse.support(),
        TextStyleSupport::Unsupported(UnsupportedTextStyleReason::RequiresSourceRangePreservation)
    );
}
```

- [ ] **Step 2: Run characterization tests**

Run:

```sh
cargo test -p surgeist-text supported_flow_controls_have_distinct_cache_keys
cargo test -p surgeist-text preserve_whitespace_flow_keeps_source_ranges_stable
cargo test -p surgeist-text flow_policy_support_matrix_keeps_behavior_gaps_unsupported
```

Expected: PASS. If any test fails, stop and report the observed mismatch rather
than broadening implementation scope. These tests characterize already-supported
flow behavior and support-matrix boundaries.

- [ ] **Step 3: Re-run existing flow regression tests**

Run:

```sh
cargo test -p surgeist-text wrap_none_preserves_single_visual_line
cargo test -p surgeist-text overflow_wrap_anywhere_breaks_unspaced_text
cargo test -p surgeist-text whitespace_collapse_reports_explicit_error
cargo test -p surgeist-text span_whitespace_collapse_reports_explicit_error
cargo test -p surgeist-text rejects_each_line_indent_without_first_line_scope
```

Expected: all pass. Existing no-wrap, overflow-wrap, whitespace rejection, and
indent rejection behavior remains unchanged.

- [ ] **Step 4: Run Group B checks, review, and commit**

Run:

```sh
cargo fmt --check
git diff --check
git status --short --branch
git diff --stat
git diff -- src/tests.rs
```

Expected: diff is test-only and documents current supported flow behavior plus
the staged unsupported policy.

After reviewer approval, commit:

```sh
git add src/tests.rs
git commit -m "test: characterize text flow behavior"
```

## Task 3: Final Holistic Checks

**Files:**
- Review: `src/style.rs`
- Review: `src/options.rs`
- Review: `src/system.rs`
- Review: `src/cache.rs`
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
# Use the BASE value recorded before Group A.
git diff --stat "$BASE"..HEAD
git diff "$BASE"..HEAD -- src/style.rs src/options.rs src/system.rs src/cache.rs src/lib.rs src/tests.rs
```

Expected:

- explicit normal-only `TextTransform`, `TextOverflow`, and `TextAlignLast`
  values are modeled as text-owned API types
- new flow policy fields are hashed and exported
- no `system.rs` projection is added for no-op fields
- current Parley-backed flow controls remain direct projections
- `WhiteSpace::Collapse`, non-default transform, non-default overflow, and
  non-default align-last behavior remain root-rejected through the support
  matrix
- no root, sibling crate, CSS parsing, generated content, layout algorithm, or
  render realization work is introduced

- [ ] **Step 4: Run final holistic review**

Assign a separate holistic reviewer to inspect all scoped commits against this
plan, the sequence plan, `AGENTS.md`, and
`guidance/surgeist-rust-modeling-guide.md`.

Expected: no Critical, Important, or Minor findings. If review fixes are
required, apply them as a new scoped fix with worker/reviewer approval, run the
relevant focused checks, and commit the fix. Keep the original pre-Group-A
`BASE` so the final diff continues to cover all scoped implementation commits
before declaring the implementation complete.

## Coordination Notes For Root And Later Plans

- Root can lower default `text-transform: none` into `TextTransform::None`.
- Root can lower default `text-overflow: clip` into `TextOverflow::Clip`.
- Root can lower default `text-align-last: auto` into `TextAlignLast::Auto`.
- Root must reject every non-default text transform, overflow marker, and
  align-last value until a later text/layout/render policy plan defines
  realization.
- Root must continue rejecting whitespace collapse for text until this crate has
  a source-range mapping model that preserves authored source positions across
  transformed or collapsed shaped text.
- Layout/render coordination is still required for ellipsis markers, clipping
  facts, line clamping, and final-line alignment if those features become
  supported later.

## Reviewer Checklist

Reviewers must check:

- new flow policy types are text-owned normalized values, not CSS syntax bags
- default/no-op modeled values cannot express unsupported non-default behavior
- cache hashing includes all newly modeled authored flow policy fields
- `system.rs` projection stays unchanged for no-op fields
- support matrix still reports `TextOverflow`, `TextAlignLast`,
  `TextTransform`, and `WhiteSpaceCollapse` as unsupported policy gaps
- tests prove default no-op construction, source-range preservation under
  preserve behavior, existing Parley-backed flow cache participation, and typed
  rejection boundaries
- no dependency, sibling crate, root adapter, layout algorithm, generated
  content, text transform, ellipsis, or render realization work is introduced
