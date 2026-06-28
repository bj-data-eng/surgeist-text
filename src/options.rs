use super::{Error, ErrorCode, ErrorDetail, NumericRequirement, Result};

/// Paragraph-level layout options.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Options {
    pub width: Option<f32>,
    pub scale: f32,
    pub alignment: Alignment,
    pub indent: Indent,
    pub quantize: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            width: None,
            scale: 1.0,
            alignment: Alignment::Start,
            indent: Indent::default(),
            quantize: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Alignment {
    Start,
    End,
    Left,
    Right,
    Center,
    Justify,
}

impl From<Alignment> for parley::Alignment {
    fn from(alignment: Alignment) -> Self {
        match alignment {
            Alignment::Start => Self::Start,
            Alignment::End => Self::End,
            Alignment::Left => Self::Left,
            Alignment::Right => Self::Right,
            Alignment::Center => Self::Center,
            Alignment::Justify => Self::Justify,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Indent {
    pub amount: f32,
    pub first_line: bool,
    pub each_line: bool,
    pub hanging: bool,
}

impl Default for Indent {
    fn default() -> Self {
        Self {
            amount: 0.0,
            first_line: true,
            each_line: false,
            hanging: false,
        }
    }
}

/// Validated layout options with parsed Parley projection data.
#[derive(Clone, Copy, Debug)]
pub struct ValidatedOptions {
    authored: Options,
    parley_indent: Option<(f32, parley::IndentOptions)>,
}

impl ValidatedOptions {
    fn new(authored: Options, parley_indent: Option<(f32, parley::IndentOptions)>) -> Self {
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

impl TryFrom<Options> for ValidatedOptions {
    type Error = Error;

    fn try_from(options: Options) -> Result<Self> {
        validate_options(&options)?;
        let parley_indent = parley_indent_options(options.indent)?;
        Ok(Self::new(options, parley_indent))
    }
}

fn validate_options(options: &Options) -> Result<()> {
    validate_positive_f32(options.scale, "text scale")?;
    if let Some(width) = options.width {
        validate_non_negative_f32(width, "layout width")?;
    }
    validate_finite_f32(options.indent.amount, "text indent")?;
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

fn validate_non_negative_f32(value: f32, name: &str) -> Result<()> {
    if !value.is_finite() || value < 0.0 {
        return Err(Error::new(
            ErrorCode::InvalidStyle,
            format!("{name} must be finite and non-negative"),
        )
        .with_detail(ErrorDetail::InvalidNumericField {
            field: numeric_field(name),
            value,
            requirement: NumericRequirement::FiniteNonNegative,
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
        "text scale" => "text scale",
        "layout width" => "layout width",
        "text indent" => "text indent",
        _ => "numeric options field",
    }
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
        )
        .with_detail(ErrorDetail::UnsupportedCombination {
            feature: "text indent",
            reason: "each-line indent without first-line indent is not expressible through Parley",
        }));
    }
    Ok(Some((
        indent.amount,
        parley::IndentOptions {
            each_line: indent.each_line,
            hanging: indent.hanging,
        },
    )))
}
