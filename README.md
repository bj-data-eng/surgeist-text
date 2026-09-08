# surgeist-text

`surgeist-text` is the Rust library for text shaping, measurement, font-facing
abstractions, and text layout in Surgeist surfaces. It turns text, resolved styles,
and layout options into glyph runs, lines, inline boxes, and interaction geometry
through Parley. The current package is version `0.1.0`, using Rust edition 2024.

The API has explicit limits: whitespace collapse and explicit base direction are
unsupported, and inline-box alignment requests do not imply backend placement.
See the [support reference](docs/reference.md) for the current contract.

## Start

From the repository root, run the focused layout test:

```sh
cargo test --offline -p surgeist-text --lib tests::builds_plain_text_layout -- --exact
```

Expected result: one passing test that shapes `hello world` into one line with
nonempty glyph runs. See [Getting started](docs/getting-started.md) for prerequisites
and the corresponding library call.

## Documentation

- [Getting started](docs/getting-started.md): verify the checkout and build a first layout.
- [How-to](docs/how-to.md): compose styled text, inspect geometry, and apply edits.
- [Reference](docs/reference.md): public interfaces, defaults, features, and support limits.
- [Explanation](docs/explanation.md): validation, caching, source identity, and ownership.

## License and attribution

The project uses the [MIT license](LICENSE). Third-party attribution and its
coverage are recorded in [NOTICE.md](NOTICE.md).
