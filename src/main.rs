use actix_web::{App, HttpServer, web, Responder, HttpResponse};
use actix_files as fs;
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::collections::{VecDeque, HashMap};
use serde::{Serialize, Deserialize};
use celestia_types::{Commitment};
use sp1_sdk::SP1ProofWithPublicValues;
use hex;
use celestia_rpc::{BlobClient, Client};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Job {
    commitment: Commitment,
    hash: Option<[u8; 32]>,
    result: Option<SP1ProofWithPublicValues>,
    status: JobStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum JobStatus {
    InQueue,
    Proving,
    Completed,
    Failed,
}

type CommitmentHash = [u8; 32];

pub struct AppState {
    job_queue: Arc<Mutex<VecDeque<Job>>>,
    job_statuses: Arc<Mutex<HashMap<CommitmentHash, Job>>>,
    client: Arc<Client>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobInfo {
    pub commitment: Commitment,
    pub height: u64,
}

async fn add_job(data: web::Data<AppState>, query: web::Query<HashMap<String, String>>) -> impl Responder {
    // Get the Client from the AppState, fetch the blob's data and namespace
    let client = &data.client;
    let blob_data = client.blob_get(height, namespace, commitment).await;


    println!("Adding job");
    // Parse commitment and height from query parameters
    let commitment = match query.get("commitment").and_then(|c| hex::decode(c).ok()) {
        Some(c) if c.len() == 32 => Commitment(c.try_into().unwrap()),
        _ => return HttpResponse::BadRequest().json("Invalid commitment"),
    };
    // this is unused for now, we will eventually use it to download the blob
    let height = match query.get("height").and_then(|h| h.parse::<u64>().ok()) {
        Some(h) => h,
        None => return HttpResponse::BadRequest().json("Invalid height"),
    };

    // Check if we have a job for this commitment, if it exists, return the job
    let mut job_statuses = data.job_statuses.lock().unwrap();
    if job_statuses.contains_key(&commitment.0) {
        return HttpResponse::Ok().json(job_statuses[&commitment.0].clone());
    }

    // Otherwise, create a job and add it to the back of the queue
    let job = Job {
        commitment,
        hash: None,
        result: None,
        status: JobStatus::InQueue,
    };
    data.job_queue.lock().unwrap().push_back(job.clone());
    job_statuses.insert(commitment.0, job.clone());
    HttpResponse::Ok().json(job)
}

async fn get_job(data: web::Data<AppState>, query: web::Query<HashMap<String, String>>) -> impl Responder {
    println!("Getting job");
    let commitment_hash: CommitmentHash = match query.get("commitment").and_then(|c| hex::decode(c).ok()) {
        Some(c) if c.len() == 32 => c.try_into().unwrap(),
        _ => return HttpResponse::BadRequest().json("Invalid commitment hash"),
    };

    let job_statuses = data.job_statuses.lock().unwrap();
    if let Some(job) = job_statuses.get(&commitment_hash) {
        HttpResponse::Ok().json(job)
    } else {
        HttpResponse::NotFound().json(format!("Job with commitment hash {} not found", hex::encode(commitment_hash)))
    }
}

fn start_worker(app_state: web::Data<AppState>) {
    println!("Starting worker");
    let state = app_state.clone();
    thread::spawn(move || {
        loop {
            let job = {
                let mut queue = state.job_queue.lock().unwrap();
                queue.pop_front()
            };
            // Simulate a job being processed by sleeping, then updating the job status
            if let Some(mut job) = job {
                println!("Processing job: {:?}", job);
                thread::sleep(Duration::from_secs(5));
                job.status = JobStatus::Completed;
                println!("Job completed: {:?}", job);

                let mut job_statuses = state.job_statuses.lock().unwrap();
                job_statuses.insert(job.commitment.0, job.clone());
            }
        }
    });
}
    

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let node_token = std::env::var("CELESTIA_NODE_AUTH_TOKEN").expect("Token not provided");
    let client = Client::new("ws://localhost:26658", Some(&node_token))
        .await
        .expect("Failed creating rpc client");


    let app_state = web::Data::new(AppState {
        job_queue: Arc::new(Mutex::new(VecDeque::new())),
        job_statuses: Arc::new(Mutex::new(HashMap::new())),
        client: Arc::new(client),
    });

    start_worker(app_state.clone());

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/add_job", web::get().to(add_job))
            .route("/get_job", web::get().to(get_job))
            .service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}