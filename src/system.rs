use std::{borrow::Cow, collections::HashMap, ops::Range as StdRange};

use parley::{
    AlignmentOptions, FontContext, FontFamily, FontFeatures, FontVariations, GenericFamily,
    LayoutContext, StyleProperty,
};

use super::{
    Brush, Id, InlineBox, Key, Layout, Options, Range, Result, Source, Span, Stats, Style,
    ValidatedOptions, ValidatedStyle, range,
};

/// Shared font and layout system.
pub struct System {
    font_context: FontContext,
    layout_context: LayoutContext<Brush>,
    cache: HashMap<Key, Layout>,
    font_generation: u64,
    stats: Stats,
}

impl System {
    pub fn new(_options: SystemOptions) -> Result<Self> {
        Ok(Self {
            font_context: FontContext::new(),
            layout_context: LayoutContext::new(),
            cache: HashMap::new(),
            font_generation: 0,
            stats: Stats::default(),
        })
    }

    pub fn builder(&mut self, text: impl Into<String>) -> Builder<'_> {
        Builder {
            system: self,
            source: Source::new(text),
            default_style: Style::default(),
            options: Options::default(),
        }
    }

    pub fn layout(&mut self, source: Source, style: Style, options: Options) -> Result<Layout> {
        let mut builder = Builder {
            system: self,
            source,
            default_style: style,
            options,
        };
        builder.build()
    }

    pub fn refresh_fonts(&mut self) -> Result<()> {
        self.font_generation = self.font_generation.saturating_add(1);
        self.stats.invalidations = self.stats.invalidations.saturating_add(self.cache.len());
        self.cache.clear();
        self.stats.font_refreshes = self.stats.font_refreshes.saturating_add(1);
        Ok(())
    }

    #[must_use]
    pub const fn stats(&self) -> Stats {
        self.stats
    }
}

impl Default for System {
    fn default() -> Self {
        Self::new(SystemOptions).expect("default text system should initialize")
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SystemOptions;

/// Builds one layout.
pub struct Builder<'a> {
    system: &'a mut System,
    source: Source,
    default_style: Style,
    options: Options,
}

impl Builder<'_> {
    pub fn identity(&mut self, id: Id, revision: u64) -> &mut Self {
        self.source.set_identity(Some(id), revision);
        self
    }

    pub fn options(&mut self, options: Options) -> &mut Self {
        self.options = options;
        self
    }

    pub fn default_style(&mut self, style: Style) -> &mut Self {
        self.default_style = style;
        self
    }

    pub fn span(&mut self, range: Range, style: Style) -> &mut Self {
        self.source.spans.push(Span::new(range, style));
        self
    }

    pub fn inline_box(&mut self, box_: InlineBox) -> &mut Self {
        self.source.boxes.push(box_);
        self
    }

    pub fn build(&mut self) -> Result<Layout> {
        validate_source(&self.source)?;
        let validated_options = ValidatedOptions::try_from(self.options)?;
        let default_style = ValidatedStyle::try_from(self.default_style.clone())?;
        let span_styles = self
            .source
            .spans
            .iter()
            .map(|span| Ok((span.range, ValidatedStyle::try_from(span.style.clone())?)))
            .collect::<Result<Vec<_>>>()?;
        let layout_source = self.source.clone();

        let key = Key::from_parts(
            &self.source,
            default_style.authored(),
            validated_options.authored(),
            self.system.font_generation,
        );
        if let Some(layout) = self.system.cache.get(&key) {
            self.system.stats.layout_hits = self.system.stats.layout_hits.saturating_add(1);
            return Ok(layout.clone());
        }

        let mut builder = self.system.layout_context.ranged_builder(
            &mut self.system.font_context,
            &layout_source.text,
            validated_options.authored().scale,
            validated_options.authored().quantize,
        );

        push_style_defaults(&mut builder, &default_style);
        for (range, style) in &span_styles {
            push_style_span(&mut builder, *range, style);
        }
        for box_ in &layout_source.boxes {
            builder.push_inline_box(parley::InlineBox {
                id: box_.id.as_u64(),
                kind: box_.kind.into(),
                index: box_.index,
                width: box_.size.width,
                height: box_.size.height,
            });
        }

        let mut layout = builder.build(&layout_source.text);
        if let Some((amount, indent_options)) = validated_options.parley_indent() {
            layout.set_text_indent(amount, indent_options);
        }
        layout.break_all_lines(validated_options.authored().width);
        layout.align(
            validated_options.authored().alignment.into(),
            AlignmentOptions::default(),
        );

        self.system.stats.layout_misses = self.system.stats.layout_misses.saturating_add(1);

        let layout = Layout {
            inner: layout,
            source: layout_source,
            default_style: self.default_style.clone(),
            key,
        };
        self.system.cache.insert(key, layout.clone());
        Ok(layout)
    }
}

fn validate_source(source: &Source) -> Result<()> {
    for span in &source.spans {
        range::validate(source.text(), span.range)?;
    }
    for box_ in &source.boxes {
        range::validate_index(source.text(), box_.index, "inline box index")?;
    }
    Ok(())
}

fn push_style_defaults(builder: &mut parley::RangedBuilder<'_, Brush>, style: &ValidatedStyle) {
    push_style_properties(builder, style, None)
}

fn push_style_span(
    builder: &mut parley::RangedBuilder<'_, Brush>,
    range: Range,
    style: &ValidatedStyle,
) {
    push_style_properties(builder, style, Some(range.into()))
}

fn push_style_properties(
    builder: &mut parley::RangedBuilder<'_, Brush>,
    style: &ValidatedStyle,
    range: Option<StdRange<usize>>,
) {
    let mut push = |property: StyleProperty<'static, Brush>| {
        if let Some(range) = range.clone() {
            builder.push(property, range);
        } else {
            builder.push_default(property);
        }
    };
    let authored = style.authored();

    let family = style
        .parley_font_family()
        .map_or(FontFamily::from(GenericFamily::SansSerif), |families| {
            FontFamily::List(Cow::Owned(families.to_vec()))
        });

    push(StyleProperty::FontFamily(family));
    push(StyleProperty::FontSize(authored.size));
    push(StyleProperty::FontWeight(authored.font.weight.into()));
    push(StyleProperty::FontWidth(authored.font.width.into()));
    push(StyleProperty::FontStyle(authored.font.slant.into()));
    push(StyleProperty::FontFeatures(
        style
            .parley_font_features()
            .map_or_else(FontFeatures::empty, |features| {
                FontFeatures::Source(Cow::Owned(features.to_owned()))
            }),
    ));
    push(StyleProperty::FontVariations(
        style
            .parley_font_variations()
            .map_or_else(FontVariations::empty, |variations| {
                FontVariations::Source(Cow::Owned(variations.to_owned()))
            }),
    ));
    push(StyleProperty::Locale(style.parley_locale()));
    push(StyleProperty::Brush(authored.brush));
    push(StyleProperty::LineHeight(authored.line_height.into()));
    push(StyleProperty::LetterSpacing(authored.letter_spacing));
    push(StyleProperty::WordSpacing(authored.word_spacing));
    push(StyleProperty::WordBreak(authored.word_break.into()));
    push(StyleProperty::OverflowWrap(authored.overflow_wrap.into()));
    push(StyleProperty::TextWrapMode(authored.wrap.into()));
    push(StyleProperty::Underline(authored.underline.enabled));
    push(StyleProperty::UnderlineOffset(authored.underline.offset));
    push(StyleProperty::UnderlineSize(authored.underline.size));
    push(StyleProperty::UnderlineBrush(authored.underline.brush));
    push(StyleProperty::Strikethrough(authored.strikethrough.enabled));
    push(StyleProperty::StrikethroughOffset(
        authored.strikethrough.offset,
    ));
    push(StyleProperty::StrikethroughSize(
        authored.strikethrough.size,
    ));
    push(StyleProperty::StrikethroughBrush(
        authored.strikethrough.brush,
    ));
}
