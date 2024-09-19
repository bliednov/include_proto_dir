//! # include_proto_dir
//!
//! `include_proto_dir` is a Rust crate that simplifies embedding and extracting Protocol Buffer (`.proto`) files into your Rust binaries.
//!
//! This crate is designed to improve the developer experience when working with Protocol Buffers in Rust, especially when creating crates for your protobuf definitions.
//!
//! The crate is a thin wrapper around [`include_dir`](https://crates.io/crates/include_dir).
//!
//! ## Examples
//!
//! Use the `include_proto_dir!` macro to include your protobuf directory into your binary:
//!
//! ```rust
//! use include_proto_dir::include_proto_dir;
//!
//! const PROTO_DIR: include_proto_dir::ProtoDir = include_proto_dir!("$CARGO_MANIFEST_DIR/proto");
//! ```
//!
//! Alternatively, if your protobuf files are located in the default `proto` directory at the root of your crate and you want, you can use the `include_proto_dir_default!` macro:
//!
//! ```rust
//! use include_proto_dir::*;
//!
//! include_proto_dir_default!();
//! // It is the same as the following line, note that PROTO_DIR is public and can be used by dependent crates:
//! // pub const PROTO_DIR: include_proto_dir::ProtoDir = include_proto_dir!("$CARGO_MANIFEST_DIR/proto");
//! ```
//!
//! In your build script `build.rs`, you can extract the embedded `.proto` files and generate Rust code using e.g. `prost-build`:
//!
//! ```rust,ignore
//! mod some_proto_crate {
//!     use include_proto_dir::include_proto_dir_default;
//!     include_proto_dir_default!();
//! }
//!
//! use some_proto_crate::PROTO_DIR;
//! use std::path::PathBuf;
//! extern crate build_deps;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
//!     let proto_dir = PROTO_DIR.extract(&out_dir)?;
//!
//!     build_deps::rerun_if_changed_paths(&proto_dir.to_glob()).map_err(|e| format!("{:?}", e))?;
//!     let mut builder = prost_build::Config::new();
//!     builder.compile_protos(proto_dir.protos(), &[proto_dir.as_path()])?;
//!
//!     Ok(())
//! }
//! ```

use anyhow::Result;
use include_dir::Dir;

use std::path::{Path, PathBuf};

/// A struct that represents a directory of embedded Protobuf files.
///
/// The `ProtoDir` struct allows you to extract the embedded `.proto` files to a specified directory,
/// which can then be used by tools like `prost-build` to generate Rust code.
pub struct ProtoDir<'a> {
    /// The embedded directory containing the Protobuf files.
    pub dir: Dir<'a>,
}

impl<'a> ProtoDir<'a> {
    /// Extracts the embedded Protobuf files into the specified output directory, adding a "proto" folder as the parent.
    ///
    /// This function extracts all the embedded `.proto` files into a `proto` subdirectory within the provided `out_dir`.
    ///
    /// # Arguments
    ///
    /// * `out_dir` - The output directory where the `proto` subdirectory will be created.
    ///
    /// # Returns
    ///
    /// Returns an `ExtractedProtoDir` struct containing the path to the extracted directory and a list of `.proto` files.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use include_proto_dir::include_proto_dir;
    /// use tempfile::tempdir;
    /// use std::path::PathBuf;
    ///
    /// const PROTO_DIR: include_proto_dir::ProtoDir = include_proto_dir!("$CARGO_MANIFEST_DIR/proto");
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let tmpdir = tempdir()?;
    ///     let extracted_proto_dir = PROTO_DIR.extract(tmpdir.path())?;
    ///     // Use extracted_proto_dir as needed
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the extraction fails.
    pub fn extract(&self, out_dir: &Path) -> Result<ExtractedProtoDir> {
        let proto_path = out_dir.join("proto");
        self.dir.extract(&proto_path)?;
        Ok(ExtractedProtoDir {
            path: proto_path,
            files: self
                .dir
                .find("**/*.proto")?
                .map(|f| f.path().to_path_buf())
                .collect::<Vec<_>>(),
        })
    }
}

impl<'a> AsRef<Dir<'a>> for ProtoDir<'a> {
    fn as_ref(&self) -> &Dir<'a> {
        &self.dir
    }
}

/// A struct that represents the extracted Protobuf directory.
///
/// After extracting the embedded `.proto` files using `ProtoDir`, an `ExtractedProtoDir` instance
/// provides access to the extracted files and their paths.
///
/// # Examples
///
/// ```rust
/// use include_proto_dir::include_proto_dir;
/// use tempfile::tempdir;
/// use std::path::PathBuf;
///
/// const PROTO_DIR: include_proto_dir::ProtoDir = include_proto_dir!("$CARGO_MANIFEST_DIR/proto");
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let tmpdir = tempdir()?;
///     let extracted_proto_dir = PROTO_DIR.extract(tmpdir.path())?;
///
///     // Get the path to the extracted directory
///     let path = extracted_proto_dir.as_path();
///
///     // Get the list of extracted .proto files
///     let protos = extracted_proto_dir.protos();
///
///     Ok(())
/// }
/// ```
pub struct ExtractedProtoDir {
    path: PathBuf,
    files: Vec<PathBuf>,
}

impl ExtractedProtoDir {
    /// Returns the glob pattern for use with `rerun-if-changed` directives in build scripts.
    ///
    /// This method generates a glob pattern that matches all files within the extracted Protobuf directory.
    /// It is useful for invalidating the build when any of the `.proto` files change.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use include_proto_dir::include_proto_dir;
    /// const PROTO_DIR: include_proto_dir::ProtoDir = include_proto_dir!("$CARGO_MANIFEST_DIR/proto");
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let tmpdir = tempfile::tempdir()?;
    ///     let extracted_proto_dir = PROTO_DIR.extract(tmpdir.path())?;
    ///     println!("cargo:rerun-if-changed={}", extracted_proto_dir.to_glob());
    ///     Ok(())
    /// }
    /// ```
    pub fn to_glob(&self) -> String {
        format!("{}/**", self.path.display())
    }

    /// Returns the path to the extracted Protobuf directory.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use include_proto_dir::include_proto_dir;
    /// const PROTO_DIR: include_proto_dir::ProtoDir = include_proto_dir!("$CARGO_MANIFEST_DIR/proto");
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let tmpdir = tempfile::tempdir()?;
    ///     let extracted_proto_dir = PROTO_DIR.extract(tmpdir.path())?;
    ///     let path = extracted_proto_dir.as_path();
    ///     // Use path as needed
    ///     Ok(())
    /// }
    /// ```
    pub fn as_path(&self) -> &Path {
        self.path.as_path()
    }

    /// Returns the list of extracted `.proto` files in the directory.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use include_proto_dir::include_proto_dir;
    /// const PROTO_DIR: include_proto_dir::ProtoDir = include_proto_dir!("$CARGO_MANIFEST_DIR/proto");
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let tmpdir = tempfile::tempdir()?;
    ///     let extracted_proto_dir = PROTO_DIR.extract(tmpdir.path())?;
    ///     let protos = extracted_proto_dir.protos();
    ///     // Iterate over the .proto files
    ///     for proto in protos {
    ///         println!("Extracted proto file: {}", proto.display());
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub fn protos(&self) -> &[PathBuf] {
        &self.files
    }
}

/// Macro to include the Protobuf directory using `include_dir`.
///
/// This macro embeds the specified directory of `.proto` files into your binary at compile time.
///
/// # Examples
///
/// ```rust
/// use include_proto_dir::include_proto_dir;
///
/// const PROTO_DIR: include_proto_dir::ProtoDir = include_proto_dir!("$CARGO_MANIFEST_DIR/proto");
/// ```
///
/// # Arguments
///
/// * `$path` - The path to the directory containing your `.proto` files. This should be a string literal.
#[macro_export]
macro_rules! include_proto_dir {
    ($path:tt) => {
        $crate::ProtoDir {
            dir: include_dir::include_dir!($path),
        }
    };
}

/// Macro to generate the default `PROTO_DIR`.
///
/// This macro assumes that your `.proto` files are located in a `proto` directory at the root of your crate.
/// It defines a `const PROTO_DIR` that can be used throughout your code and by the dependent crates if you make it public.
///
/// # Examples
///
/// ```rust
/// use include_proto_dir::include_proto_dir_default;
///
/// include_proto_dir_default!();
/// ```
#[macro_export]
macro_rules! include_proto_dir_default {
    () => {
        pub const PROTO_DIR: $crate::ProtoDir =
            $crate::include_proto_dir!("$CARGO_MANIFEST_DIR/proto");
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // Helper function to create a mock ProtoDir
    fn create_mock_proto_dir() -> ProtoDir<'static> {
        // Assuming there's a proto file at "./proto/v1/foo/foo.proto" for testing
        ProtoDir {
            dir: include_dir::include_dir!("proto"),
        }
    }

    #[test]
    fn test_extract() -> Result<()> {
        let proto_dir = create_mock_proto_dir();
        let tmp_dir = tempdir()?;
        let extracted_proto_dir = proto_dir.extract(tmp_dir.path())?;

        // Check that the extracted directory exists
        assert!(extracted_proto_dir.as_path().exists());

        // Check that the extracted directory contains the expected files
        let expected_proto_path = extracted_proto_dir.as_path().join("foo/v1/foo.proto");
        assert!(expected_proto_path.exists());

        Ok(())
    }

    #[test]
    fn test_to_glob() -> Result<()> {
        let proto_dir = create_mock_proto_dir();
        let tmp_dir = tempdir()?;
        let extracted_proto_dir = proto_dir.extract(tmp_dir.path())?;

        let glob_pattern = extracted_proto_dir.to_glob();
        let expected_pattern = format!("{}/**", extracted_proto_dir.as_path().display());
        assert_eq!(glob_pattern, expected_pattern);

        Ok(())
    }

    #[test]
    fn test_as_path() -> Result<()> {
        let proto_dir = create_mock_proto_dir();
        let tmp_dir = tempdir()?;
        let extracted_proto_dir = proto_dir.extract(tmp_dir.path())?;

        // Verify that as_path returns the correct path
        assert_eq!(extracted_proto_dir.as_path(), &tmp_dir.path().join("proto"));

        Ok(())
    }

    #[test]
    fn test_protos() -> Result<()> {
        let proto_dir = create_mock_proto_dir();
        let tmp_dir = tempdir()?;
        let extracted_proto_dir = proto_dir.extract(tmp_dir.path())?;

        let protos = extracted_proto_dir.protos();
        // Assuming we have at least one .proto file
        assert!(!protos.is_empty());

        // Check that each file has a .proto extension
        for proto in protos {
            assert_eq!(proto.extension().and_then(|s| s.to_str()), Some("proto"));
        }

        Ok(())
    }

    #[test]
    fn test_macro_include_proto_dir() -> Result<()> {
        const PROTO_DIR: ProtoDir = include_proto_dir!("proto");
        let tmp_dir = tempdir()?;
        let extracted_proto_dir = PROTO_DIR.extract(tmp_dir.path())?;

        // Check that the extracted directory contains the expected files
        let expected_proto_path = extracted_proto_dir.as_path().join("foo/v1/foo.proto");
        assert!(expected_proto_path.exists());

        Ok(())
    }

    #[test]
    fn test_macro_include_proto_dir_default() -> Result<()> {
        // Define the macro in the test scope
        include_proto_dir_default!();

        let tmp_dir = tempdir()?;
        let extracted_proto_dir = PROTO_DIR.extract(tmp_dir.path())?;

        // Check that the extracted directory contains the expected files
        let expected_proto_path = extracted_proto_dir.as_path().join("foo/v1/foo.proto");
        assert!(expected_proto_path.exists());

        Ok(())
    }
}
