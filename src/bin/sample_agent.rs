use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    // Read initial input
    let header_line = lines.next().unwrap().unwrap();
    let header: Vec<&str> = header_line.split_whitespace().collect();
    let m: usize = header[1].parse().unwrap();
    let t: usize = header[3].parse().unwrap();

    for _ in 0..t {
        // Read current floors
        let _ = lines.next().unwrap().unwrap();

        // Read elevator states
        for _ in 0..m {
            let _ = lines.next().unwrap().unwrap();
        }

        // Read floor states
        let n = 10; // fixed N
        for _ in 0..n {
            let _ = lines.next().unwrap().unwrap();
        }

        // Output actions
        for _ in 0..m {
            println!("STAY");
        }
    }
}
