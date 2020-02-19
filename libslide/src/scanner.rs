mod types;

pub struct Scanner {
    input: String, 
    output: Vec<token>
}

impl Scanner {
    pub fn new(input: &str) -> Scanner {
        Scanner{
            input: input.to_owned(),
            output: Vec::new()
        }
    }

    pub fn scan(&mut self){

