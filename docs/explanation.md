# Explanation

## The layout boundary

`surgeist-text` receives authored text with resolved styles and produces text
layout facts. `Source` carries the string, style spans, optional identity and
revision, and inline boxes. `Style` supplies the default resolved text style;
`Options` supplies paragraph constraints. [System](../src/system.rs) validates
these inputs, derives a cache key, and either returns a cached layout or asks
Parley to shape, break lines, and align the result.

The resulting [Layout](../src/layout.rs) is an immutable snapshot containing
authored source and the shaped layout. It exposes line metrics, resolved font
data, glyphs, clusters, decorations, inline boxes, hit tests, selection geometry,
and low-level movement. These facts let a caller build its own surface or editor
without making the text crate own widgets or application commands.

[The public front door](../src/lib.rs) excludes rendering, widgets, document
identity, style resolution, and application commands from the intended boundary.
The repository guide assigns cross-Surgeist adapters and API audit generation to
the root `surgeist` repository. There is a current ownership discrepancy:
`Layout::push_render_text` in [src/layout.rs](../src/layout.rs) implements a
feature-gated cross-crate render adapter in this leaf. That describes the present
code; it does not resolve or revise the boundary stated in [AGENTS.md](../AGENTS.md).

## Source coordinates remain authored coordinates

Text ranges use UTF-8 byte offsets. Preserving the authored string lets style
spans, glyph cluster ranges, cursor positions, and edit ranges refer to the same
source. A byte offset must still lie on a character boundary; it is not a count
of visible characters or grapheme clusters. Cluster movement comes from layout
information rather than arithmetic on byte offsets.

This contract helps explain why whitespace collapse is explicitly rejected:
transforming the string would need an additional mapping back to authored
coordinates. Similarly, style resolution and symbolic colors require caller
policy beyond the concrete values accepted here. The
[support matrix](reference.md) makes those limits observable instead of silently
substituting behavior.

Inline-box alignment illustrates another distinction. Authored alignment and
baseline-shift requests survive validation and appear in metric facts, while the
current backend determines the actual box rectangle without applying those
requests. Consumers must distinguish retained intent from completed placement.

## Identity, edits, and caching

A source identity is optional caller-supplied metadata; the crate does not define
a document identity system. The [cache key](../src/cache.rs) includes source ID,
revision, text and inline-box content, default and span styles, layout options,
and a font generation. Identity alone therefore does not stand in for the
contents of a layout request.

[TextEdit](../src/edit.rs) normalizes insertion, replacement, or deletion to a
validated range and replacement string. Applying it rechecks the range against
the target source, projects span endpoints and inline-box anchors, and increments
the source revision once using saturating arithmetic. It does not require that
the target source have the identity or revision of the source used to construct
the edit. A caller that needs stale-edit detection must enforce that separately.

Projection retains the existing spans and boxes. Span starts inside replaced
text move to the edit start; span ends inside or at the replaced end move to the
end of inserted text. Box anchors at or before the edit start stay put, while
anchors inside the replaced interval move to the inserted end. Later coordinates
shift by the byte-length difference. Applying an edit returns a `Source`; a new
layout request performs validation and shaping for that version.

Each `System` owns its contexts and an in-memory layout cache. Repeated identical
requests reuse cached layouts; counters expose hits, misses, font refreshes, and
invalidations. `refresh_fonts()` changes the font generation and clears layouts.
The current implementation does not recreate its font context or impose a cache
size limit. See [src/system.rs](../src/system.rs) for the exact lifecycle.
