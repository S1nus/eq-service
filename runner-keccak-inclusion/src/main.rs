use sp1_sdk::{SP1Stdin, ProverClient};
use std::fs;
use eq_common::KeccakInclusionToDataRootProofInput;

const KECCAK_INCLUSION_ELF: &[u8] = include_bytes!("../../target/elf-compilation/riscv32im-succinct-zkvm-elf/release/eq-program-keccak-inclusion");


fn main() {
    sp1_sdk::utils::setup_logger();

    let input = fs::read_to_string("proof_input.json").expect("Failed reading proof input");
    let input: KeccakInclusionToDataRootProofInput = serde_json::from_str(&input).expect("Failed deserializing proof input");

    let input_bytes = bincode::serialize(&input)
        .map_err(|e| {
            println!("Serialization error: {:?}", e);
            e
        })
        .expect("Failed serializing proof input");
    
    println!("Successfully serialized {} bytes", input_bytes.len());
    
    let from_bytes: KeccakInclusionToDataRootProofInput = bincode::deserialize(&input_bytes)
        .map_err(|e| {
            println!("Deserialization error: {:?}", e);
            e
        })
        .expect("Failed deserializing proof input");

    /*let client = ProverClient::new();
    let mut stdin = SP1Stdin::new();
    stdin.write(&input);
    client.execute(KECCAK_INCLUSION_ELF, stdin).run().expect("Failed executing program");*/
}