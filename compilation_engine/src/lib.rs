use anyhow::{anyhow, Ok, Result};
use jack_tokenizer::{JackTokenizer, TokenType};
use std::{
    io::Write,
    sync::{Arc, Mutex},
};

pub struct CompilationEngine {
    tokenizer: JackTokenizer,
    writer: Arc<Mutex<dyn Write>>,
}

impl CompilationEngine {
    pub fn new(tokenizer: JackTokenizer, writer: Arc<Mutex<dyn Write>>) -> Result<Self> {
        Ok(Self { tokenizer, writer })
    }

    pub fn compile_class(&mut self) -> Result<()> {
        let tag_name = "class";
        self.tokenizer.advance()?;
        self.write_start_xml_tag(tag_name)?;
        self.process_token("class")?;
        self.process_identifier()?;
        self.process_token("{")?;
        self.compile_class_var_dec()?;
        self.compile_subroutine()?;
        self.process_token("}")?;
        self.write_end_xml_tag(tag_name)?;
        Ok(())
    }

    pub fn compile_class_var_dec(&mut self) -> Result<()> {
        let tag_name = "classVarDec";
        self.write_start_xml_tag(tag_name)?;
        // "static"|"field"
        {
            self.process_token("static").or_else(|_| {
                self.process_token("field")?;
                Ok(())
            })?;
        }
        // type -> "int"|"char"|"boolean"|className
        {
            self.process_type()?;
        }
        self.process_identifier()?;
        self.process_token(";")?;
        self.write_end_xml_tag(tag_name)?;

        // classVarDecが複数存在する場合
        if self.tokenizer.token_type()? == TokenType::KeyWord
            && matches!(
                self.tokenizer
                    .keyword()?
                    .as_ref()
                    .to_string()
                    .to_lowercase()
                    .as_str(),
                "static" | "field"
            )
        {
            self.compile_class_var_dec()?;
        }

        Ok(())
    }

    pub fn compile_subroutine(&mut self) -> Result<()> {
        let tag_name = "subroutineDec";
        self.write_start_xml_tag(tag_name)?;
        // "constructor"|"function"|"method"
        {
            self.process_token("constructor").or_else(|_| {
                self.process_token("function").or_else(|_| {
                    self.process_token("method")?;
                    Ok(())
                })
            })?;
        }
        // "void"|type
        {
            self.process_token("void").or_else(|_| {
                self.process_type()?;
                Ok(())
            })?;
        }
        self.process_identifier()?;
        self.process_token("(")?;
        self.compile_parameter_list()?;
        self.process_token(")")?;
        self.compile_subroutine_body()?;
        self.write_end_xml_tag(tag_name)?;

        // subroutineDecが複数存在する場合
        if self.tokenizer.token_type()? == TokenType::KeyWord
            && matches!(self.tokenizer.keyword()?.as_ref().to_lowercase().as_str(), "constructor"|"function"|"method"){
                self.compile_subroutine()?;
            }
        Ok(())
    }

    pub fn compile_parameter_list(&mut self) -> Result<()> {
        todo!()
    }

    pub fn compile_subroutine_body(&mut self) -> Result<()> {
        todo!()
    }

    pub fn compile_var_dec(&mut self) -> Result<()> {
        todo!()
    }

    pub fn compile_statements(&mut self) -> Result<()> {
        todo!()
    }

    pub fn compile_let(&mut self) -> Result<()> {
        todo!()
    }

    pub fn compile_if(&mut self) -> Result<()> {
        todo!()
    }

    pub fn compile_while(&mut self) -> Result<()> {
        self.write_start_xml_tag("whileStatement")?;
        self.process_token("while")?;
        self.process_token("(")?;
        self.compile_expression()?;
        self.process_token(")")?;
        self.process_token("{")?;
        self.compile_statements()?;
        self.process_token("}")?;
        self.write_end_xml_tag("whileStatement")?;
        Ok(())
    }

    pub fn compile_do(&mut self) -> Result<()> {
        todo!()
    }

    pub fn compile_return(&mut self) -> Result<()> {
        todo!()
    }

    pub fn compile_expression(&mut self) -> Result<()> {
        todo!()
    }

    pub fn compile_term(&mut self) -> Result<()> {
        todo!()
    }

    pub fn compile_expression_list(&mut self) -> Result<()> {
        todo!()
    }

    fn process_token(&mut self, token: &str) -> Result<()> {
        let current_token = match self.tokenizer.token_type()? {
            jack_tokenizer::TokenType::KeyWord => self
                .tokenizer
                .keyword()?
                .as_ref()
                .to_string()
                .as_str()
                .to_lowercase(),
            jack_tokenizer::TokenType::Symbol => self.tokenizer.symbol()?,
            jack_tokenizer::TokenType::Identifier => self.tokenizer.identifer()?,
            jack_tokenizer::TokenType::IntConst => self.tokenizer.int_val()?.to_string(),
            jack_tokenizer::TokenType::StringConst => self.tokenizer.string_val()?,
        };
        if current_token == token.to_lowercase() {
            self.write_xml(
                &self
                    .tokenizer
                    .token_type()?
                    .as_ref()
                    .to_string()
                    .to_lowercase(),
                &current_token,
            )?;
        } else {
            return Err(anyhow!(
                "syntax error token: {:?}, current_token: {:?}",
                token,
                current_token
            ));
        }
        self.tokenizer.advance()?;
        Ok(())
    }

    fn process_identifier(&mut self) -> Result<()> {
        if self.tokenizer.token_type()? == TokenType::Identifier {
            self.write_xml(
                &self
                    .tokenizer
                    .token_type()?
                    .as_ref()
                    .to_string()
                    .to_lowercase(),
                &self.tokenizer.identifer()?,
            )?;
        } else {
            return Err(anyhow!(
                "syntax error current token type is not identifier: {:?}",
                self.tokenizer.token_type()?
            ));
        }
        self.tokenizer.advance()?;

        // 次のトークンを先読みして","であれば複数varNameが存在するので対応する
        if self.tokenizer.token_type()? == TokenType::Symbol && self.tokenizer.symbol()? == "," {
            self.process_token(",")?;
            self.process_identifier()?;
        }

        Ok(())
    }

    fn process_type(&mut self) -> Result<()> {
        self.process_token("int").or_else(|_| {
            self.process_token("char").or_else(|_| {
                self.process_token("boolean").or_else(|_| {
                    self.process_identifier()?;
                    Ok(())
                })
            })
        })?;
        Ok(())
    }

    fn write_start_xml_tag(&mut self, tag_name: &str) -> Result<()> {
        self.write(&format!("<{tag_name}>\n"))?;
        Ok(())
    }

    fn write_end_xml_tag(&mut self, tag_name: &str) -> Result<()> {
        self.write(&format!("</{tag_name}>\n"))?;
        Ok(())
    }

    fn write_xml(&mut self, tag_name: &str, content: &str) -> Result<()> {
        self.write(&format!("<{tag_name}>"))?;
        self.write(&format!(" {content} "))?;
        self.write(&format!("</{tag_name}>\n"))?;
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
    use pretty_assertions::assert_eq;
    use std::{
        io::Cursor,
        sync::{Arc, Mutex},
    };

    use jack_tokenizer::JackTokenizer;

    use crate::CompilationEngine;
    use anyhow::Result;

    #[test]
    fn test_compilation_engine() -> Result<()> {
        let jack_code = Cursor::new("method");
        let output = Arc::new(Mutex::new(Cursor::new(Vec::new())));
        let mut tokenizer = JackTokenizer::new(jack_code)?;
        tokenizer.advance()?;
        let mut compilation_engine = CompilationEngine::new(tokenizer, output.clone())?;
        compilation_engine.process_token("method")?;
        let expect = "<keyword> method </keyword>\n";
        let output = output.lock().unwrap();
        let actual = String::from_utf8_lossy(output.get_ref());

        assert_eq!(expect, actual);
        Ok(())
    }
}
