use libslide::{evaluate, parse_expression, scan, EvaluatorContext};

use std::env;

fn main() -> Result<(), String> {
    let program = match env::args().nth(1) {
        Some(prog) => prog,
        None => {
            return Err("Must supply a program.".into());
        }
    };

    let tokens = scan(program);
    let (parse_tree, errors) = parse_expression(tokens);
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    // TODO: handle errors
    let simplified = evaluate(parse_tree, EvaluatorContext::default()).unwrap();

    println!("{}", simplified.to_string());

    Ok(())
}
