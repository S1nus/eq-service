use nmt_rs::{
    simple_merkle::{db::MemDb, proof::Proof, tree::{MerkleTree, MerkleHash}},
    TmSha2Hasher,
};
use tendermint::{hash::Hash as TmHash};
use tendermint_proto::{
    v0_37::{types::BlockId as RawBlockId, version::Consensus as RawConsensusVersion},
    Protobuf,
};

pub fn create_header_field_tree(header: &celestia_types::ExtendedHeader) -> (MerkleTree<MemDb<[u8; 32]>, TmSha2Hasher>, Proof<TmSha2Hasher>) {
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