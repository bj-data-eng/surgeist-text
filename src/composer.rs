use super::{
    Id, InlineBox, InlineBoxKind, Range, Result, Size, Source, SourcePosition, SourceRange, Span,
    Style,
};

pub fn source(children: impl FnOnce(&mut Composer)) -> Source {
    let mut composer = Composer::new();
    children(&mut composer);
    composer.finish()
}

#[must_use]
pub fn compose() -> Composer {
    Composer::new()
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Composer {
    source: Source,
}

impl Composer {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn identity(&mut self, id: Id, revision: u64) -> &mut Self {
        self.source.set_identity(Some(id), revision);
        self
    }

    pub fn revision(&mut self, revision: u64) -> &mut Self {
        self.source.revision = revision;
        self
    }

    pub fn push(&mut self, text: impl AsRef<str>) -> Mark {
        Mark {
            range: self.source.push(text),
        }
    }

    #[must_use]
    pub fn mark(&self) -> Mark {
        let index = self.source.text().len();
        Mark {
            range: Range::new(index, index),
        }
    }

    pub fn span(&mut self, mark: Mark, style: Style) -> &mut Self {
        self.source.span(mark.range, style);
        self
    }

    pub fn with(&mut self, style: Style, children: impl FnOnce(&mut Composer)) -> Mark {
        let start = self.source.text().len();
        let span_index = self.source.spans().len();
        children(self);
        let mark = Mark {
            range: Range::new(start, self.source.text().len()),
        };
        self.source
            .spans
            .insert(span_index, Span::new(mark.range, style));
        mark
    }

    pub fn box_(&mut self, id: Id, kind: InlineBoxKind, size: Size) -> &mut Self {
        let index = self.source.text().len();
        self.source
            .inline_box(InlineBox::new(id, kind, index, size));
        self
    }

    pub fn try_span(&mut self, range: Range, style: Style) -> Result<&mut Self> {
        let range = SourceRange::try_new(self.source.text(), range.start, range.end)?;
        self.source.span(range.range(), style);
        Ok(self)
    }

    pub fn try_inline_box(&mut self, box_: InlineBox) -> Result<&mut Self> {
        SourcePosition::try_new(self.source.text(), box_.index)?;
        self.source.inline_box(box_);
        Ok(self)
    }

    #[must_use]
    pub const fn source(&self) -> &Source {
        &self.source
    }

    #[must_use]
    pub fn finish(self) -> Source {
        self.source
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Mark {
    range: Range,
}

impl Mark {
    #[must_use]
    pub const fn range(self) -> Range {
        self.range
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.range.is_empty()
    }
}
