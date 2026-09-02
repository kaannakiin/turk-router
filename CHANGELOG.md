# Changelog

## Unreleased

- Rust client: `build_find_route_instruction`, the ten venue modules, and the wire constants held
  against the manifest. The crate builds an instruction; it is not yet published to crates.io.
- Wire artifacts: the manifest carries `find_route.base_mints` and `find_route.prefix_account_metas`
  (additive, epoch unchanged); the fixture corpus covers every menu kind and preserves the
  program-wide addresses each window fixes.
- Tests: fixture round-trip and per-slot window conformance, manifest agreement for every constant
  and every venue's window lengths, the crate-root lint pin, and the Whirlpool compile-fail proof.
- Documentation: `ARCHITECTURE.md`; CONTRIBUTING gains the documentation rules and the `cargo doc`
  check that CI runs.
- Cross-language corpus: `clients/golden/find_route.json`, the bytes and account list the Rust
  client emits for a fixed sweep of inputs, generated and verified by `tests/cross_language.rs`.
  The TypeScript client will verify the same file.

Repository scaffold: the wire manifest and the synthetic fixture corpus are committed; nothing is
published to crates.io or npm.
