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
