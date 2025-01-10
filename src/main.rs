use std::sync::Arc;
use tonic::{transport::Server, Request, Response, Status};

pub mod eqs {
    include!("generated/eqs.rs");
}
use eqs::eq_server::{Eq, EqServer};
use eqs::{GetKeccakInclusionRequest, GetKeccakInclusionResponse};

use celestia_rpc::{BlobClient, Client, HeaderClient};
use celestia_types::nmt::Namespace;
use celestia_types::blob::Commitment;
use tendermint::{hash::Algorithm, Hash as TmHash};
use tendermint_proto::{
    v0_37::{types::BlockId as RawBlockId, version::Consensus as RawConsensusVersion},
    Protobuf,
};

use nmt_rs::{
    simple_merkle::{db::MemDb, proof::Proof, tree::{MerkleTree, MerkleHash}},
    TmSha2Hasher,
};

pub struct EqService {
    client: Arc<Client>,
}

#[tonic::async_trait]
impl Eq for EqService {
    async fn get_keccak_inclusion(
        &self,
        request: Request<GetKeccakInclusionRequest>,
    ) -> Result<Response<GetKeccakInclusionResponse>, Status> {
        let request = request.into_inner();
        let height = request.height;
        let commitment = Commitment::new(
            request.commitment
            .try_into()
            .map_err(|_| Status::invalid_argument("Invalid commitment"))?
        );
        let namespace = Namespace::from_raw(&request.namespace)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let blob = self.client.blob_get(height, namespace, commitment).await
            .map_err(|e| Status::internal(e.to_string()))?;

        // Get the ExtendedHeader
        let header = self.client.header_get_by_height(height)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let eds_row_roots = header.dah.row_roots();
        let eds_column_roots = header.dah.column_roots();
    
        // Compute these values needed for proving inclusion
        let eds_size: u64 = eds_row_roots.len().try_into().unwrap();
        let ods_size = eds_size / 2;

        // Header hash is a merkle tree of the header fields
        // We can use this to prove the data hash is in the celestia header
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

        let computed_header_hash = header_field_tree.root();
        let (data_hash_bytes_from_tree, data_hash_proof) = header_field_tree.get_index_with_proof(6);
        let data_hash_from_tree = TmHash::decode_vec(&data_hash_bytes_from_tree).unwrap();
        assert_eq!(
            data_hash_from_tree.as_bytes(),
            header.header.data_hash.unwrap().as_bytes()
        );
        assert_eq!(header.hash().as_ref(), header_field_tree.root());

        // Sanity check, verify the data hash merkle proof
        let hasher = TmSha2Hasher {};
        data_hash_proof
            .verify_range(
                &header_field_tree.root(),
                &[hasher.hash_leaf(&data_hash_bytes_from_tree)],
            )
            .unwrap();

        Ok(Response::new(GetKeccakInclusionResponse { status: 0 }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let node_token = std::env::var("CELESTIA_NODE_AUTH_TOKEN").expect("Token not provided");
    let client = Client::new("ws://localhost:26658", Some(&node_token))
        .await
        .expect("Failed creating celestia rpc client");

    let addr = "[::1]:50051".parse()?;
    let eq = EqService{
        client: Arc::new(client),
    };

    Server::builder()
        .add_service(EqServer::new(eq))
        .serve(addr)
        .await?;

    Ok(())
}