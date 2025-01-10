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

mod utils;
use utils::create_header_field_tree;

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

        let nmt_multiproofs = self.client
            .blob_get_proof(height, namespace, commitment)
            .await
            .map_err(|e| Status::internal(format!("Failed to get blob proof: {}", e)))?;

        let (_header_field_tree, _data_hash_proof) = create_header_field_tree(&header);

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