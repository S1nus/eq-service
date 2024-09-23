# Take-Home Assignment
Part of Solutions Engineering at Celestia Labs involves on-boarding to new codebases and tech stacks to integrate them with Celestia. This asssignment is meant to test your ability to on-board to a new codebase, in a new language and tech stack. Here you'll find a server with a web API interface that allows a user to submit jobs, and check the status of those jobs.

This is a simplified, educational version of a real service we are currently developing, which computes zero-knowledge "commitment equivalence" proofs for usage in integrations with certain rollups.

# Your task
As it's written, this service keeps all of its data in memory, but we need the service to persist the completed proofs to disk, in case the service crashes and to index the completed proofs.
1. [Install Rust](https://rustup.rs/)
2. Clone and run this repo
    - Manually test it, to verify that it works
    - On a Mac, these curl commands should work:
    ```
    curl localhost:8080/add_job\?height=64\&namespace=deadbeef\&commitment=2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b981e
    curl localhost:8080/get_job\?commitment=2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b981e
    ```
3. Upgrade the repo to persist the completed proofs to a datastore.
    - Rust has many options for a datastore, such as [RocksDB](https://github.com/rust-rocksdb/rust-rocksdb) and [Sled](https://github.com/spacejam/sled). Choose whichever datastore you think is best for the task.