use std::hash::{Hash, Hasher};

use super::{
    Brush, Decoration, Font, Id, Indent, InlineBox, LineHeight, Options, Size, Slant, Source, Span,
    Style,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Stats {
    pub layout_hits: usize,
    pub layout_misses: usize,
    pub font_refreshes: usize,
    pub invalidations: usize,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Key {
    pub source: SourceKey,
    pub styles: StyleKey,
    pub options: OptionsKey,
    pub font_generation: u64,
    pub(crate) options_width: Option<OrderedF32>,
}

impl Key {
    #[must_use]
    pub fn new(
        source: SourceKey,
        styles: StyleKey,
        options: OptionsKey,
        font_generation: u64,
    ) -> Self {
        Self {
            source,
            styles,
            options,
            font_generation,
            options_width: None,
        }
    }

    pub(crate) fn from_parts(
        source: &Source,
        style: &Style,
        options: Options,
        font_generation: u64,
    ) -> Self {
        let source_key = SourceKey::new(source.id, source.revision, stable_hash_source(source));
        let style_key = StyleKey::new(0, stable_hash_style(style, source.spans()));
        let options_key = OptionsKey::new(stable_hash_options(options));
        Self {
            source: source_key,
            styles: style_key,
            options: options_key,
            font_generation,
            options_width: options.width.map(OrderedF32),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SourceKey {
    pub id: Option<Id>,
    pub revision: u64,
    pub hash: u64,
}

impl SourceKey {
    #[must_use]
    pub const fn new(id: Option<Id>, revision: u64, hash: u64) -> Self {
        Self { id, revision, hash }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StyleKey {
    pub revision: u64,
    pub hash: u64,
}

impl StyleKey {
    #[must_use]
    pub const fn new(revision: u64, hash: u64) -> Self {
        Self { revision, hash }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct OptionsKey {
    pub hash: u64,
}

impl OptionsKey {
    #[must_use]
    pub const fn new(hash: u64) -> Self {
        Self { hash }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct OrderedF32(pub(crate) f32);

impl PartialEq for OrderedF32 {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for OrderedF32 {}

impl std::hash::Hash for OrderedF32 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

fn stable_hash_source(source: &Source) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    source.text.hash(&mut hasher);
    source.boxes.len().hash(&mut hasher);
    for box_ in &source.boxes {
        hash_inline_box(*box_, &mut hasher);
    }
    hasher.finish()
}

fn stable_hash_style(style: &Style, spans: &[Span]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    hash_style(style, &mut hasher);
    spans.len().hash(&mut hasher);
    for span in spans {
        span.range.hash(&mut hasher);
        hash_style(&span.style, &mut hasher);
    }
    hasher.finish()
}

fn stable_hash_options(options: Options) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    hash_option_f32(options.width, &mut hasher);
    hash_f32(options.scale, &mut hasher);
    options.alignment.hash(&mut hasher);
    hash_indent(options.indent, &mut hasher);
    options.quantize.hash(&mut hasher);
    hasher.finish()
}

fn hash_inline_box<H: Hasher>(box_: InlineBox, hasher: &mut H) {
    box_.id.hash(hasher);
    box_.kind.hash(hasher);
    box_.index.hash(hasher);
    hash_size(box_.size, hasher);
}

fn hash_style<H: Hasher>(style: &Style, hasher: &mut H) {
    hash_font(&style.font, hasher);
    hash_f32(style.size, hasher);
    hash_line_height(style.line_height, hasher);
    hash_f32(style.letter_spacing, hasher);
    hash_f32(style.word_spacing, hasher);
    hash_brush(style.brush, hasher);
    hash_decoration(style.underline, hasher);
    hash_decoration(style.strikethrough, hasher);
    style.locale.hash(hasher);
    style.direction.hash(hasher);
    style.white_space.hash(hasher);
    style.word_break.hash(hasher);
    style.wrap.hash(hasher);
    style.overflow_wrap.hash(hasher);
}

fn hash_font<H: Hasher>(font: &Font, hasher: &mut H) {
    font.family.hash(hasher);
    font.weight.hash(hasher);
    font.width.hash(hasher);
    hash_slant(font.slant, hasher);
    font.features.hash(hasher);
    font.variations.hash(hasher);
}

fn hash_slant<H: Hasher>(slant: Slant, hasher: &mut H) {
    std::mem::discriminant(&slant).hash(hasher);
    if let Slant::Oblique(angle) = slant {
        hash_option_f32(angle, hasher);
    }
}

fn hash_line_height<H: Hasher>(line_height: LineHeight, hasher: &mut H) {
    std::mem::discriminant(&line_height).hash(hasher);
    match line_height {
        LineHeight::MetricsRelative(value)
        | LineHeight::FontSizeRelative(value)
        | LineHeight::Absolute(value) => hash_f32(value, hasher),
    }
}

fn hash_decoration<H: Hasher>(decoration: Decoration, hasher: &mut H) {
    decoration.enabled.hash(hasher);
    hash_option_f32(decoration.offset, hasher);
    hash_option_f32(decoration.size, hasher);
    if let Some(brush) = decoration.brush {
        true.hash(hasher);
        hash_brush(brush, hasher);
    } else {
        false.hash(hasher);
    }
}

fn hash_brush<H: Hasher>(brush: Brush, hasher: &mut H) {
    hash_f32(brush.r, hasher);
    hash_f32(brush.g, hasher);
    hash_f32(brush.b, hasher);
    hash_f32(brush.a, hasher);
}

fn hash_indent<H: Hasher>(indent: Indent, hasher: &mut H) {
    hash_f32(indent.amount, hasher);
    indent.first_line.hash(hasher);
    indent.each_line.hash(hasher);
    indent.hanging.hash(hasher);
}

fn hash_size<H: Hasher>(size: Size, hasher: &mut H) {
    hash_f32(size.width, hasher);
    hash_f32(size.height, hasher);
}

fn hash_option_f32<H: Hasher>(value: Option<f32>, hasher: &mut H) {
    match value {
        Some(value) => {
            true.hash(hasher);
            hash_f32(value, hasher);
        }
        None => false.hash(hasher),
    }
}

fn hash_f32<H: Hasher>(value: f32, hasher: &mut H) {
    value.to_bits().hash(hasher);
}
