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
// TODO: look into cloning
//
// TODO: Expand this to accept arbitrary txs in indexed hashes. 
fn solve_with_proof(challenge: String, difficulty: u64) -> Vec<String> {
    let mut hash = challenge;
    let mut proof = Vec::with_capacity(difficulty as usize);
    for _ in 0..difficulty {
        hash = digest(hash.clone());
        proof.push(hash.clone());  
    }
    return proof
}

// verify takes a challenge string, a proof, and a difficulty and returns true if the proof is valid.
// Parallelization allows verification to be faster than solving.
fn verify(challenge: String, proof: Vec<String>, difficulty: u64) -> bool {

    if proof[0] != challenge {
        return false
    }

    let mut handles = Vec::with_capacity(difficulty as usize);

    // Assume 8 CPU cores, divide computations into 8 threads. 
    // Theoretically you could parallelize these computations further with CUDA or similar. 
    let chunk_size:usize = (difficulty as usize) / 8;
    let chunks_owned: Vec<Vec<String>> = proof.chunks(chunk_size).map(|chunk| chunk.to_owned()).collect();

    for chunk in chunks_owned {
        let handle: JoinHandle<bool>= thread::spawn(move || {
            for idx in 0..chunk.len()-1 {
                let input_cln = chunk[idx].clone();
                let output_cln = chunk[idx + 1].clone();
                let hash = digest(input_cln);
                if hash != output_cln {
                    return false
                }
            }
            return true;
        });
        handles.push(handle);
    }

    for handle in handles {
        if !handle.join().unwrap() {
            return false
        }
    }

    return true
}