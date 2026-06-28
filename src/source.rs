use super::{Id, Range, Size, SourcePosition, SourceRange, Style};

/// Text source plus resolved style spans and inline boxes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Source {
    pub id: Option<Id>,
    pub revision: u64,
    pub text: String,
    pub spans: Vec<Span>,
    pub boxes: Vec<InlineBox>,
}

impl Source {
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            id: None,
            revision: 0,
            text: text.into(),
            spans: Vec::new(),
            boxes: Vec::new(),
        }
    }

    #[must_use]
    pub fn identified(text: impl Into<String>, id: Id, revision: u64) -> Self {
        Self {
            id: Some(id),
            revision,
            text: text.into(),
            spans: Vec::new(),
            boxes: Vec::new(),
        }
    }

    pub fn set_identity(&mut self, id: Option<Id>, revision: u64) -> &mut Self {
        self.id = id;
        self.revision = revision;
        self
    }

    pub fn push(&mut self, text: impl AsRef<str>) -> Range {
        let start = SourcePosition::from_unchecked(self.text.len());
        self.text.push_str(text.as_ref());
        SourceRange::from_unchecked(start, SourcePosition::from_unchecked(self.text.len())).range()
    }

    pub fn span(&mut self, range: Range, style: Style) -> &mut Self {
        self.spans.push(Span::new(range, style));
        self
    }

    pub fn inline_box(&mut self, box_: InlineBox) -> &mut Self {
        self.boxes.push(box_);
        self
    }

    #[must_use]
    pub const fn id(&self) -> Option<Id> {
        self.id
    }

    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    #[must_use]
    pub fn spans(&self) -> &[Span] {
        &self.spans
    }

    #[must_use]
    pub fn boxes(&self) -> &[InlineBox] {
        &self.boxes
    }
}

/// Style override for one source range.
#[derive(Clone, Debug, PartialEq)]
pub struct Span {
    pub range: Range,
    pub style: Style,
}

impl Span {
    #[must_use]
    pub const fn new(range: Range, style: Style) -> Self {
        Self { range, style }
    }
}

/// A box laid out with text.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InlineBox {
    pub id: Id,
    pub kind: InlineBoxKind,
    pub index: usize,
    pub size: Size,
}

impl InlineBox {
    #[must_use]
    pub const fn new(id: Id, kind: InlineBoxKind, index: usize, size: Size) -> Self {
        Self {
            id,
            kind,
            index,
            size,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum InlineBoxKind {
    InFlow,
    OutOfFlow,
}

impl From<InlineBoxKind> for parley::InlineBoxKind {
    fn from(kind: InlineBoxKind) -> Self {
        match kind {
            InlineBoxKind::InFlow => Self::InFlow,
            InlineBoxKind::OutOfFlow => Self::OutOfFlow,
        }
    }
}
