# Inline Alignment And Layout-Ready Metric Facts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expose text-owned vertical alignment input and layout-ready inline metric facts that root/layout can consume without making `surgeist-text` depend on `surgeist-layout`.

**Architecture:** Keep authored inline box data in `src/source.rs`, keep style validation in `src/style.rs`, and project backend-derived layout facts from `src/layout.rs`. Parley 0.9 only accepts inline box kind, id, byte index, width, and height, so this plan stores requested alignment as text-owned metadata, hashes it with the source key, and reports both requested and actual backend placement as public facts.

**Tech Stack:** Rust 2024, Parley 0.9, crate-local typed models, existing `System`/`Layout` APIs, Cargo tests and clippy.

---

## Context

This is Plan 5 from `plans/2026-07-07-style-text-support-sequence.md`.

The previous plans already established:

- direct style intake and unsupported-feature reporting
- font weight, width, variant, and line-height policy
- text flow fields and explicit staged unsupported values
- decoration values and selection paint projection

This plan should not add CSS parsing, style crate dependencies, root adapters, `surgeist-layout` dependencies, render changes, or layout algorithms owned by another crate.

Backwards compatibility shims are not required at this phase in development.

## Current Backend Boundary

Parley 0.9 accepts inline boxes as:

```rust
parley::InlineBox {
    id: u64,
    kind: parley::InlineBoxKind,
    index: usize,
    width: f32,
    height: f32,
}
```

Parley positions inline boxes at `line_baseline - height`. It does not expose a text API for CSS `vertical-align` variants or arbitrary baseline shifts in the input. `surgeist-text` must therefore model vertical alignment honestly as text-owned requested metadata and projection facts. It must not claim that all CSS `vertical-align` behavior is implemented by the backend.

## Files

- Modify: `src/source.rs`
  - Add `VerticalAlign` and `BaselineShift` typed models.
  - Store `vertical_align` on `InlineBox`.
  - Validate finite baseline shifts during `ValidatedSource` construction.

- Modify: `src/layout.rs`
  - Extend `PositionedInlineBox` with requested alignment and line index.
  - Extend `Run` with its text range so metric projection uses backend run
    ranges instead of inferring from a glyph.
  - Add public metric projection structs: `InlineMetricFacts`, `LineMetricFact`, `RunMetricFact`, `InlineBoxMetricFact`, `BaselineShiftFact`.
  - Add `Layout::inline_metric_facts()`.

- Modify: `src/cache.rs`
  - Hash inline box `vertical_align` and baseline shift values.

- Modify: `src/lib.rs`
  - Export the new front-door types.

- Modify: `src/style_support.rs`
  - Add `TextStyleFeature::InlineBoxVerticalAlign` for the supported
    text-owned subset represented by `VerticalAlign`.
  - Keep broad CSS `TextStyleFeature::VerticalAlign` unsupported because
    table-cell alignment, percentage shifts, font-relative shifts, SVG baseline
    behavior, and parent-context values remain outside this crate.
  - Add `UnsupportedTextStyleReason::RequiresBroadVerticalAlignPolicy` for
    broad CSS `vertical-align`.

- Modify: `src/tests.rs`
  - Add focused tests for vertical-align validation, cache identity, positioned inline box projection, and layout-ready metric facts.

## Public Model

Use these exact public type names.

In `src/source.rs`:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VerticalAlign {
    Baseline,
    TextTop,
    TextBottom,
    Middle,
    Sub,
    Super,
    Shift(BaselineShift),
}

impl VerticalAlign {
    #[must_use]
    pub const fn baseline() -> Self {
        Self::Baseline
    }

    pub fn try_shift(value: f32) -> Result<Self> {
        Ok(Self::Shift(BaselineShift::try_new(value)?))
    }
}

impl Default for VerticalAlign {
    fn default() -> Self {
        Self::Baseline
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BaselineShift(f32);

impl BaselineShift {
    pub fn try_new(value: f32) -> Result<Self> {
        validate_baseline_shift(value)?;
        Ok(Self(value))
    }

    #[must_use]
    pub const fn get(self) -> f32 {
        self.0
    }
}
```

Extend `InlineBox`:

```rust
pub struct InlineBox {
    pub(crate) id: Id,
    pub(crate) kind: InlineBoxKind,
    pub(crate) index: usize,
    pub(crate) size: Size,
    pub(crate) vertical_align: VerticalAlign,
}

impl InlineBox {
    #[must_use]
    pub const fn new(id: Id, kind: InlineBoxKind, index: usize, size: Size) -> Self {
        Self {
            id,
            kind,
            index,
            size,
            vertical_align: VerticalAlign::Baseline,
        }
    }

    #[must_use]
    pub const fn with_vertical_align(mut self, vertical_align: VerticalAlign) -> Self {
        self.vertical_align = vertical_align;
        self
    }

    #[must_use]
    pub const fn vertical_align(self) -> VerticalAlign {
        self.vertical_align
    }
}
```

In `src/layout.rs`, extend `PositionedInlineBox`:

```rust
pub struct PositionedInlineBox {
    id: Id,
    kind: InlineBoxKind,
    rect: Rect,
    index: usize,
    vertical_align: VerticalAlign,
    line: usize,
}
```

Add these layout-facing facts:

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct InlineMetricFacts {
    metrics: Metrics,
    lines: Vec<LineMetricFact>,
    runs: Vec<RunMetricFact>,
    inline_boxes: Vec<InlineBoxMetricFact>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineMetricFact {
    line: Line,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RunMetricFact {
    line: usize,
    range: Range,
    font_size: f32,
    baseline: f32,
    offset: f32,
    advance: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InlineBoxMetricFact {
    id: Id,
    kind: InlineBoxKind,
    index: usize,
    rect: Rect,
    line: usize,
    vertical_align: VerticalAlign,
    baseline_shift: BaselineShiftFact,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BaselineShiftFact {
    BackendBottomOnBaseline,
    Requested(BaselineShift),
}
```

`BaselineShiftFact::BackendBottomOnBaseline` means Parley positioned the inline box with its bottom edge on the line baseline. `BaselineShiftFact::Requested` carries a text-owned request that root/layout can use for a later placement pass; it must not imply Parley already applied that shift.

## Task 1: Model Vertical Alignment On Inline Boxes

**Files:**
- Modify: `src/source.rs`
- Modify: `src/lib.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Add failing tests for the new public input model**

Add this test to `src/tests.rs`:

```rust
#[test]
fn inline_box_vertical_align_is_public_and_validated() {
    let align = VerticalAlign::try_shift(3.5).expect("finite shift is valid");
    let box_ = InlineBox::new(
        Id::from_u64(42),
        InlineBoxKind::InFlow,
        1,
        Size::new(8.0, 10.0),
    )
    .with_vertical_align(align);

    assert_eq!(box_.vertical_align(), align);

    let mut system = System::default();
    let mut builder = system.builder("ab");
    builder.inline_box(box_);

    builder.build().expect("finite aligned inline box is valid");
}

#[test]
fn inline_box_baseline_shift_rejects_non_finite_values() {
    let error = VerticalAlign::try_shift(f32::INFINITY).expect_err("infinite shift is invalid");

    assert_eq!(error.code, ErrorCode::InvalidStyle);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::InvalidNumericField {
            field: "baseline shift",
            value: f32::INFINITY,
            requirement: NumericRequirement::Finite,
        })
    );
}

#[test]
fn validated_source_rejects_duplicate_inline_box_ids() {
    let mut source = Source::new("ab");
    source.inline_box(InlineBox::new(
        Id::from_u64(4),
        InlineBoxKind::InFlow,
        0,
        Size::new(4.0, 4.0),
    ));
    source.inline_box(InlineBox::new(
        Id::from_u64(4),
        InlineBoxKind::InFlow,
        1,
        Size::new(5.0, 5.0),
    ));

    let error = ValidatedSource::try_from(source).expect_err("duplicate IDs are ambiguous");

    assert_eq!(error.code, ErrorCode::InvalidStyle);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::UnsupportedCombination {
            feature: "inline box id",
            reason: "inline box ids must be unique within a source",
        })
    );
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```sh
cargo test -p surgeist-text inline_box_vertical_align_is_public_and_validated
cargo test -p surgeist-text inline_box_baseline_shift_rejects_non_finite_values
cargo test -p surgeist-text validated_source_rejects_duplicate_inline_box_ids
```

Expected: the first two tests fail to compile because `VerticalAlign` and `BaselineShift` are not defined or exported yet; the duplicate-ID test fails until validation is added.

- [ ] **Step 3: Implement vertical-align input types**

In `src/source.rs`, add `VerticalAlign` and `BaselineShift` near `InlineBox`.

Add:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VerticalAlign {
    Baseline,
    TextTop,
    TextBottom,
    Middle,
    Sub,
    Super,
    Shift(BaselineShift),
}

impl VerticalAlign {
    #[must_use]
    pub const fn baseline() -> Self {
        Self::Baseline
    }

    pub fn try_shift(value: f32) -> Result<Self> {
        Ok(Self::Shift(BaselineShift::try_new(value)?))
    }
}

impl Default for VerticalAlign {
    fn default() -> Self {
        Self::Baseline
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BaselineShift(f32);

impl BaselineShift {
    pub fn try_new(value: f32) -> Result<Self> {
        validate_baseline_shift(value)?;
        Ok(Self(value))
    }

    #[must_use]
    pub const fn get(self) -> f32 {
        self.0
    }
}

fn validate_baseline_shift(value: f32) -> Result<()> {
    if !value.is_finite() {
        return Err(
            Error::new(ErrorCode::InvalidStyle, "baseline shift must be finite").with_detail(
                ErrorDetail::InvalidNumericField {
                    field: "baseline shift",
                    value,
                    requirement: NumericRequirement::Finite,
                },
            ),
        );
    }
    Ok(())
}
```

Extend `InlineBox` as shown in the Public Model section. Keep fields private to preserve construction invariants.

- [ ] **Step 4: Validate source inline box alignment**

In `validate_source`, after `validate_inline_box_size(box_.size)?;`, add:

```rust
validate_inline_box_vertical_align(box_.vertical_align)?;
```

Before iterating inline boxes, add:

```rust
let mut inline_box_ids = std::collections::HashSet::new();
```

Inside the inline box loop, after the vertical-align validation, add:

```rust
if !inline_box_ids.insert(box_.id) {
    return Err(Error::new(
        ErrorCode::InvalidStyle,
        "inline box ids must be unique within a source",
    )
    .with_detail(ErrorDetail::UnsupportedCombination {
        feature: "inline box id",
        reason: "inline box ids must be unique within a source",
    }));
}
```

Add this helper:

```rust
fn validate_inline_box_vertical_align(vertical_align: VerticalAlign) -> Result<()> {
    if let VerticalAlign::Shift(shift) = vertical_align {
        validate_baseline_shift(shift.get())?;
    }
    Ok(())
}
```

- [ ] **Step 5: Export public input types**

In `src/lib.rs`, extend the source model export:

```rust
pub use source_model::{
    BaselineShift, InlineBox, InlineBoxKind, Source, SourceIdentity, SourceRevision, Span,
    ValidatedSource, VerticalAlign,
};
```

- [ ] **Step 6: Run focused tests**

Run:

```sh
cargo test -p surgeist-text inline_box_vertical_align_is_public_and_validated
cargo test -p surgeist-text inline_box_baseline_shift_rejects_non_finite_values
cargo test -p surgeist-text validated_source_rejects_duplicate_inline_box_ids
cargo fmt --check
git diff --check
```

Expected: tests pass, formatting passes, and diff check is clean.

- [ ] **Step 7: Review and commit Task 1**

Coordinator must assign a separate reviewer before commit.

After reviewer is clean, rerun the focused checks:

```sh
cargo test -p surgeist-text inline_box_vertical_align_is_public_and_validated
cargo test -p surgeist-text inline_box_baseline_shift_rejects_non_finite_values
cargo test -p surgeist-text validated_source_rejects_duplicate_inline_box_ids
cargo fmt --check
git diff --check
```

Expected: tests pass, formatting passes, and diff check is clean.

Then commit:

```sh
git add src/source.rs src/lib.rs src/tests.rs
git commit -m "refactor: model inline vertical alignment"
```

## Task 2: Project Positioned Inline Box Alignment And Hash It

**Files:**
- Modify: `src/layout.rs`
- Modify: `src/cache.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Add failing tests for projection and cache identity**

Add these tests to `src/tests.rs`:

```rust
#[test]
fn positioned_inline_boxes_preserve_requested_vertical_align_and_line() {
    let mut system = System::default();
    let mut builder = system.builder("a\nb");
    builder.inline_box(
        InlineBox::new(
            Id::from_u64(7),
            InlineBoxKind::InFlow,
            2,
            Size::new(6.0, 9.0),
        )
        .with_vertical_align(VerticalAlign::Super),
    );

    let layout = builder.build().expect("inline box layout should build");
    let boxes = layout.inline_boxes();

    assert_eq!(boxes.len(), 1);
    assert_eq!(boxes[0].id(), Id::from_u64(7));
    assert_eq!(boxes[0].line(), 1);
    assert_eq!(boxes[0].vertical_align(), VerticalAlign::Super);
}

#[test]
fn inline_box_vertical_align_changes_source_cache_key() {
    let mut baseline = Source::new("ab");
    baseline.inline_box(InlineBox::new(
        Id::from_u64(9),
        InlineBoxKind::InFlow,
        1,
        Size::new(4.0, 5.0),
    ));

    let mut shifted = Source::new("ab");
    shifted.inline_box(
        InlineBox::new(
            Id::from_u64(9),
            InlineBoxKind::InFlow,
            1,
            Size::new(4.0, 5.0),
        )
        .with_vertical_align(VerticalAlign::try_shift(2.0).expect("finite shift")),
    );

    let options = ValidatedOptions::try_from(Options::default()).expect("valid options");
    let style = ValidatedStyle::try_from(Style::default()).expect("valid style");
    let baseline_source = ValidatedSource::try_from(baseline).expect("valid source");
    let shifted_source = ValidatedSource::try_from(shifted).expect("valid source");

    let baseline_key = Key::from_validated(
        &baseline_source,
        &style,
        options,
        FontGeneration::initial(),
    );
    let shifted_key = Key::from_validated(
        &shifted_source,
        &style,
        options,
        FontGeneration::initial(),
    );

    assert_ne!(baseline_key.source(), shifted_key.source());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```sh
cargo test -p surgeist-text positioned_inline_boxes_preserve_requested_vertical_align_and_line
cargo test -p surgeist-text inline_box_vertical_align_changes_source_cache_key
```

Expected: first test fails because `PositionedInlineBox::line()` is missing and projection does not carry alignment; second test fails because cache hashing ignores `vertical_align`.

- [ ] **Step 3: Extend positioned inline box projection**

In `src/layout.rs`, import `VerticalAlign`.

Change `PositionedInlineBox` to include:

```rust
vertical_align: VerticalAlign,
line: usize,
```

Update `PositionedInlineBox::new` to accept both fields, and add accessors:

```rust
#[must_use]
pub const fn vertical_align(self) -> VerticalAlign {
    self.vertical_align
}

#[must_use]
pub const fn line(self) -> usize {
    self.line
}
```

In `Layout::inline_boxes`, enumerate lines:

```rust
self.inner
    .lines()
    .enumerate()
    .flat_map(|(line_index, line)| {
        line.items().filter_map(move |item| {
            let PositionedLayoutItem::InlineBox(box_) = item else {
                return None;
            };
            let source_box = self
                .source
                .boxes
                .iter()
                .find(|source_box| source_box.id.as_u64() == box_.id);
            Some(PositionedInlineBox::new(
                Id::from_u64(box_.id),
                match box_.kind {
                    parley::InlineBoxKind::InFlow => InlineBoxKind::InFlow,
                    parley::InlineBoxKind::OutOfFlow
                    | parley::InlineBoxKind::CustomOutOfFlow => InlineBoxKind::OutOfFlow,
                },
                Rect::new(box_.x, box_.y, box_.width, box_.height),
                source_box.map_or(0, |source_box| source_box.index),
                source_box.map_or(VerticalAlign::Baseline, |source_box| {
                    source_box.vertical_align
                }),
                line_index,
            ))
        })
    })
```

When matching the source inline box, use it for both `index` and `vertical_align`. If the source box is not found, the projection must use `index = 0` and `VerticalAlign::Baseline`.

- [ ] **Step 4: Hash vertical-align on inline boxes**

In `src/cache.rs`, extend `hash_inline_box`:

```rust
hash_vertical_align(box_.vertical_align, hasher);
```

Add:

```rust
fn hash_vertical_align<H: Hasher>(vertical_align: VerticalAlign, hasher: &mut H) {
    std::mem::discriminant(&vertical_align).hash(hasher);
    if let VerticalAlign::Shift(value) = vertical_align {
        hash_f32(value.get(), hasher);
    }
}
```

Import `VerticalAlign`.

- [ ] **Step 5: Run focused tests**

Run:

```sh
cargo test -p surgeist-text positioned_inline_boxes_preserve_requested_vertical_align_and_line
cargo test -p surgeist-text inline_box_vertical_align_changes_source_cache_key
cargo test -p surgeist-text inline_box_participates_in_layout
cargo test -p surgeist-text out_of_flow_inline_box_preserves_metrics_and_reports_anchor
cargo fmt --check
git diff --check
```

Expected: tests pass, existing in-flow and out-of-flow inline box projection remains intact, formatting passes, and diff check is clean.

- [ ] **Step 6: Review and commit Task 2**

Coordinator must assign a separate reviewer before commit.

After reviewer is clean, rerun the focused checks:

```sh
cargo test -p surgeist-text positioned_inline_boxes_preserve_requested_vertical_align_and_line
cargo test -p surgeist-text inline_box_vertical_align_changes_source_cache_key
cargo test -p surgeist-text inline_box_participates_in_layout
cargo test -p surgeist-text out_of_flow_inline_box_preserves_metrics_and_reports_anchor
cargo fmt --check
git diff --check
```

Expected: tests pass, existing in-flow and out-of-flow inline box projection remains intact, formatting passes, and diff check is clean.

Then commit:

```sh
git add src/layout.rs src/cache.rs src/tests.rs
git commit -m "refactor: project inline box alignment facts"
```

## Task 3: Expose Layout-Ready Inline Metric Facts

**Files:**
- Modify: `src/layout.rs`
- Modify: `src/lib.rs`
- Modify: `src/style_support.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Add failing tests for metric facts and support matrix**

Add these tests to `src/tests.rs`:

```rust
#[test]
fn inline_metric_facts_report_lines_runs_and_inline_boxes() {
    let mut system = System::default();
    let mut builder = system.builder("hello world");
    builder.inline_box(
        InlineBox::new(
            Id::from_u64(15),
            InlineBoxKind::InFlow,
            5,
            Size::new(12.0, 7.0),
        )
        .with_vertical_align(VerticalAlign::Middle),
    );

    let layout = builder.build().expect("layout should build");
    let facts = layout.inline_metric_facts();

    assert_eq!(facts.metrics(), layout.metrics());
    assert_eq!(facts.lines().len(), layout.lines().len());
    assert_eq!(facts.runs().len(), layout.glyph_runs().len());
    assert_eq!(facts.inline_boxes().len(), 1);
    assert_eq!(facts.inline_boxes()[0].id(), Id::from_u64(15));
    assert_eq!(facts.inline_boxes()[0].vertical_align(), VerticalAlign::Middle);
    assert_eq!(
        facts.inline_boxes()[0].baseline_shift(),
        BaselineShiftFact::BackendBottomOnBaseline
    );
}

#[test]
fn inline_metric_facts_report_requested_baseline_shift() {
    let shift = BaselineShift::try_new(4.0).expect("finite shift");
    let mut system = System::default();
    let mut builder = system.builder("ab");
    builder.inline_box(
        InlineBox::new(
            Id::from_u64(16),
            InlineBoxKind::InFlow,
            1,
            Size::new(8.0, 8.0),
        )
        .with_vertical_align(VerticalAlign::Shift(shift)),
    );

    let layout = builder.build().expect("layout should build");
    let facts = layout.inline_metric_facts();

    assert_eq!(
        facts.inline_boxes()[0].baseline_shift(),
        BaselineShiftFact::Requested(shift)
    );
}

#[test]
fn support_matrix_reports_inline_box_vertical_align_supported() {
    assert_eq!(
        TextStyleFeature::InlineBoxVerticalAlign.support(),
        TextStyleSupport::Supported
    );
    assert_eq!(
        TextStyleFeature::VerticalAlign.support(),
        TextStyleSupport::Unsupported(UnsupportedTextStyleReason::RequiresBroadVerticalAlignPolicy)
    );
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```sh
cargo test -p surgeist-text inline_metric_facts_report_lines_runs_and_inline_boxes
cargo test -p surgeist-text inline_metric_facts_report_requested_baseline_shift
cargo test -p surgeist-text support_matrix_reports_inline_box_vertical_align_supported
```

Expected: metric fact tests fail to compile because the fact types and `Layout::inline_metric_facts()` are missing; support matrix test fails until `InlineBoxVerticalAlign` is added as the supported subset while broad `VerticalAlign` remains unsupported.

- [ ] **Step 3: Preserve run ranges, preserve run lines, and implement metric fact structs**

In `src/layout.rs`, add `range: Range` and `line: usize` to `Run`. In `Layout::glyph_runs()`, enumerate lines before iterating items, pass `line_index` and `fallback_range` into `Run::new`, and add:

```rust
#[must_use]
pub const fn range(&self) -> Range {
    self.range
}

#[must_use]
pub const fn line(&self) -> usize {
    self.line
}
```

Then add the structs and accessors from the Public Model section. Keep fields private. Accessors must return slices for vectors:

```rust
impl InlineMetricFacts {
    #[must_use]
    pub fn new(
        metrics: Metrics,
        lines: Vec<LineMetricFact>,
        runs: Vec<RunMetricFact>,
        inline_boxes: Vec<InlineBoxMetricFact>,
    ) -> Self {
        Self {
            metrics,
            lines,
            runs,
            inline_boxes,
        }
    }

    #[must_use]
    pub const fn metrics(&self) -> Metrics {
        self.metrics
    }

    #[must_use]
    pub fn lines(&self) -> &[LineMetricFact] {
        &self.lines
    }

    #[must_use]
    pub fn runs(&self) -> &[RunMetricFact] {
        &self.runs
    }

    #[must_use]
    pub fn inline_boxes(&self) -> &[InlineBoxMetricFact] {
        &self.inline_boxes
    }
}
```

Add these constructors and accessors for the remaining fact types:

```rust
impl LineMetricFact {
    #[must_use]
    pub const fn new(line: Line) -> Self {
        Self { line }
    }

    #[must_use]
    pub const fn line(self) -> Line {
        self.line
    }
}

impl RunMetricFact {
    #[must_use]
    pub const fn new(
        line: usize,
        range: Range,
        font_size: f32,
        baseline: f32,
        offset: f32,
        advance: f32,
    ) -> Self {
        Self {
            line,
            range,
            font_size,
            baseline,
            offset,
            advance,
        }
    }

    #[must_use]
    pub const fn line(self) -> usize {
        self.line
    }

    #[must_use]
    pub const fn range(self) -> Range {
        self.range
    }

    #[must_use]
    pub const fn font_size(self) -> f32 {
        self.font_size
    }

    #[must_use]
    pub const fn baseline(self) -> f32 {
        self.baseline
    }

    #[must_use]
    pub const fn offset(self) -> f32 {
        self.offset
    }

    #[must_use]
    pub const fn advance(self) -> f32 {
        self.advance
    }
}

impl InlineBoxMetricFact {
    #[must_use]
    pub const fn new(
        id: Id,
        kind: InlineBoxKind,
        index: usize,
        rect: Rect,
        line: usize,
        vertical_align: VerticalAlign,
        baseline_shift: BaselineShiftFact,
    ) -> Self {
        Self {
            id,
            kind,
            index,
            rect,
            line,
            vertical_align,
            baseline_shift,
        }
    }

    #[must_use]
    pub const fn id(self) -> Id {
        self.id
    }

    #[must_use]
    pub const fn kind(self) -> InlineBoxKind {
        self.kind
    }

    #[must_use]
    pub const fn index(self) -> usize {
        self.index
    }

    #[must_use]
    pub const fn rect(self) -> Rect {
        self.rect
    }

    #[must_use]
    pub const fn line(self) -> usize {
        self.line
    }

    #[must_use]
    pub const fn vertical_align(self) -> VerticalAlign {
        self.vertical_align
    }

    #[must_use]
    pub const fn baseline_shift(self) -> BaselineShiftFact {
        self.baseline_shift
    }
}
```

- [ ] **Step 4: Implement `Layout::inline_metric_facts()`**

Add:

```rust
#[must_use]
pub fn inline_metric_facts(&self) -> InlineMetricFacts {
    let lines = self
        .lines()
        .into_iter()
        .map(LineMetricFact::new)
        .collect();
    let runs = self
        .glyph_runs()
        .into_iter()
        .map(|run| {
            RunMetricFact::new(
                run.line(),
                run.range(),
                run.font_size(),
                run.baseline(),
                run.offset(),
                run.advance(),
            )
        })
        .collect();
    let inline_boxes = self
        .inline_boxes()
        .into_iter()
        .map(|box_| {
            InlineBoxMetricFact::new(
                box_.id(),
                box_.kind(),
                box_.index(),
                box_.rect(),
                box_.line(),
                box_.vertical_align(),
                match box_.vertical_align() {
                    VerticalAlign::Shift(shift) => BaselineShiftFact::Requested(shift),
                    _ => BaselineShiftFact::BackendBottomOnBaseline,
                },
            )
        })
        .collect();

    InlineMetricFacts::new(self.metrics(), lines, runs, inline_boxes)
}
```

- [ ] **Step 5: Add the multiline run-line regression test**

Add this test to `src/tests.rs`:

```rust
#[test]
fn inline_metric_facts_report_run_line_indices_at_line_boundaries() {
    let mut system = System::default();
    let layout = system
        .builder("a\nb")
        .build()
        .expect("multiline layout should build");
    let facts = layout.inline_metric_facts();

    assert!(
        facts
            .runs()
            .iter()
            .any(|run| run.line() == 1 && run.range().start <= 2 && 2 < run.range().end),
        "the run containing byte index 2 should be reported on the second line"
    );
}
```

- [ ] **Step 6: Export metric fact types and update support matrix**

In `src/lib.rs`, add these to the layout exports:

```rust
BaselineShiftFact, InlineBoxMetricFact, InlineMetricFacts, LineMetricFact, RunMetricFact,
```

In `src/style_support.rs`, add `TextStyleFeature::InlineBoxVerticalAlign` to `TextStyleFeature::ALL` and return `TextStyleSupport::Supported` for it. Add `UnsupportedTextStyleReason::RequiresBroadVerticalAlignPolicy` and keep `TextStyleFeature::VerticalAlign` mapped to that reason.

In the existing `public_text_style_contract_is_enumerable` test in `src/tests.rs`, keep:

```rust
assert!(unsupported.contains(&TextStyleFeature::VerticalAlign));
```

Add:

```rust
assert!(TextStyleFeature::ALL.contains(&TextStyleFeature::InlineBoxVerticalAlign));
assert!(!unsupported.contains(&TextStyleFeature::InlineBoxVerticalAlign));
assert_eq!(
    TextStyleFeature::InlineBoxVerticalAlign.support(),
    TextStyleSupport::Supported,
    "text exposes vertical-align as requested inline-box alignment facts"
);
```

- [ ] **Step 7: Run focused tests**

Run:

```sh
cargo test -p surgeist-text inline_metric_facts_report_lines_runs_and_inline_boxes
cargo test -p surgeist-text inline_metric_facts_report_requested_baseline_shift
cargo test -p surgeist-text support_matrix_reports_inline_box_vertical_align_supported
cargo test -p surgeist-text inline_metric_facts_report_run_line_indices_at_line_boundaries
cargo test -p surgeist-text public_text_style_contract_is_enumerable
cargo fmt --check
git diff --check
```

Expected: tests pass, public contract enumeration passes, formatting passes, and diff check is clean.

- [ ] **Step 8: Review and commit Task 3**

Coordinator must assign a separate reviewer before commit.

After reviewer is clean, rerun the focused checks:

```sh
cargo test -p surgeist-text inline_metric_facts_report_lines_runs_and_inline_boxes
cargo test -p surgeist-text inline_metric_facts_report_requested_baseline_shift
cargo test -p surgeist-text support_matrix_reports_inline_box_vertical_align_supported
cargo test -p surgeist-text inline_metric_facts_report_run_line_indices_at_line_boundaries
cargo test -p surgeist-text public_text_style_contract_is_enumerable
cargo fmt --check
git diff --check
```

Expected: tests pass, public contract enumeration passes, formatting passes, and diff check is clean.

Then commit:

```sh
git add src/layout.rs src/lib.rs src/style_support.rs src/tests.rs
git commit -m "refactor: expose inline metric facts"
```

## Final Verification

After all task-scoped worker/reviewer cycles are clean, assign a final clean-context holistic reviewer to inspect:

- the full diff against this plan
- `guidance/surgeist-rust-modeling-guide.md`
- `plans/2026-07-07-style-text-support-sequence.md`
- crate boundary rules in `AGENTS.md`
- public exports and unsupported-feature support matrix
- source/cache/layout coherence for inline box vertical alignment
- whether metric facts are honest about backend-applied placement versus requested alignment

Commit any reviewer-required fixes as logical follow-up commits after scoped review.

After the final holistic reviewer is clean, run:

```sh
cargo test -p surgeist-text
cargo test -p surgeist-text --features text-render
cargo clippy -p surgeist-text --all-targets -- -D warnings
cargo clippy -p surgeist-text --all-targets --features text-render -- -D warnings
cargo fmt --check
git diff --check
```

Expected: all commands pass.

If any reviewer-required follow-up commit is made after these commands, rerun the full final verification command set.

## Coordination Notes

- Root can lower style-normalized inline-box alignment values to `VerticalAlign` only for the supported `InlineBoxVerticalAlign` subset: baseline, text-top, text-bottom, middle, sub, super, and finite absolute baseline shifts.
- Root must still reject or defer broad CSS `vertical-align`, percentage baseline shifts, font-relative shifts, table-cell alignment, SVG dominant/alignment-baseline behavior, and values that require parent layout context.
- Layout can consume `InlineMetricFacts` and `InlineBoxMetricFact` without depending on text internals. Layout remains responsible for any later placement algorithms that adjust boxes according to requested vertical alignment.
- Text records requested alignment and exposes Parley's actual positioned rectangle. It does not claim backend placement has applied `TextTop`, `TextBottom`, `Middle`, `Sub`, `Super`, or `Shift`.
- Render has no new requirement in this plan. Render should continue consuming glyph/decorations through existing projection APIs unless a future layout/render plan introduces inline box painting.
