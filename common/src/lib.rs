use celestia_types::nmt::{NamespaceProof, NamespacedHash};
use nmt_rs::{TmSha2Hasher, simple_merkle::proof::Proof};
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
    pub nmt_multiproofs: Vec<NamespaceProof>,
    pub row_root_multiproof: Proof<TmSha2Hasher>,
    // these types are wrong.
    // TODO: fix this
    pub row_roots: Vec<NamespacedHash>,
    pub data_root: NamespacedHash,
    pub keccak_hash: [u8; 32],
}

pub struct KeccakInclusionToDataRootProofOutput {
    pub keccak_hash: [u8; 32],
    pub data_root: NamespacedHash,
}
