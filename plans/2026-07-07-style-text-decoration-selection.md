# Style Text Decoration Selection Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Represent text-owned decoration and selection paint facts with concrete, typed APIs root can target, while keeping unsupported decoration variants and final paint policy outside `surgeist-text`.

**Architecture:** Tighten the existing underline/strikethrough model around typed decoration offset, thickness, and brush values, then add a concrete selection paint projection API that combines caller-provided paint with existing selection geometry. Keep Parley as the shaping/projection chokepoint, keep `text-render` as an optional adapter for existing decoration fills, and keep overline, non-solid decoration styles, symbolic colors, and automatic selection painting out of this crate.

**Tech Stack:** Rust 2024, Parley 0.9, optional `surgeist-render` through the existing `text-render` feature, current `surgeist-text` style/cache/layout APIs, `guidance/surgeist-rust-modeling-guide.md`.

---

## Scope

This plan implements Plan 4 from:

```text
plans/2026-07-07-style-text-support-sequence.md
```

It covers:

- typed underline and strikethrough decoration values
- decoration offset policy: auto or finite absolute offset
- decoration thickness policy: auto or finite positive absolute thickness
- decoration brush policy: text color or concrete color
- concrete selection paint as text-owned projection data
- support-matrix updates for concrete selection color
- cache-key updates for refactored decoration values
- optional `text-render` adapter checks for existing decoration fills

It does not implement CSS parsing, root adapters, `currentColor` or symbolic
color resolution, overline, non-solid decoration stroke styles, final render
paint policy, automatic selection rendering, layout algorithms, or edits to
sibling crates.

## Current Decoration And Selection Snapshot

Already supported:

- `Style::underline` and `Style::strikethrough` as solid decorations
- decoration offset and thickness projection to Parley
- concrete decoration brush overrides
- fallback to the text run brush when no decoration brush override is provided
- `Layout::decorations()` exposing text-owned decoration rectangles, brush, and
  kind
- optional `Layout::push_render_text()` encoding decoration rectangles as
  render fills under the `text-render` feature
- `Layout::selection()` exposing selection rectangles and line indexes

Current gaps:

- `Decoration` stores offset, thickness, and brush as public loose fields.
- Selection geometry has rectangles but no text-owned paint fact.
- `TextStyleFeature::SelectionColor` remains unsupported even though text can
  safely combine concrete caller-provided paint with selection geometry.
- Overline, decoration stroke style, symbolic color, and final render paint
  policy are not modeled.

## Modeling Direction

Use text-owned normalized values, not CSS syntax values:

- `DecorationOffset` models either auto backend metrics or a finite absolute
  text-space offset.
- `DecorationThickness` models either auto backend metrics or a finite positive
  absolute text-space thickness.
- `DecorationBrush` models either the resolved text brush for the run or a
  concrete color brush. It does not model symbolic colors.
- `SelectionPaint` models a concrete brush that has already been resolved by
  root/style. It is caller-provided at projection time because selection is a
  runtime range, not a stored document style.
- `PaintedSelectionGeometry` and `PaintedSelectionRect` pair existing selection
  rectangles with concrete paint without claiming final render ownership.

This plan follows the modeling guide by keeping invalid numeric states behind
constructors, naming the text-owned phase explicitly, and keeping conversion
boundaries narrow: root resolves CSS/symbolic inputs, text projects geometry and
concrete facts, render realizes final paint.

## Target File Responsibilities

- `src/style.rs`: Replace loose decoration fields with typed
  `DecorationOffset`, `DecorationThickness`, `DecorationBrush`, and add
  `SelectionPaint`.
- `src/system.rs`: Convert typed decoration fields into current Parley
  `StyleProperty` values at the existing projection chokepoint.
- `src/layout.rs`: Read typed decoration style through helper methods, add
  painted selection projection structs and methods, and keep optional
  `text-render` decoration encoding working.
- `src/cache.rs`: Hash typed decoration values.
- `src/style_support.rs`: Mark concrete selection color supported; keep
  overline, decoration style, and symbolic decoration color unsupported.
- `src/lib.rs`: Export the new public decoration and selection projection
  types.
- `src/tests.rs`: Add focused tests for typed decoration construction,
  validation, projection, render encoding, selection paint projection, support
  matrix boundaries, and cache participation.

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
  `refactor: type text decoration values`.
- Group B: Task 2. Commit message:
  `refactor: project selection paint facts`.
- Group C: Task 3 final holistic checks. Commit only if review fixes are
  required after Group B.

## Decoration And Selection Policy Matrix

Supported after this plan:

- underline as a solid decoration
- strikethrough as a solid decoration
- decoration offset auto
- decoration offset finite absolute value
- decoration thickness auto
- decoration thickness finite positive absolute value
- decoration brush inherited from text color
- decoration brush concrete color
- concrete selection color passed as `SelectionPaint`
- selection geometry paired with concrete paint facts

Rejected or coordination-only after this plan:

- overline: `TextStyleFeature::Overline` remains unsupported with
  `UnsupportedTextStyleReason::RequiresDecorationSelectionPolicy`
- non-solid decoration stroke styles: `TextStyleFeature::DecorationStyle`
  remains unsupported with
  `UnsupportedTextStyleReason::RequiresDecorationSelectionPolicy`
- symbolic text or decoration colors:
  `UnsupportedTextStyleReason::RequiresColorResolution`
- automatic selection rendering: root/render must use text selection paint facts
  and decide final draw order

## Task 1: Typed Decoration Values

**Files:**
- Modify: `src/style.rs`
- Modify: `src/system.rs`
- Modify: `src/layout.rs`
- Modify: `src/cache.rs`
- Modify: `src/lib.rs`
- Test: `src/tests.rs`
- Test with feature: `src/tests.rs`

- [ ] **Step 0: Check starting status**

Run:

```sh
git status --short --branch
```

Expected: clean except for previously committed sequence/plan work.

- [ ] **Step 1: Write failing typed decoration tests**

Update existing test code that calls `Decoration::solid(Some(...))` to use the
new builder shape from this task:

```rust
Decoration::solid().with_brush(DecorationBrush::Color(brush))
```

Then add these tests near existing decoration tests in `src/tests.rs`:

```rust
#[test]
fn typed_decoration_values_project_to_layout_runs() {
    let mut system = System::default();
    let brush = Brush::color(0.2, 0.4, 0.6, 1.0);
    let decoration_brush = Brush::color(0.7, 0.2, 0.1, 1.0);
    let style = Style {
        brush,
        underline: Decoration::solid()
            .with_offset(DecorationOffset::try_absolute(2.0).expect("offset is finite"))
            .with_thickness(
                DecorationThickness::try_absolute(1.5).expect("thickness is positive"),
            )
            .with_brush(DecorationBrush::Color(decoration_brush)),
        ..Style::default()
    };
    let mut builder = system.builder("decor");
    builder.default_style(style);

    let layout = builder.build().expect("typed decoration should build");
    let underline = layout
        .decorations()
        .into_iter()
        .find(|decoration| decoration.kind() == DecorationKind::Underline)
        .expect("underline decoration should project");

    assert_eq!(underline.brush(), decoration_brush);
    assert_eq!(underline.rect().size.height, 1.5);
}

#[test]
fn decoration_text_color_brush_uses_resolved_run_brush() {
    let mut system = System::default();
    let brush = Brush::color(0.2, 0.4, 0.6, 1.0);
    let style = Style {
        brush,
        underline: Decoration::solid(),
        ..Style::default()
    };
    let mut builder = system.builder("decor");
    builder.default_style(style);

    let layout = builder.build().expect("text-color decoration should build");
    let underline = layout
        .decorations()
        .into_iter()
        .find(|decoration| decoration.kind() == DecorationKind::Underline)
        .expect("underline decoration should project");

    assert_eq!(underline.brush(), brush);
}

#[test]
fn decoration_metric_values_reject_invalid_numbers() {
    let nan_offset =
        DecorationOffset::try_absolute(f32::NAN).expect_err("nan offset is invalid");
    let zero_thickness =
        DecorationThickness::try_absolute(0.0).expect_err("zero thickness is invalid");

    assert_eq!(nan_offset.code, ErrorCode::InvalidStyle);
    assert_eq!(zero_thickness.code, ErrorCode::InvalidStyle);
}
```

- [ ] **Step 2: Run focused tests and confirm RED**

Run:

```sh
cargo test -p surgeist-text typed_decoration_values_project_to_layout_runs
cargo test -p surgeist-text decoration_text_color_brush_uses_resolved_run_brush
cargo test -p surgeist-text decoration_metric_values_reject_invalid_numbers
```

Expected: FAIL because the typed decoration values and builder methods do not
exist yet.

- [ ] **Step 3: Add typed decoration values**

In `src/style.rs`, replace the loose `Decoration` struct with this typed shape:

```rust
/// Solid text decoration.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Decoration {
    enabled: bool,
    offset: DecorationOffset,
    thickness: DecorationThickness,
    brush: DecorationBrush,
}

impl Decoration {
    #[must_use]
    pub const fn none() -> Self {
        Self {
            enabled: false,
            offset: DecorationOffset::Auto,
            thickness: DecorationThickness::Auto,
            brush: DecorationBrush::TextColor,
        }
    }

    #[must_use]
    pub const fn solid() -> Self {
        Self {
            enabled: true,
            offset: DecorationOffset::Auto,
            thickness: DecorationThickness::Auto,
            brush: DecorationBrush::TextColor,
        }
    }

    #[must_use]
    pub const fn with_offset(mut self, offset: DecorationOffset) -> Self {
        self.offset = offset;
        self
    }

    #[must_use]
    pub const fn with_thickness(mut self, thickness: DecorationThickness) -> Self {
        self.thickness = thickness;
        self
    }

    #[must_use]
    pub const fn with_brush(mut self, brush: DecorationBrush) -> Self {
        self.brush = brush;
        self
    }

    #[must_use]
    pub const fn enabled(self) -> bool {
        self.enabled
    }

    #[must_use]
    pub const fn offset(self) -> DecorationOffset {
        self.offset
    }

    #[must_use]
    pub const fn thickness(self) -> DecorationThickness {
        self.thickness
    }

    #[must_use]
    pub const fn brush(self) -> DecorationBrush {
        self.brush
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DecorationOffset {
    Auto,
    Absolute(DecorationOffsetValue),
}

impl DecorationOffset {
    pub fn try_absolute(value: f32) -> Result<Self> {
        validate_finite_f32(value, "decoration offset")?;
        Ok(Self::Absolute(DecorationOffsetValue(value)))
    }

    #[must_use]
    pub const fn to_parley(self) -> Option<f32> {
        match self {
            Self::Auto => None,
            Self::Absolute(value) => Some(value.get()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DecorationOffsetValue(f32);

impl DecorationOffsetValue {
    #[must_use]
    pub const fn get(self) -> f32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DecorationThickness {
    Auto,
    Absolute(DecorationThicknessValue),
}

impl DecorationThickness {
    pub fn try_absolute(value: f32) -> Result<Self> {
        validate_positive_f32(value, "decoration thickness")?;
        Ok(Self::Absolute(DecorationThicknessValue(value)))
    }

    #[must_use]
    pub const fn to_parley(self) -> Option<f32> {
        match self {
            Self::Auto => None,
            Self::Absolute(value) => Some(value.get()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DecorationThicknessValue(f32);

impl DecorationThicknessValue {
    #[must_use]
    pub const fn get(self) -> f32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DecorationBrush {
    TextColor,
    Color(Brush),
}

impl DecorationBrush {
    #[must_use]
    pub const fn to_parley(self) -> Option<Brush> {
        match self {
            Self::TextColor => None,
            Self::Color(brush) => Some(brush),
        }
    }
}
```

Keep `impl Default for Decoration` returning `Self::none()`.

Update `validate_decoration` in `src/style.rs`:

```rust
fn validate_decoration(decoration: Decoration, name: &str) -> Result<()> {
    if !decoration.enabled() {
        return Ok(());
    }
    match decoration.brush() {
        DecorationBrush::TextColor => {}
        DecorationBrush::Color(brush) => validate_brush(brush, &format!("{name} brush"))?,
    }
    Ok(())
}
```

Add numeric field names:

```rust
"decoration offset" => "decoration offset",
"decoration thickness" => "decoration thickness",
```

The old `"underline offset"`, `"underline size"`, `"strikethrough offset"`,
and `"strikethrough size"` names can remain for unrelated existing diagnostics
until the whole crate stops using them.

- [ ] **Step 4: Update projection, layout, cache, and exports**

In `src/system.rs`, replace direct field access for underline/strikethrough:

```rust
push(StyleProperty::Underline(authored.underline.enabled()));
push(StyleProperty::UnderlineOffset(authored.underline.offset().to_parley()));
push(StyleProperty::UnderlineSize(
    authored.underline.thickness().to_parley(),
));
push(StyleProperty::UnderlineBrush(authored.underline.brush().to_parley()));
push(StyleProperty::Strikethrough(authored.strikethrough.enabled()));
push(StyleProperty::StrikethroughOffset(
    authored.strikethrough.offset().to_parley(),
));
push(StyleProperty::StrikethroughSize(
    authored.strikethrough.thickness().to_parley(),
));
push(StyleProperty::StrikethroughBrush(
    authored.strikethrough.brush().to_parley(),
));
```

In `src/cache.rs`, replace `hash_decoration` with:

```rust
fn hash_decoration<H: Hasher>(decoration: Decoration, hasher: &mut H) {
    decoration.enabled().hash(hasher);
    hash_decoration_offset(decoration.offset(), hasher);
    hash_decoration_thickness(decoration.thickness(), hasher);
    hash_decoration_brush(decoration.brush(), hasher);
}

fn hash_decoration_offset<H: Hasher>(offset: DecorationOffset, hasher: &mut H) {
    std::mem::discriminant(&offset).hash(hasher);
    if let DecorationOffset::Absolute(value) = offset {
        hash_f32(value.get(), hasher);
    }
}

fn hash_decoration_thickness<H: Hasher>(thickness: DecorationThickness, hasher: &mut H) {
    std::mem::discriminant(&thickness).hash(hasher);
    if let DecorationThickness::Absolute(value) = thickness {
        hash_f32(value.get(), hasher);
    }
}

fn hash_decoration_brush<H: Hasher>(brush: DecorationBrush, hasher: &mut H) {
    std::mem::discriminant(&brush).hash(hasher);
    if let DecorationBrush::Color(brush) = brush {
        hash_brush(brush, hasher);
    }
}
```

Import `DecorationBrush`, `DecorationOffset`, and `DecorationThickness` in
`src/cache.rs`.

In `src/layout.rs`, no decoration rectangle algorithm change should be needed
because Parley still returns resolved decoration facts. Existing
`Layout::decorations()` should keep reading Parley's `style.underline` and
`style.strikethrough`.

In `src/lib.rs`, export:

```rust
DecorationBrush, DecorationOffset, DecorationOffsetValue, DecorationThickness,
DecorationThicknessValue,
```

- [ ] **Step 5: Update existing tests and optional render test**

Update existing tests in `src/tests.rs` from:

```rust
Decoration::solid(None)
Decoration::solid(Some(brush))
```

to:

```rust
Decoration::solid()
Decoration::solid().with_brush(DecorationBrush::Color(brush))
```

In the `#[cfg(feature = "text-render")]` test
`render_projection_encodes_decorations`, use:

```rust
underline: Decoration::solid()
    .with_brush(DecorationBrush::Color(Brush::color(1.0, 0.0, 0.0, 1.0))),
```

- [ ] **Step 6: Run Group A checks, review, and commit**

Run:

```sh
cargo test -p surgeist-text typed_decoration_values_project_to_layout_runs
cargo test -p surgeist-text decoration_text_color_brush_uses_resolved_run_brush
cargo test -p surgeist-text decoration_metric_values_reject_invalid_numbers
cargo test -p surgeist-text glyph_runs_preserve_resolved_brush
cargo test -p surgeist-text style_span_changes_cache_key
cargo test -p surgeist-text --features text-render render_projection_encodes_decorations
cargo fmt --check
git diff --check
git status --short --branch
git diff --stat
git diff -- src/style.rs src/system.rs src/layout.rs src/cache.rs src/lib.rs src/tests.rs
```

Expected: focused tests, optional render decoration test, formatting, and diff
check pass. The diff only types decoration values, updates projections/hashing,
exports the new types, and updates tests.

After reviewer approval, commit:

```sh
git add src/style.rs src/system.rs src/layout.rs src/cache.rs src/lib.rs src/tests.rs
git commit -m "refactor: type text decoration values"
```

## Task 2: Selection Paint Projection Facts

**Files:**
- Modify: `src/style.rs`
- Modify: `src/layout.rs`
- Modify: `src/style_support.rs`
- Modify: `src/lib.rs`
- Test: `src/tests.rs`

- [ ] **Step 0: Check starting status**

Run:

```sh
git status --short --branch
```

Expected: clean after Group A commit.

- [ ] **Step 1: Write failing selection paint tests**

Add these tests near existing selection tests in `src/tests.rs`:

```rust
#[test]
fn selection_paint_projects_concrete_brush_with_geometry() {
    let mut system = System::default();
    let mut builder = system.builder("hello world");
    let layout = builder.build().expect("layout should build");
    let paint =
        SelectionPaint::try_color(Brush::color(0.1, 0.2, 0.9, 0.5)).expect("paint is valid");
    let selection = Selection::new(
        Cursor::new(0, Affinity::After),
        Cursor::new(5, Affinity::Before),
    );

    let painted = layout.painted_selection(selection, paint);

    assert_eq!(
        painted.rects().len(),
        layout.selection(selection).rects().len()
    );
    assert!(
        painted
            .rects()
            .iter()
            .all(|rect| rect.paint() == paint && rect.line() < layout.metrics().line_count())
    );
}

#[test]
fn selection_paint_rejects_invalid_brush() {
    let error = SelectionPaint::try_color(Brush::color(f32::NAN, 0.0, 0.0, 1.0))
        .expect_err("invalid selection brush should fail");

    assert_eq!(error.code, ErrorCode::InvalidStyle);
}

#[test]
fn support_matrix_reports_concrete_selection_color_supported() {
    assert_eq!(
        TextStyleFeature::SelectionColor.support(),
        TextStyleSupport::Supported
    );
    assert_eq!(
        TextStyleFeature::Overline.support(),
        TextStyleSupport::Unsupported(
            UnsupportedTextStyleReason::RequiresDecorationSelectionPolicy
        )
    );
    assert_eq!(
        TextStyleFeature::DecorationStyle.support(),
        TextStyleSupport::Unsupported(
            UnsupportedTextStyleReason::RequiresDecorationSelectionPolicy
        )
    );
    assert_eq!(
        TextStyleFeature::SymbolicDecorationColor.support(),
        TextStyleSupport::Unsupported(UnsupportedTextStyleReason::RequiresColorResolution)
    );
}
```

- [ ] **Step 2: Run focused tests and confirm RED**

Run:

```sh
cargo test -p surgeist-text selection_paint_projects_concrete_brush_with_geometry
cargo test -p surgeist-text selection_paint_rejects_invalid_brush
cargo test -p surgeist-text support_matrix_reports_concrete_selection_color_supported
```

Expected: FAIL because `SelectionPaint`, `Layout::painted_selection`, painted
selection projection types, and the support-matrix update do not exist yet.

- [ ] **Step 3: Add `SelectionPaint`**

In `src/style.rs`, add near `Brush`:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelectionPaint {
    brush: Brush,
}

impl SelectionPaint {
    pub fn try_color(brush: Brush) -> Result<Self> {
        validate_brush(brush, "selection brush")?;
        Ok(Self { brush })
    }

    #[must_use]
    pub const fn brush(self) -> Brush {
        self.brush
    }
}
```

Add brush channel field names:

```rust
("selection brush", "red") => "selection brush red channel",
("selection brush", "green") => "selection brush green channel",
("selection brush", "blue") => "selection brush blue channel",
("selection brush", "alpha") => "selection brush alpha channel",
```

- [ ] **Step 4: Add painted selection projection structs**

In `src/layout.rs`, import `SelectionPaint` and add:

```rust
pub fn painted_selection(
    &self,
    selection: Selection,
    paint: SelectionPaint,
) -> PaintedSelectionGeometry {
    PaintedSelectionGeometry::new(
        self.selection(selection)
            .rects()
            .iter()
            .map(|rect| PaintedSelectionRect::new(rect.rect(), rect.line(), paint))
            .collect(),
    )
}
```

Add structs near `SelectionGeometry`:

```rust
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PaintedSelectionGeometry {
    rects: Vec<PaintedSelectionRect>,
}

impl PaintedSelectionGeometry {
    #[must_use]
    pub fn new(rects: Vec<PaintedSelectionRect>) -> Self {
        Self { rects }
    }

    #[must_use]
    pub fn rects(&self) -> &[PaintedSelectionRect] {
        &self.rects
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaintedSelectionRect {
    rect: Rect,
    line: usize,
    paint: SelectionPaint,
}

impl PaintedSelectionRect {
    #[must_use]
    pub const fn new(rect: Rect, line: usize, paint: SelectionPaint) -> Self {
        Self { rect, line, paint }
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
    pub const fn paint(self) -> SelectionPaint {
        self.paint
    }
}
```

Do not make `push_render_text` draw painted selections. Selection rendering
requires caller range, z-order, and paint policy that text does not own.

- [ ] **Step 5: Export and update support matrix**

In `src/lib.rs`, export:

```rust
PaintedSelectionGeometry, PaintedSelectionRect
```

from `layout`, and export `SelectionPaint` from `style`.

In `src/style_support.rs`, move `TextStyleFeature::SelectionColor` into the
supported branch. Keep `Overline` and `DecorationStyle` unsupported with
`RequiresDecorationSelectionPolicy`, and keep `SymbolicDecorationColor`
unsupported with `RequiresColorResolution`.

Update existing tests that expected `SelectionColor` to be unsupported so they
now expect it to be supported. In `public_text_style_contract_is_enumerable`,
replace:

```rust
assert!(unsupported.contains(&TextStyleFeature::SelectionColor));
```

with:

```rust
assert!(!unsupported.contains(&TextStyleFeature::SelectionColor));
assert_eq!(
    TextStyleFeature::SelectionColor.support(),
    TextStyleSupport::Supported,
    "concrete selection color is projected through SelectionPaint"
);
```

- [ ] **Step 6: Run Group B checks, review, and commit**

Run:

```sh
cargo test -p surgeist-text selection_paint_projects_concrete_brush_with_geometry
cargo test -p surgeist-text selection_paint_rejects_invalid_brush
cargo test -p surgeist-text support_matrix_reports_concrete_selection_color_supported
cargo test -p surgeist-text selection_geometry_for_non_empty_range
cargo test -p surgeist-text selection_geometry_for_multi_line_range
cargo fmt --check
git diff --check
git status --short --branch
git diff --stat
git diff -- src/style.rs src/layout.rs src/style_support.rs src/lib.rs src/tests.rs
```

Expected: focused tests, existing selection geometry tests, formatting, and diff
check pass. The diff only adds concrete selection paint projection facts,
exports, support-matrix update, and tests.

After reviewer approval, commit:

```sh
git add src/style.rs src/layout.rs src/style_support.rs src/lib.rs src/tests.rs
git commit -m "refactor: project selection paint facts"
```

## Task 3: Final Holistic Checks

**Files:**
- Review: `src/style.rs`
- Review: `src/system.rs`
- Review: `src/layout.rs`
- Review: `src/cache.rs`
- Review: `src/style_support.rs`
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
cargo test -p surgeist-text --features text-render
cargo clippy -p surgeist-text --all-targets -- -D warnings
cargo clippy -p surgeist-text --all-targets --features text-render -- -D warnings
cargo fmt --check
git diff --check
```

Expected: all pass.

- [ ] **Step 3: Review the final branch diff**

Run:

```sh
# Use the BASE value recorded before Group A.
git diff --stat "$BASE"..HEAD
git diff "$BASE"..HEAD -- src/style.rs src/system.rs src/layout.rs src/cache.rs src/style_support.rs src/lib.rs src/tests.rs
```

Expected:

- decoration offset and thickness invalid numeric states are behind typed
  constructors
- decoration brush distinguishes text color from concrete color
- underline and strikethrough still project through Parley and render adapter
- concrete selection paint projects with selection rectangles
- selection rendering is not automatically inserted into `push_render_text`
- `SelectionColor` support reflects concrete paint projection
- overline, decoration style, symbolic colors, and final render paint policy
  remain unsupported or external
- no root, sibling crate, CSS parsing, layout algorithm, or render crate edit is
  introduced

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

- Root can lower concrete underline and strikethrough decorations into
  `Decoration::solid()` plus typed offset, thickness, and brush values.
- Root should continue rejecting overline and non-solid decoration stroke styles
  until a later text/render coordination plan defines metrics and paint
  realization.
- Root must resolve `currentColor`, system colors, and other symbolic colors
  before constructing text `Brush`, `DecorationBrush::Color`, or
  `SelectionPaint`.
- Root can use `Layout::painted_selection(selection, paint)` to pair concrete
  selection paint with text-owned selection rectangles.
- Render/root still own final selection draw order, clipping, compositing, and
  whether selection paint is encoded into a render scene.

## Reviewer Checklist

Reviewers must check:

- typed decoration offset and thickness constructors reject invalid numeric
  states
- `DecorationBrush::TextColor` and `DecorationBrush::Color` preserve the
  existing text-color fallback and concrete brush override behavior
- cache hashing includes all refactored decoration values
- Parley projection remains in `src/system.rs` and render conversion remains in
  the optional `text-render` adapter
- `SelectionPaint` validates concrete brushes and does not model symbolic
  colors
- painted selection projection is geometry-plus-paint data only, not automatic
  render realization
- support matrix marks only concrete selection color newly supported and keeps
  overline, decoration style, and symbolic colors unsupported
- no sibling crate, root adapter, CSS dependency, layout algorithm, or render
  crate changes are introduced
