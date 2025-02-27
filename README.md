# tomling

[![Build Status](https://github.com/zeenix/tomling/actions/workflows/rust.yml/badge.svg)](https://github.com/zeenix/tomling/actions/workflows/rust.yml) [![API Documentation](https://docs.rs/tomling/badge.svg)](https://docs.rs/tomling/) [![crates.io](https://img.shields.io/crates/v/tomling)](https://crates.io/crates/tomling)

`tomling` is a simple TOML parser API, that is designed to have minimal dependencies and is `no_std`
compatible. The main target is Cargo manifests (`Cargo.toml` files) and hence why specific
API is provided for that purpose as well.

## Usage

```rust
use tomling::{
    cargo::{BuildDependency, Dependency, Manifest, ResolverVersion, RustEdition},
    Value, parse,
};

//
// Using the `Cargo.toml` specific API:
//

let manifest: Manifest = tomling::from_str(CARGO_TOML).unwrap();

assert_eq!(manifest.package().name(), "example");
assert_eq!(manifest.package().version(), "0.1.0");
assert_eq!(manifest.package().edition().unwrap(), RustEdition::E2021);
assert_eq!(manifest.package().resolver().unwrap(), ResolverVersion::V2);
let authors = manifest.package().authors().unwrap();
let alice = &authors[0];
assert_eq!(alice.name(), "Alice Great");
assert_eq!(alice.email(), Some("foo@bar.com"));
let bob = &authors[1];
assert_eq!(bob.name(), "Bob Less");
assert_eq!(bob.email(), None);

let serde = match manifest.dependencies().unwrap().by_name("serde").unwrap() {
    Dependency::Full(serde) => serde,
    _ => panic!(),
};
assert_eq!(serde.version(), "1.0");
assert_eq!(serde.features(), Some(&["std", "derive"][..]));

let regex = match manifest.dependencies().unwrap().by_name("regex").unwrap() {
    Dependency::VersionOnly(regex) => *regex,
    _ => panic!(),
};
assert_eq!(regex, "1.5");

let cc = match manifest
    .targets()
    .unwrap()
    .by_name("cfg(unix)")
    .unwrap()
    .build_dependencies()
    .unwrap()
    .by_name("cc")
    .unwrap()
{
    BuildDependency::VersionOnly(cc) => *cc,
    _ => panic!(),
};
assert_eq!(cc, "1.0.3");

let default = manifest.features().unwrap().by_name("default").unwrap();
assert_eq!(default, &["serde"]);

let binary = &manifest.binaries().unwrap()[0];
assert_eq!(binary.name(), "some-binary");
assert_eq!(binary.path(), Some("src/bin/my-binary.rs"));

//
// Using the generic raw `TOML` parsing API:
//
let manifest = parse(CARGO_TOML).unwrap();
let package = match manifest.get("package").unwrap() {
    Value::Table(package) => package,
    _ => panic!(),
};
assert_eq!(package.get("name").unwrap(), &Value::String("example"));
assert_eq!(package.get("version").unwrap(), &Value::String("0.1.0"));
assert_eq!(package.get("edition").unwrap(), &Value::String("2021"));
assert_eq!(package.get("resolver").unwrap(), &Value::String("2"));

let deps = match manifest.get("dependencies").unwrap() {
    Value::Table(deps) => deps,
    _ => panic!(),
};
let serde = match deps.get("serde").unwrap() {
    Value::Table(serde) => serde,
    _ => panic!(),
};
assert_eq!(serde.get("version").unwrap(), &Value::String("1.0"));
let serde_features = match serde.get("features").unwrap() {
    Value::Array(features) => features.as_slice(),
    _ => panic!(),
};
assert_eq!(serde_features, &[Value::String("std"), Value::String("derive")]);
let regex = match deps.get("regex").unwrap() {
    Value::String(regex) => *regex,
    _ => panic!(),
};
assert_eq!(regex, "1.5");

const CARGO_TOML: &'static str = r#"
[package]
name = "example"
version = "0.1.0"
edition = "2021"
authors = ["Alice Great <foo@bar.com>", "Bob Less"]
resolver = "2"

[dependencies]
serde = { version = "1.0", features = [
    "std",
    "derive", # and here.
] }
regex = "1.5"

[target.'cfg(unix)'.build-dependencies]
cc = "1.0.3"

[features]
default = ["serde"]

[[bin]]
name = "some-binary"
path = "src/bin/my-binary.rs"

"#;
```

## Dependencies

- `winnow` with `alloc` and `simd` features enabled.
- `serde` (optional) with `alloc` and `derive` features enabled.

## Features

- `serde` - Enables Serde support.
- `cargo-toml` - Enables Cargo manifest specific API. This requires `serde`.
- `simd` - Enables the `simd` feature of `winnow` for SIMD acceleration for parsing.
- `std` - Enables some features, like `std::error::Error` implementation for `Error` type. It also
  enables `std` feature of `winnow` and `serde`.

All features are enabled by default.

## Comparison with `toml` crate

The [`toml`] crate is great but it being based on `toml-edit`, it ends up requiring `indexmap` crate
and its dependencies. `tomling` was created specifically to avoid most of these dependencies by
focusing completely on the parsing of `TOML` documents only.

## Goals

- Simple parser/deserializer API.
- Minimum dependencies. The only mandatory dependency is `winnow` with only 2 features enabled.
- Primary target: Cargo manifests.

## Non-goals

- Strict compliance with the specification. This can change if sufficient demand arises. 😉
- Encoder/Serializer API.

## License

[MIT](LICENSE-MIT)

## The Name

The name "tomling" is a portmanteau of "TOML" and "ling" (a suffix meaning "a small thing").
Coincidentally, it also means a "male kitten" in English, with all the stress on the "kitten"
part 😸 and none on the "male" part.

[`toml`]: https://crates.io/crates/toml
