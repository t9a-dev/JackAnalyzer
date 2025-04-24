use anyhow::Result;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

const COMMENT_OUT_TOKEN: &str = "//";
// const KEYWORD_CLASS: &str = "class";
// const KEYWORD_CONSTRUCTOR: &str = "constructor";
// const KEYWORD_FUNCTION: &str = "function";
// const KEYWORD_METHOD: &str = "method";
// const KEYWORD_FIELD: &str = "field";
// const KEYWORD_STATIC: &str = "static";
// const KEYWORD_VAR: &str = "var";
// const KEYWORD_INT: &str = "int";
// const KEYWORD_CHAR: &str = "char";
// const KEYWORD_BOOLEAN: &str = "boolean";
// const KEYWORD_VOID: &str = "void";
// const KEYWORD_TRUE: &str = "true";
// const KEYWORD_FALSE: &str = "false";
// const KEYWORD_NULL: &str = "null";
// const KEYWORD_THIS: &str = "this";
// const KEYWORD_LET: &str = "let";
// const KEYWORD_DO: &str = "do";
// const KEYWORD_IF: &str = "if";
// const KEYWORD_ELSE: &str = "else";
// const KEYWORD_WHILE: &str = "while";
// const KEYWORD_RETURN: &str = "return";

pub enum TokenType {
    KeyWord,
    Symbol,
    Identifier,
    IntConst,
    StringConst,
}

pub enum KeyWord {
    Class,
    Method,
    Function,
    Constructor,
    Int,
    Boolean,
    Char,
    Void,
    Var,
    Static,
    Field,
    Let,
    Do,
    If,
    Else,
    While,
    Return,
    True,
    False,
    Null,
    This,
}

pub struct JackTokenizer {
    jack_code: Box<dyn BufRead>,
    current_token: Option<String>,
}

impl JackTokenizer {
    pub fn new(jack_file_path: &str) -> Self {
        Self {
            jack_code: Box::new(BufReader::new(
                File::open(Path::new(jack_file_path)).unwrap(),
            )),
            current_token: None,
        }
    }

    pub fn has_more_tokens(&mut self) -> Result<bool> {
        Ok(self.jack_code.fill_buf()?.iter().next().is_some())
    }

    pub fn advance(&mut self) -> Result<()> {
        // //で始まるコメント行と空白を無視して次の行を読み込む
        while self.has_more_tokens()? {
            self.current_token = match self.jack_code.as_mut().lines().next().unwrap() {
                Ok(line) if line.chars().all(char::is_whitespace) => None, //空白の場合は無視
                Ok(line) if line.trim().starts_with(COMMENT_OUT_TOKEN) => None, //コメント行の場合は無視
                Ok(line) => Some(line.trim().to_string()),
                Err(e) => panic!("tokenizer advance error: {:?}", e),
            };
            if self.current_token.is_some() {
                break;
            }
        }
        Ok(())
    }

    pub fn token_type(&self) -> Result<TokenType> {
        todo!()
    }

    pub fn keyword(&self) -> Result<KeyWord> {
        todo!()
    }

    pub fn symbol(&self) -> Result<String> {
        todo!()
    }

    pub fn identifer(&self) -> Result<String> {
        todo!()
    }

    pub fn int_val(&self) -> Result<u16> {
        todo!()
    }

    pub fn string_val(&self) -> Result<String> {
        todo!()
    }
}

fn parse_tokens(input: &str) -> Result<Vec<String>> {
    let mut tokens: Vec<String> = Vec::new();
    // todo to const
    let symbols = vec!['=', '(', ')', '<', '>', '{', '}', ';'];

    // ignore comment line
    let _input = input
        .lines()
        .filter(|line| !line.trim().starts_with(COMMENT_OUT_TOKEN))
        .collect::<String>();
    let mut input = _input.as_str();

    while input.chars().next().is_some() {
        match input.chars().next() {
            // whitespace
            Some(c) if c.is_whitespace() => {
                let mut chars = input.chars();
                chars.next();
                input = chars.as_str();
            }
            // keyword,identifer
            Some(_c @ ('"' | 'a'..'z' | 'A'..'Z')) => {
                let mut chars = input.chars();
                let mut token = chars.next().unwrap().to_string();
                input = chars.as_str();
                while matches!(
                    input.chars().next(),
                    Some(_c @ ('"' | 'a'..'z' | 'A'..'Z' | '0'..='9' | '_'))
                ) {
                    let mut chars = input.chars();
                    token += &chars.next().unwrap().to_string();
                    input = chars.as_str();
                }
                tokens.push(token);
            }
            // keyword,identifer
            Some(c) if symbols.iter().any(|symbol| c == *symbol) => {
                let mut chars = input.chars();
                let token = chars.next().unwrap().to_string();
                input = chars.as_str();
                tokens.push(token);
            }
            Some(_c @ ('0'..='9')) => {
                let mut chars = input.chars();
                let mut token = chars.next().unwrap().to_string();
                input = chars.as_str();
                while matches!(input.chars().next(), Some(_c @ ('0'..='9'))) {
                    let mut chars = input.chars();
                    token += &chars.next().unwrap().to_string();
                    input = chars.as_str();
                }
                tokens.push(token);
            }
            None => (),
            c => panic!("un supported token: {:?}", c),
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_token() {
        let input = r#"if (x < 0) {
    // comment
    let sign = "negative";
    let sign_2 = "positive";
}"#;
        let actual = vec![
            "if",
            "(",
            "x",
            "<",
            "0",
            ")",
            "{",
            "let",
            "sign",
            "=",
            "\"negative\"",
            ";",
            "let",
            "sign_2",
            "=",
            "\"positive\"",
            ";",
            "}",
        ];
        assert_eq!(parse_tokens(input).unwrap(), actual);
    }
}
