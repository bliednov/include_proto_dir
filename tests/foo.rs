mod some_proto_crate {
    use include_proto_dir::include_proto_dir_default;
    include_proto_dir_default!();
}

use some_proto_crate::PROTO_DIR;
use std::path::PathBuf;
extern crate build_deps;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
    let proto_dir = PROTO_DIR.extract(&out_dir)?;

    build_deps::rerun_if_changed_paths(&proto_dir.to_glob()).map_err(|e| format!("{:?}", e))?;
    let mut builder = prost_build::Config::new();
    builder.compile_protos(proto_dir.protos(), &[proto_dir.as_path()])?;

    Ok(())
}
