use celestia_types::{nmt::NamespaceProof, blob::Blob};
use nmt_rs::{TmSha2Hasher, simple_merkle::proof::Proof, NamespacedHash};
use serde::{Deserialize, Serialize};

/*
    The types of proofs we expect to support:
    1. KeccakInclusionToDataRootProof
    2. KeccakInclusionToBlockHashProof
    3. PayyPoseidonToDataRootProof
    4. PayyPoseidonToBlockHashProof
*/

#[derive(Serialize, Deserialize)]
pub struct KeccakInclusionToDataRootProofInput {
    pub blob: Blob,
    pub nmt_multiproofs: Vec<NamespaceProof>,
    pub row_root_multiproof: Proof<TmSha2Hasher>,
    pub row_roots: Vec<NamespacedHash<29>>,
    pub data_root: Vec<u8>,
    pub keccak_hash: [u8; 32],
}

#[derive(Serialize, Deserialize)]
pub struct KeccakInclusionToDataRootProofOutput {
    pub keccak_hash: [u8; 32],
    pub data_root: Vec<u8>,
}
