use sp1_sdk::{SP1Stdin, ProverClient};
use std::fs;
use eq_common::KeccakInclusionToDataRootProofInput;
use celestia_types::nmt::NamespaceProof;

const KECCAK_INCLUSION_ELF: &[u8] = include_bytes!("../../target/elf-compilation/riscv32im-succinct-zkvm-elf/release/eq-program-keccak-inclusion");


fn main() {
    sp1_sdk::utils::setup_logger();

    let input_json = fs::read_to_string("proof_input.json").expect("Failed reading proof input");
    let input: KeccakInclusionToDataRootProofInput = serde_json::from_str(&input_json).expect("Failed deserializing proof input");

    let client = ProverClient::builder().cpu().build();
    let mut stdin = SP1Stdin::new();
    stdin.write(&input);
    client.execute(KECCAK_INCLUSION_ELF, &stdin).run().expect("Failed executing program");
}