use std::thread::{self, JoinHandle};
use std::time::Instant;
use sha256::digest;

fn main() {
    let input = String::from("Genesis string");

    let solve_start = Instant::now();
    let proof = solve_with_proof(input, 10000);
    let solve_duration = solve_start.elapsed();

    let verify_start = Instant::now();
    let valid = verify(proof[0].clone(), proof.clone(), 10000);
    let verify_duration = verify_start.elapsed();
    println!("Final proof: {:?}", proof[9999]);
    println!("Valid: {:?}", valid);
    println!("Solve duration: {:?}", solve_duration);
    println!("Verify duration: {:?}", verify_duration);

    // invalid proof 
    // TODO: Make UT
    let mut invalid = proof.clone();
    invalid[998] = String::from("invalid");

    let valid = verify(invalid[0].clone(), invalid.clone(), 10000);
    println!("Valid: {:?}", valid)
}


// solve_with_proof hashes the challenge string a set amount of times 
// while storing all hashes in the process. This serves as Solana's simple VDF (verifiable delay function).
// This function has to run sequentially on a single core by nature. 
//
// TODO: Expand this to accept arbitrary txs in indexed hashes. 
fn solve_with_proof(challenge: String, difficulty: u64) -> Vec<String> {
    let mut hash = challenge;
    let mut proof = Vec::with_capacity(difficulty as usize);
    for _ in 0..difficulty {
        hash = digest(hash);
        proof.push(hash.clone()); // Pass clone to vector, as hash has been moved and is needed for future iterations.
    }
    proof
}

// verify takes a challenge string, a proof, and a difficulty and returns true if the proof is valid.
// Parallelization allows verification to be faster than solving.
//
// Note OS threads are used for concurrency here, 
// but async may be more appropriate: https://rust-lang.github.io/async-book/01_getting_started/01_chapter.html. 
fn verify(challenge: String, proof: Vec<String>, difficulty: u64) -> bool {

    if proof[0] != challenge {
        return false
    }

    let mut handles = Vec::with_capacity(difficulty as usize);

    // Assume 8 CPU cores, divide computations into 8 threads. 
    // Theoretically you could parallelize these computations further with CUDA or similar. 
    let chunk_size:usize = (difficulty as usize) / 8;
    let chunks = proof.chunks(chunk_size);

    for chunk in chunks {
        let chunk_owned = chunk.to_owned();
        let handle: JoinHandle<bool>= thread::spawn(move || {
            for idx in 0..chunk_owned.len()-1 {
                let input = chunk_owned[idx].to_owned();
                let expected_output = chunk_owned[idx + 1].to_owned();
                let hash = digest(input);
                if hash != expected_output {
                    return false
                }
            }
            return true;
        });
        handles.push(handle);
    }

    // Wait for threads to finish and return false if any hashes are invalid.
    for handle in handles {
        if !handle.join().unwrap() {
            return false
        }
    }

    true
}
