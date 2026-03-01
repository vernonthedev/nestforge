fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .compile_protos(&["proto/greeter.proto"], &["proto"])?;

    println!("cargo:rerun-if-changed=proto/greeter.proto");
    Ok(())
}
