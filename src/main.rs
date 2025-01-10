use std::sync::Arc;
use tonic::{transport::Server, Request, Response, Status};

pub mod eqs {
    include!("generated/eqs.rs");
}
use eqs::eq_server::{Eq, EqServer};
use eqs::{SubmitJobRequest, SubmitJobResponse};

use celestia_rpc::{BlobClient, Client};
use celestia_types::nmt::Namespace;
use celestia_types::blob::Commitment;

pub struct EqService {
    client: Arc<Client>,
}

#[tonic::async_trait]
impl Eq for EqService {
    async fn submit_job(
        &self,
        request: Request<SubmitJobRequest>,
    ) -> Result<Response<SubmitJobResponse>, Status> {
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
        Ok(Response::new(SubmitJobResponse { status: 0 }))
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