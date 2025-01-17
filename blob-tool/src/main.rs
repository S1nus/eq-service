use celestia_rpc::{BlobClient, Client, HeaderClient};
use clap::{command, Parser};
use celestia_types::nmt::Namespace;
use celestia_types::blob::Commitment;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    height: u64,
    #[arg(short, long)]
    namespace: String,
    #[arg(short, long)]
    commitment: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let node_token = std::env::var("CELESTIA_NODE_AUTH_TOKEN").expect("Token not provided");
    let client = Client::new("ws://localhost:26658", Some(&node_token))
        .await
        .expect("Failed creating celestia rpc client");

    let header = client.header_get_by_height(args.height)
        .await
        .expect("Failed getting header");

    let commitment = Commitment::new(
        hex::decode(&args.commitment)
            .expect("Invalid commitment hex")
            .try_into()
            .expect("Invalid commitment length")
    );
    let namespace = Namespace::from_raw(args.namespace.as_bytes())
        .expect("Invalid namespace");

    let blob = client.blob_get(args.height, namespace, commitment)
        .await
        .expect("Failed getting blob");

    let nmt_multiproofs = client.blob_get_proof(args.height, namespace, commitment)
        .await
        .expect("Failed getting nmt multiproofs");

    let input = create_inclusion_proof_input(&blob, &header, nmt_multiproofs)?;

}