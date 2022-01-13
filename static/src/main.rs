use std::collections::HashSet;
use std::env::args;
use std::fs::File;
use std::io::Read;
use std::process;
use wordlib::{best_guesses, char_freq, show_freq};

const TARGET_WORD_LEN: usize = 5;

fn main() -> Result<(), anyhow::Error> {
    let arguments: Vec<String> = args().collect();
    if arguments.len() < 2 {
        println!("Usage: {} <file> [disallowed_words ...]", arguments[0]);
        process::exit(1);
    } else {
        // Parse args
        let input_filename = arguments[1].as_str();
        let disallowed: HashSet<String> = arguments[2..].iter()
            .cloned()
            .collect();

        // Read the word list
        let mut file = File::open(input_filename)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let all_words: Vec<&str> = contents.lines().collect();

        // Filter out proper names, words of the wrong length, and disallowed words
        let target_words: Vec<&str> = all_words.iter()
            .filter(|s| s.chars().all(|ch| ch.is_lowercase()))
            .filter(|s| s.len() == TARGET_WORD_LEN)
            .filter(|s| !disallowed.contains(**s))
            .copied()
            .collect();

        // Frequency analysis. Characters are worth points equal to their number of appearances in the word list
        let freq = char_freq(&target_words);

        show_freq(&freq);

        // Find the words which give you the best character coverage
        let mut coverage= HashSet::new();
        let guesses = best_guesses(&target_words, &freq, &mut coverage);
        for guess in guesses {
            println!("{}", guess);
        }

        Ok(())
    }
}
