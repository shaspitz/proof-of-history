use sha256::digest;

fn main() {
    let input = "hello";
    let val = digest(input);
    println!("{} => {}", input, val)
}

