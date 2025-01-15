use tonic::Status;
use nmt_rs::{
    simple_merkle::{db::MemDb, proof::Proof, tree::{MerkleTree, MerkleHash}},
    TmSha2Hasher,
};
use tendermint::{hash::Hash as TmHash};
use tendermint_proto::{
    v0_37::{types::BlockId as RawBlockId, version::Consensus as RawConsensusVersion},
    Protobuf,
};
use celestia_types::{nmt::{NamespaceProof, NamespacedHashExt}, blob::Blob, ExtendedHeader};
use std::cmp::max;
use eq_common::KeccakInclusionToDataRootProofInput;
use sha3::{Keccak256, Digest};
pub fn create_inclusion_proof_input(blob: &Blob, header: &ExtendedHeader, nmt_multiproofs: Vec<NamespaceProof>) -> Result<KeccakInclusionToDataRootProofInput, Status> {

    let eds_row_roots = header.dah.row_roots();
    let eds_column_roots = header.dah.column_roots();

    // Compute these values needed for proving inclusion
    let eds_size: u64 = eds_row_roots.len().try_into().unwrap();
    let ods_size = eds_size / 2;

    let blob_index = blob.index.ok_or_else(|| Status::internal("Blob index not found"))?;
    let blob_size: u64 = max(1, blob.to_shares().unwrap().len() as u64);
    let first_row_index: u64 = blob_index.div_ceil(eds_size) - 1;
    let ods_index = blob_index - (first_row_index * ods_size);

    let last_row_index: u64 = (ods_index + blob_size).div_ceil(ods_size) - 1;

    let hasher = TmSha2Hasher {};
    let mut row_root_tree: MerkleTree<MemDb<[u8; 32]>, TmSha2Hasher> =
        MerkleTree::with_hasher(hasher);

    let leaves = eds_row_roots
        .iter()
        .chain(eds_column_roots.iter())
        .map(|root| root.to_array())
        .collect::<Vec<[u8; 90]>>();

    for root in &leaves {
        row_root_tree.push_raw_leaf(root);
    }

    // assert that the row root tree equals the data hash
    assert_eq!(row_root_tree.root(), header.header.data_hash.unwrap().as_bytes());
    // Get range proof of the row roots spanned by the blob
    // +1 is so we include the last row root
    let row_root_multiproof =
        row_root_tree.build_range_proof(first_row_index as usize..(last_row_index + 1) as usize);
    // Sanity check, verify the row root range proof
    let hasher = TmSha2Hasher {};
    let leaves_hashed = leaves
        .iter()
        .map(|leaf| hasher.hash_leaf(leaf))
        .collect::<Vec<[u8; 32]>>();
    row_root_multiproof
        .verify_range(
            header.header.data_hash.unwrap().as_bytes().try_into().unwrap(),
            &leaves_hashed[first_row_index as usize..(last_row_index + 1) as usize],
        )
        .map_err(|_| Status::internal("Failed sanity check on row root inclusion multiproof"))?;

    let mut hasher = Keccak256::new();
    hasher.update(&blob.data);
    let hash: [u8; 32] = hasher.finalize().try_into().map_err(|_| Status::internal("Failed to convert keccak hash to array"))?;

    Ok(KeccakInclusionToDataRootProofInput {
        blob: blob.clone(),
        keccak_hash: hash,
        nmt_multiproofs,
        row_root_multiproof,
        row_roots: eds_row_roots.to_vec(),
        data_root: header.header.data_hash.unwrap().as_bytes().try_into().unwrap(),
    })
}

pub fn create_header_field_tree(header: &ExtendedHeader) -> (MerkleTree<MemDb<[u8; 32]>, TmSha2Hasher>, Proof<TmSha2Hasher>) {
    let hasher = TmSha2Hasher {};
    let mut header_field_tree: MerkleTree<MemDb<[u8; 32]>, TmSha2Hasher> =
        MerkleTree::with_hasher(hasher);

    let field_bytes = vec![
        Protobuf::<RawConsensusVersion>::encode_vec(header.header.version),
        header.header.chain_id.clone().encode_vec(),
        header.header.height.encode_vec(),
        header.header.time.encode_vec(),
        Protobuf::<RawBlockId>::encode_vec(header.header.last_block_id.unwrap_or_default()),
        header
            .header
            .last_commit_hash
            .unwrap_or_default()
            .encode_vec(),
        header.header.data_hash.unwrap_or_default().encode_vec(),
        header.header.validators_hash.encode_vec(),
        header.header.next_validators_hash.encode_vec(),
        header.header.consensus_hash.encode_vec(),
        header.header.app_hash.clone().encode_vec(),
        header
            .header
            .last_results_hash
            .unwrap_or_default()
            .encode_vec(),
        header.header.evidence_hash.unwrap_or_default().encode_vec(),
        header.header.proposer_address.encode_vec(),
    ];

    for leaf in field_bytes {
        header_field_tree.push_raw_leaf(&leaf);
    }

    let (data_hash_bytes_from_tree, data_hash_proof) = header_field_tree.get_index_with_proof(6);
    
    // Verify the data hash
    let data_hash_from_tree = TmHash::decode_vec(&data_hash_bytes_from_tree).unwrap();
    assert_eq!(
        data_hash_from_tree.as_bytes(),
        header.header.data_hash.unwrap().as_bytes()
    );
    assert_eq!(header.hash().as_ref(), header_field_tree.root());

    // Verify the proof
    let hasher = TmSha2Hasher {};
    data_hash_proof
        .verify_range(
            &header_field_tree.root(),
            &[hasher.hash_leaf(&data_hash_bytes_from_tree)],
        )
        .unwrap();

    (header_field_tree, data_hash_proof)
}