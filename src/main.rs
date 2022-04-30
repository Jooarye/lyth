mod lexer;

fn main() {
    let inp = std::fs::read_to_string("test.ly").unwrap();
    let input = inp.as_ref();
    let lex = lexer::Lexer::new("test.ly", input);

    for tok in lex {
        println!("{}: {:?} {}", tok.loc, tok.kind, tok.text);
    }
}
