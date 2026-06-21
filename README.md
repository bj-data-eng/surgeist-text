# surgeist-text

Text shaping, measurement, font-facing abstractions, and text layout contracts for Surgeist.

## API Artifact

The committed API coordination artifact lives at `api/public-api.txt`.

Refresh it explicitly with:

```sh
cargo run --manifest-path api/generator/Cargo.toml
```

API refresh tooling is command-only and must not run as part of normal `cargo test`.
