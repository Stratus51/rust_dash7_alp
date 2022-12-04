[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]
[![Documentation][doc-badge]][doc-url]

[crates-badge]: https://img.shields.io/crates/v/dash7_alp.svg
[crates-url]: https://crates.io/crates/dash7_alp
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: LICENSE
[doc-badge]: https://docs.rs/dash7_alp/badge.svg
[doc-url]: https://docs.rs/dash7_alp

Implementation of a [Dash7](https://dash7-alliance.org/) ALP protocol codec from its public specification.

This library is currently intended for desktop grade usage. It liberally uses
Vec and Box, thus making allocations on its own (when decoding).

# Status

The current specification is fully implemented.

This library was not used in any project yet, so there are probably a few bugs lying around.
