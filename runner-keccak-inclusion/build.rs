fn main() -> Result<(), Box<dyn std::error::Error>> {
    // SP1 ELF/Program build
    let sp1_program_crate_path = "../program-keccak-inclusion";
    let sp1_elf_path = "../target/release/eq-program-keccak-inclusion";

    if std::path::Path::new(sp1_elf_path).exists() {
        println!(
            "cargo:warning=File '{}' existed. SP1 program ELF generation skipped",
            sp1_elf_path
        );
    } else {
        println!(
            "cargo:warning=File '{}' does not exist! SP1 Program ELF generated",
            sp1_elf_path
        );
        // Run `cargo build` for the other crate
        let sp1_build_status = std::process::Command::new("cargo")
            .arg("build")
            .arg("--release") // Optionally build in release mode
            .current_dir(sp1_program_crate_path)
            .status()
            .expect("Failed to execute cargo build for the other crate");

        // Check if the command succeeded
        if !sp1_build_status.success() {
            panic!("Building sp1 proof failed!");
        }
    }

    Ok(())
}
