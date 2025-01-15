#![no_main]
sp1_zkvm::entrypoint!(main);
use eq_common::{KeccakInclusionToDataRootProof};
use celestia_types::blob::Blob;

pub fn main() {
    let input: KeccakInclusionToDataRootProof = sp1_zkvm::io::read();
    let blob: Blob = sp1_zkvm::io::read();
}