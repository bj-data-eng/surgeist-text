# Reference

## Package and features

[Cargo.toml](../Cargo.toml) defines package `surgeist-text` version `0.1.0`, library
name `surgeist_text`, and Rust edition 2024. No `rust-version` minimum is declared.
The implementation uses Parley `0.9.0` in [src/system.rs](../src/system.rs).
The manifest also declares `cosmic-text` `0.19.0` and `fontdb` `0.23.0`; these are
not selectable alternate backends in the current public API.

| Feature | Default | Effect and dependency |
| --- | --- | --- |
| `text-accessibility` | Off | Enables `Accessibility` and layout-to-AccessKit methods; selects AccessKit `0.24.1` and `parley/accesskit`. |
| `text-render` | Off | Enables `Layout::push_render_text`; requires `surgeist-render` `0.1.0` at `../surgeist-render`. |

The default feature set is empty. Feature-gated methods and their tests are in
[src/layout.rs](../src/layout.rs) and [src/tests.rs](../src/tests.rs). The render
adapter emits glyph runs and decoration rectangles into a render scene. The
accessibility adapter emits an AccessKit tree update and maps text positions and
selections.

## Public surface

All public exports are defined by [src/lib.rs](../src/lib.rs); its modules are
private implementation organization.

| Area | Main types and entry points | Source |
| --- | --- | --- |
| Authored text | `Source`, `Span`, `InlineBox`, `SourceIdentity`, `SourceRevision` | [source.rs](../src/source.rs) |
| Composition | `source`, `compose`, `Composer`, `Mark` | [composer.rs](../src/composer.rs) |
| Style and fonts | `Style`, `Font`, `Brush`, decorations, flow settings | [style.rs](../src/style.rs) |
| Paragraph layout | `Options`, `Alignment`, `Indent`, `ValidatedOptions` | [options.rs](../src/options.rs) |
| Layout construction | `System`, `Builder`, `SystemOptions` | [system.rs](../src/system.rs) |
| Output and interaction | `Layout`, glyph runs, line/box facts, cursors, selections, movement | [layout.rs](../src/layout.rs) |
| Ranges and edits | `Range`, `SourcePosition`, `SourceRange`, `Edit`, `TextEdit` | [range.rs](../src/range.rs), [edit.rs](../src/edit.rs) |
| Cache identity | `Key`, `SourceKey`, `StyleKey`, `OptionsKey`, `FontGeneration`, `Stats` | [cache.rs](../src/cache.rs) |
| Errors | `Error`, `ErrorCode`, `ErrorDetail`, `Result` | [error.rs](../src/error.rs) |

## Defaults and validation

`Options::default()` has no width constraint, scale `1.0`, start alignment,
quantization enabled, `TextOverflow::Clip`, `TextAlignLast::Auto`, and zero indent.
The indent defaults to first-line scope. A nonzero indent with `each_line = true`,
`first_line = false`, and `hanging = false` is unsupported by the backend mapping.

`Style::default()` requests a 16-unit font with normal weight, width, and slant;
the empty family list maps to generic sans-serif. It uses metric-relative line
height `1.0`, zero letter/word spacing, opaque black, no decorations, automatic
direction, preserved whitespace, word wrapping, normal word-break and
overflow-wrap, and no text transform.

Ranges, cursor indices, glyph/cluster ranges, and inline-box anchors use UTF-8
byte offsets. `Range::new` does not validate ordering or character boundaries.
`SourcePosition::try_new` and `SourceRange::try_new` validate against specific
text. Layout construction validates source spans, style values, options, inline
box dimensions, and unique inline-box IDs before cache lookup. Font size and
scale must be finite and positive; layout width and box dimensions must be finite
and nonnegative; brush channels must be finite and within `0..=1`.

## Style support

[TextStyleFeature::ALL and support()](../src/style_support.rs) enumerate the
current style-facing contract. This is a capability report, not a CSS parser or
style resolver.

| Status | Features |
| --- | --- |
| Supported | Family lists; named/numeric weights; basic/expanded stretch; normal/italic/oblique slant; feature and variation settings; font size; line height; letter/word spacing; concrete text color; locale. |
| Supported | Preserved whitespace; word-break, wrapping, and overflow-wrap controls; paragraph alignment; supported indent shapes; inline-box alignment facts; underline/strikethrough; decoration offset, thickness, and concrete color; concrete selection color. |
| Unsupported: backend direction control | Explicit text direction. Automatic direction still resolves to left-to-right or right-to-left in `Layout::direction()`. |
| Unsupported: source-range policy | Whitespace collapse. |
| Unsupported: font or text-flow policy | Non-normal font variants, text overflow behavior beyond the accepted default, last-line alignment behavior beyond the accepted default, and text transformation. |
| Unsupported: decoration policy | Overline and decoration styles beyond the exposed solid decoration contract. |
| Unsupported: color resolution | Symbolic text and decoration colors. |
| Unsupported: broader vertical alignment policy | General text vertical alignment. |

`FontVariant::Normal`, `TextOverflow::Clip`, `TextAlignLast::Auto`, and
`TextTransform::None` are the exposed default values. Their acceptance does not
mean the broader feature is implemented; `Clip` does not add a clipping stage to
the layout pipeline. Explicit direction and collapsed whitespace produce typed
`UnsupportedFeature` errors during validation.

Inline-box `VerticalAlign` values are validated, cached, and preserved in output
facts. The current [Parley projection](../src/system.rs) passes box kind, source
index, width, and height but does not apply these alignment requests.
`BaselineShiftFact::Requested` records an authored shift;
`BackendBottomOnBaseline` describes the backend fact used for other values.
Neither means that a requested middle, super, or shifted placement was performed.

## Focused verification

[src/tests.rs](../src/tests.rs) owns the crate's test cases, including feature-gated
render and accessibility cases. [AGENTS.md](../AGENTS.md) owns the repository
command inventory. [Getting started](getting-started.md) gives the focused default
layout proof; enabling an optional feature requires its declared dependencies to
be available locally as well.
