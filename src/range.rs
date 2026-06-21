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

pub(crate) fn validate(text: &str, range: Range) -> Result<()> {
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

pub(crate) fn validate_index(text: &str, index: usize, name: &str) -> Result<()> {
    if index > text.len() || !text.is_char_boundary(index) {
        return Err(Error::new(
            ErrorCode::InvalidRange,
            format!("{name} {index} is not a valid UTF-8 boundary"),
        ));
    }
    Ok(())
}
