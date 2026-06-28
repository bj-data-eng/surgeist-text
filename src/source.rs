use super::{Id, Range, Result, Size, SourcePosition, SourceRange, Style, ValidatedStyle, range};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceRevision(u64);

impl SourceRevision {
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
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

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct SourceIdentity {
    id: Option<Id>,
    revision: SourceRevision,
}

impl SourceIdentity {
    #[must_use]
    pub const fn new(id: Option<Id>, revision: SourceRevision) -> Self {
        Self { id, revision }
    }

    #[must_use]
    pub const fn id(self) -> Option<Id> {
        self.id
    }

    #[must_use]
    pub const fn revision(self) -> SourceRevision {
        self.revision
    }
}

#[derive(Clone, Debug)]
pub struct ValidatedSource {
    source: Source,
    span_styles: Vec<ValidatedSpan>,
}

#[derive(Clone, Debug)]
pub(crate) struct ValidatedSpan {
    range: Range,
    style: ValidatedStyle,
}

impl ValidatedSpan {
    #[must_use]
    pub(crate) const fn range(&self) -> Range {
        self.range
    }

    #[must_use]
    pub(crate) const fn style(&self) -> &ValidatedStyle {
        &self.style
    }
}

impl ValidatedSource {
    #[must_use]
    pub const fn source(&self) -> &Source {
        &self.source
    }

    #[must_use]
    pub const fn identity(&self) -> SourceIdentity {
        SourceIdentity::new(self.source.id, SourceRevision::new(self.source.revision))
    }

    #[must_use]
    pub(crate) fn span_styles(&self) -> &[ValidatedSpan] {
        &self.span_styles
    }
}

impl TryFrom<Source> for ValidatedSource {
    type Error = super::Error;

    fn try_from(source: Source) -> Result<Self> {
        let span_styles = validate_source(&source)?;
        Ok(Self {
            source,
            span_styles,
        })
    }
}

fn validate_source(source: &Source) -> Result<Vec<ValidatedSpan>> {
    let mut span_styles = Vec::with_capacity(source.spans.len());
    for span in &source.spans {
        range::validate(source.text(), span.range)?;
        span_styles.push(ValidatedSpan {
            range: span.range,
            style: ValidatedStyle::try_from(span.style.clone())?,
        });
    }
    for box_ in &source.boxes {
        range::validate_index(source.text(), box_.index, "inline box index")?;
    }
    Ok(span_styles)
}

/// Text source plus resolved style spans and inline boxes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Source {
    pub(crate) id: Option<Id>,
    pub(crate) revision: u64,
    pub(crate) text: String,
    pub(crate) spans: Vec<Span>,
    pub(crate) boxes: Vec<InlineBox>,
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
    pub(crate) range: Range,
    pub(crate) style: Style,
}

impl Span {
    #[must_use]
    pub const fn new(range: Range, style: Style) -> Self {
        Self { range, style }
    }

    #[must_use]
    pub const fn range(&self) -> Range {
        self.range
    }

    #[must_use]
    pub const fn style(&self) -> &Style {
        &self.style
    }
}

/// A box laid out with text.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InlineBox {
    pub(crate) id: Id,
    pub(crate) kind: InlineBoxKind,
    pub(crate) index: usize,
    pub(crate) size: Size,
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

    #[must_use]
    pub const fn id(self) -> Id {
        self.id
    }

    #[must_use]
    pub const fn kind(self) -> InlineBoxKind {
        self.kind
    }

    #[must_use]
    pub const fn index(self) -> usize {
        self.index
    }

    #[must_use]
    pub const fn size(self) -> Size {
        self.size
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
