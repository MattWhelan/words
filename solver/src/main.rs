use std::collections::HashSet;
use std::env::args;
use std::fs::File;
use std::io::Read;
use std::process;

use regex::Regex;
use wordlib::{best_guesses, char_freq};

const TARGET_WORD_LEN: usize = 5;

fn main() -> Result<(), anyhow::Error> {
    let arguments: Vec<String> = args().collect();
    if arguments.len() < 2 {
        println!("Usage: {} <file> <pattern> <chs> [<tried> ...]", arguments[0]);
        process::exit(1);
    } else {
        // Parse args
        let input_filename = arguments[1].as_str();

        let pattern = arguments[2].as_str();
        let re = Regex::new(pattern)?;

        let hit_chars: Vec<char> = arguments[3].chars().collect();

        let tried: Vec<String> = arguments[4..].iter()
            .cloned()
            .collect();
        let miss_chars: HashSet<char> = tried.iter()
            .flat_map(|w| w.chars())
            .filter(|ch| !hit_chars.contains(ch))
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
            .copied()
            .collect();

        let candidates = candidate_words(&target_words, |w| {
            re.is_match(w)
                && hit_chars.iter().all(|ch| w.chars().any(|wch| wch == *ch))
                && !w.chars().any(|ch| miss_chars.contains(&ch))
        });

        if candidates.len() < 3 {
            println!("Candidates ({}):", candidates.len());
            for w in candidates.iter() {
                println!("  {}", w);
            }
            return Ok(())
        } else {
            println!("Candidates: {}", candidates.len());
        }

        let freq = char_freq(&candidates);

        // Find the words which give you the best character coverage
        let mut coverage= HashSet::new();
        coverage.extend(miss_chars.iter());
        coverage.extend(hit_chars.iter());

        let guesses = best_guesses(&target_words, &freq, &mut coverage);
        for guess in guesses {
            println!("{}", guess);
        }

    }
    Ok(())
}

fn candidate_words<'a, F: Fn(&str) -> bool>(words: &[&'a str], pred: F) -> Vec<&'a str> {
    words.iter().filter(|w|pred(w)).copied().collect()
}