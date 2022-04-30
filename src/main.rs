mod lexer;
mod parser;

fn main() {
    let inp = std::fs::read_to_string("test.ly").unwrap();
    let input = inp.as_ref();
    let mut par = parser::Parser::new("test.ly", input);

    let ast = par.parse();

    for d in ast {
        println!("{:?}", d);
    }
}
