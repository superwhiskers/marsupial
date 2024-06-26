# changelog

## [unreleased]

### changed

- fixed remaining incorrect documentation comments referencing a panic after
  repeated finalization

## [0.0.5] - 2024-06-26

### added

- support for xkcp/k12's `ARMv8Asha3` implementation behind
  `TargetImplementation::Armv8Asha3`
  - note: it doesn't seem rustc is reliable at detecting support for the arm
    sha3 extensions
- parsing of doxygen comments in `marsupial-sys`
- parameterization of `Hash`, permitting constant time equality for hashes of
  arbitrary length
- support for getting out hashes of `KT256` in its canonical length (64 bytes)
- property checking of hash functionality
- fuzzing of hash functionality
- made use-after-finalization impossible through the type system

### changed

- enabled `target-cpu=native` during benchmarking and tests
- the manner in which the security level is specified

### removed

- hexadecimal formatting of `Hash` (for now)

## [0.0.4] - 2024-06-22

## changed

- got `marsupial` and `marsupial-sys` on the same version, with `marsupial`
  depending upon the correct version of `marsupial-sys`

## [0.0.3] - 2024-06-22

### changed

- improved benchmarking program
- slightly more idiomatic `Debug` implementation techniques
- addressed a clippy lint in `sys/build.rs`
- parameterized `Hasher` to support variable `SECURITY_LEVEL`s, and
  made `RATE` an associated constant of `Hasher` to support this correctly
- made all of the documentation comments uniform

### added

- comparison against the [tiny-keccak](https://crates.io/crates/tiny-keccak)
  crate in the benchmark suite, in addition to comparing responses agains it
  in the test suite
- support for KT256 (described
  [here](https://datatracker.ietf.org/doc/draft-irtf-cfrg-kangarootwelve/))
- full set of test vectors for KT256 from the above document
- benchmarks of marsupial's KT256 bindings
- `strip.sh` script, which is useful for stripping whitespace from the test
  vectors provided in the IETF internet draft above

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
