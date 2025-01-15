#![no_main]
sp1_zkvm::entrypoint!(main);
use eq_common::{KeccakInclusionToDataRootProofInput, KeccakInclusionToDataRootProofOutput};
use celestia_types::blob::Blob;

pub fn main() {
    let input: KeccakInclusionToDataRootProofInput = sp1_zkvm::io::read();
    let blob: Blob = sp1_zkvm::io::read();
}
