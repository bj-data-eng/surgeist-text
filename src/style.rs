use parley::{FontFamilyName, FontFeature, FontStyle, FontVariation, FontWeight, FontWidth};

use super::{
    Error, ErrorCode, ErrorDetail, NumericRequirement, Result, TextStyleFeature,
    UnsupportedTextStyleReason,
};

/// RGBA brush in text-space terms.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Brush {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Brush {
    #[must_use]
    pub const fn color(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

impl Default for Brush {
    fn default() -> Self {
        Self::color(0.0, 0.0, 0.0, 1.0)
    }
}

/// Font request.
#[derive(Clone, Debug, PartialEq)]
pub struct Font {
    pub family: Vec<String>,
    pub weight: Weight,
    pub width: Width,
    pub slant: Slant,
    pub variant: FontVariant,
    pub features: Vec<String>,
    pub variations: Vec<String>,
}

impl Font {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn family(mut self, family: impl Into<String>) -> Self {
        self.family.push(family.into());
        self
    }

    #[must_use]
    pub const fn weight(mut self, weight: Weight) -> Self {
        self.weight = weight;
        self
    }

    #[must_use]
    pub const fn width(mut self, width: Width) -> Self {
        self.width = width;
        self
    }

    #[must_use]
    pub const fn style(mut self, slant: Slant) -> Self {
        self.slant = slant;
        self
    }

    #[must_use]
    pub const fn variant(mut self, variant: FontVariant) -> Self {
        self.variant = variant;
        self
    }

    #[must_use]
    pub fn feature(mut self, feature: impl Into<String>) -> Self {
        self.features.push(feature.into());
        self
    }

    #[must_use]
    pub fn variation(mut self, variation: impl Into<String>) -> Self {
        self.variations.push(variation.into());
        self
    }
}

impl Default for Font {
    fn default() -> Self {
        Self {
            family: Vec::new(),
            weight: Weight::Normal,
            width: Width::Normal,
            slant: Slant::Normal,
            variant: FontVariant::Normal,
            features: Vec::new(),
            variations: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FontVariant {
    Normal,
}

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

impl From<Weight> for FontWeight {
    fn from(weight: Weight) -> Self {
        match weight {
            Weight::Thin => Self::THIN,
            Weight::ExtraLight => Self::EXTRA_LIGHT,
            Weight::Light => Self::LIGHT,
            Weight::Normal => Self::NORMAL,
            Weight::Medium => Self::MEDIUM,
            Weight::SemiBold => Self::SEMI_BOLD,
            Weight::Bold => Self::BOLD,
            Weight::ExtraBold => Self::EXTRA_BOLD,
            Weight::Black => Self::BLACK,
            Weight::Number(value) => Self::new(value.get()),
        }
    }
}

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

impl From<Width> for FontWidth {
    fn from(width: Width) -> Self {
        match width {
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
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Slant {
    Normal,
    Italic,
    Oblique(Option<f32>),
}

impl From<Slant> for FontStyle {
    fn from(slant: Slant) -> Self {
        match slant {
            Slant::Normal => Self::Normal,
            Slant::Italic => Self::Italic,
            Slant::Oblique(angle) => Self::Oblique(angle),
        }
    }
}

/// Resolved text style accepted by the text layout engine.
#[derive(Clone, Debug, PartialEq)]
pub struct Style {
    pub font: Font,
    pub size: f32,
    pub line_height: LineHeight,
    pub letter_spacing: f32,
    pub word_spacing: f32,
    pub brush: Brush,
    pub underline: Decoration,
    pub strikethrough: Decoration,
    pub locale: Option<String>,
    pub direction: Direction,
    pub white_space: WhiteSpace,
    pub word_break: WordBreak,
    pub wrap: Wrap,
    pub overflow_wrap: OverflowWrap,
    pub text_transform: TextTransform,
}

impl Style {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            font: Font::default(),
            size: 16.0,
            line_height: LineHeight::MetricsRelative(1.0),
            letter_spacing: 0.0,
            word_spacing: 0.0,
            brush: Brush::default(),
            underline: Decoration::none(),
            strikethrough: Decoration::none(),
            locale: None,
            direction: Direction::Auto,
            white_space: WhiteSpace::Preserve,
            word_break: WordBreak::Normal,
            wrap: Wrap::Word,
            overflow_wrap: OverflowWrap::Normal,
            text_transform: TextTransform::None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LineHeight {
    MetricsRelative(f32),
    FontSizeRelative(f32),
    Absolute(f32),
}

impl From<LineHeight> for parley::LineHeight {
    fn from(line_height: LineHeight) -> Self {
        match line_height {
            LineHeight::MetricsRelative(value) => Self::MetricsRelative(value),
            LineHeight::FontSizeRelative(value) => Self::FontSizeRelative(value),
            LineHeight::Absolute(value) => Self::Absolute(value),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WhiteSpace {
    Collapse,
    Preserve,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WordBreak {
    Normal,
    BreakAll,
    KeepAll,
}

impl From<WordBreak> for parley::WordBreak {
    fn from(word_break: WordBreak) -> Self {
        match word_break {
            WordBreak::Normal => Self::Normal,
            WordBreak::BreakAll => Self::BreakAll,
            WordBreak::KeepAll => Self::KeepAll,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Wrap {
    None,
    Word,
}

impl From<Wrap> for parley::TextWrapMode {
    fn from(wrap: Wrap) -> Self {
        match wrap {
            Wrap::None => Self::NoWrap,
            Wrap::Word => Self::Wrap,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum OverflowWrap {
    Normal,
    Anywhere,
    BreakWord,
}

impl From<OverflowWrap> for parley::OverflowWrap {
    fn from(overflow_wrap: OverflowWrap) -> Self {
        match overflow_wrap {
            OverflowWrap::Normal => Self::Normal,
            OverflowWrap::Anywhere => Self::Anywhere,
            OverflowWrap::BreakWord => Self::BreakWord,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TextTransform {
    None,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Direction {
    Auto,
    LeftToRight,
    RightToLeft,
}

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

impl Default for Decoration {
    fn default() -> Self {
        Self::none()
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

/// Validated style input with parsed Parley projection data.
#[derive(Clone, Debug)]
pub struct ValidatedStyle {
    authored: Style,
    locale: Option<parley::Language>,
    font_family: Option<Vec<parley::FontFamilyName<'static>>>,
    font_features: Option<String>,
    font_variations: Option<String>,
}

impl ValidatedStyle {
    fn new(
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
    pub fn font_features(&self) -> &[String] {
        &self.authored.font.features
    }

    #[must_use]
    pub fn font_variations(&self) -> &[String] {
        &self.authored.font.variations
    }

    #[must_use]
    pub fn text_transform(&self) -> TextTransform {
        self.authored.text_transform
    }

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

impl TryFrom<Style> for ValidatedStyle {
    type Error = Error;

    fn try_from(style: Style) -> Result<Self> {
        let parsed = validate_style(&style)?;
        Ok(Self::new(
            style,
            parsed.locale,
            parsed.font_family,
            parsed.font_features,
            parsed.font_variations,
        ))
    }
}

struct ParsedStyle {
    locale: Option<parley::Language>,
    font_family: Option<Vec<parley::FontFamilyName<'static>>>,
    font_features: Option<String>,
    font_variations: Option<String>,
}

fn validate_style(style: &Style) -> Result<ParsedStyle> {
    if style.direction != Direction::Auto {
        return Err(Error::new(
            ErrorCode::UnsupportedFeature,
            "explicit text direction is not supported until Parley exposes public base-direction controls",
        )
        .with_detail(ErrorDetail::UnsupportedTextStyle {
            feature: TextStyleFeature::ExplicitTextDirection,
            reason: UnsupportedTextStyleReason::RequiresParleyBaseDirection,
        }));
    }
    if style.white_space != WhiteSpace::Preserve {
        return Err(Error::new(
            ErrorCode::UnsupportedFeature,
            "whitespace collapse is not supported until text layout preserves authored source ranges",
        )
        .with_detail(ErrorDetail::UnsupportedTextStyle {
            feature: TextStyleFeature::WhiteSpaceCollapse,
            reason: UnsupportedTextStyleReason::RequiresSourceRangePreservation,
        }));
    }
    validate_positive_f32(style.size, "font size")?;
    validate_line_height(style.line_height)?;
    validate_finite_f32(style.letter_spacing, "letter spacing")?;
    validate_finite_f32(style.word_spacing, "word spacing")?;
    validate_slant(style.font.slant)?;
    validate_brush(style.brush, "text brush")?;
    validate_decoration(style.underline, "underline")?;
    validate_decoration(style.strikethrough, "strikethrough")?;
    let locale = style.locale.as_deref().map(parse_language).transpose()?;
    let font_family = parse_font_families(&style.font.family)?;
    let font_features = font_settings_source(&style.font.features);
    if let Some(features) = &font_features {
        validate_font_features(features)?;
    }
    let font_variations = font_settings_source(&style.font.variations);
    if let Some(variations) = &font_variations {
        validate_font_variations(variations)?;
    }
    Ok(ParsedStyle {
        locale,
        font_family,
        font_features,
        font_variations,
    })
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

fn validate_slant(slant: Slant) -> Result<()> {
    if let Slant::Oblique(Some(angle)) = slant {
        validate_finite_f32(angle, "oblique angle")?;
    }
    Ok(())
}

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
            )
            .with_detail(ErrorDetail::InvalidNumericField {
                field: brush_channel_field(name, channel),
                value,
                requirement: NumericRequirement::UnitInterval,
            }));
        }
    }
    Ok(())
}

fn validate_positive_f32(value: f32, name: &str) -> Result<()> {
    if !value.is_finite() || value <= 0.0 {
        return Err(Error::new(
            ErrorCode::InvalidStyle,
            format!("{name} must be finite and greater than 0"),
        )
        .with_detail(ErrorDetail::InvalidNumericField {
            field: numeric_field(name),
            value,
            requirement: NumericRequirement::FiniteGreaterThanZero,
        }));
    }
    Ok(())
}

fn validate_finite_f32(value: f32, name: &str) -> Result<()> {
    if !value.is_finite() {
        return Err(
            Error::new(ErrorCode::InvalidStyle, format!("{name} must be finite")).with_detail(
                ErrorDetail::InvalidNumericField {
                    field: numeric_field(name),
                    value,
                    requirement: NumericRequirement::Finite,
                },
            ),
        );
    }
    Ok(())
}

fn numeric_field(name: &str) -> &'static str {
    match name {
        "font size" => "font size",
        "metrics-relative line height" => "metrics-relative line height",
        "font-size-relative line height" => "font-size-relative line height",
        "absolute line height" => "absolute line height",
        "font weight" => "font weight",
        "font width ratio" => "font width ratio",
        "letter spacing" => "letter spacing",
        "word spacing" => "word spacing",
        "oblique angle" => "oblique angle",
        "decoration offset" => "decoration offset",
        "decoration thickness" => "decoration thickness",
        "underline offset" => "underline offset",
        "underline size" => "underline size",
        "strikethrough offset" => "strikethrough offset",
        "strikethrough size" => "strikethrough size",
        _ => "numeric style field",
    }
}

fn brush_channel_field(name: &str, channel: &str) -> &'static str {
    match (name, channel) {
        ("text brush", "red") => "text brush red channel",
        ("text brush", "green") => "text brush green channel",
        ("text brush", "blue") => "text brush blue channel",
        ("text brush", "alpha") => "text brush alpha channel",
        ("underline brush", "red") => "underline brush red channel",
        ("underline brush", "green") => "underline brush green channel",
        ("underline brush", "blue") => "underline brush blue channel",
        ("underline brush", "alpha") => "underline brush alpha channel",
        ("strikethrough brush", "red") => "strikethrough brush red channel",
        ("strikethrough brush", "green") => "strikethrough brush green channel",
        ("strikethrough brush", "blue") => "strikethrough brush blue channel",
        ("strikethrough brush", "alpha") => "strikethrough brush alpha channel",
        _ => "brush channel",
    }
}

fn parse_language(locale: &str) -> Result<parley::Language> {
    parley::Language::parse(locale).map_err(|_| {
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
