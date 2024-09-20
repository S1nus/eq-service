pub type Proof = Vec<u8>;

pub fn generate_proof(input: &[u8]) -> Proof {
    // Wait 5 seconds, then return a random 32 byte array
    thread::sleep(Duration::from_secs(5));
    let mut rng = rand::thread_rng();
    let proof = (0..32).map(|_| rng.gen::<u8>()).collect();
    proof
}