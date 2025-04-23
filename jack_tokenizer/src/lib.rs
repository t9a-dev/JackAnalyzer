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
    current_command: Option<String>,
}

impl JackTokenizer {
    pub fn new(jack_file_path: &str) -> Self {
        Self {
            jack_code: Box::new(BufReader::new(
                File::open(Path::new(jack_file_path)).unwrap(),
            )),
            current_command: None,
        }
    }

    pub fn has_more_lines(&mut self) -> Result<bool> {
        Ok(self.jack_code.fill_buf()?.iter().next().is_some())
    }

    pub fn advance(&mut self) -> Result<()> {
        // //で始まるコメント行と空白を無視して次の行を読み込む
        while self.has_more_lines()? {
            self.current_command = match self.jack_code.as_mut().lines().next().unwrap() {
                Ok(line) if line.chars().all(char::is_whitespace) => None, //空白の場合は無視
                Ok(line) if line.trim().starts_with(COMMENT_OUT_TOKEN) => None, //コメント行の場合は無視
                Ok(line) => Some(line.trim().to_string()),
                Err(_) => None,
            };
            if self.current_command.is_some() {
                break;
            }
        }
        Ok(())
    }

    pub fn token_type(&self) -> Result<TokenType> {
        let cmd: String = self
            .current_command
            .clone()
            .unwrap()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();

        match cmd.chars().next() {
            Some(_c @ ('a'..'z' | 'A'..'Z')) => {
                while matches!(
                    cmd.chars().next(),
                    Some(_c @ ('_' | 'a'..'z' | 'A'..'Z' | '0'..'9'))
                ) {
                    todo!()
                }
            }
            None => todo!(),
            _ => todo!(),
        }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert!(true)
    }
}
