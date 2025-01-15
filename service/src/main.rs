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
use clap::{Parser};
use nmt_rs::{
    simple_merkle::{db::MemDb, proof::Proof, tree::{MerkleTree, MerkleHash}},
    TmSha2Hasher,
};
use sp1_sdk::{ProverClient, SP1Stdin};

use eq_common::KeccakInclusionToDataRootProof;

mod utils;
use utils::{create_inclusion_proof_input};

// Using rkyv for serialization
#[derive(Archive, RkyvDeserialize, RkyvSerialize)]
pub struct Job {
    pub height: u64,
    pub namespace: Vec<u8>,
    pub commitment: Vec<u8>,
}

pub struct InclusionService {
    client: Arc<Client>,
    db: sled::Db,
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
            .map_err(|e| Status::internal(format!("Failed to get header: {}", e.to_string())))?;

        let nmt_multiproofs = self.client
            .blob_get_proof(height, namespace, commitment)
            .await
            .map_err(|e| Status::internal(format!("Failed to get blob proof: {}", e.to_string())))?;

        let inclusion_proof_input = create_inclusion_proof_input(&blob, &header, nmt_multiproofs)?;

        Ok(Response::new(GetKeccakInclusionResponse { status: 0 }))
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    db_path: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let args = Args::parse();
    let db = sled::open(args.db_path)?;

    let node_token = std::env::var("CELESTIA_NODE_AUTH_TOKEN").expect("Token not provided");
    let client = Client::new("ws://localhost:26658", Some(&node_token))
        .await
        .expect("Failed creating celestia rpc client");

    let addr = "[::1]:50051".parse()?;
    let inclusion_service = InclusionService{
        client: Arc::new(client),
        db: db,
    };

    Server::builder()
        .add_service(InclusionServer::new(inclusion_service))
        .serve(addr)
        .await?;

    Ok(())
}