# Getting started

Build a layout for `hello world` and verify that it contains one line and shaped
glyph runs. The executable proof is `builds_plain_text_layout` in
[the crate's tests](../src/tests.rs).

## Prerequisites

- A local checkout of this repository and Rust/Cargo with edition-2024 support.
- The dependencies declared in [Cargo.toml](../Cargo.toml) already available in
  the local Cargo cache. The command below uses offline mode and will fail if
  required packages are absent.
- All sibling path dependencies required by the local manifests: currently
  `../surgeist-render`, declared by this crate, and `../surgeist-window`, declared
  by `surgeist-render`. Cargo resolves these optional path manifests even when
  their features are disabled. Keep their relative checkout paths intact.
- Fonts available to Parley's default font context. The test uses the default
  sans-serif request; this repository does not include a font fixture.

The manifest does not declare a `rust-version` minimum. Its edition is not a
separate compatibility promise.

## Verify the layout path

1. Open a terminal at the repository root.
2. Run the existing test with default features:

   ```sh
   cargo test --offline -p surgeist-text --lib tests::builds_plain_text_layout -- --exact
   ```

3. Confirm the result reports one passing test. The test constructs a `System`,
   supplies a width of `100.0`, and asserts one line and nonempty glyph runs.
   A dependency-resolution error occurs before this proof runs; it is not a
   layout result.

## The library call

The [public entry point](../src/lib.rs) exports the crate as `surgeist_text`.
The same operation in a Rust target that depends on this crate is:

```rust
use surgeist_text::{Options, System};

let mut system = System::default();
let mut builder = system.builder("hello world");
builder.options(Options {
    width: Some(100.0),
    ..Options::default()
});
let layout = builder.build().expect("layout should build");

assert_eq!(layout.metrics().line_count(), 1);
assert!(!layout.glyph_runs().is_empty());
```

`build()` validates inputs before shaping or looking up a cached layout. Keep the
`System` when creating subsequent layouts to reuse its font context and cache.
Continue with [How-to](how-to.md) to add styles, inspect geometry, or edit text.
