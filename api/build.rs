fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=public/gtfs-r.proto");

    prost_build::compile_protos(&["public/gtfs-r.proto"], &["public"])?;

    Ok(())
}
