extern crate proof;
extern crate time;
extern crate rand;

use rand::{Rng};

fn main () {
    let input = rand::thread_rng()
        .gen_ascii_chars()
        .take(10)
        .collect::<String>();
    let before = time::precise_time_ns();
    let x = proof::get_proof(&input.into_bytes(), 2);
    let elapsed = (time::precise_time_ns() - before) as f64;
    let as_ms = elapsed/1000000.0;
    println!("{}, took {}ms", x, as_ms);
}
