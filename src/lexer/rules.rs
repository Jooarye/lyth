use super::token::TokenKind;
use lazy_static::*;
use regex::Regex;

macro_rules! rule {
    ($k: expr, $a: expr) => {
        Rule {
            kind: $k,
            matches: |input| match_one(input, $a),
        }
    };

    ($k: expr, $a: expr, $b: expr) => {
        Rule {
            kind: $k,
            matches: |input| match_two(input, $a, $b),
        }
    };
}

fn match_one(input: &str, c: char) -> Option<usize> {
    input
        .chars()
        .next()
        .and_then(|ch| if ch == c { Some(1) } else { None })
}

fn match_two(input: &str, a: char, b: char) -> Option<usize> {
    if input.len() >= 2 {
        match_one(input, a).and_then(|_| match_one(&input[1..], b).map(|_| 2))
    } else {
        None
    }
}

fn match_word(input: &str, word: &str) -> Option<usize> {
    input.starts_with(word).then(|| word.len())
}

fn match_regex(input: &str, r: &Regex) -> Option<usize> {
    r.find(input).map(|regex_match| regex_match.end())
}

pub struct Rule {
    pub kind: TokenKind,
    pub matches: fn(&str) -> Option<usize>,
}

lazy_static! {
    static ref STRING_REGEX: Regex = Regex::new(r#"^"((\\"|\\\\)|[^\\"])*""#).unwrap();
    static ref INTEGER_REGEX: Regex =
        Regex::new(r#"^((0o[0-7]+)|(0b[01]+)|(0x[0-9A-Fa-f]+)|([0-9]+))"#).unwrap();
    static ref IDENTIFIER_REGEX: Regex = Regex::new(r##"^([A-Za-z]|_)([A-Za-z]|_|\d)*"##).unwrap();
}

pub fn get_rules() -> Vec<Rule> {
    vec![
        rule!(TokenKind::Equal, '=', '='),
        rule!(TokenKind::UnEqual, '!', '='),
        rule!(TokenKind::LessThan, '<'),
        rule!(TokenKind::GreaterThan, '>'),
        rule!(TokenKind::LessEqual, '<', '='),
        rule!(TokenKind::GreaterEqual, '>', '='),
        rule!(TokenKind::Plus, '+'),
        rule!(TokenKind::Minus, '-'),
        rule!(TokenKind::Asterisk, '*'),
        rule!(TokenKind::Slash, '/'),
        rule!(TokenKind::Percent, '%'),
        rule!(TokenKind::And, '&'),
        rule!(TokenKind::Pipe, '|'),
        rule!(TokenKind::Caret, '^'),
        rule!(TokenKind::Bang, '!'),
        rule!(TokenKind::Tilde, '~'),
        rule!(TokenKind::Assign, '='),
        rule!(TokenKind::Dot, '.'),
        rule!(TokenKind::Comma, ','),
        rule!(TokenKind::Colon, ':'),
        rule!(TokenKind::SemiColon, ';'),
        rule!(TokenKind::OpenParen, '('),
        rule!(TokenKind::ClosedParen, ')'),
        rule!(TokenKind::OpenBrace, '{'),
        rule!(TokenKind::ClosedBrace, '}'),
        Rule {
            kind: TokenKind::Struct,
            matches: |input| match_word(input, "struct"),
        },
        Rule {
            kind: TokenKind::Function,
            matches: |input| match_word(input, "fn"),
        },
        Rule {
            kind: TokenKind::Let,
            matches: |input| match_word(input, "let"),
        },
        Rule {
            kind: TokenKind::If,
            matches: |input| match_word(input, "if"),
        },
        Rule {
            kind: TokenKind::Else,
            matches: |input| match_word(input, "else"),
        },
        Rule {
            kind: TokenKind::For,
            matches: |input| match_word(input, "for"),
        },
        Rule {
            kind: TokenKind::Loop,
            matches: |input| match_word(input, "loop"),
        },
        Rule {
            kind: TokenKind::Break,
            matches: |input| match_word(input, "break"),
        },
        Rule {
            kind: TokenKind::Continue,
            matches: |input| match_word(input, "continue"),
        },
        Rule {
            kind: TokenKind::Return,
            matches: |input| match_word(input, "return"),
        },
        Rule {
            kind: TokenKind::Inline,
            matches: |input| match_word(input, "inline"),
        },
        Rule {
            kind: TokenKind::Boolean,
            matches: |input| match_word(input, "true"),
        },
        Rule {
            kind: TokenKind::Boolean,
            matches: |input| match_word(input, "false"),
        },
        Rule {
            kind: TokenKind::Identifier,
            matches: |input| match_regex(input, &IDENTIFIER_REGEX),
        },
        Rule {
            kind: TokenKind::String,
            matches: |input| match_regex(input, &STRING_REGEX),
        },
        Rule {
            kind: TokenKind::Integer,
            matches: |input| match_regex(input, &INTEGER_REGEX),
        },
    ]
}
