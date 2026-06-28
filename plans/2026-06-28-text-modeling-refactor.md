# Text Modeling Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor `surgeist-text` so source positions, layout options, style values, edit commands, cache keys, and projection outputs encode their invariants at crate-owned boundaries instead of relying on late validation of public primitive fields.

**Architecture:** Keep the current front-door modules and public layout behavior, but add typed normalization layers between authored caller data and Parley/render/accessibility projections. Backwards compatibility shims are not required at this phase; each task may remove or reshape public fields, constructors, and helper APIs when doing so makes the crate-owned model cleaner and the generated API artifact reflects the intentional change.

**Tech Stack:** Rust 2024, Parley 0.9, optional `surgeist-render`, optional `accesskit`, root-owned API artifact generator.

---

## Review Findings

This plan is based on `guidance/surgeist-rust-modeling-guide.md` and the current `surgeist-text` source.

1. Public structs expose invalid states through ordinary literals. `Source`, `Span`, `InlineBox`, `Range`, `Brush`, `Style`, `Options`, `Indent`, `Cursor`, `Selection`, layout output structs, `Stats`, and cache key structs all expose fields directly. Several have documented or tested invariants that are enforced later in `Builder::build`, `Composer::try_*`, `Layout::try_apply`, or not at all.
2. Source positions are untyped byte offsets. `Range::new`, `InlineBox::index`, `Cursor::index`, `Edit::Insert.index`, `Line.range`, `Glyph.range`, `Cluster.range`, and cache projection code all use `usize` directly. This lets non-UTF-8-boundary positions cross module boundaries until late validation.
3. Authored and normalized phases are mixed. `Style` and `Options` represent caller-authored requests, layout-ready values, and unsupported Parley combinations in one public shape. `Direction::LeftToRight`, `WhiteSpace::Collapse`, and unsupported indent combinations can be constructed freely and only fail inside layout building.
4. Font settings are stored as raw CSS fragments in `Vec<String>`. Family names, feature settings, variation settings, and locale tags are parsed repeatedly in `system.rs`; valid parsed values are not retained across validation and projection.
5. Cache keys are partly semantic and partly hash bags. `SourceKey`, `StyleKey`, `OptionsKey`, and `Key` expose raw revisions and hashes, with `Key::new` producing a value that omits `options_width`, while `Key::from_parts` secretly adds it for overflow reporting.
6. Edits are interpreted ad hoc when applied. `Edit` mixes insert, replace, and delete commands with raw indices/ranges; range projection and revision advancement live inside `Layout::try_apply` instead of a reusable normalized text edit boundary.
7. Projection output structs are broad public data bags. Glyphs, runs, metrics, lines, selection rectangles, positioned inline boxes, and decorations expose primitive fields with no distinction between text-space units, source byte positions, Parley identifiers, and render-facing data.
8. Errors carry mostly string detail. `ErrorCode` groups invalid style, invalid range, unsupported feature, and layout failure, but the invalid value, boundary, and phase are embedded in prose messages rather than structured diagnostics.

## Target File Responsibilities

- `src/range.rs`: Own source byte positions and source ranges, including validated UTF-8-boundary positions.
- `src/geometry.rs`: Own text-space geometry units and non-negative size constructors.
- `src/style.rs`: Own authored text style API plus crate-local validated style values for Parley projection.
- `src/options.rs`: Own authored layout options plus crate-local validated layout options.
- `src/source.rs`: Own source identity, authored source construction, and validated source snapshots.
- `src/edit.rs`: New module for authored edit commands, validated text edits, edit application, and range/index projection.
- `src/cache.rs`: Own cache key construction only through crate-owned source/style/options/font generation facts.
- `src/system.rs`: Use validated source/style/options/font settings; project to Parley at one narrow boundary.
- `src/layout.rs`: Consume normalized source snapshots and normalized edits; keep layout projection outputs stable while adding semantic constructors/accessors.
- `src/error.rs`: Add structured error detail without removing the current `ErrorCode` front door.
- `src/tests.rs`: Preserve current coverage and add focused tests for new invariant boundaries.
- `api/public-api.txt`: Regenerate only after intentional public API changes. The generator is owned by the root `/Users/codex/Development/surgeist` repo and writes the artifact in that root checkout's `crates/surgeist-text` submodule. For this standalone crate checkout, source changes that require API regeneration must first be committed and made available to the root checkout, then the generated artifact commit must be brought back into this crate repo.

## Task 1: Source Positions And Ranges

**Files:**
- Modify: `src/range.rs`
- Modify: `src/source.rs`
- Modify: `src/composer.rs`
- Modify: `src/system.rs`
- Modify: `src/layout.rs`
- Modify: `src/lib.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Add failing tests for validated source positions**

Add these tests near the existing invalid UTF-8 range tests in `src/tests.rs`:

```rust
#[test]
fn source_position_accepts_only_utf8_boundaries() {
    let text = "é text";

    let position = SourcePosition::try_new(text, 2).expect("boundary should validate");
    let error = SourcePosition::try_new(text, 1).expect_err("middle of scalar should fail");

    assert_eq!(position.get(), 2);
    assert_eq!(error.code, ErrorCode::InvalidRange);
}

#[test]
fn source_range_accepts_only_ordered_utf8_boundaries() {
    let text = "é text";

    let range = SourceRange::try_new(text, 0, 2).expect("valid source range");
    let reversed = SourceRange::try_new(text, 2, 0).expect_err("reversed range should fail");
    let split = SourceRange::try_new(text, 0, 1).expect_err("split scalar should fail");

    assert_eq!(range.start().get(), 0);
    assert_eq!(range.end().get(), 2);
    assert_eq!(reversed.code, ErrorCode::InvalidRange);
    assert_eq!(split.code, ErrorCode::InvalidRange);
}
```

- [ ] **Step 2: Run the focused tests and confirm they fail**

Run:

```sh
cargo test -p surgeist-text source_position_accepts_only_utf8_boundaries
cargo test -p surgeist-text source_range_accepts_only_ordered_utf8_boundaries
```

Expected: FAIL because `SourcePosition` and `SourceRange` do not exist yet.

- [ ] **Step 3: Implement source position and range wrappers**

In `src/range.rs`, add validated wrappers. Keep `Range` only where it remains the explicit authored range type for this task; do not retain duplicate APIs solely for backwards compatibility:

```rust
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourcePosition(usize);

impl SourcePosition {
    #[must_use]
    pub const fn from_unchecked(index: usize) -> Self {
        Self(index)
    }

    pub fn try_new(text: &str, index: usize) -> Result<Self> {
        validate_index(text, index, "source position")?;
        Ok(Self(index))
    }

    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct SourceRange {
    start: SourcePosition,
    end: SourcePosition,
}

impl SourceRange {
    #[must_use]
    pub const fn from_unchecked(start: SourcePosition, end: SourcePosition) -> Self {
        Self { start, end }
    }

    pub fn try_new(text: &str, start: usize, end: usize) -> Result<Self> {
        let range = Range::new(start, end);
        validate(text, range)?;
        Ok(Self {
            start: SourcePosition(start),
            end: SourcePosition(end),
        })
    }

    #[must_use]
    pub const fn start(self) -> SourcePosition {
        self.start
    }

    #[must_use]
    pub const fn end(self) -> SourcePosition {
        self.end
    }

    #[must_use]
    pub const fn range(self) -> Range {
        Range::new(self.start.0, self.end.0)
    }
}
```

Export both types from `src/lib.rs`:

```rust
pub use range::{Range, SourcePosition, SourceRange};
```

- [ ] **Step 4: Route internal validation through the new wrappers**

Update `validate`, `validate_index`, `validate_source`, `Composer::try_span`, `Composer::try_inline_box`, `Layout::try_apply`, and cursor/edit helper code to construct `SourcePosition` or `SourceRange` at validation boundaries. Keep public methods accepting `Range` and `usize` in this task.

- [ ] **Step 5: Run task checks**

Run:

```sh
cargo test -p surgeist-text source_position_accepts_only_utf8_boundaries
cargo test -p surgeist-text source_range_accepts_only_ordered_utf8_boundaries
cargo test -p surgeist-text
cargo fmt --check
```

Expected: all pass.

- [ ] **Step 6: Refresh API artifact after the new public source position exports**

Run:

```sh
# from /Users/codex/Development/surgeist after this crate commit is available there
cargo run --manifest-path /Users/codex/Development/surgeist/api/generator/Cargo.toml -- --crate surgeist-text
```

Expected: `api/public-api.txt` includes `SourcePosition` and `SourceRange`.

- [ ] **Step 7: Commit**

```sh
git add src/range.rs src/source.rs src/composer.rs src/system.rs src/layout.rs src/lib.rs src/tests.rs api/public-api.txt
git commit -m "refactor: model source text positions"
```

## Task 2: Validated Style And Options Boundaries

**Files:**
- Modify: `src/style.rs`
- Modify: `src/options.rs`
- Modify: `src/system.rs`
- Modify: `src/lib.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Add failing tests for normalized style and options**

Add tests near existing style and options validation tests:

```rust
#[test]
fn validated_style_preserves_parsed_font_inputs() {
    let style = Style {
        font: Font::new()
            .family("Arial")
            .feature(r#""liga" on"#)
            .variation(r#""wght" 700"#),
        locale: Some("en-US".to_owned()),
        ..Style::default()
    };

    let validated = ValidatedStyle::try_from(style).expect("style should validate");

    assert_eq!(validated.font_families(), ["Arial".to_owned()]);
    assert_eq!(validated.locale_tag(), Some("en-US"));
    assert_eq!(validated.authored().font.features, [r#""liga" on"#.to_owned()]);
}

#[test]
fn validated_options_reject_unsupported_indent_shape() {
    let options = Options {
        indent: Indent {
            amount: 10.0,
            first_line: false,
            each_line: true,
            hanging: false,
        },
        ..Options::default()
    };

    let error = ValidatedOptions::try_from(options).expect_err("unsupported shape should fail");

    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
}
```

- [ ] **Step 2: Run the focused tests and confirm they fail**

Run:

```sh
cargo test -p surgeist-text validated_style_preserves_parsed_font_inputs
cargo test -p surgeist-text validated_options_reject_unsupported_indent_shape
```

Expected: FAIL because `ValidatedStyle` and `ValidatedOptions` do not exist.

- [ ] **Step 3: Add validated style model**

In `src/style.rs`, add a crate-owned normalized style type:

```rust
#[derive(Clone, Debug)]
pub struct ValidatedStyle {
    authored: Style,
    locale: Option<parley::Language>,
    font_family: Option<Vec<parley::FontFamilyName<'static>>>,
    font_features: Option<String>,
    font_variations: Option<String>,
}

impl ValidatedStyle {
    pub(crate) fn new(
        authored: Style,
        locale: Option<parley::Language>,
        font_family: Option<Vec<parley::FontFamilyName<'static>>>,
        font_features: Option<String>,
        font_variations: Option<String>,
    ) -> Self {
        Self {
            authored,
            locale,
            font_family,
            font_features,
            font_variations,
        }
    }

    #[must_use]
    pub const fn authored(&self) -> &Style {
        &self.authored
    }

    #[must_use]
    pub fn locale_tag(&self) -> Option<&str> {
        self.authored.locale.as_deref()
    }

    #[must_use]
    pub fn font_families(&self) -> &[String] {
        &self.authored.font.family
    }

    #[must_use]
    pub fn font_features(&self) -> Option<&str> {
        self.font_features.as_deref()
    }

    #[must_use]
    pub fn font_variations(&self) -> Option<&str> {
        self.font_variations.as_deref()
    }
}
```

Keep Parley-specific parsed values out of the public API by exposing them only to crate internals:

```rust
impl ValidatedStyle {
    pub(crate) const fn parley_locale(&self) -> Option<parley::Language> {
        self.locale
    }

    pub(crate) fn parley_font_family(&self) -> Option<&[parley::FontFamilyName<'static>]> {
        self.font_family.as_deref()
    }

    pub(crate) fn parley_font_features(&self) -> Option<&str> {
        self.font_features.as_deref()
    }

    pub(crate) fn parley_font_variations(&self) -> Option<&str> {
        self.font_variations.as_deref()
    }
}
```

- [ ] **Step 4: Add validated options model**

In `src/options.rs`, add:

```rust
#[derive(Clone, Copy, Debug)]
pub struct ValidatedOptions {
    authored: Options,
    parley_indent: Option<(f32, parley::IndentOptions)>,
}

impl ValidatedOptions {
    pub(crate) fn new(
        authored: Options,
        parley_indent: Option<(f32, parley::IndentOptions)>,
    ) -> Self {
        Self {
            authored,
            parley_indent,
        }
    }

    #[must_use]
    pub const fn authored(self) -> Options {
        self.authored
    }

    #[must_use]
    pub const fn indent(self) -> Indent {
        self.authored.indent
    }

    pub(crate) fn parley_indent(self) -> Option<(f32, parley::IndentOptions)> {
        self.parley_indent
    }
}
```

- [ ] **Step 5: Move style/options validation into `TryFrom` implementations**

Move `validate_style`, `parse_language`, `parse_font_families`, `validate_font_features`, `validate_font_variations`, `font_settings_source`, `validate_options`, and `parley_indent_options` behind `TryFrom<Style> for ValidatedStyle` and `TryFrom<Options> for ValidatedOptions`. Keep helper functions private to the owning module.

- [ ] **Step 6: Update `System::build` projection**

Update `src/system.rs` so `Builder::build` creates one `ValidatedStyle` for the default style, one per span, and one `ValidatedOptions`. Pass parsed values through the `pub(crate)` Parley projection accessors instead of parsing CSS fragments again. Keep `Layout.default_style` as the authored `Style` only because glyph-run style reporting currently needs the authored style value.

- [ ] **Step 7: Export validated models as intentional front doors**

Export the normalized style and options front doors from `src/lib.rs` so tests and sibling crates can validate authored values without building a layout:

```rust
pub use options::{Alignment, Indent, Options, ValidatedOptions};
pub use style::{
    Brush, Decoration, Direction, Font, LineHeight, OverflowWrap, Slant, Style, ValidatedStyle,
    Weight, WhiteSpace, Width, WordBreak, Wrap,
};
```

- [ ] **Step 8: Run task checks**

Run:

```sh
cargo test -p surgeist-text validated_style_preserves_parsed_font_inputs
cargo test -p surgeist-text validated_options_reject_unsupported_indent_shape
cargo test -p surgeist-text
cargo fmt --check
```

Expected: all pass.

- [ ] **Step 9: Regenerate API artifact**

Run:

```sh
# from /Users/codex/Development/surgeist after this crate commit is available there
cargo run --manifest-path /Users/codex/Development/surgeist/api/generator/Cargo.toml -- --crate surgeist-text
```

Expected: generated artifact matches intentional exports.

- [ ] **Step 10: Commit**

```sh
git add src/style.rs src/options.rs src/system.rs src/lib.rs src/tests.rs api/public-api.txt
git commit -m "refactor: normalize text style inputs"
```

## Task 3: Source Snapshot And Cache Key Ownership

**Files:**
- Modify: `src/source.rs`
- Modify: `src/cache.rs`
- Modify: `src/system.rs`
- Modify: `src/layout.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Add failing tests for source snapshot and cache key constructors**

Add tests near existing cache tests:

```rust
#[test]
fn validated_source_snapshot_rejects_invalid_boxes_before_cache_keying() {
    let mut source = Source::new("é");
    source.inline_box(InlineBox::new(
        Id::from_u64(11),
        InlineBoxKind::InFlow,
        1,
        Size::new(1.0, 1.0),
    ));

    let error = ValidatedSource::try_from(source).expect_err("invalid box anchor should fail");

    assert_eq!(error.code, ErrorCode::InvalidRange);
}

#[test]
fn cache_key_requires_normalized_parts() {
    let source = ValidatedSource::try_from(Source::identified("hello", Id::from_u64(1), 2))
        .expect("source validates");
    let style = ValidatedStyle::try_from(Style::default()).expect("style validates");
    let options = ValidatedOptions::try_from(Options::default()).expect("options validate");

    let key = Key::from_validated(&source, &style, options, FontGeneration::initial());

    assert_eq!(key.source().id(), Some(Id::from_u64(1)));
    assert_eq!(key.source().revision().get(), 2);
}
```

- [ ] **Step 2: Run the focused tests and confirm they fail**

Run:

```sh
cargo test -p surgeist-text validated_source_snapshot_rejects_invalid_boxes_before_cache_keying
cargo test -p surgeist-text cache_key_requires_normalized_parts
```

Expected: FAIL because `ValidatedSource`, `FontGeneration`, and accessor methods do not exist.

- [ ] **Step 3: Add source identity and validated source snapshot**

In `src/source.rs`, add:

```rust
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceRevision(u64);

impl SourceRevision {
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct SourceIdentity {
    id: Option<Id>,
    revision: SourceRevision,
}

impl SourceIdentity {
    #[must_use]
    pub const fn new(id: Option<Id>, revision: SourceRevision) -> Self {
        Self { id, revision }
    }

    #[must_use]
    pub const fn id(self) -> Option<Id> {
        self.id
    }

    #[must_use]
    pub const fn revision(self) -> SourceRevision {
        self.revision
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ValidatedSource {
    source: Source,
}

impl ValidatedSource {
    #[must_use]
    pub const fn source(&self) -> &Source {
        &self.source
    }

    #[must_use]
    pub const fn identity(&self) -> SourceIdentity {
        SourceIdentity::new(self.source.id, SourceRevision::new(self.source.revision))
    }

    pub(crate) fn into_source(self) -> Source {
        self.source
    }
}
```

Implement `TryFrom<Source> for ValidatedSource` using the range validation from Task 1.

- [ ] **Step 4: Add font generation newtype and cache key accessors**

In `src/cache.rs`, add:

```rust
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FontGeneration(u64);

impl FontGeneration {
    #[must_use]
    pub const fn initial() -> Self {
        Self(0)
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}
```

Make `Key::from_validated(&ValidatedSource, &ValidatedStyle, ValidatedOptions, FontGeneration)` the only constructor. Add `source()`, `styles()`, `options()`, and `font_generation()` accessors. Remove `Key::new` unless implementation discovers a current crate-owned semantic use that cannot be represented by `from_validated`; do not keep it as a backwards compatibility shim.

Add `SourceKey` accessors used by the focused test:

```rust
impl SourceKey {
    #[must_use]
    pub const fn id(self) -> Option<Id> {
        self.id
    }

    #[must_use]
    pub const fn revision(self) -> SourceRevision {
        SourceRevision::new(self.revision)
    }

    #[must_use]
    pub const fn hash(self) -> u64 {
        self.hash
    }
}
```

- [ ] **Step 5: Update `System` cache state**

Change `System.font_generation: u64` to `FontGeneration`. Update `refresh_fonts` to call `.next()`. Update `Builder::build` so cache keys are built after source/style/options validation.

- [ ] **Step 6: Export source and cache identity front doors**

Export the new source and cache identity front doors from `src/lib.rs`:

```rust
pub use cache::{FontGeneration, Key, OptionsKey, SourceKey, Stats, StyleKey};
pub use source_model::{
    InlineBox, InlineBoxKind, Source, SourceIdentity, SourceRevision, Span, ValidatedSource,
};
```

- [ ] **Step 7: Run task checks**

Run:

```sh
cargo test -p surgeist-text validated_source_snapshot_rejects_invalid_boxes_before_cache_keying
cargo test -p surgeist-text cache_key_requires_normalized_parts
cargo test -p surgeist-text
cargo fmt --check
```

Expected: all pass.

- [ ] **Step 8: Refresh API artifact after the new public identity exports**

Run:

```sh
# from /Users/codex/Development/surgeist after this crate commit is available there
cargo run --manifest-path /Users/codex/Development/surgeist/api/generator/Cargo.toml -- --crate surgeist-text
```

Expected: cache key and source identity additions are reflected.

- [ ] **Step 9: Commit**

```sh
git add src/source.rs src/cache.rs src/system.rs src/layout.rs src/lib.rs src/tests.rs api/public-api.txt
git commit -m "refactor: own text cache key inputs"
```

## Task 4: Normalized Edits And Projection

**Files:**
- Create: `src/edit.rs`
- Modify: `src/layout.rs`
- Modify: `src/lib.rs`
- Modify: `src/source.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Add failing tests for normalized edits**

Add tests near existing edit tests:

```rust
#[test]
fn text_edit_normalizes_insert_replace_and_delete() {
    let source = Source::identified("hello", Id::from_u64(4), 10);

    let insert = TextEdit::insert(&source, 2, "y").expect("insert validates");
    let replace = TextEdit::replace(&source, Range::new(1, 4), "ipp").expect("replace validates");
    let delete = TextEdit::delete(&source, Range::new(1, 4)).expect("delete validates");

    assert_eq!(insert.range(), Range::new(2, 2));
    assert_eq!(insert.inserted_text(), "y");
    assert_eq!(replace.range(), Range::new(1, 4));
    assert_eq!(replace.inserted_text(), "ipp");
    assert_eq!(delete.range(), Range::new(1, 4));
    assert_eq!(delete.inserted_text(), "");
}

#[test]
fn text_edit_application_advances_source_revision_once() {
    let source = Source::identified("hello", Id::from_u64(4), 10);
    let edit = TextEdit::insert(&source, 2, "y").expect("insert validates");

    let edited = edit.apply_to(source).expect("edit applies to original source");

    assert_eq!(edited.text(), "heyllo");
    assert_eq!(edited.revision(), 11);
}

#[test]
fn text_edit_revalidates_target_source_before_applying() {
    let source = Source::new("hello");
    let edit = TextEdit::replace(&source, Range::new(1, 4), "ipp").expect("replace validates");

    let error = edit
        .apply_to(Source::new("é"))
        .expect_err("edit range should be invalid for different source");

    assert_eq!(error.code, ErrorCode::InvalidRange);
}
```

- [ ] **Step 2: Run the focused tests and confirm they fail**

Run:

```sh
cargo test -p surgeist-text text_edit_normalizes_insert_replace_and_delete
cargo test -p surgeist-text text_edit_application_advances_source_revision_once
cargo test -p surgeist-text text_edit_revalidates_target_source_before_applying
```

Expected: FAIL because `TextEdit` does not exist.

- [ ] **Step 3: Move edit normalization into `src/edit.rs`**

Create `src/edit.rs`:

```rust
use std::ops::Range as StdRange;

use super::{Range, Result, Source, range};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextEdit {
    range: Range,
    inserted_text: String,
}

impl TextEdit {
    pub fn insert(source: &Source, index: usize, text: impl Into<String>) -> Result<Self> {
        Self::replace(source, Range::new(index, index), text)
    }

    pub fn replace(source: &Source, range: Range, text: impl Into<String>) -> Result<Self> {
        range::validate(source.text(), range)?;
        Ok(Self {
            range,
            inserted_text: text.into(),
        })
    }

    pub fn delete(source: &Source, range: Range) -> Result<Self> {
        Self::replace(source, range, "")
    }

    #[must_use]
    pub const fn range(&self) -> Range {
        self.range
    }

    #[must_use]
    pub fn inserted_text(&self) -> &str {
        &self.inserted_text
    }

    pub fn apply_to(&self, mut source: Source) -> Result<Source> {
        range::validate(source.text(), self.range)?;
        project_edit_ranges(&mut source, self.range, self.inserted_text.len());
        source.revision = source.revision.saturating_add(1);
        source
            .text
            .replace_range(StdRange::from(self.range), &self.inserted_text);
        Ok(source)
    }
}
```

Move `project_edit_ranges`, `project_edit_start`, `project_edit_end`, and `project_edit_anchor` from `src/layout.rs` into `src/edit.rs`.

- [ ] **Step 4: Keep `Edit` as the authored edit command API**

Move or re-export the current public `Edit` enum through `src/edit.rs` as the authored command type that normalizes into `TextEdit`. Add:

```rust
impl Edit {
    pub fn normalize(self, source: &Source) -> Result<TextEdit> {
        match self {
            Self::Insert { index, text } => TextEdit::insert(source, index, text),
            Self::Replace { range, text } => TextEdit::replace(source, range, text),
            Self::Delete { range } => TextEdit::delete(source, range),
        }
    }
}
```

Update `Layout::try_apply` to call `edit.normalize(&self.source)?.apply_to(self.source.clone())`.

- [ ] **Step 5: Export edit types**

In `src/lib.rs`, add:

```rust
mod edit;
pub use edit::{Edit, TextEdit};
```

Remove `Edit` from the `layout` export list after moving it.

- [ ] **Step 6: Run task checks**

Run:

```sh
cargo test -p surgeist-text text_edit_normalizes_insert_replace_and_delete
cargo test -p surgeist-text text_edit_application_advances_source_revision_once
cargo test -p surgeist-text text_edit_revalidates_target_source_before_applying
cargo test -p surgeist-text
cargo fmt --check
```

Expected: all pass.

- [ ] **Step 7: Refresh API artifact**

Run:

```sh
# from /Users/codex/Development/surgeist after this crate commit is available there
cargo run --manifest-path /Users/codex/Development/surgeist/api/generator/Cargo.toml -- --crate surgeist-text
```

Expected: `TextEdit` appears and `Edit` remains available from `surgeist_text`.

- [ ] **Step 8: Commit**

```sh
git add src/edit.rs src/layout.rs src/lib.rs src/source.rs src/tests.rs api/public-api.txt
git commit -m "refactor: normalize text edits"
```

## Task 5: Semantic Diagnostics

**Files:**
- Modify: `src/error.rs`
- Modify: `src/lib.rs`
- Modify: `src/range.rs`
- Modify: `src/style.rs`
- Modify: `src/options.rs`
- Modify: `src/source.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Add failing tests for structured error details**

Add near the validation tests:

```rust
#[test]
fn invalid_range_error_names_rejected_boundary() {
    let error = SourceRange::try_new("é", 0, 1).expect_err("split scalar should fail");

    assert_eq!(error.code, ErrorCode::InvalidRange);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::InvalidSourceRange {
            start: 0,
            end: 1,
            text_len: 2,
        })
    );
}

#[test]
fn invalid_style_error_names_rejected_field() {
    let style = Style {
        size: 0.0,
        ..Style::default()
    };

    let error = ValidatedStyle::try_from(style).expect_err("zero size should fail");

    assert_eq!(error.code, ErrorCode::InvalidStyle);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::InvalidNumericField {
            field: "font size",
            value: 0.0,
            requirement: NumericRequirement::FiniteGreaterThanZero,
        })
    );
}
```

- [ ] **Step 2: Run the focused tests and confirm they fail**

Run:

```sh
cargo test -p surgeist-text invalid_range_error_names_rejected_boundary
cargo test -p surgeist-text invalid_style_error_names_rejected_field
```

Expected: FAIL because `ErrorDetail` and `NumericRequirement` do not exist.

- [ ] **Step 3: Add error detail model**

In `src/error.rs`, add:

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum ErrorDetail {
    InvalidSourceRange {
        start: usize,
        end: usize,
        text_len: usize,
    },
    InvalidSourceIndex {
        name: &'static str,
        index: usize,
        text_len: usize,
    },
    InvalidNumericField {
        field: &'static str,
        value: f32,
        requirement: NumericRequirement,
    },
    UnsupportedCombination {
        feature: &'static str,
        reason: &'static str,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NumericRequirement {
    Finite,
    FiniteNonNegative,
    FiniteGreaterThanZero,
    UnitInterval,
}
```

Add `detail: Option<ErrorDetail>` to `Error`, plus:

```rust
pub struct Error {
    pub code: ErrorCode,
    pub message: String,
    detail: Option<ErrorDetail>,
    pub source: Option<Box<dyn error::Error + Send + Sync>>,
}

impl Error {
    #[must_use]
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            detail: None,
            source: None,
        }
    }

    #[must_use]
    pub fn with_detail(mut self, detail: ErrorDetail) -> Self {
        self.detail = Some(detail);
        self
    }

    #[must_use]
    pub const fn detail(&self) -> Option<&ErrorDetail> {
        self.detail.as_ref()
    }
}
```

Export the detail types from `src/lib.rs`:

```rust
pub use error::{Error, ErrorCode, ErrorDetail, NumericRequirement, Result};
```

- [ ] **Step 4: Populate detail at validation chokepoints**

Update range, source, style, and option validation helpers to attach `ErrorDetail` while preserving existing `ErrorCode` and message strings. Use `InvalidSourceRange` for invalid ranges, `InvalidSourceIndex` for invalid indices, `InvalidNumericField` for finite/range checks, and `UnsupportedCombination` for unsupported direction, whitespace collapse, and unsupported indent shapes.

- [ ] **Step 5: Run task checks**

Run:

```sh
cargo test -p surgeist-text invalid_range_error_names_rejected_boundary
cargo test -p surgeist-text invalid_style_error_names_rejected_field
cargo test -p surgeist-text
cargo fmt --check
```

Expected: all pass.

- [ ] **Step 6: Refresh API artifact**

Run:

```sh
# from /Users/codex/Development/surgeist after this crate commit is available there
cargo run --manifest-path /Users/codex/Development/surgeist/api/generator/Cargo.toml -- --crate surgeist-text
```

Expected: structured diagnostics are reflected in `api/public-api.txt`.

- [ ] **Step 7: Commit**

```sh
git add src/error.rs src/lib.rs src/range.rs src/style.rs src/options.rs src/source.rs src/tests.rs api/public-api.txt
git commit -m "refactor: add semantic text diagnostics"
```

## Task 6: Layout Projection Accessors And Public API Audit

**Files:**
- Modify: `src/layout.rs`
- Modify: `src/geometry.rs`
- Modify: `src/cache.rs`
- Modify: `src/source.rs`
- Modify: `src/tests.rs`
- Modify: `api/public-api.txt`

- [ ] **Step 1: Add tests for accessor-based projection use**

Add tests near projection tests:

```rust
#[test]
fn projection_outputs_expose_semantic_accessors() {
    let mut system = System::default();
    let mut builder = system.builder("hello");
    let layout = builder.build().expect("layout should build");

    let metrics = layout.metrics();
    let line = layout.lines().into_iter().next().expect("line exists");
    let run = layout.glyph_runs().into_iter().next().expect("run exists");

    assert_eq!(metrics.line_count(), 1);
    assert_eq!(line.range(), Range::new(0, 5));
    assert!(run.font_size().is_finite());
}

#[test]
fn cache_stats_expose_counters_without_field_access() {
    let mut system = System::default();
    system.builder("hello").build().expect("layout builds");

    let stats = system.stats();

    assert_eq!(stats.layout_misses(), 1);
    assert_eq!(stats.layout_hits(), 0);
}
```

- [ ] **Step 2: Run focused tests and confirm they fail**

Run:

```sh
cargo test -p surgeist-text projection_outputs_expose_semantic_accessors
cargo test -p surgeist-text cache_stats_expose_counters_without_field_access
```

Expected: FAIL because accessors do not exist yet.

- [ ] **Step 3: Add accessors to public data bags**

For `Stats`, `Key`, `SourceKey`, `StyleKey`, `OptionsKey`, `Metrics`, `Line`, `Run`, `Glyph`, `Cluster`, `PositionedInlineBox`, `DecorationRun`, `Cursor`, `Selection`, `SelectionGeometry`, and `SelectionRect`, add accessors for every currently public field. Example:

```rust
impl Metrics {
    #[must_use]
    pub const fn size(self) -> Size {
        self.size
    }

    #[must_use]
    pub const fn line_count(self) -> usize {
        self.line_count
    }
}

impl Run {
    #[must_use]
    pub const fn font_size(&self) -> f32 {
        self.font_size
    }

    #[must_use]
    pub fn glyphs(&self) -> &[Glyph] {
        &self.glyphs
    }
}
```

- [ ] **Step 4: Replace public field construction with constructors and accessors**

Add `new` or `from_parts` constructors for projection types that should remain constructible by callers, and make fields private when accessors/constructors preserve the intended model. Backwards compatibility shims are not required; remove direct public field construction where it exposes invalid states or muddles authored, normalized, and projection phases.

- [ ] **Step 5: Run full API and behavior checks**

Run:

```sh
cargo test -p surgeist-text projection_outputs_expose_semantic_accessors
cargo test -p surgeist-text cache_stats_expose_counters_without_field_access
cargo test -p surgeist-text
cargo clippy -p surgeist-text --all-targets -- -D warnings
cargo fmt --check
# from /Users/codex/Development/surgeist after this crate commit is available there
cargo run --manifest-path /Users/codex/Development/surgeist/api/generator/Cargo.toml -- --crate surgeist-text
```

Expected: all checks pass and `api/public-api.txt` reflects intentional accessor additions.

- [ ] **Step 6: Audit public field cleanup**

Inspect:

```sh
git diff -- api/public-api.txt
rg -n "pub (id|revision|hash|start|end|index|range|size|width|height|rect|glyphs|spans|boxes|text):" src
```

Expected: remaining public fields are either intentionally part of the authored/projection data model or are called out with a concrete reason they cannot be made private in this refactor. Do not defer field cleanup solely for backwards compatibility.

- [ ] **Step 7: Commit**

```sh
git add src/layout.rs src/geometry.rs src/cache.rs src/source.rs src/tests.rs api/public-api.txt
git commit -m "refactor: add semantic projection accessors"
```

## Final Verification And Review

- [ ] **Step 1: Confirm clean working state before final checks**

Run:

```sh
git status --short --branch
```

Expected: on `main`, clean after the task commits.

- [ ] **Step 2: Run crate baseline checks**

Run:

```sh
cargo test -p surgeist-text
cargo clippy -p surgeist-text --all-targets -- -D warnings
cargo fmt --check
```

Expected: all pass.

- [ ] **Step 3: Run feature-gated checks when sibling path dependencies are available**

Run:

```sh
cargo test -p surgeist-text --features text-accessibility
cargo test -p surgeist-text --features text-render
```

Expected: both pass. If either cannot run because the sibling optional dependency is unavailable or platform-specific renderer setup fails, report the exact command, error, and affected API surface.

- [ ] **Step 4: Audit generated public API artifact**

Run:

```sh
# from /Users/codex/Development/surgeist after this crate commit is available there
cargo run --manifest-path /Users/codex/Development/surgeist/api/generator/Cargo.toml -- --crate surgeist-text
git diff -- api/public-api.txt
```

Expected: generated API changes match the intended new front doors and do not expose accidental crate-internal normalized machinery.

- [ ] **Step 5: Holistic reviewer gate**

Assign a separate reviewer to inspect the full result against:

- `guidance/surgeist-rust-modeling-guide.md`
- this plan
- crate boundary rules in `AGENTS.md`
- public API artifact changes
- final test output

The implementation is complete only when the holistic reviewer reports no Critical or Important findings, or all such findings have been fixed and re-reviewed cleanly.

## Coordinator Notes

- This plan intentionally keeps work inside `/Users/codex/Development/surgeist-text`.
- Before implementation begins, commit this plan as a documentation/planning commit or include it in the first task commit so final clean-state checks are meaningful.
- Do not edit sibling crate repositories or root submodule pointers from this crate thread.
- Run one worker/reviewer cycle per task before committing the task.
- Tell every worker they are not alone in the codebase and must not revert others' work.
- Commit task-sized logical points on `main`; do not create a branch unless the user asks.
- Use `guidance/surgeist-rust-modeling-guide.md` for every task that changes models or public APIs.
- Public API breaking changes are acceptable when they align with the modeling guide and are captured by `api/public-api.txt`; do not keep backwards compatibility shims solely to avoid API churn.
