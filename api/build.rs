fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=public/gtfs-r.proto");

    unsafe {
        std::env::set_var(
            "PROTOC",
            protoc_bin_vendored::protoc_bin_path()?,
        );
    }

    prost_build::compile_protos(&["public/gtfs-r.proto"], &["public"])?;

    Ok(())
}
