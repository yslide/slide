use libslide::{evaluate, parse, scan, ScannerOptions};

use std::env;

fn main() -> Result<(), &'static str> {
    let program = match env::args().nth(1) {
        Some(prog) => prog,
        None => {
            return Err("Must supply a program.");
        }
    };

    let tokens = scan(program, ScannerOptions::default());
    let parse_tree = parse(tokens);
    let simplified = evaluate(parse_tree);

    println!("{}", simplified.to_string());

    Ok(())
}
