use libslide::{evaluate, parse, scan};

use std::env;

fn main() -> Result<(), &'static str> {
    let program = match env::args().nth(1) {
        Some(prog) => prog,
        None => {
            return Err("Must supply a program.");
        }
    };

    let parse_tree = parse(scan(program));
    let simplified = evaluate(parse_tree);

    println!("{}", simplified.to_string());

    Ok(())
}
