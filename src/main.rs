use std::{
    fs::File,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use anyhow::{anyhow, Result};
use jack_tokenizer::JackTokenizer;
use tokenized_xml_writer::TokenizedXmlWriter;

const JACK_FILE_EXTENSION: &str = "jack";
const OUTPUT_FILE_EXTENSION: &str = "xml";

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
    analyze_target_paths
        .iter()
        .try_for_each(|jack_file| -> Result<()> {
            let output_file_path = jack_file
                .parent() 
                .unwrap()
                .join(format!("{}T.{}", jack_file.file_stem().unwrap().to_string_lossy().to_string(), OUTPUT_FILE_EXTENSION));
            let output_file = Arc::new(Mutex::new(File::create(&output_file_path)?));
            let mut tokenized_xml = TokenizedXmlWriter::new(output_file);
            let mut tokenizer = JackTokenizer::new(File::open(jack_file)?)
                .expect(&format!("jack_toknizer initialize failed: {:?}", jack_file));
            tokenized_xml.write_xml(&mut tokenizer)?;
            Ok(())
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use std::fs::{self, File};

    use rand::distr::{Alphanumeric, SampleString};
    use walkdir::WalkDir;

    use super::*;

    const TEST_DIR: &str = "target/test/data";
    const TEST_JACK_DIR: &str = "test_files";

    fn create_test_file(test_dir: Option<&str>, test_file_extension: &str) -> Result<String> {
        let test_dir = test_dir.or(Some("target/test/data")).unwrap();
        fs::create_dir_all(test_dir)?;
        let mut test_file_name = Alphanumeric.sample_string(&mut rand::rng(), 5);
        test_file_name = format!("{}.{}", test_file_name, test_file_extension);
        let file_path = Path::new(test_dir).join(&test_file_name);
        File::create(&file_path)?;
        Ok(file_path.to_string_lossy().to_string())
    }

    fn find_files_with_extension(dir: &Path, extension: &str) -> Result<Vec<String>> {
        let mut paths: Vec<String> = Vec::new();

        for entry in WalkDir::new(dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            if let Some(ext) = entry.path().extension() {
                if ext == extension {
                    paths.push(entry.path().display().to_string());
                }
            }
        }

        Ok(paths)
    }

    #[test]
    fn playground() -> Result<()> {
        let jack_files = find_files_with_extension(Path::new(TEST_JACK_DIR), JACK_FILE_EXTENSION)?;
        println!("jack files: {:?}", jack_files);
        Ok(())
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
    fn run_analyze() -> Result<()> {
        let jack_file_paths =
            find_files_with_extension(Path::new(TEST_JACK_DIR), JACK_FILE_EXTENSION)?;

        jack_file_paths
            .iter()
            .try_for_each(|jack_file_path| jack_analyzer(&jack_file_path))?;

        Ok(())
    }
}
