/// Text-owned style-facing feature names for root/style integration.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TextStyleFeature {
    FontFamilyList,
    NamedFontWeight,
    NumericFontWeight,
    BasicFontStretch,
    ExpandedFontStretch,
    FontStyle,
    ObliqueSlant,
    FontVariant,
    FontFeatureSettings,
    FontVariationSettings,
    FontSize,
    LineHeight,
    LetterSpacing,
    WordSpacing,
    ConcreteTextColor,
    SymbolicTextColor,
    Locale,
    ExplicitTextDirection,
    WhiteSpacePreserve,
    WhiteSpaceCollapse,
    WordBreak,
    TextWrap,
    OverflowWrap,
    TextOverflow,
    TextAlignment,
    TextAlignLast,
    TextIndent,
    VerticalAlign,
    Underline,
    Strikethrough,
    Overline,
    DecorationOffset,
    DecorationThickness,
    DecorationStyle,
    ConcreteDecorationColor,
    SymbolicDecorationColor,
    TextTransform,
    SelectionColor,
}

impl TextStyleFeature {
    pub const ALL: &'static [Self] = &[
        Self::FontFamilyList,
        Self::NamedFontWeight,
        Self::NumericFontWeight,
        Self::BasicFontStretch,
        Self::ExpandedFontStretch,
        Self::FontStyle,
        Self::ObliqueSlant,
        Self::FontVariant,
        Self::FontFeatureSettings,
        Self::FontVariationSettings,
        Self::FontSize,
        Self::LineHeight,
        Self::LetterSpacing,
        Self::WordSpacing,
        Self::ConcreteTextColor,
        Self::SymbolicTextColor,
        Self::Locale,
        Self::ExplicitTextDirection,
        Self::WhiteSpacePreserve,
        Self::WhiteSpaceCollapse,
        Self::WordBreak,
        Self::TextWrap,
        Self::OverflowWrap,
        Self::TextOverflow,
        Self::TextAlignment,
        Self::TextAlignLast,
        Self::TextIndent,
        Self::VerticalAlign,
        Self::Underline,
        Self::Strikethrough,
        Self::Overline,
        Self::DecorationOffset,
        Self::DecorationThickness,
        Self::DecorationStyle,
        Self::ConcreteDecorationColor,
        Self::SymbolicDecorationColor,
        Self::TextTransform,
        Self::SelectionColor,
    ];

    #[must_use]
    pub const fn support(self) -> TextStyleSupport {
        match self {
            Self::FontFamilyList
            | Self::NamedFontWeight
            | Self::NumericFontWeight
            | Self::BasicFontStretch
            | Self::ExpandedFontStretch
            | Self::FontStyle
            | Self::ObliqueSlant
            | Self::FontFeatureSettings
            | Self::FontVariationSettings
            | Self::FontSize
            | Self::LineHeight
            | Self::LetterSpacing
            | Self::WordSpacing
            | Self::ConcreteTextColor
            | Self::Locale
            | Self::WhiteSpacePreserve
            | Self::WordBreak
            | Self::TextWrap
            | Self::OverflowWrap
            | Self::TextAlignment
            | Self::TextIndent
            | Self::Underline
            | Self::Strikethrough
            | Self::DecorationOffset
            | Self::DecorationThickness
            | Self::ConcreteDecorationColor => TextStyleSupport::Supported,
            Self::ExplicitTextDirection => TextStyleSupport::Unsupported(
                UnsupportedTextStyleReason::RequiresParleyBaseDirection,
            ),
            Self::WhiteSpaceCollapse => TextStyleSupport::Unsupported(
                UnsupportedTextStyleReason::RequiresSourceRangePreservation,
            ),
            Self::FontVariant => {
                TextStyleSupport::Unsupported(UnsupportedTextStyleReason::RequiresFontPolicy)
            }
            Self::TextAlignLast | Self::TextOverflow | Self::TextTransform => {
                TextStyleSupport::Unsupported(UnsupportedTextStyleReason::RequiresTextFlowPolicy)
            }
            Self::Overline | Self::DecorationStyle | Self::SelectionColor => {
                TextStyleSupport::Unsupported(
                    UnsupportedTextStyleReason::RequiresDecorationSelectionPolicy,
                )
            }
            Self::SymbolicTextColor | Self::SymbolicDecorationColor => {
                TextStyleSupport::Unsupported(UnsupportedTextStyleReason::RequiresColorResolution)
            }
            Self::VerticalAlign => TextStyleSupport::Unsupported(
                UnsupportedTextStyleReason::RequiresInlineMetricContract,
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TextStyleSupport {
    Supported,
    Unsupported(UnsupportedTextStyleReason),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UnsupportedTextStyleReason {
    RequiresParleyBaseDirection,
    RequiresSourceRangePreservation,
    RequiresFontPolicy,
    RequiresTextFlowPolicy,
    RequiresDecorationSelectionPolicy,
    RequiresColorResolution,
    RequiresInlineMetricContract,
    IndentShapeNotExpressibleByCurrentBackend,
}
