# Style Text Font Policy Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expand `surgeist-text` font intake so root can pass text-ready numeric font weights and richer font stretch values, while font variant and font resource behavior remain explicitly bounded.

**Architecture:** Keep font handling inside the existing `Font`/`Style` front door, but replace keyword-only font axes with small typed scalar wrappers at the text boundary. Project supported values through Parley at the existing `system.rs` chokepoint, update cache hashing for new scalar values, and keep `@font-face`/host font loading out of this crate.

**Tech Stack:** Rust 2024, Parley 0.9, current `surgeist-text` style validation/cache system, `guidance/surgeist-rust-modeling-guide.md`.

---

## Scope

This plan implements Plan 2 from:

```text
plans/2026-07-07-style-text-support-sequence.md
```

It covers font family policy, numeric font weights, richer font stretch/width
values, font variant intake policy, oblique slant validation, font feature and
variation setting boundaries, and cache-key impact.

It does not add CSS parsing, root adapters, a dependency on `surgeist-style` or
`surgeist-css`, font file loading, host font discovery, `@font-face` handling,
generated content, layout algorithms, or render behavior.

## Modeling Direction

Use text-owned normalized values, not CSS syntax types:

- `FontWeightValue` is a text-domain scalar for Parley font-weight projection.
- `FontWidthRatio` is a text-domain ratio for Parley font-width projection.
- `FontVariant` is modeled as `Normal` only in this crate for now. Root must
  reject every non-normal variant before constructing `surgeist-text` values
  until a later font-variant realization plan defines text-owned variants.
- Font families remain symbolic strings validated through Parley
  `FontFamilyName` parsing. Empty family lists continue to mean the text crate's
  sans-serif fallback.
- Font features and variations remain CSS setting-list strings for now because
  Parley already accepts those strings and validation exists in `style.rs`.

This plan should reduce root coordination by making accepted font values
constructible in `surgeist-text`, while keeping unsupported font categories
visible through the support matrix for root-side rejection.

## Target File Responsibilities

- `src/style.rs`: Add `FontWeightValue`, `FontWidthRatio`, expanded `Weight`,
  expanded `Width`, and a normal-only `FontVariant`. Validate scalar ranges.
- `src/system.rs`: No direct change expected; existing `StyleProperty`
  projection should continue to use `authored.font.weight.into()` and
  `authored.font.width.into()` after the `From` implementations are expanded.
- `src/cache.rs`: Hash new numeric weight, width ratio, and variant values.
- `src/style_support.rs`: Mark numeric font weight and expanded font stretch as
  supported, while keeping `FontVariant` unsupported.
- `src/lib.rs`: Export new public font value types.
- `src/tests.rs`: Add focused tests for numeric weight, width ratio/keywords,
  font variant rejection, support-matrix updates, cache keys, and existing
  feature/variation validation boundaries.

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
  `refactor: model numeric font weights`.
- Group B: Task 2. Commit message:
  `refactor: model font width ratios`.
- Group C: Task 3. Commit message:
  `refactor: bound font variant policy`.
- Group D: Task 4 final holistic checks. Commit only if review fixes are
  required after Group C.

## Font Policy Matrix

Supported after this plan:

- family list strings accepted by `parley::FontFamilyName::parse`
- empty family list as text-owned generic sans-serif fallback
- named weights: thin, extra-light, light, normal, medium, semi-bold, bold,
  extra-bold, black
- numeric weights through `FontWeightValue::try_new`
- width keywords: ultra-condensed, extra-condensed, condensed, semi-condensed,
  normal, semi-expanded, expanded, extra-expanded, ultra-expanded
- width ratios through `FontWidthRatio::try_new`
- normal, italic, and oblique slant with finite optional oblique angle
- feature settings and variation settings as validated CSS setting-list strings

Rejected after this plan:

- non-finite or non-positive numeric weight values
- non-finite or non-positive width ratios
- all non-normal font variants before text construction. Root should report
  them as `TextStyleFeature::FontVariant` with
  `UnsupportedTextStyleReason::RequiresFontPolicy` until a later text-owned
  font-variant realization plan exists.

Coordination-only boundaries:

- `@font-face`, font file loading, host font discovery, and root resource policy
  stay outside `surgeist-text`.
- Root may lower style font resources into existing loaded font data only after
  a separate root/text resource policy plan.
- Font variant realization waits for a later plan because it may require feature
  synthesis, text transform-like behavior, or style-specific OpenType mapping.

## Task 1: Numeric Font Weights

**Files:**
- Modify: `src/style.rs`
- Modify: `src/cache.rs`
- Modify: `src/style_support.rs`
- Modify: `src/lib.rs`
- Test: `src/tests.rs`

- [ ] **Step 0: Check starting status**

Run:

```sh
git status --short --branch
```

Expected: clean except for previously committed sequence/plan work.

- [ ] **Step 1: Write failing numeric weight tests**

Add these tests near existing font tests in `src/tests.rs`:

```rust
#[test]
fn numeric_font_weight_shapes_and_changes_cache_key() {
    let mut system = System::default();
    let first_style = Style {
        font: Font::new().weight(Weight::Number(
            FontWeightValue::try_new(450.0).expect("weight is valid"),
        )),
        ..Style::default()
    };
    let second_style = Style {
        font: Font::new().weight(Weight::Number(
            FontWeightValue::try_new(500.0).expect("weight is valid"),
        )),
        ..Style::default()
    };

    let mut first = system.builder("weight");
    first.default_style(first_style);
    let first = first.build().expect("numeric weight should shape");

    let mut second = system.builder("weight");
    second.default_style(second_style);
    let second = second.build().expect("numeric weight should shape");

    assert_ne!(first.key(), second.key());
}

#[test]
fn font_weight_value_rejects_invalid_values() {
    let zero = FontWeightValue::try_new(0.0).expect_err("zero weight is invalid");
    let nan = FontWeightValue::try_new(f32::NAN).expect_err("nan weight is invalid");

    assert_eq!(zero.code, ErrorCode::InvalidStyle);
    assert_eq!(nan.code, ErrorCode::InvalidStyle);
}

#[test]
fn text_style_support_reports_numeric_font_weight_supported() {
    assert_eq!(
        TextStyleFeature::NumericFontWeight.support(),
        TextStyleSupport::Supported
    );
}
```

- [ ] **Step 2: Run focused tests and confirm RED**

Run:

```sh
cargo test -p surgeist-text numeric_font_weight_shapes_and_changes_cache_key
cargo test -p surgeist-text font_weight_value_rejects_invalid_values
cargo test -p surgeist-text text_style_support_reports_numeric_font_weight_supported
```

Expected: FAIL because `FontWeightValue` and `Weight::Number` do not exist, and
the support matrix still reports numeric weight as unsupported.

- [ ] **Step 3: Add `FontWeightValue` and expand `Weight`**

In `src/style.rs`, add a text-owned scalar before `Weight`:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FontWeightValue(f32);

impl FontWeightValue {
    pub fn try_new(value: f32) -> Result<Self> {
        validate_positive_f32(value, "font weight")?;
        Ok(Self(value))
    }

    #[must_use]
    pub const fn get(self) -> f32 {
        self.0
    }
}
```

Change `Weight` to remove `Eq` and `Hash`, and add the numeric variant:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Weight {
    Thin,
    ExtraLight,
    Light,
    Normal,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
    Number(FontWeightValue),
}
```

Update `From<Weight> for FontWeight`:

```rust
Weight::Number(value) => Self::new(value.get()),
```

Add `"font weight" => "font weight"` to `numeric_field`.

- [ ] **Step 4: Hash numeric weights**

In `src/cache.rs`, replace direct `font.weight.hash(hasher)` with:

```rust
hash_weight(font.weight, hasher);
```

Add:

```rust
fn hash_weight<H: Hasher>(weight: Weight, hasher: &mut H) {
    std::mem::discriminant(&weight).hash(hasher);
    if let Weight::Number(value) = weight {
        hash_f32(value.get(), hasher);
    }
}
```

- [ ] **Step 5: Export and update support matrix**

In `src/lib.rs`, export `FontWeightValue` beside the other style types.

In `src/style_support.rs`, move `TextStyleFeature::NumericFontWeight` from the
font-policy unsupported branch into the supported branch.

Keep `FontVariant` unsupported.

- [ ] **Step 6: Run Group A checks, review, and commit**

Run:

```sh
cargo test -p surgeist-text numeric_font_weight_shapes_and_changes_cache_key
cargo test -p surgeist-text font_weight_value_rejects_invalid_values
cargo test -p surgeist-text text_style_support_reports_numeric_font_weight_supported
cargo fmt --check
git diff --check
git status --short --branch
git diff --stat
git diff -- src/style.rs src/cache.rs src/style_support.rs src/lib.rs src/tests.rs
```

Expected: focused tests, formatting, and diff check pass. The diff only models
numeric weights, projection, hashing, exports, support-matrix update, and tests.

After reviewer approval, commit:

```sh
git add src/style.rs src/cache.rs src/style_support.rs src/lib.rs src/tests.rs
git commit -m "refactor: model numeric font weights"
```

## Task 2: Richer Font Width Values

**Files:**
- Modify: `src/style.rs`
- Modify: `src/cache.rs`
- Modify: `src/style_support.rs`
- Modify: `src/lib.rs`
- Test: `src/tests.rs`

- [ ] **Step 0: Check starting status**

Run:

```sh
git status --short --branch
```

Expected: clean after Group A commit.

- [ ] **Step 1: Write failing width tests**

Add these tests near the numeric weight tests in `src/tests.rs`:

```rust
#[test]
fn font_width_ratio_shapes_and_changes_cache_key() {
    let mut system = System::default();
    let condensed = Style {
        font: Font::new().width(Width::Ratio(
            FontWidthRatio::try_new(0.875).expect("width is valid"),
        )),
        ..Style::default()
    };
    let expanded = Style {
        font: Font::new().width(Width::ExtraExpanded),
        ..Style::default()
    };

    let mut first = system.builder("width");
    first.default_style(condensed);
    let first = first.build().expect("width ratio should shape");

    let mut second = system.builder("width");
    second.default_style(expanded);
    let second = second.build().expect("expanded width should shape");

    assert_ne!(first.key(), second.key());
}

#[test]
fn font_width_ratio_rejects_invalid_values() {
    let zero = FontWidthRatio::try_new(0.0).expect_err("zero width is invalid");
    let nan = FontWidthRatio::try_new(f32::NAN).expect_err("nan width is invalid");

    assert_eq!(zero.code, ErrorCode::InvalidStyle);
    assert_eq!(nan.code, ErrorCode::InvalidStyle);
}

#[test]
fn text_style_support_reports_expanded_font_stretch_supported() {
    assert_eq!(
        TextStyleFeature::ExpandedFontStretch.support(),
        TextStyleSupport::Supported
    );
}
```

- [ ] **Step 2: Run focused tests and confirm RED**

Run:

```sh
cargo test -p surgeist-text font_width_ratio_shapes_and_changes_cache_key
cargo test -p surgeist-text font_width_ratio_rejects_invalid_values
cargo test -p surgeist-text text_style_support_reports_expanded_font_stretch_supported
```

Expected: FAIL because `FontWidthRatio`, expanded width variants, and support
matrix changes do not exist yet.

- [ ] **Step 3: Add `FontWidthRatio` and expand `Width`**

In `src/style.rs`, add:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FontWidthRatio(f32);

impl FontWidthRatio {
    pub fn try_new(value: f32) -> Result<Self> {
        validate_positive_f32(value, "font width ratio")?;
        Ok(Self(value))
    }

    #[must_use]
    pub const fn get(self) -> f32 {
        self.0
    }
}
```

Change `Width` to remove `Eq` and `Hash`, and use this text-domain shape:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Width {
    UltraCondensed,
    ExtraCondensed,
    Condensed,
    SemiCondensed,
    Normal,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
    UltraExpanded,
    Ratio(FontWidthRatio),
}
```

Update the `Font::default()` value to `Width::Normal`.

Add `"font width ratio" => "font width ratio"` to `numeric_field`.

- [ ] **Step 4: Project and hash width values**

Update `From<Width> for FontWidth` in `src/style.rs`:

```rust
Width::UltraCondensed => Self::ULTRA_CONDENSED,
Width::ExtraCondensed => Self::EXTRA_CONDENSED,
Width::Condensed => Self::CONDENSED,
Width::SemiCondensed => Self::SEMI_CONDENSED,
Width::Normal => Self::NORMAL,
Width::SemiExpanded => Self::SEMI_EXPANDED,
Width::Expanded => Self::EXPANDED,
Width::ExtraExpanded => Self::EXTRA_EXPANDED,
Width::UltraExpanded => Self::ULTRA_EXPANDED,
Width::Ratio(value) => Self::from_ratio(value.get()),
```

In `src/cache.rs`, replace direct `font.width.hash(hasher)` with:

```rust
hash_width(font.width, hasher);
```

Add:

```rust
fn hash_width<H: Hasher>(width: Width, hasher: &mut H) {
    std::mem::discriminant(&width).hash(hasher);
    if let Width::Ratio(value) = width {
        hash_f32(value.get(), hasher);
    }
}
```

- [ ] **Step 5: Export and update support matrix**

In `src/lib.rs`, export `FontWidthRatio`.

In `src/style_support.rs`, move `TextStyleFeature::ExpandedFontStretch` from the
font-policy unsupported branch into the supported branch.

- [ ] **Step 6: Run Group B checks, review, and commit**

Run:

```sh
cargo test -p surgeist-text font_width_ratio_shapes_and_changes_cache_key
cargo test -p surgeist-text font_width_ratio_rejects_invalid_values
cargo test -p surgeist-text text_style_support_reports_expanded_font_stretch_supported
cargo fmt --check
git diff --check
git status --short --branch
git diff --stat
git diff -- src/style.rs src/cache.rs src/style_support.rs src/lib.rs src/tests.rs
```

Expected: focused tests, formatting, and diff check pass. The diff only models
width values, projection, hashing, exports, support-matrix update, and tests.

After reviewer approval, commit:

```sh
git add src/style.rs src/cache.rs src/style_support.rs src/lib.rs src/tests.rs
git commit -m "refactor: model font width ratios"
```

## Task 3: Font Variant And Resource Boundaries

**Files:**
- Modify: `src/style.rs`
- Modify: `src/cache.rs`
- Modify: `src/lib.rs`
- Test: `src/tests.rs`

- [ ] **Step 0: Check starting status**

Run:

```sh
git status --short --branch
```

Expected: clean after Group B commit.

- [ ] **Step 1: Write failing font variant tests**

Add these tests near the other font policy tests in `src/tests.rs`:

```rust
#[test]
fn font_variant_normal_is_default_noop() {
    let style = Style {
        font: Font::new().variant(FontVariant::Normal),
        ..Style::default()
    };

    let validated = ValidatedStyle::try_from(style).expect("normal variant should validate");

    assert_eq!(validated.authored().font.variant, FontVariant::Normal);
    assert_eq!(
        TextStyleFeature::FontVariant.support(),
        TextStyleSupport::Unsupported(UnsupportedTextStyleReason::RequiresFontPolicy)
    );
}

#[test]
fn root_must_reject_non_normal_font_variants_before_text() {
    assert_eq!(
        TextStyleFeature::FontVariant.support(),
        TextStyleSupport::Unsupported(UnsupportedTextStyleReason::RequiresFontPolicy)
    );
}
```

- [ ] **Step 2: Run focused tests and confirm RED**

Run:

```sh
cargo test -p surgeist-text font_variant_normal_is_default_noop
cargo test -p surgeist-text root_must_reject_non_normal_font_variants_before_text
```

Expected: FAIL because `FontVariant` and `Font::variant` do not exist yet.

- [ ] **Step 3: Add normal-only `FontVariant` to the font model**

In `src/style.rs`, add:

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FontVariant {
    Normal,
}
```

Add `pub variant: FontVariant` to `Font`, default it to `FontVariant::Normal`,
and add the builder:

```rust
#[must_use]
pub const fn variant(mut self, variant: FontVariant) -> Self {
    self.variant = variant;
    self
}
```

- [ ] **Step 4: Hash font variant**

In `src/cache.rs`, hash the new field:

```rust
font.variant.hash(hasher);
```

No `system.rs` projection is needed because `FontVariant::Normal` is a no-op.
There is no non-normal `FontVariant` value in this crate yet; root must reject
non-normal font variants before constructing text values.

- [ ] **Step 5: Export and keep support matrix unchanged**

In `src/lib.rs`, export `FontVariant`.

Do not mark `TextStyleFeature::FontVariant` supported in `src/style_support.rs`;
the feature remains unsupported until a later plan defines actual variant
realization.

- [ ] **Step 6: Run Group C checks, review, and commit**

Run:

```sh
cargo test -p surgeist-text font_variant_normal_is_default_noop
cargo test -p surgeist-text root_must_reject_non_normal_font_variants_before_text
cargo test -p surgeist-text rejects_invalid_font_settings
cargo test -p surgeist-text validated_style_rejects_invalid_oblique_angle
cargo fmt --check
git diff --check
git status --short --branch
git diff --stat
git diff -- src/style.rs src/cache.rs src/lib.rs src/tests.rs
```

Expected: focused tests, formatting, and diff check pass. Feature/variation and
oblique validation remain unchanged. There is no typed text error emitter for
font variants in this plan because non-normal variants cannot be constructed as
text values yet; root uses the support matrix to reject them before text intake.

After reviewer approval, commit:

```sh
git add src/style.rs src/cache.rs src/lib.rs src/tests.rs
git commit -m "refactor: bound font variant policy"
```

## Task 4: Final Holistic Checks

**Files:**
- Review: `src/style.rs`
- Review: `src/system.rs`
- Review: `src/cache.rs`
- Review: `src/style_support.rs`
- Review: `src/lib.rs`
- Review: `src/tests.rs`

- [ ] **Step 1: Check status after scoped commits**

Run:

```sh
git status --short --branch
```

Expected: clean after Group A, Group B, and Group C commits.

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
git diff "$BASE"..HEAD -- src/style.rs src/system.rs src/cache.rs src/style_support.rs src/lib.rs src/tests.rs
```

Expected:

- numeric font weight support with bounded text-owned scalar
- richer font width support with bounded text-owned ratio
- font variant `Normal` accepted and non-normal variants explicitly left to root
  rejection through the support matrix
- support matrix updated only for numeric weight and expanded font stretch
- no dependency changes
- no font file loading, host font discovery, `@font-face`, root adapter, CSS
  parser, layout, or render work

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

- Root can lower numeric font weights into `FontWeightValue::try_new` and width
  ratios into `FontWidthRatio::try_new`.
- Root should reject every non-normal font variant for text until a later
  font-variant realization plan exists.
- Root should keep `@font-face`, font fetching, host font discovery, and loaded
  font resource policy outside this crate for now.
- Feature and variation setting strings remain validated CSS setting-list
  payloads. A later plan may replace them with parsed setting lists if that
  removes real duplication or improves diagnostics.
- Line-height and font-size remain existing `Style` fields in this plan; inline
  metric derivation is intentionally left to the inline metrics plan.

## Reviewer Checklist

Reviewers must check:

- typed font scalars keep invalid numeric states out of ordinary construction
- `Weight` and `Width` remain one semantic domain each, not transport bags
- hash updates include all new cache-keyed font fields
- support matrix changes reflect actual implemented support
- `FontVariant` remains normal-only in text, and non-normal variant rejection is
  clearly assigned to root through the support matrix
- no dependency, sibling crate, root adapter, or font resource loading work is
  introduced
- tests prove RED/GREEN behavior, cache-key impact, supported matrix updates,
  and typed rejection behavior
