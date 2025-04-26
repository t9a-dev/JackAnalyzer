use anyhow::Result;
use std::{
    io::Write,
    sync::{Arc, Mutex},
};

use jack_tokenizer::JackTokenizer;

pub struct TokenizedXmlWriter {
    writer: Arc<Mutex<dyn Write>>,
}

impl TokenizedXmlWriter {
    pub fn new(writer: Arc<Mutex<dyn Write>>) -> Self {
        Self { writer: writer }
    }

    pub fn write_xml(&mut self, tokenizer: &mut JackTokenizer) -> Result<()> {
        self.write(&format!("<tokens>\n"))?;
        while tokenizer.has_more_tokens()? {
            tokenizer.advance()?;

            match tokenizer.token_type()? {
                jack_tokenizer::TokenType::KeyWord => {
                    self.write_xml_tag(
                        &tokenizer.token_type()?.as_ref().to_lowercase(),
                        &tokenizer.keyword()?.as_ref().to_lowercase(),
                    )?;
                }
                jack_tokenizer::TokenType::Symbol => {
                    self.write_xml_tag(
                        &tokenizer.token_type()?.as_ref().to_lowercase(),
                        &tokenizer.symbol()?,
                    )?;
                }
                jack_tokenizer::TokenType::Identifier => {
                    self.write_xml_tag(
                        &tokenizer.token_type()?.as_ref().to_lowercase(),
                        &tokenizer.identifer()?,
                    )?;
                }
                jack_tokenizer::TokenType::IntConst => {
                    self.write_xml_tag(
                        &tokenizer.token_type()?.as_ref(),
                        &tokenizer.int_val()?.to_string(),
                    )?;
                }
                jack_tokenizer::TokenType::StringConst => {
                    self.write_xml_tag(
                        &tokenizer.token_type()?.as_ref(),
                        &tokenizer.string_val()?.to_string(),
                    )?;
                }
            }
        }
        self.write(&format!("</tokens>"))?;
        Ok(())
    }

    fn escape_xml_symbol<'a>(&self, v: &'a str) -> &'a str {
        match v {
            "<" => "&lt;",
            ">" => "&gt;",
            "\"" => "&quot;",
            "&" => "&amp;",
            _ => v,
        }
    }

    fn write_xml_tag(&mut self, tag_name: &str, content: &str) -> Result<()> {
        let content = self.escape_xml_symbol(content);
        self.write(&format!("<{tag_name}> {content} </{tag_name}>\n"))?;
        Ok(())
    }

    fn write(&mut self, content: &str) -> Result<()> {
        self.writer.lock().unwrap().write(content.as_bytes())?;
        self.writer.lock().unwrap().flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::io::Cursor;

    fn normalize(s: &str) -> String {
        s.lines().map(str::trim).collect::<Vec<_>>().join("")
    }

    #[test]
    fn test_tokenize_and_write_xml() -> Result<()> {
        let jack_code = r#"if (x < 0) {
    // comment
    let quit = "yes";
}"#;
        let expect_buf = Arc::new(Mutex::new(Cursor::new(Vec::new())));
        let mut tokenizer = JackTokenizer::new(Cursor::new(jack_code.as_bytes()));
        let mut tokenized_xml_writer = TokenizedXmlWriter::new(expect_buf.clone());

        tokenized_xml_writer.write_xml(&mut tokenizer)?;
        let expect = String::from_utf8_lossy(expect_buf.lock().unwrap().get_ref()).to_string();
        let actual = "<tokens>
        <keyword> if </keyword>
        <symbol> ( </symbol>
        <identifier> x </identifier>
        <symbol> &lt; </symbol>
        <integerConstant> 0 </integerConstant>
        <symbol> ) </symbol>
        <symbol> { </symbol>
        <keyword> let </keyword>
        <identifier> quit </identifier>
        <symbol> = </symbol>
        <stringConstant> yes </stringConstant>
        <symbol> ; </symbol>
        <symbol> } </symbol>
        </tokens>
        ";

        assert_eq!(normalize(&expect), normalize(actual));

        Ok(())
    }
}
