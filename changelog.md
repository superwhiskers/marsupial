# changelog

## [unreleased]

### changed

- improved benchmarking program
- slightly more idiomatic `Debug` implementation techniques
- addressed a clippy lint in `sys/build.rs`
- parameterized `Hasher` to support variable `SECURITY_LEVEL`s, and
  made `RATE` an associated constant of `Hasher` to support this correctly

### added

- comparison against the [tiny-keccak](https://crates.io/crates/tiny-keccak)
  crate in the benchmark suite, in addition to comparing responses agains it
  in the test suite
- support for KT256 (described
  [here](https://datatracker.ietf.org/doc/draft-irtf-cfrg-kangarootwelve/))
- full set of test vectors for KT256 from the above document
- benchmarks of marsupial's KT256 bindings

## [0.0.2] - 2024-06-21

### added

- `.editorconfig` file

### changed

- moved ffi bindings to a separate crate, `marsupial-sys`
- some adjustments to documentation comments
- contained custom license combination to `marsupial-sys`, `marsupial` is
  entirely cc0

## [0.0.1] - 2024-06-21

initial version. changes are documented relative to code previously at
https://github.com/oconnor663/kangarootwelve_xkcp.rs

### changed

- updated crate dependencies to their latest version
- updated xkcp/k12 and moved to using a git submodule
- adjusted branding to `marsupial`
- moved benchmarking to use the [criterion](https://lib.rs/crates/criterion)
  crate to permit usage of stable rust for benchmarking
- removed the need to manually run `generate_bindings.sh` by integrating
  bindgen into the `build.rs` script

### removed

- deleted `k12sum`
