mod compiler;
mod lexer;
mod parser;

fn main() {
    let inp = std::fs::read_to_string("test.ly").unwrap();
    let input = inp.as_ref();
    let mut c = compiler::Compiler::new("test.ly", input);

    c.compile("a.out");
}
