use std::ops::Range as StdRange;

use super::{Error, ErrorCode, Result};

/// Byte range in source text.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Range {
    pub start: usize,
    pub end: usize,
}

impl Range {
    #[must_use]
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    #[must_use]
    pub const fn len(self) -> usize {
        self.end - self.start
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }

    #[must_use]
    pub const fn contains(self, index: usize) -> bool {
        self.start <= index && index < self.end
    }
}

impl From<Range> for StdRange<usize> {
    fn from(range: Range) -> Self {
        range.start..range.end
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourcePosition(usize);

impl SourcePosition {
    #[must_use]
    pub const fn from_unchecked(index: usize) -> Self {
        Self(index)
    }

    pub fn try_new(text: &str, index: usize) -> Result<Self> {
        validate_index_raw(text, index, "source position")?;
        Ok(Self(index))
    }

    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct SourceRange {
    start: SourcePosition,
    end: SourcePosition,
}

impl SourceRange {
    #[must_use]
    pub const fn from_unchecked(start: SourcePosition, end: SourcePosition) -> Self {
        Self { start, end }
    }

    pub fn try_new(text: &str, start: usize, end: usize) -> Result<Self> {
        let range = Range::new(start, end);
        validate_raw(text, range)?;
        Ok(Self {
            start: SourcePosition(start),
            end: SourcePosition(end),
        })
    }

    #[must_use]
    pub const fn start(self) -> SourcePosition {
        self.start
    }

    #[must_use]
    pub const fn end(self) -> SourcePosition {
        self.end
    }

    #[must_use]
    pub const fn range(self) -> Range {
        Range::new(self.start.0, self.end.0)
    }
}

pub(crate) fn validate(text: &str, range: Range) -> Result<()> {
    SourceRange::try_new(text, range.start, range.end)?;
    Ok(())
}

pub(crate) fn validate_index(text: &str, index: usize, name: &str) -> Result<()> {
    SourcePosition::try_new_with_name(text, index, name)?;
    Ok(())
}

impl SourcePosition {
    fn try_new_with_name(text: &str, index: usize, name: &str) -> Result<Self> {
        validate_index_raw(text, index, name)?;
        Ok(Self(index))
    }
}

fn validate_raw(text: &str, range: Range) -> Result<()> {
    if range.start > range.end || range.end > text.len() {
        return Err(Error::new(
            ErrorCode::InvalidRange,
            format!(
                "invalid range {}..{} for text length {}",
                range.start,
                range.end,
                text.len()
            ),
        ));
    }
    if !text.is_char_boundary(range.start) || !text.is_char_boundary(range.end) {
        return Err(Error::new(
            ErrorCode::InvalidRange,
            format!(
                "range {}..{} does not align to UTF-8 boundaries",
                range.start, range.end
            ),
        ));
    }
    Ok(())
}

fn validate_index_raw(text: &str, index: usize, name: &str) -> Result<()> {
    if index > text.len() || !text.is_char_boundary(index) {
        return Err(Error::new(
            ErrorCode::InvalidRange,
            format!("{name} {index} is not a valid UTF-8 boundary"),
        ));
    }
    Ok(())
}
