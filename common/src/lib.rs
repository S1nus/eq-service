use celestia_types::nmt::{NamespaceProof};
use nmt_rs::{TmSha2Hasher, simple_merkle::proof::Proof};

pub struct InclusionProofInput {
    pub nmt_multiproofs: Vec<NamespaceProof>,
    pub row_root_multiproof: Proof<TmSha2Hasher>,
}