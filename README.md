# include_proto_dir

`include_proto_dir` is a Rust crate that simplifies embedding and extracting Protocol Buffer (`.proto`) files into your Rust binaries. It enhances the developer experience when working with Protobufs in Rust, especially when creating crates for your protobuf definitions.

The crate is a thin wrapper around [`include_dir`](https://crates.io/crates/include_dir).

By using `include_proto_dir`, you can embed your `.proto` files directly into your crate, ensuring that they are always available during build time of the dependent crates. This approach eliminates the need to manage external `.proto` files and simplifies the build process for your Protobuf crates.

## Features

For the **proto crate**:
- Embed your `.proto` files directly into your binary.
```rust
use include_proto_dir::*;

pub const PROTO_DIR: include_proto_dir::ProtoDir = include_proto_dir!("$CARGO_MANIFEST_DIR/proto");
// that is the same as:
include_proto_dir_default!();
```
- Extract your `.proto` files during the build process in `build.rs`.
```rust
let proto_dir = PROTO_DIR.extract(&PathBuf::from(std::env::var("OUT_DIR")?));
```
- Configure rerun-if-changed for your `.proto` files to ensure that the generated Rust code is up-to-date.
```rust
println!("cargo:rerun-if-changed={}", proto_dir.to_glob());
```
- Generate Rust code from your `.proto` files using `prost-build`.
```rust
prost_build::Config::new()
    .compile_protos(proto_dir.protos(), &[proto_dir.as_path()])?;
```
- Include the generated Rust code in your tests to ensure that your Protobuf definitions compile correctly.
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[macro_export]
    macro_rules! include_proto {
        ($package: tt) => {
            include!(concat!(env!("OUT_DIR"), concat!("/", $package, ".rs")));
        };
    }

    include_proto!("foo.v1");

    #[test]
    fn test_protobuf_compilation() {
        // Use the generated code in some way to ensure it compiles.
    }
}
```

For the **dependent crate**:
- Add the **proto crate** into `[build-dependencies]` in your `Cargo.toml`.
```toml
[build-dependencies]
proto = { path = "../proto" }
```
- Extract the `.proto` files from the **proto crate** during the build process in `build.rs` and generate Rust code from the extracted `.proto` files using `prost-build` the way you like.
```rust
use include_proto_dir::include_proto_dir_default;

include_proto_dir_default!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_dir = PROTO_DIR.extract(&PathBuf::from(std::env::var("OUT_DIR")?));

    println!("cargo:rerun-if-changed={}", proto_dir.to_glob());

    prost_build::Config::new()
        .compile_protos(proto_dir.protos(), &[proto_dir.as_path()])?;

    Ok(())
}
```
- Profit!

## Installation

Add `include_proto_dir` to your `Cargo.toml`:

```toml
[dependencies]
include_proto_dir = "0.1.0"
```

## Contributing

Contributions are welcome! Please submit issues and pull requests on GitHub.

## License

This project is licensed under the MIT License..
