# How-to

These procedures assume the [first layout test](getting-started.md) succeeds and
the calling Rust target already depends on `surgeist-text`.

## Compose text with styled spans

Use the [composer](../src/composer.rs) when constructing text incrementally so
that ranges follow the UTF-8 text you append:

```rust
use surgeist_text::{Options, Style, System, source};

let source = source(|text| {
    text.push("Hello ");
    text.with(Style { size: 20.0, ..Style::default() }, |text| {
        text.push("world");
    });
});
assert_eq!(source.text(), "Hello world");
assert_eq!(source.spans()[0].range(), surgeist_text::Range::new(6, 11));

let layout = System::default()
    .layout(source, Style::default(), Options::default())
    .expect("composed text should build");
assert!(!layout.glyph_runs().is_empty());
```

`with` styles only the text appended inside its closure. For manual ranges, use
`SourceRange::try_new` or `Composer::try_span` to check boundaries against the
current source. A span holds a complete resolved `Style`; overlapping spans are
applied in declaration order, with later spans taking precedence. The composer
places an outer `with` span before its nested spans.

Verify the authored text and span ranges through `Layout::source()`, and inspect
the resulting styles through `Layout::glyph_runs()`. The composer and overlapping
span tests in [src/tests.rs](../src/tests.rs) demonstrate both contracts.

## Query interaction and inline-box geometry

With a built `Layout`, use `hit(Point)` to distinguish text cursors, inline boxes,
and empty space. Use `cursor(Cursor)` for caret geometry, `selection(Selection)`
for selection rectangles, and `move_cursor` or `move_selection` for supported
cluster, word, line, and document movements.

For inline content, create an `InlineBox` at a UTF-8 byte boundary, assign an ID
unique within the source, and supply finite, nonnegative dimensions. Add it with
`Source::inline_box`, `Builder::inline_box`, or `Composer::box_` at the current text
end. Choose `InFlow` when the box should participate in flow, or `OutOfFlow` for a
positioned anchor that does not contribute the same flow metrics.

Verify the box's ID, source index, line, and rectangle with `inline_boxes()`.
Use `inline_metric_facts()` when an adapter also needs requested alignment and
baseline-shift facts. These requests are retained separately from the backend's
rectangle; consult the [alignment limits](reference.md) before interpreting them.
Geometry and movement behavior is implemented in [src/layout.rs](../src/layout.rs)
and exercised by the inline-box, selection, hit, and movement tests in
[src/tests.rs](../src/tests.rs).

## Apply an edit and lay out the new source

Use the fallible edit path for ranges that can come from callers:

```rust
use surgeist_text::{Edit, Options, Source, Style, System};

let mut system = System::default();
let layout = system
    .layout(Source::new("hello"), Style::default(), Options::default())
    .expect("initial layout should build");
let edited = layout
    .try_apply(Edit::Insert { index: 2, text: "y".into() })
    .expect("edit boundary should be valid");

assert_eq!(edited.text(), "heyllo");
assert_eq!(edited.revision(), 1);
assert_eq!(layout.source().text(), "hello");

let updated = system
    .layout(edited, Style::default(), Options::default())
    .expect("edited source should build");
assert_eq!(updated.source().text(), "heyllo");
```

The edit returns a new `Source`; it does not mutate or reflow the existing layout.
Supply the desired default style and options again when rebuilding. For edits
before layout, use `TextEdit::insert`, `replace`, or `delete`, then `apply_to`.
See [the edit implementation](../src/edit.rs) for span and inline-anchor projection.

## Inspect cache reuse

Reuse one `System` and submit identical source, styles, and options. Compare
`Layout::key()` and the `layout_hits()` / `layout_misses()` counters returned by
`System::stats()`. The `repeated_layout_uses_cache` test in
[src/tests.rs](../src/tests.rs) demonstrates a repeat hit.

`System::refresh_fonts()` advances the font generation, clears cached layouts,
and records invalidations. Verify the next build is a miss and has a different
font-generation key. The current method performs cache invalidation; it does not
recreate the backend font context. See [src/system.rs](../src/system.rs).
