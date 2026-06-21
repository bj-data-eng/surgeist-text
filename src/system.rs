use std::{borrow::Cow, collections::HashMap, ops::Range as StdRange};

use parley::{
    AlignmentOptions, FontContext, FontFamily, FontFamilyName, FontFeature, FontFeatures,
    FontVariation, FontVariations, GenericFamily, Language, LayoutContext, StyleProperty,
};

use super::{
    Brush, Decoration, Direction, Error, ErrorCode, Id, Indent, InlineBox, Key, Layout, LineHeight,
    Options, Range, Result, Source, Span, Stats, Style, WhiteSpace, range,
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
        validate_options(&self.options)?;
        validate_style(&self.default_style)?;
        for span in &self.source.spans {
            validate_style(&span.style)?;
        }
        let layout_source = self.source.clone();

        let key = Key::from_parts(
            &self.source,
            &self.default_style,
            self.options,
            self.system.font_generation,
        );
        if let Some(layout) = self.system.cache.get(&key) {
            self.system.stats.layout_hits = self.system.stats.layout_hits.saturating_add(1);
            return Ok(layout.clone());
        }

        let mut builder = self.system.layout_context.ranged_builder(
            &mut self.system.font_context,
            &layout_source.text,
            self.options.scale,
            self.options.quantize,
        );

        push_style_defaults(&mut builder, &self.default_style)?;
        for span in &layout_source.spans {
            push_style_span(&mut builder, span)?;
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
        if let Some((amount, indent_options)) = parley_indent_options(self.options.indent)? {
            layout.set_text_indent(amount, indent_options);
        }
        layout.break_all_lines(self.options.width);
        layout.align(self.options.alignment.into(), AlignmentOptions::default());

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

pub(crate) fn validate_range(text: &str, range: Range) -> Result<()> {
    range::validate(text, range)
}

fn validate_options(options: &Options) -> Result<()> {
    validate_positive_f32(options.scale, "text scale")?;
    if let Some(width) = options.width {
        validate_non_negative_f32(width, "layout width")?;
    }
    validate_finite_f32(options.indent.amount, "text indent")?;
    Ok(())
}

fn validate_style(style: &Style) -> Result<()> {
    if style.direction != Direction::Auto {
        return Err(Error::new(
            ErrorCode::UnsupportedFeature,
            "explicit text direction is not supported until Parley exposes public base-direction controls",
        ));
    }
    if style.white_space != WhiteSpace::Preserve {
        return Err(Error::new(
            ErrorCode::UnsupportedFeature,
            "whitespace collapse is not supported until text layout preserves authored source ranges",
        ));
    }
    validate_positive_f32(style.size, "font size")?;
    validate_line_height(style.line_height)?;
    validate_finite_f32(style.letter_spacing, "letter spacing")?;
    validate_finite_f32(style.word_spacing, "word spacing")?;
    validate_brush(style.brush, "text brush")?;
    validate_decoration(style.underline, "underline")?;
    validate_decoration(style.strikethrough, "strikethrough")?;
    if let Some(locale) = &style.locale {
        parse_language(locale)?;
    }
    parse_font_families(&style.font.family)?;
    if let Some(features) = font_settings_source(&style.font.features) {
        validate_font_features(&features)?;
    }
    if let Some(variations) = font_settings_source(&style.font.variations) {
        validate_font_variations(&variations)?;
    }
    Ok(())
}

fn validate_line_height(line_height: LineHeight) -> Result<()> {
    match line_height {
        LineHeight::MetricsRelative(value) => {
            validate_positive_f32(value, "metrics-relative line height")
        }
        LineHeight::FontSizeRelative(value) => {
            validate_positive_f32(value, "font-size-relative line height")
        }
        LineHeight::Absolute(value) => validate_positive_f32(value, "absolute line height"),
    }
}

fn validate_decoration(decoration: Decoration, name: &str) -> Result<()> {
    if !decoration.enabled {
        return Ok(());
    }
    if let Some(offset) = decoration.offset {
        validate_finite_f32(offset, &format!("{name} offset"))?;
    }
    if let Some(size) = decoration.size {
        validate_positive_f32(size, &format!("{name} size"))?;
    }
    if let Some(brush) = decoration.brush {
        validate_brush(brush, &format!("{name} brush"))?;
    }
    Ok(())
}

fn validate_brush(brush: Brush, name: &str) -> Result<()> {
    for (channel, value) in [
        ("red", brush.r),
        ("green", brush.g),
        ("blue", brush.b),
        ("alpha", brush.a),
    ] {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(Error::new(
                ErrorCode::InvalidStyle,
                format!("{name} {channel} channel must be finite and between 0 and 1"),
            ));
        }
    }
    Ok(())
}

fn validate_positive_f32(value: f32, name: &str) -> Result<()> {
    if !value.is_finite() || value <= 0.0 {
        return Err(Error::new(
            ErrorCode::InvalidStyle,
            format!("{name} must be finite and greater than 0"),
        ));
    }
    Ok(())
}

fn validate_non_negative_f32(value: f32, name: &str) -> Result<()> {
    if !value.is_finite() || value < 0.0 {
        return Err(Error::new(
            ErrorCode::InvalidStyle,
            format!("{name} must be finite and non-negative"),
        ));
    }
    Ok(())
}

fn validate_finite_f32(value: f32, name: &str) -> Result<()> {
    if !value.is_finite() {
        return Err(Error::new(
            ErrorCode::InvalidStyle,
            format!("{name} must be finite"),
        ));
    }
    Ok(())
}

fn parse_language(locale: &str) -> Result<Language> {
    Language::parse(locale).map_err(|_| {
        Error::new(
            ErrorCode::InvalidStyle,
            format!("locale {locale:?} is not a valid BCP 47 language tag"),
        )
    })
}

fn parse_font_families(families: &[String]) -> Result<Option<Vec<FontFamilyName<'static>>>> {
    if families.is_empty() {
        return Ok(None);
    }
    let mut parsed = Vec::with_capacity(families.len());
    for family in families {
        if family.trim().is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidStyle,
                "font family names must not be empty",
            ));
        }
        let family = FontFamilyName::parse(family).ok_or_else(|| {
            Error::new(
                ErrorCode::InvalidStyle,
                format!("font family {family:?} is not valid CSS font-family syntax"),
            )
        })?;
        parsed.push(family.into_owned());
    }
    Ok(Some(parsed))
}

fn font_settings_source(settings: &[String]) -> Option<String> {
    if settings.is_empty() {
        return None;
    }
    Some(settings.join(", "))
}

fn validate_font_features(source: &str) -> Result<()> {
    if source.trim().is_empty() {
        return Err(Error::new(
            ErrorCode::InvalidStyle,
            "font feature settings must not be empty",
        ));
    }
    let mut count = 0;
    for feature in FontFeature::parse_css_list(source) {
        feature.map_err(|error| {
            Error::new(
                ErrorCode::InvalidStyle,
                format!("font feature settings {source:?} are invalid: {error}"),
            )
        })?;
        count += 1;
    }
    if count == 0 {
        return Err(Error::new(
            ErrorCode::InvalidStyle,
            "font feature settings must contain at least one setting",
        ));
    }
    Ok(())
}

fn validate_font_variations(source: &str) -> Result<()> {
    if source.trim().is_empty() {
        return Err(Error::new(
            ErrorCode::InvalidStyle,
            "font variation settings must not be empty",
        ));
    }
    let mut count = 0;
    for variation in FontVariation::parse_css_list(source) {
        variation.map_err(|error| {
            Error::new(
                ErrorCode::InvalidStyle,
                format!("font variation settings {source:?} are invalid: {error}"),
            )
        })?;
        count += 1;
    }
    if count == 0 {
        return Err(Error::new(
            ErrorCode::InvalidStyle,
            "font variation settings must contain at least one setting",
        ));
    }
    Ok(())
}

fn parley_indent_options(indent: Indent) -> Result<Option<(f32, parley::IndentOptions)>> {
    if indent.amount == 0.0 {
        return Ok(None);
    }
    if !indent.first_line && !indent.each_line && !indent.hanging {
        return Ok(None);
    }
    if !indent.first_line && indent.each_line && !indent.hanging {
        return Err(Error::new(
            ErrorCode::UnsupportedFeature,
            "each-line indent without first-line indent is not expressible through Parley",
        ));
    }
    Ok(Some((
        indent.amount,
        parley::IndentOptions {
            each_line: indent.each_line,
            hanging: indent.hanging,
        },
    )))
}

fn push_style_defaults(
    builder: &mut parley::RangedBuilder<'_, Brush>,
    style: &Style,
) -> Result<()> {
    push_style_properties(builder, style, None)
}

fn push_style_span(builder: &mut parley::RangedBuilder<'_, Brush>, span: &Span) -> Result<()> {
    push_style_properties(builder, &span.style, Some(span.range.into()))
}

fn push_style_properties(
    builder: &mut parley::RangedBuilder<'_, Brush>,
    style: &Style,
    range: Option<StdRange<usize>>,
) -> Result<()> {
    let mut push = |property: StyleProperty<'static, Brush>| {
        if let Some(range) = range.clone() {
            builder.push(property, range);
        } else {
            builder.push_default(property);
        }
    };

    let family = parse_font_families(&style.font.family)?
        .map_or(FontFamily::from(GenericFamily::SansSerif), |families| {
            FontFamily::List(Cow::Owned(families))
        });

    push(StyleProperty::FontFamily(family));
    push(StyleProperty::FontSize(style.size));
    push(StyleProperty::FontWeight(style.font.weight.into()));
    push(StyleProperty::FontWidth(style.font.width.into()));
    push(StyleProperty::FontStyle(style.font.slant.into()));
    push(StyleProperty::FontFeatures(
        font_settings_source(&style.font.features).map_or_else(FontFeatures::empty, |features| {
            FontFeatures::Source(Cow::Owned(features))
        }),
    ));
    push(StyleProperty::FontVariations(
        font_settings_source(&style.font.variations)
            .map_or_else(FontVariations::empty, |variations| {
                FontVariations::Source(Cow::Owned(variations))
            }),
    ));
    push(StyleProperty::Locale(
        style.locale.as_deref().map(parse_language).transpose()?,
    ));
    push(StyleProperty::Brush(style.brush));
    push(StyleProperty::LineHeight(style.line_height.into()));
    push(StyleProperty::LetterSpacing(style.letter_spacing));
    push(StyleProperty::WordSpacing(style.word_spacing));
    push(StyleProperty::WordBreak(style.word_break.into()));
    push(StyleProperty::OverflowWrap(style.overflow_wrap.into()));
    push(StyleProperty::TextWrapMode(style.wrap.into()));
    push(StyleProperty::Underline(style.underline.enabled));
    push(StyleProperty::UnderlineOffset(style.underline.offset));
    push(StyleProperty::UnderlineSize(style.underline.size));
    push(StyleProperty::UnderlineBrush(style.underline.brush));
    push(StyleProperty::Strikethrough(style.strikethrough.enabled));
    push(StyleProperty::StrikethroughOffset(
        style.strikethrough.offset,
    ));
    push(StyleProperty::StrikethroughSize(style.strikethrough.size));
    push(StyleProperty::StrikethroughBrush(style.strikethrough.brush));
    Ok(())
}
