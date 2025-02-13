//! Module creating the expected input for the test runner
//!
//! It reads dictionary file to get list of words
//! then shuffles this list
//! optionally replaces words with numbers if flag `numbers` is set to true in config
//! and returns as a string
//!
//! Dictionary file should be a text file in format of single words per line.

use anyhow::Context;
use mockall::automock;
use rand::{seq::SliceRandom, thread_rng, Rng};
use std::io::Read;

use crate::config::Config;

/// Struct used by runner to hold generate the text used for validation and as a placeholder
#[derive(Debug)]
pub struct ExpectedInput {
    str: String,
}

// todo: move it to config, read it from config file or args as other config options
/// If `numbers` option in config is set to `true` specifies what should be the ratio of numbers to
/// normal words in expected input.
/// Should be a floating point value between 0 and 1.
const NUMBERS_RATIO: f64 = 0.05;

impl ExpectedInput {
    /// Create new struct instance by reading the dictionary file
    ///
    /// After reading the file shuffle its content
    /// then replace some words with numbers if specified in config
    /// then save one long string to memory
    pub fn new(config: &Config) -> Result<Self, anyhow::Error> {
        let mut file = std::fs::File::open(config.dictionary_path.clone())
            .context("Unable to open dictionary file")?;
        let mut str = String::new();
        file.read_to_string(&mut str)
            .context("Unable to read dictionary file")?;

        let mut rng = thread_rng();
        let mut str_vec = str.split("\n").collect::<Vec<&str>>();
        let mut string_vec: Vec<String> = str_vec.iter().map(|s| s.to_string()).collect();
        str_vec.shuffle(&mut rng);

        if config.numbers == true {
            replace_words_with_numbers(&mut string_vec, &mut rng, NUMBERS_RATIO);

            str_vec = string_vec.iter().map(|s| s.as_str()).collect();
            str_vec.shuffle(&mut rng);
        }

        let str = str_vec.join(" ").trim().to_string();

        Ok(Self { str })
    }
}

/// In given vector of words replace some of them
///
/// with words consisting only of numbers
/// number_ratio should be between [0, 1.0]
/// and tells how many percent of words should become numbers
fn replace_words_with_numbers(
    string_vec: &mut Vec<String>,
    rng: &mut rand::rngs::ThreadRng,
    numbers_ratio: f64,
) {
    let change_to_num_treshold = (numbers_ratio * string_vec.len() as f64).round() as usize;

    *string_vec = string_vec
        .iter()
        .enumerate()
        .map(|(index, word)| {
            if index < change_to_num_treshold {
                let random_digits: String = (0..word.len())
                    .map(|_| rng.gen_range(b'0'..=b'9') as char)
                    .collect();
                return random_digits;
            }
            return word.to_string();
        })
        .collect();
}

/// extracted to trait to create mock with `mockall` crate
#[automock]
pub trait ExpectedInputInterface {
    fn get_string(&self, len: usize) -> String;
}

impl ExpectedInputInterface for ExpectedInput {
    /// Cuts string saved in ExpectedInput at specified length instance and returns it
    ///
    /// If string is shorter than the specified length it repeats it enough times for it to be long
    /// enough.
    fn get_string(&self, len: usize) -> String {
        let s = self.str.clone() + " ";
        let s = s.repeat(len / s.len() + 1);
        let (s, _) = s.split_at(len);

        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::{io::Write, time::Duration};
    use rand::{thread_rng, Rng}; // Add this import

    use super::*;
    use super::super::*; // Add this import

    #[test]
    fn new_expected_input_should_correctly_convert_to_str() {
        let config = Config::default();
        let expected_input = ExpectedInput::new(&config).expect("Unable to create expected input");

        assert_eq!(expected_input.get_string(12).len(), 12);
    }

    #[test]
    fn should_read_file() {
        let mut config_file = tempfile::NamedTempFile::new().expect("Unable to create temp file");
        config_file
            .write_all(b"halo")
            .expect("Unable to write to temp file");
        let config = Config {
            duration: Duration::from_secs(30),
            numbers: false,
            dictionary_path: config_file.path().to_path_buf(),
        };

        let expected_input = ExpectedInput::new(&config).expect("Unable to create expected input");

        assert_eq!(expected_input.get_string(4), "halo");
    }

    #[test]
    fn should_trim_string_to_match_len() {
        let expected_input = ExpectedInput {
            str: "abcdef".to_string(),
        };

        assert_eq!(expected_input.get_string(3), "abc");
    }

    #[test]
    fn should_repeat_string_if_len_is_too_big() {
        let expected_input = ExpectedInput {
            str: "abc".to_string(),
        };

        assert_eq!(expected_input.get_string(11), "abc abc abc");
    }

    #[test]
    fn should_replace_words_with_numbers() {
        let mut string_vec = vec![
            "item1".to_string(),
            "item2".to_string(),
            "item3".to_string(),
            "item4".to_string(),
            "item5".to_string(),
            "item6".to_string(),
            "item7".to_string(),
            "item8".to_string(),
        ];
        let mut rng = thread_rng(); // Initialize RNG
        let numbers_ratio = 0.5;

        replace_words_with_numbers(&mut string_vec, &mut rng, numbers_ratio);

        let items_with_only_digits: Vec<&String> = string_vec
            .iter()
            .filter(|item| item.chars().all(|c| c.is_digit(10)))
            .collect();

        let change_to_num_threshold = (numbers_ratio * string_vec.len() as f64).round() as usize;
        assert_eq!(change_to_num_threshold, 4);
        assert_eq!(
            items_with_only_digits.len(),
            4,
            "At least 4 items contain only digits"
        );
    }
}

