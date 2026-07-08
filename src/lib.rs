//! Parley-backed text layout boundary for Surgeist.
//!
//! This module owns text layout facts: shaping, line breaking, glyph runs,
//! inline boxes, cursor geometry, selection geometry, and low-level movement.
//! It does not own rendering, widgets, document identity, style resolution, or
//! application commands.

mod cache;
mod composer;
mod edit;
mod error;
mod geometry;
mod id;
mod layout;
mod options;
mod range;
#[path = "source.rs"]
mod source_model;
mod style;
mod style_support;
mod system;

pub use cache::{FontGeneration, Key, OptionsKey, SourceKey, Stats, StyleKey};
pub use composer::{Composer, Mark, compose, source};
pub use edit::{Edit, TextEdit};
pub use error::{Error, ErrorCode, ErrorDetail, NumericRequirement, Result};
pub use geometry::{Point, Rect, Size};
pub use id::Id;
#[cfg(feature = "text-accessibility")]
pub use layout::Accessibility;
pub use layout::{
    Affinity, Cluster, Cursor, CursorGeometry, DecorationKind, DecorationRun, FontData, FontRef,
    Glyph, Hit, Layout, Line, Metrics, Movement, PositionedInlineBox, Run, RunMetrics, Selection,
    SelectionGeometry, SelectionRect,
};
pub use options::{Alignment, Indent, Options, ValidatedOptions};
pub use range::{Range, SourcePosition, SourceRange};
pub use source_model::{
    InlineBox, InlineBoxKind, Source, SourceIdentity, SourceRevision, Span, ValidatedSource,
};
pub use style::{
    Brush, Decoration, Direction, Font, FontVariant, FontWeightValue, FontWidthRatio, LineHeight,
    OverflowWrap, Slant, Style, ValidatedStyle, Weight, WhiteSpace, Width, WordBreak, Wrap,
};
pub use style_support::{TextStyleFeature, TextStyleSupport, UnsupportedTextStyleReason};
pub use system::{Builder, System, SystemOptions};

#[cfg(test)]
mod tests;
