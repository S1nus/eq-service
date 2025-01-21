fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Protobuff generation
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/generated")
        .compile_protos(&["proto/eqservice.proto"], &["proto/"])?;

    // SP1 ELF build
    let sp1_program_path = "../program-keccak-inclusion";

    // Run `cargo build` for the other crate
    let sp1_build_status = std::process::Command::new("cargo")
        .arg("build")
        .arg("--release") // Optionally build in release mode
        .current_dir(sp1_program_path)
        .status()
        .expect("Failed to execute cargo build for the other crate");

    // Check if the command succeeded
    if !sp1_build_status.success() {
        panic!("Building sp1 proof failed!");
    }

    // Optionally, emit build artifacts to Cargo
    println!("cargo:rerun-if-changed={}", sp1_program_path);

    Ok(())
}
