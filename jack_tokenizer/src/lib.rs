use anyhow::Result;
use std::{
    io::{BufReader, Read},
    str::FromStr,
};
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, EnumIter, EnumString};

const COMMENT_TOKEN: &str = "//";
const SYMBOLS: [char; 19] = [
    '{', '}', '(', ')', '[', ']', '.', ',', ';', '+', '-', '*', '/', '&', '|', '<', '>', '=', '~',
];

#[derive(Debug, PartialEq, AsRefStr, EnumIter)]
pub enum TokenType {
    KeyWord,
    Symbol,
    Identifier,
    IntConst,
    StringConst,
}

#[derive(Debug, PartialEq, AsRefStr, EnumIter, EnumString)]
#[strum(ascii_case_insensitive)]
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
    tokens: Vec<String>,
    current_token: Option<String>,
}

impl JackTokenizer {
    pub fn new<R: Read>(reader: R) -> Self {
        let mut buf = String::new();
        let mut jack_code = BufReader::new(reader);
        jack_code.read_to_string(&mut buf).unwrap();

        Self {
            tokens: parse_tokens(&buf).unwrap(),
            current_token: None,
        }
    }

    pub fn has_more_tokens(&mut self) -> Result<bool> {
        Ok(self.tokens.iter().next().is_some())
    }

    pub fn advance(&mut self) -> Result<()> {
        if self.has_more_tokens()? {
            self.current_token = self.tokens.iter().next().cloned();
            let mut tokens = self.tokens.clone().into_iter();
            tokens.next();
            self.tokens = tokens.collect();
        }
        Ok(())
    }

    pub fn token_type(&self) -> Result<TokenType> {
        match &self.current_token {
            Some(t) if KeyWord::iter().any(|k| k.as_ref().to_lowercase() == *t) => {
                Ok(TokenType::KeyWord)
            }
            Some(t) if SYMBOLS.iter().any(|s| *s == t.chars().next().unwrap()) => {
                Ok(TokenType::Symbol)
            }
            Some(t) if matches!(t.chars().next().unwrap(), _c @('0'..='9')) => {
                Ok(TokenType::IntConst)
            }
            Some(t) if t.chars().next().unwrap() == '"' => Ok(TokenType::StringConst),
            Some(t) if matches!(t.chars().next().unwrap(), _c @('_' | 'a'..'z' | 'A'..'Z')) => {
                Ok(TokenType::Identifier)
            }
            None => panic!("curret token is empty"),
            t => panic!("un supported token type: {:?}", t),
        }
    }

    pub fn keyword(&self) -> Result<KeyWord> {
        match &self.current_token {
            Some(token) => match KeyWord::from_str(&token) {
                Ok(keyword) => Ok(keyword),
                Err(e) => panic!(
                    "KeywordEnum parse error: {:?} token: {:?}",
                    e, self.current_token
                ),
            },
            None => panic!("current token is empty"),
        }
    }

    pub fn symbol(&self) -> Result<String> {
        Ok(self.current_token.clone().unwrap())
    }

    pub fn identifer(&self) -> Result<String> {
        Ok(self.current_token.clone().unwrap())
    }

    pub fn int_val(&self) -> Result<u16> {
        Ok(self
            .current_token
            .clone()
            .unwrap()
            .parse::<u16>()
            .expect("int_val parse failed"))
    }

    pub fn string_val(&self) -> Result<String> {
        Ok(self
            .current_token
            .clone()
            .unwrap()
            .as_str()
            .chars()
            .filter(|c| *c != '"')
            .collect())
    }
}

fn parse_tokens(input: &str) -> Result<Vec<String>> {
    let mut tokens: Vec<String> = Vec::new();

    let ignore_comment_input = input
        .lines()
        .filter(|line| !line.trim().starts_with(COMMENT_TOKEN))
        .collect::<String>();
    let mut input = ignore_comment_input.as_str();

    while input.chars().next().is_some() {
        match input.chars().next() {
            // whitespace
            Some(c) if c.is_whitespace() => {
                let mut chars = input.chars();
                chars.next();
                input = chars.as_str();
            }
            // keyword,identifer
            Some(_c @ ('_' | '"' | 'a'..'z' | 'A'..'Z')) => {
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
            // symbol
            Some(c) if SYMBOLS.iter().any(|symbol| c == *symbol) => {
                let mut chars = input.chars();
                let token = chars.next().unwrap().to_string();
                input = chars.as_str();
                tokens.push(token);
            }
            // integer
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
    fn playground() -> Result<()> {
        TokenType::iter()
            .for_each(|token_type| println!("{:?}", token_type.as_ref().to_lowercase()));
        Ok(())
    }

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

    #[test]
    fn test_jack_tokenizer() -> Result<()> {
        let file_content = std::io::Cursor::new(
            r#"if (x < 0) {
    // comment
    let sign = "negative";
    let sign_2 = "positive";
}"#
            .as_bytes(),
        );
        let mut tokenizer = JackTokenizer::new(file_content);

        assert_eq!(tokenizer.current_token.clone(), None);

        tokenizer.advance()?;
        assert_eq!(tokenizer.current_token.clone().unwrap(), "if".to_string());
        assert_eq!(tokenizer.token_type()?, TokenType::KeyWord);

        tokenizer.advance()?;
        assert_eq!(tokenizer.current_token.clone().unwrap(), "(".to_string());
        assert_eq!(tokenizer.token_type()?, TokenType::Symbol);

        tokenizer.advance()?;
        assert_eq!(tokenizer.current_token.clone().unwrap(), "x".to_string());
        assert_eq!(tokenizer.token_type()?, TokenType::Identifier);

        Ok(())
    }
}
