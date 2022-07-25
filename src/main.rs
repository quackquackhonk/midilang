use midilang;
use std::{env, collections::BinaryHeap};

fn main() {
    let arguments: Vec<String> = env::args().collect();

    if arguments.len() < 2 {
        // UNIMPLEMENTED
        midilang::run_interactive();
    } else {
        midilang::run(&arguments[1]);
    }
    
}