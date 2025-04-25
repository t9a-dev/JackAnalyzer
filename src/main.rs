use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use jack_tokenizer::JackTokenizer;

const JACK_FILE_EXTENSION: &str = "jack";
const OUTPUT_FILE_EXTENSION: &str = "xml";

struct TokenizedXmlWriter<'a> {
    writer: &'a mut dyn Write,
}

impl<'a> TokenizedXmlWriter<'a> {
    pub fn new(writer: &'a mut dyn Write) -> Self {
        Self { writer: writer }
    }

    pub fn write_xml(&mut self, tokenizer: &mut JackTokenizer) -> Result<()> {
        self.write_start_xml_tag("tokens")?;
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
                        &tokenizer.token_type()?.as_ref().to_lowercase(),
                        &tokenizer.int_val()?.to_string(),
                    )?;
                }
                jack_tokenizer::TokenType::StringConst => {
                    self.write_xml_tag(
                        &tokenizer.token_type()?.as_ref().to_lowercase(),
                        &tokenizer.string_val()?.to_string(),
                    )?;
                }
            }
        }
        self.write_end_xml_tag("tokens")?;
        Ok(())
    }

    fn write_xml_tag(&mut self, tag_name: &str, content: &str) -> Result<()> {
        self.write_start_xml_tag(tag_name)?;
        self.write(&format!(" {} \n", content))?;
        self.write_end_xml_tag(tag_name)?;
        Ok(())
    }

    fn write_start_xml_tag(&mut self, tag_name: &str) -> Result<()> {
        self.write(&format!("<{}>\n", tag_name))
    }

    fn write_end_xml_tag(&mut self, tag_name: &str) -> Result<()> {
        self.write(&format!("</{}>", tag_name))
    }

    fn write(&mut self, content: &str) -> Result<()> {
        self.writer.write(content.as_bytes())?;
        self.writer.flush()?;
        Ok(())
    }
}

fn main() -> Result<()> {
    if let Err(e) = jack_analyzer(&parse_arg(std::env::args().collect())?) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
    Ok(())
}

fn parse_arg(args: Vec<String>) -> Result<String> {
    let current_dir = "./".to_string();
    match args.get(1) {
        Some(arg) if arg.is_empty() => Ok(current_dir),
        Some(arg) => Ok(arg.to_string()),
        _ => Ok(current_dir),
    }
}

fn parse_analyze_target_path(path: &Path) -> Result<Vec<PathBuf>> {
    let mut jack_files: Vec<PathBuf> = Vec::new();
    if path.is_dir() {
        for entry in path.read_dir()? {
            if let Ok(entry) = entry {
                if entry.path().is_file() {
                    match entry.path().extension() {
                        Some(file_extension) if file_extension == JACK_FILE_EXTENSION => {
                            jack_files.push(entry.path().to_path_buf());
                        }
                        _ => (),
                    }
                }
            }
        }
    } else {
        if let Some(extension) = path.extension() {
            if extension == JACK_FILE_EXTENSION {
                jack_files.push(path.to_path_buf());
            } else {
                return Err(anyhow!("un supported file: {:?}", path));
            }
        }
    }
    Ok(jack_files)
}

fn jack_analyzer(path_str: &str) -> Result<()> {
    let path = Path::new(path_str);
    let analyze_target_paths = parse_analyze_target_path(path)?;
    let output_file_name = if path.is_dir() {
        path.file_name().unwrap().to_string_lossy().to_string()
    } else {
        analyze_target_paths
            .get(0)
            .unwrap()
            .to_path_buf()
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string()
    };
    let output_file_path = path
        .parent()
        .unwrap()
        .join(format!("{}T.{}", output_file_name, OUTPUT_FILE_EXTENSION));

    let mut output_file = File::create(&output_file_path)?;
    let mut tokenized_xml = TokenizedXmlWriter::new(&mut output_file);
    analyze_target_paths
        .iter()
        .try_for_each(|jack_file| -> Result<()> {
            let mut tokenizer = JackTokenizer::new(File::open(jack_file)?);
            tokenized_xml.write_xml(&mut tokenizer)?;
            Ok(())
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File},
        io::Cursor,
    };

    use rand::distr::{Alphanumeric, SampleString};

    use super::*;

    const TEST_DIR: &str = "target/test/data";

    fn create_test_file(test_dir: Option<&str>, test_file_extension: &str) -> Result<String> {
        let test_dir = test_dir.or(Some("target/test/data")).unwrap();
        fs::create_dir_all(test_dir)?;
        let mut test_file_name = Alphanumeric.sample_string(&mut rand::rng(), 5);
        test_file_name = format!("{}.{}", test_file_name, test_file_extension);
        let file_path = Path::new(test_dir).join(&test_file_name);
        File::create(&file_path)?;
        Ok(file_path.to_string_lossy().to_string())
    }

    #[test]
    fn test_parse_analyze_target_path_when_dirctory() -> Result<()> {
        let test_files = vec![
            create_test_file(Some(TEST_DIR), JACK_FILE_EXTENSION)?,
            create_test_file(Some(TEST_DIR), JACK_FILE_EXTENSION)?,
        ];
        let mut expect: Vec<PathBuf> = test_files
            .iter()
            .map(|f| Path::new(f).to_path_buf())
            .collect();
        let mut actual = parse_analyze_target_path(Path::new(TEST_DIR))?;

        assert_eq!(expect.sort(), actual.sort());

        test_files
            .iter()
            .try_for_each(|test_file| fs::remove_file(test_file))?;
        Ok(())
    }

    #[test]
    fn test_tokenize_and_write_xml() -> Result<()> {
        let jack_code = r#"if (x < 0) {
    // comment
    let quit = "yes";
}"#;
        let mut expect_buf = Cursor::new(Vec::new());
        let mut tokenizer = JackTokenizer::new(Cursor::new(jack_code.as_bytes()));
        let mut tokenized_xml_writer = TokenizedXmlWriter::new(&mut expect_buf);

        tokenized_xml_writer.write_xml(&mut tokenizer)?;
        let expect = String::from_utf8_lossy(&expect_buf.into_inner()).to_string();
        let actual = "";

        assert_eq!(&expect, actual);

        Ok(())
    }
}
