use std::{fmt, ops::Range as StdRange};

use parley::PositionedLayoutItem;

use super::geometry::rect_from_bounds;
use super::system::validate_range;
use super::{
    Brush, Direction, Id, InlineBoxKind, Key, Point, Range, Rect, Result, Size, Source, Style,
};

/// Immutable shaped and line-broken layout.
#[derive(Clone)]
pub struct Layout {
    pub(crate) inner: parley::Layout<Brush>,
    pub(crate) source: Source,
    pub(crate) default_style: Style,
    pub(crate) key: Key,
}

impl fmt::Debug for Layout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Layout")
            .field("metrics", &self.metrics())
            .field("key", &self.key)
            .finish_non_exhaustive()
    }
}

impl Layout {
    #[must_use]
    pub const fn source(&self) -> &Source {
        &self.source
    }

    #[must_use]
    pub fn metrics(&self) -> Metrics {
        Metrics {
            size: Size::new(self.inner.width(), self.inner.height()),
            full_width: self.inner.full_width(),
            line_count: self.inner.len(),
            first_baseline: self.inner.get(0).map(|line| line.metrics().baseline),
            last_baseline: self
                .inner
                .get(self.inner.len().saturating_sub(1))
                .map(|line| line.metrics().baseline),
            overflow: self
                .key
                .options_width
                .is_some_and(|width| self.inner.full_width() > width.0),
        }
    }

    #[must_use]
    pub fn key(&self) -> Key {
        self.key
    }

    #[must_use]
    pub fn direction(&self) -> Direction {
        if self.inner.is_rtl() {
            Direction::RightToLeft
        } else {
            Direction::LeftToRight
        }
    }

    pub fn lines(&self) -> Vec<Line> {
        self.inner
            .lines()
            .enumerate()
            .map(|(index, line)| {
                let metrics = line.metrics();
                Line {
                    index,
                    range: Range::new(line.text_range().start, line.text_range().end),
                    baseline: metrics.baseline,
                    bounds: Rect::new(
                        metrics.inline_min_coord,
                        metrics.block_min_coord,
                        metrics.inline_max_coord - metrics.inline_min_coord,
                        metrics.block_max_coord - metrics.block_min_coord,
                    ),
                }
            })
            .collect()
    }

    pub fn glyph_runs(&self) -> Vec<Run> {
        self.inner
            .lines()
            .flat_map(|line| line.items())
            .filter_map(|item| match item {
                PositionedLayoutItem::GlyphRun(run) => {
                    let run_range = run.run().text_range();
                    let fallback_range = Range::new(run_range.start, run_range.end);
                    let brush = run.style().brush;
                    let mut ranges = run.run().visual_clusters().flat_map(|cluster| {
                        let range = cluster.text_range();
                        let range = Range::new(range.start, range.end);
                        cluster.glyphs().map(move |_| range)
                    });
                    let glyphs: Vec<Glyph> = run
                        .positioned_glyphs()
                        .map(|glyph| Glyph {
                            id: glyph.id,
                            x: glyph.x,
                            y: glyph.y,
                            advance: glyph.advance,
                            range: ranges.next().unwrap_or(fallback_range),
                        })
                        .collect();
                    let style = self.style_for_run(fallback_range, &glyphs, brush);
                    Some(Run {
                        font: FontRef::from_parley(run.run().font()),
                        style,
                        font_size: run.run().font_size(),
                        brush,
                        baseline: run.baseline(),
                        offset: run.offset(),
                        advance: run.advance(),
                        glyphs,
                    })
                }
                PositionedLayoutItem::InlineBox(_) => None,
            })
            .collect()
    }

    fn style_for_run(&self, fallback_range: Range, glyphs: &[Glyph], brush: Brush) -> Style {
        glyphs
            .iter()
            .map(|glyph| self.style_for_range(glyph.range))
            .find(|style| style.brush == brush)
            .or_else(|| {
                glyphs
                    .first()
                    .map(|glyph| self.style_for_range(glyph.range))
            })
            .unwrap_or_else(|| self.style_for_range(fallback_range))
    }

    fn style_for_range(&self, range: Range) -> Style {
        if range.start < range.end {
            self.style_at(range.start)
        } else if range.start > 0 {
            self.style_at(range.start - 1)
        } else {
            self.default_style.clone()
        }
    }

    fn style_at(&self, index: usize) -> Style {
        self.source
            .spans
            .iter()
            .rfind(|span| span.range.contains(index))
            .map(|span| span.style.clone())
            .unwrap_or_else(|| self.default_style.clone())
    }

    pub fn inline_boxes(&self) -> Vec<PositionedInlineBox> {
        self.inner
            .lines()
            .flat_map(|line| line.items())
            .filter_map(|item| match item {
                PositionedLayoutItem::InlineBox(box_) => Some(PositionedInlineBox {
                    id: Id::from_u64(box_.id),
                    kind: match box_.kind {
                        parley::InlineBoxKind::InFlow => InlineBoxKind::InFlow,
                        parley::InlineBoxKind::OutOfFlow
                        | parley::InlineBoxKind::CustomOutOfFlow => InlineBoxKind::OutOfFlow,
                    },
                    rect: Rect::new(box_.x, box_.y, box_.width, box_.height),
                    index: self
                        .source
                        .boxes
                        .iter()
                        .find(|source_box| source_box.id.as_u64() == box_.id)
                        .map_or(0, |source_box| source_box.index),
                }),
                PositionedLayoutItem::GlyphRun(_) => None,
            })
            .collect()
    }

    pub fn clusters(&self) -> Vec<Cluster> {
        let mut clusters = Vec::new();
        for line in self.inner.lines() {
            let line_metrics = line.metrics();
            for run in line.runs() {
                for cluster in run.clusters() {
                    let range = cluster.text_range();
                    let x = cluster.visual_offset().unwrap_or(line_metrics.offset);
                    clusters.push(Cluster {
                        range: Range::new(range.start, range.end),
                        bounds: Rect::new(
                            x,
                            line_metrics.block_min_coord,
                            cluster.advance(),
                            line_metrics.block_max_coord - line_metrics.block_min_coord,
                        ),
                    });
                }
            }
        }
        clusters
    }

    pub fn decorations(&self) -> Vec<DecorationRun> {
        let mut decorations = Vec::new();
        for line in self.inner.lines() {
            for item in line.items() {
                let PositionedLayoutItem::GlyphRun(run) = item else {
                    continue;
                };
                let style = run.style();
                let x = run.offset();
                let width = run.advance();
                if let Some(decoration) = &style.underline {
                    decorations.push(DecorationRun {
                        rect: Rect::new(
                            x,
                            decoration_top(
                                run.baseline(),
                                decoration
                                    .offset
                                    .unwrap_or_else(|| run.run().metrics().underline_offset),
                            ),
                            width,
                            decoration
                                .size
                                .unwrap_or_else(|| run.run().metrics().underline_size),
                        ),
                        brush: decoration.brush,
                        kind: DecorationKind::Underline,
                    });
                }
                if let Some(decoration) = &style.strikethrough {
                    decorations.push(DecorationRun {
                        rect: Rect::new(
                            x,
                            decoration_top(
                                run.baseline(),
                                decoration
                                    .offset
                                    .unwrap_or_else(|| run.run().metrics().strikethrough_offset),
                            ),
                            width,
                            decoration
                                .size
                                .unwrap_or_else(|| run.run().metrics().strikethrough_size),
                        ),
                        brush: decoration.brush,
                        kind: DecorationKind::Strikethrough,
                    });
                }
            }
        }
        decorations
    }

    #[must_use]
    pub fn hit(&self, point: Point) -> Hit {
        for box_ in self.inline_boxes() {
            if box_.rect.contains(point) {
                return Hit::InlineBox(box_.id);
            }
        }
        if self.inner.is_empty() {
            return Hit::None;
        }
        let metrics = self.metrics();
        if !Rect::new(0.0, 0.0, metrics.size.width, metrics.size.height).contains(point) {
            return Hit::None;
        }
        Hit::Text(Cursor::from_parley(parley::Cursor::from_point(
            &self.inner,
            point.x,
            point.y,
        )))
    }

    #[must_use]
    pub fn cursor(&self, cursor: Cursor) -> CursorGeometry {
        let cursor = cursor.to_parley(&self.inner);
        CursorGeometry {
            rect: rect_from_bounds(cursor.geometry(&self.inner, 1.0)),
        }
    }

    pub fn selection(&self, selection: Selection) -> SelectionGeometry {
        let selection = selection.to_parley(&self.inner);
        SelectionGeometry {
            rects: selection
                .geometry(&self.inner)
                .into_iter()
                .map(|(bounds, line)| SelectionRect {
                    rect: rect_from_bounds(bounds),
                    line,
                })
                .collect(),
        }
    }

    #[must_use]
    pub fn move_cursor(&self, cursor: Cursor, movement: Movement) -> Cursor {
        let cursor = cursor.to_parley(&self.inner);
        Cursor::from_parley(match movement {
            Movement::PreviousCluster => cursor.previous_visual(&self.inner),
            Movement::NextCluster => cursor.next_visual(&self.inner),
            Movement::PreviousWord => cursor.previous_logical_word(&self.inner),
            Movement::NextWord => cursor.next_logical_word(&self.inner),
            Movement::LineStart => self.line_boundary(cursor.index(), LineBoundary::Start),
            Movement::LineEnd => self.line_boundary(cursor.index(), LineBoundary::End),
            Movement::PreviousLine => self.vertical_movement(cursor, VerticalMovement::Previous),
            Movement::NextLine => self.vertical_movement(cursor, VerticalMovement::Next),
            Movement::DocumentStart => {
                parley::Cursor::from_byte_index(&self.inner, 0, parley::Affinity::Downstream)
            }
            Movement::DocumentEnd => parley::Cursor::from_byte_index(
                &self.inner,
                self.source.text.len(),
                parley::Affinity::Upstream,
            ),
        })
    }

    #[must_use]
    pub fn move_selection(
        &self,
        selection: Selection,
        movement: Movement,
        extend: bool,
    ) -> Selection {
        let moved = self.move_cursor(selection.focus, movement);
        if extend {
            Selection::new(selection.anchor, moved)
        } else {
            Selection::collapsed(moved)
        }
    }

    #[cfg(feature = "text-render")]
    pub fn push_render_text(
        &self,
        scene: &mut surgeist_render::Scene,
        transform: surgeist_render::Transform,
    ) {
        for run in self.glyph_runs() {
            let glyphs: Vec<_> = run
                .glyphs
                .iter()
                .map(|glyph| surgeist_render::TextGlyph {
                    id: glyph.id,
                    x: glyph.x,
                    y: glyph.y,
                    advance: glyph.advance,
                })
                .collect();
            let mut font = surgeist_render::FontRef::new(run.font.id);
            if let Some(data) = run.font.data() {
                font = font.with_data(data.to_render());
            }
            scene.text_run(surgeist_render::TextRun {
                font,
                size: run.font_size,
                transform,
                paint: surgeist_render::TextPaint {
                    fill: render_color(run.brush).into(),
                },
                glyphs: &glyphs,
            });
        }
        let decorations = self.decorations();
        if !decorations.is_empty() {
            scene.transform(transform, |scene| {
                for decoration in decorations {
                    scene.fill(render_rect(decoration.rect), render_color(decoration.brush));
                }
            });
        }
    }

    #[cfg(feature = "text-accessibility")]
    pub fn accessibility_update(
        &self,
        state: &mut Accessibility,
        parent: accesskit::NodeId,
        focus: accesskit::NodeId,
        origin: Point,
    ) -> accesskit::TreeUpdate {
        let mut update = accesskit::TreeUpdate {
            nodes: Vec::new(),
            tree: None,
            tree_id: accesskit::TreeId::ROOT,
            focus,
        };
        let mut parent_node = accesskit::Node::new(accesskit::Role::Paragraph);
        parent_node.set_bounds(accesskit::Rect {
            x0: origin.x as f64,
            y0: origin.y as f64,
            x1: (origin.x + self.metrics().size.width) as f64,
            y1: (origin.y + self.metrics().size.height) as f64,
        });
        let mut next_id = state.next_id;
        state.inner.build_nodes(
            &self.source.text,
            &self.inner,
            &mut update,
            &mut parent_node,
            || {
                let id = accesskit::NodeId(next_id);
                next_id = next_id.saturating_add(1);
                id
            },
            origin.x as f64,
            origin.y as f64,
            |_node, _style| {},
        );
        state.next_id = next_id;
        update.nodes.push((parent, parent_node));
        update
    }

    #[cfg(feature = "text-accessibility")]
    pub fn access_position(
        &self,
        state: &Accessibility,
        cursor: Cursor,
    ) -> Option<accesskit::TextPosition> {
        cursor
            .to_parley(&self.inner)
            .to_access_position(&self.inner, &state.inner)
    }

    #[cfg(feature = "text-accessibility")]
    pub fn access_selection(
        &self,
        state: &Accessibility,
        selection: Selection,
    ) -> Option<accesskit::TextSelection> {
        selection
            .to_parley(&self.inner)
            .to_access_selection(&self.inner, &state.inner)
    }

    fn line_boundary(&self, index: usize, boundary: LineBoundary) -> parley::Cursor {
        let Some(line) = self.line_for_index(index) else {
            return parley::Cursor::from_byte_index(
                &self.inner,
                match boundary {
                    LineBoundary::Start => 0,
                    LineBoundary::End => self.source.text.len(),
                },
                match boundary {
                    LineBoundary::Start => parley::Affinity::Downstream,
                    LineBoundary::End => parley::Affinity::Upstream,
                },
            );
        };
        match boundary {
            LineBoundary::Start => parley::Cursor::from_byte_index(
                &self.inner,
                line.range.start,
                parley::Affinity::Downstream,
            ),
            LineBoundary::End => parley::Cursor::from_byte_index(
                &self.inner,
                self.line_visible_end(line.range),
                parley::Affinity::Upstream,
            ),
        }
    }

    fn vertical_movement(
        &self,
        cursor: parley::Cursor,
        movement: VerticalMovement,
    ) -> parley::Cursor {
        let lines = self.lines();
        let Some(current_index) = self.line_index_for_cursor(cursor.index(), &lines) else {
            return cursor;
        };
        let next_index = match movement {
            VerticalMovement::Previous => current_index.checked_sub(1),
            VerticalMovement::Next => {
                let index = current_index + 1;
                (index < lines.len()).then_some(index)
            }
        };
        let Some(next_index) = next_index else {
            return cursor;
        };
        let geometry = cursor.geometry(&self.inner, 1.0);
        let line = lines[next_index];
        let y = line.bounds.origin.y + line.bounds.size.height * 0.5;
        parley::Cursor::from_point(&self.inner, geometry.x0 as f32, y)
    }

    fn line_for_index(&self, index: usize) -> Option<Line> {
        let lines = self.lines();
        self.line_index_for_cursor(index, &lines)
            .and_then(|line_index| lines.get(line_index).copied())
    }

    fn line_index_for_cursor(&self, index: usize, lines: &[Line]) -> Option<usize> {
        lines
            .iter()
            .position(|line| index >= line.range.start && index < line.range.end)
            .or_else(|| {
                lines
                    .iter()
                    .rposition(|line| index == line.range.end && line.range.end != line.range.start)
            })
            .or_else(|| {
                (!lines.is_empty() && index >= self.source.text.len()).then_some(lines.len() - 1)
            })
    }

    fn line_visible_end(&self, range: Range) -> usize {
        let mut end = range.end;
        while end > range.start {
            let Some(ch) = self.source.text[..end].chars().next_back() else {
                break;
            };
            if ch == '\n' || ch == '\r' {
                end -= ch.len_utf8();
            } else {
                break;
            }
        }
        end
    }

    #[must_use]
    pub fn apply(&self, edit: Edit) -> Source {
        self.try_apply(edit)
            .expect("Layout::apply expects edit ranges produced from the layout source")
    }

    pub fn try_apply(&self, edit: Edit) -> Result<Source> {
        let mut source = self.source.clone();
        match edit {
            Edit::Insert { index, text } => {
                validate_range(&source.text, Range::new(index, index))?;
                project_edit_ranges(&mut source, Range::new(index, index), text.len());
                source.revision = source.revision.saturating_add(1);
                source.text.insert_str(index, &text);
            }
            Edit::Replace { range, text } => {
                validate_range(&source.text, range)?;
                project_edit_ranges(&mut source, range, text.len());
                source.revision = source.revision.saturating_add(1);
                source.text.replace_range(StdRange::from(range), &text);
            }
            Edit::Delete { range } => {
                validate_range(&source.text, range)?;
                project_edit_ranges(&mut source, range, 0);
                source.revision = source.revision.saturating_add(1);
                source.text.replace_range(StdRange::from(range), "");
            }
        }
        Ok(source)
    }
}

fn project_edit_ranges(source: &mut Source, range: Range, inserted_len: usize) {
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

pub(crate) fn decoration_top(baseline: f32, offset_from_baseline: f32) -> f32 {
    baseline - offset_from_baseline
}

#[cfg(feature = "text-render")]
fn render_rect(rect: Rect) -> surgeist_render::Rect {
    surgeist_render::Rect::new(
        f64::from(rect.origin.x),
        f64::from(rect.origin.y),
        f64::from(rect.size.width),
        f64::from(rect.size.height),
    )
}

#[cfg(feature = "text-render")]
fn render_color(brush: Brush) -> surgeist_render::Color {
    surgeist_render::Color::rgba(brush.r, brush.g, brush.b, brush.a)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Metrics {
    pub size: Size,
    pub full_width: f32,
    pub line_count: usize,
    pub first_baseline: Option<f32>,
    pub last_baseline: Option<f32>,
    pub overflow: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Line {
    pub index: usize,
    pub range: Range,
    pub baseline: f32,
    pub bounds: Rect,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Run {
    pub font: FontRef,
    pub style: Style,
    pub font_size: f32,
    pub brush: Brush,
    pub baseline: f32,
    pub offset: f32,
    pub advance: f32,
    pub glyphs: Vec<Glyph>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FontRef {
    pub id: u64,
    data: Option<FontData>,
}

impl FontRef {
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self { id, data: None }
    }

    #[must_use]
    pub fn with_data(mut self, data: FontData) -> Self {
        self.data = Some(data);
        self
    }

    #[must_use]
    pub fn data(&self) -> Option<&FontData> {
        self.data.as_ref()
    }

    fn from_parley(data: &parley::FontData) -> Self {
        Self {
            id: data.data.id(),
            data: Some(FontData::from_parley(data.clone())),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FontData {
    data: parley::FontData,
}

impl FontData {
    fn from_parley(data: parley::FontData) -> Self {
        Self { data }
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        self.data.data.as_ref()
    }

    #[must_use]
    pub const fn index(&self) -> u32 {
        self.data.index
    }

    #[cfg(feature = "text-render")]
    fn to_render(&self) -> surgeist_render::FontData {
        surgeist_render::FontData::from_bytes(self.data.data.as_ref().to_vec(), self.data.index)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glyph {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub advance: f32,
    pub range: Range,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cluster {
    pub range: Range,
    pub bounds: Rect,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PositionedInlineBox {
    pub id: Id,
    pub kind: InlineBoxKind,
    pub rect: Rect,
    pub index: usize,
}

#[cfg(feature = "text-accessibility")]
#[derive(Clone)]
pub struct Accessibility {
    inner: parley::LayoutAccessibility,
    next_id: u64,
}

#[cfg(feature = "text-accessibility")]
impl Accessibility {
    #[must_use]
    pub fn new(first_node_id: u64) -> Self {
        Self {
            inner: parley::LayoutAccessibility::default(),
            next_id: first_node_id,
        }
    }
}

#[cfg(feature = "text-accessibility")]
impl Default for Accessibility {
    fn default() -> Self {
        Self::new(1)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Hit {
    Text(Cursor),
    InlineBox(Id),
    None,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DecorationRun {
    pub rect: Rect,
    pub brush: Brush,
    pub kind: DecorationKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecorationKind {
    Underline,
    Strikethrough,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Cursor {
    pub index: usize,
    pub affinity: Affinity,
}

impl Cursor {
    #[must_use]
    pub const fn new(index: usize, affinity: Affinity) -> Self {
        Self { index, affinity }
    }

    fn from_parley(cursor: parley::Cursor) -> Self {
        Self {
            index: cursor.index(),
            affinity: cursor.affinity().into(),
        }
    }

    fn to_parley(self, layout: &parley::Layout<Brush>) -> parley::Cursor {
        parley::Cursor::from_byte_index(layout, self.index, self.affinity.into())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Affinity {
    Before,
    After,
}

impl From<parley::Affinity> for Affinity {
    fn from(affinity: parley::Affinity) -> Self {
        match affinity {
            parley::Affinity::Upstream => Self::Before,
            parley::Affinity::Downstream => Self::After,
        }
    }
}

impl From<Affinity> for parley::Affinity {
    fn from(affinity: Affinity) -> Self {
        match affinity {
            Affinity::Before => Self::Upstream,
            Affinity::After => Self::Downstream,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CursorGeometry {
    pub rect: Rect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Selection {
    pub anchor: Cursor,
    pub focus: Cursor,
}

impl Selection {
    #[must_use]
    pub const fn new(anchor: Cursor, focus: Cursor) -> Self {
        Self { anchor, focus }
    }

    #[must_use]
    pub const fn collapsed(cursor: Cursor) -> Self {
        Self {
            anchor: cursor,
            focus: cursor,
        }
    }

    #[must_use]
    pub const fn is_collapsed(self) -> bool {
        self.anchor.index == self.focus.index
            && self.anchor.affinity as u8 == self.focus.affinity as u8
    }

    #[must_use]
    pub fn range(self) -> Range {
        Range::new(
            self.anchor.index.min(self.focus.index),
            self.anchor.index.max(self.focus.index),
        )
    }

    fn to_parley(self, layout: &parley::Layout<Brush>) -> parley::Selection {
        parley::Selection::new(self.anchor.to_parley(layout), self.focus.to_parley(layout))
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SelectionGeometry {
    pub rects: Vec<SelectionRect>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelectionRect {
    pub rect: Rect,
    pub line: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Movement {
    PreviousCluster,
    NextCluster,
    PreviousWord,
    NextWord,
    LineStart,
    LineEnd,
    PreviousLine,
    NextLine,
    DocumentStart,
    DocumentEnd,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LineBoundary {
    Start,
    End,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum VerticalMovement {
    Previous,
    Next,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Edit {
    Insert { index: usize, text: String },
    Replace { range: Range, text: String },
    Delete { range: Range },
}
