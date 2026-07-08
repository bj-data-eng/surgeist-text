use std::hash::{Hash, Hasher};

use super::source_model::ValidatedSpan;
use super::{
    Brush, Decoration, DecorationBrush, DecorationOffset, DecorationThickness, Font, Id, Indent,
    InlineBox, LineHeight, Options, Size, Slant, Source, SourceRevision, Style, ValidatedOptions,
    ValidatedSource, ValidatedStyle, Weight, Width,
};

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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Stats {
    layout_hits: usize,
    layout_misses: usize,
    font_refreshes: usize,
    invalidations: usize,
}

impl Stats {
    #[must_use]
    pub const fn layout_hits(self) -> usize {
        self.layout_hits
    }

    #[must_use]
    pub const fn layout_misses(self) -> usize {
        self.layout_misses
    }

    #[must_use]
    pub const fn font_refreshes(self) -> usize {
        self.font_refreshes
    }

    #[must_use]
    pub const fn invalidations(self) -> usize {
        self.invalidations
    }

    pub(crate) fn record_layout_hit(&mut self) {
        self.layout_hits = self.layout_hits.saturating_add(1);
    }

    pub(crate) fn record_layout_miss(&mut self) {
        self.layout_misses = self.layout_misses.saturating_add(1);
    }

    pub(crate) fn record_font_refresh(&mut self) {
        self.font_refreshes = self.font_refreshes.saturating_add(1);
    }

    pub(crate) fn record_invalidations(&mut self, count: usize) {
        self.invalidations = self.invalidations.saturating_add(count);
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Key {
    source: SourceKey,
    styles: StyleKey,
    options: OptionsKey,
    font_generation: FontGeneration,
    pub(crate) options_width: Option<OrderedF32>,
}

impl Key {
    #[must_use]
    pub fn from_validated(
        source: &ValidatedSource,
        style: &ValidatedStyle,
        options: ValidatedOptions,
        font_generation: FontGeneration,
    ) -> Self {
        let identity = source.identity();
        let authored_options = options.authored();
        let source_key = SourceKey::new(
            identity.id(),
            identity.revision().get(),
            stable_hash_source(source.source()),
        );
        let style_key = StyleKey::new(0, stable_hash_style(style.authored(), source.span_styles()));
        let options_key = OptionsKey::new(stable_hash_options(authored_options));
        Self {
            source: source_key,
            styles: style_key,
            options: options_key,
            font_generation,
            options_width: authored_options.width.map(OrderedF32),
        }
    }

    #[must_use]
    pub const fn source(self) -> SourceKey {
        self.source
    }

    #[must_use]
    pub const fn styles(self) -> StyleKey {
        self.styles
    }

    #[must_use]
    pub const fn options(self) -> OptionsKey {
        self.options
    }

    #[must_use]
    pub const fn font_generation(self) -> FontGeneration {
        self.font_generation
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SourceKey {
    id: Option<Id>,
    revision: u64,
    hash: u64,
}

impl SourceKey {
    #[must_use]
    pub(crate) const fn new(id: Option<Id>, revision: u64, hash: u64) -> Self {
        Self { id, revision, hash }
    }

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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StyleKey {
    revision: u64,
    hash: u64,
}

impl StyleKey {
    #[must_use]
    pub(crate) const fn new(revision: u64, hash: u64) -> Self {
        Self { revision, hash }
    }

    #[must_use]
    pub const fn revision(self) -> u64 {
        self.revision
    }

    #[must_use]
    pub const fn hash(self) -> u64 {
        self.hash
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct OptionsKey {
    hash: u64,
}

impl OptionsKey {
    #[must_use]
    pub(crate) const fn new(hash: u64) -> Self {
        Self { hash }
    }

    #[must_use]
    pub const fn hash(self) -> u64 {
        self.hash
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

fn stable_hash_style(style: &Style, spans: &[ValidatedSpan]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    hash_style(style, &mut hasher);
    spans.len().hash(&mut hasher);
    for span in spans {
        span.range().hash(&mut hasher);
        hash_style(span.style().authored(), &mut hasher);
    }
    hasher.finish()
}

fn stable_hash_options(options: Options) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    hash_option_f32(options.width, &mut hasher);
    hash_f32(options.scale, &mut hasher);
    options.alignment.hash(&mut hasher);
    options.text_overflow.hash(&mut hasher);
    options.text_align_last.hash(&mut hasher);
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
    style.text_transform.hash(hasher);
}

fn hash_font<H: Hasher>(font: &Font, hasher: &mut H) {
    font.family.hash(hasher);
    hash_weight(font.weight, hasher);
    hash_width(font.width, hasher);
    hash_slant(font.slant, hasher);
    font.variant.hash(hasher);
    font.features.hash(hasher);
    font.variations.hash(hasher);
}

fn hash_weight<H: Hasher>(weight: Weight, hasher: &mut H) {
    std::mem::discriminant(&weight).hash(hasher);
    if let Weight::Number(value) = weight {
        hash_f32(value.get(), hasher);
    }
}

fn hash_width<H: Hasher>(width: Width, hasher: &mut H) {
    std::mem::discriminant(&width).hash(hasher);
    if let Width::Ratio(value) = width {
        hash_f32(value.get(), hasher);
    }
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
