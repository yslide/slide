use libslide::{evaluate, parse, scan, ParsingStrategy};

use std::env;

fn main() -> Result<(), String> {
    let program = match env::args().nth(1) {
        Some(prog) => prog,
        None => {
            return Err("Must supply a program.".into());
        }
    };

    let tokens = scan(program);
    let (parse_tree, errors) = parse(tokens, ParsingStrategy::Expression);
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    let simplified = evaluate(parse_tree);

    println!("{}", simplified.to_string());

    Ok(())
}
