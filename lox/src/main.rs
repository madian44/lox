fn main() {
    println!("Hello, world!");

    match lox::run("text to scan") {
        Ok(m) => println!("Yippee: {m}"),
        Err(m) => println!("Hmm, is this expected?: {m}"),
    }
}
