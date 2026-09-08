# surgeist-text

## Dependencies

This notice covers the six direct dependencies declared for `surgeist-text`
0.1.0 in [Cargo.toml](Cargo.toml), including the default configuration, the
`text-accessibility` and `text-render` features, and development dependencies.
It records source-level dependency attribution, not a resolved binary, release,
or transitive dependency inventory. A dependency declaration does not establish
that its code is redistributed in a particular artifact. Upstream example and
test assets are not bundled in this repository.

The project's own MIT license remains in [LICENSE](LICENSE). Keep this notice
and its accompanying `licenses/` files together.

### accesskit 0.24.1

This product depends on accesskit, distributed by The AccessKit contributors:

* License: [MIT](licenses/accesskit/LICENSE-MIT) or
  [Apache License 2.0](licenses/accesskit/LICENSE-APACHE)
* Homepage: [AccessKit](https://accesskit.dev/)
* Additional material: [source notices](licenses/accesskit/NOTICE.md),
  [Chromium BSD license](licenses/accesskit/LICENSE.chromium), and
  [copyright authors](licenses/accesskit/AUTHORS)

This optional dependency is enabled by `text-accessibility`. AccessKit contains
code derived from Chromium and kurbo; the accompanying source notices retain
those attributions and their applicable license terms.

### cosmic-text 0.19.0

This product depends on cosmic-text, distributed by Jeremy Soller:

* License: [MIT](licenses/cosmic-text/LICENSE-MIT) or
  [Apache License 2.0](licenses/cosmic-text/LICENSE-APACHE)
* Homepage: [COSMIC Text](https://github.com/pop-os/cosmic-text)

This is a declared normal dependency with no direct use in the current `src/`
tree. The MIT license retains System76's copyright notice.

### fontdb 0.23.0

This product depends on fontdb, distributed by Yevhenii Reizner:

* License: [MIT](licenses/fontdb/LICENSE)
* Homepage: [fontdb](https://github.com/RazrFalcon/fontdb)

This is a declared normal dependency with no direct use in the current `src/`
tree.

### parley 0.9.0

This product depends on parley, distributed by the Parley Authors:

* License: [Apache License 2.0](licenses/parley/LICENSE-APACHE) or
  [MIT](licenses/parley/LICENSE-MIT)
* Homepage: [Parley](https://github.com/linebender/parley)

Parley provides the text layout implementation in the default configuration.
The `text-accessibility` feature also enables Parley's AccessKit integration.

### pollster 0.4.0

This product depends on pollster, distributed by Joshua Barretto:

* License: [Apache License 2.0](licenses/pollster/LICENSE-APACHE) or
  [MIT](licenses/pollster/LICENSE-MIT)
* Homepage: [Pollster](https://github.com/zesterer/pollster)

This development dependency is used by a renderer test under `text-render`.

### surgeist-render 0.1.0

This product depends on surgeist-render, distributed by bj-data-eng:

* License: [MIT](licenses/surgeist-render/LICENSE)
* Homepage: [surgeist-render](https://github.com/bj-data-eng/surgeist-render)

This optional sibling path dependency is enabled by `text-render`. Its license
is included from the sibling repository; its dependencies are outside this
notice's direct-dependency scope.
