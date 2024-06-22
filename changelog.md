# changelog

## [unreleased]

### changed

- improved benchmarking program

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
