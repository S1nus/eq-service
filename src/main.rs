use std::sync::Arc;
use tonic::{transport::Server, Request, Response, Status};

pub mod eqs {
    include!("generated/eqs.rs");
}
use eqs::inclusion_server::{Inclusion, InclusionServer};
use eqs::{GetKeccakInclusionRequest, GetKeccakInclusionResponse};

use celestia_rpc::{BlobClient, Client, HeaderClient};
use celestia_types::nmt::{Namespace, NamespacedHashExt};
use celestia_types::blob::Commitment;
use tendermint::{hash::Algorithm, Hash as TmHash};
use tendermint_proto::{
    v0_37::{types::BlockId as RawBlockId, version::Consensus as RawConsensusVersion},
    Protobuf,
};
use std::cmp::max;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize };

use nmt_rs::{
    simple_merkle::{db::MemDb, proof::Proof, tree::{MerkleTree, MerkleHash}},
    TmSha2Hasher,
};

mod utils;
use utils::create_header_field_tree;

// Using rkyv for serialization
#[derive(Archive, RkyvDeserialize, RkyvSerialize)]
pub struct Job {
    pub height: u64,
    pub namespace: Vec<u8>,
    pub commitment: Vec<u8>,
}

pub struct InclusionService {
    client: Arc<Client>,
}

#[tonic::async_trait]
impl Inclusion for InclusionService {
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

        let nmt_multiproofs = self.client
            .blob_get_proof(height, namespace, commitment)
            .await
            .map_err(|e| Status::internal(format!("Failed to get blob proof: {}", e)))?;

        let (mut header_field_tree, _data_hash_proof) = create_header_field_tree(&header);
        let data_hash_from_tree = TmHash::decode_vec(&header_field_tree.root())
            .map_err(|e| Status::internal(format!("Failed to decode data hash: {}", e)))?;

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
        assert_eq!(row_root_tree.root(), data_hash_from_tree.as_bytes());
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
                data_hash_from_tree.as_bytes().try_into().unwrap(),
                &leaves_hashed[first_row_index as usize..(last_row_index + 1) as usize],
            )
            .map_err(|_| Status::internal("Failed sanity check on row root inclusion multiproof"))?;

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
    let eq = InclusionService{
        client: Arc::new(client),
    };

    Server::builder()
        .add_service(InclusionServer::new(eq))
        .serve(addr)
        .await?;

    Ok(())
}