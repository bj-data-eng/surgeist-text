use std::ops::Range as StdRange;

use super::{Range, Result, Source, SourceRange, range};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextEdit {
    range: Range,
    inserted_text: String,
}

impl TextEdit {
    pub fn insert(source: &Source, index: usize, text: impl Into<String>) -> Result<Self> {
        Self::replace(source, Range::new(index, index), text)
    }

    pub fn replace(source: &Source, range: Range, text: impl Into<String>) -> Result<Self> {
        range::validate(source.text(), range)?;
        Ok(Self {
            range,
            inserted_text: text.into(),
        })
    }

    pub fn delete(source: &Source, range: Range) -> Result<Self> {
        Self::replace(source, range, "")
    }

    #[must_use]
    pub const fn range(&self) -> Range {
        self.range
    }

    #[must_use]
    pub fn inserted_text(&self) -> &str {
        &self.inserted_text
    }

    pub fn apply_to(&self, mut source: Source) -> Result<Source> {
        let source_range = SourceRange::try_new(source.text(), self.range.start, self.range.end)?;
        project_edit_ranges(&mut source, source_range, self.inserted_text.len());
        source.revision = source.revision.saturating_add(1);
        source
            .text
            .replace_range(StdRange::from(self.range), &self.inserted_text);
        Ok(source)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Edit {
    Insert { index: usize, text: String },
    Replace { range: Range, text: String },
    Delete { range: Range },
}

impl Edit {
    pub fn normalize(self, source: &Source) -> Result<TextEdit> {
        match self {
            Self::Insert { index, text } => TextEdit::insert(source, index, text),
            Self::Replace { range, text } => TextEdit::replace(source, range, text),
            Self::Delete { range } => TextEdit::delete(source, range),
        }
    }
}

fn project_edit_ranges(source: &mut Source, source_range: SourceRange, inserted_len: usize) {
    let range = source_range.range();
    let removed_len = range.len();
    for span in &mut source.spans {
        span.range = Range::new(
            project_edit_start(span.range.start, range, inserted_len, removed_len),
            project_edit_end(span.range.end, range, inserted_len, removed_len),
        );
    }
    for box_ in &mut source.boxes {
        box_.index = project_edit_anchor(box_.index, range, inserted_len, removed_len);
    }
}

fn project_edit_start(
    index: usize,
    range: Range,
    inserted_len: usize,
    removed_len: usize,
) -> usize {
    if index <= range.start {
        index
    } else if index < range.end {
        range.start
    } else {
        index + inserted_len - removed_len
    }
}

fn project_edit_end(index: usize, range: Range, inserted_len: usize, removed_len: usize) -> usize {
    if index < range.start {
        index
    } else if index <= range.end {
        range.start + inserted_len
    } else {
        index + inserted_len - removed_len
    }
}

fn project_edit_anchor(
    index: usize,
    range: Range,
    inserted_len: usize,
    removed_len: usize,
) -> usize {
    if index <= range.start {
        index
    } else if index <= range.end {
        range.start + inserted_len
    } else {
        index + inserted_len - removed_len
    }
}
