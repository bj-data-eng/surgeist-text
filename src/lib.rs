//! Parley-backed text layout boundary for Surgeist.
//!
//! This module owns text layout facts: shaping, line breaking, glyph runs,
//! inline boxes, cursor geometry, selection geometry, and low-level movement.
//! It does not own rendering, widgets, document identity, style resolution, or
//! application commands.

mod cache;
mod composer;
mod error;
mod geometry;
mod id;
mod layout;
mod options;
mod range;
#[path = "source.rs"]
mod source_model;
mod style;
mod system;

pub use cache::{FontGeneration, Key, OptionsKey, SourceKey, Stats, StyleKey};
pub use composer::{Composer, Mark, compose, source};
pub use error::{Error, ErrorCode, Result};
pub use geometry::{Point, Rect, Size};
pub use id::Id;
#[cfg(feature = "text-accessibility")]
pub use layout::Accessibility;
pub use layout::{
    Affinity, Cluster, Cursor, CursorGeometry, DecorationKind, DecorationRun, Edit, FontData,
    FontRef, Glyph, Hit, Layout, Line, Metrics, Movement, PositionedInlineBox, Run, Selection,
    SelectionGeometry, SelectionRect,
};
pub use options::{Alignment, Indent, Options, ValidatedOptions};
pub use range::{Range, SourcePosition, SourceRange};
pub use source_model::{
    InlineBox, InlineBoxKind, Source, SourceIdentity, SourceRevision, Span, ValidatedSource,
};
pub use style::{
    Brush, Decoration, Direction, Font, LineHeight, OverflowWrap, Slant, Style, ValidatedStyle,
    Weight, WhiteSpace, Width, WordBreak, Wrap,
};
pub use system::{Builder, System, SystemOptions};

#[cfg(test)]
mod tests;
