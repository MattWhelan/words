use std::collections::HashSet;
use std::env::args;
use std::process;

use strsim::hamming;

use wordlib::{char_freq, freq_scored_guesses, Knowledge, words_from_file};

const TARGET_WORD_LEN: usize = 5;

fn main() -> Result<(), anyhow::Error> {
    let arguments: Vec<String> = args().collect();
    if arguments.len() < 2 {
        println!(
            "Usage: {} <file> <pattern> <chs> [<tried> ...]",
            arguments[0]
        );
        process::exit(1);
    } else {
        // Parse args
        let input_filename = arguments[1].as_str();

        let pattern = arguments[2].as_str();

        let present_chs: &str = arguments[3].as_str();

        let tries: Vec<&str> = arguments[4..].iter()
            .map(|s| s.as_str())
            .collect();

        let knowledge = Knowledge::from_tries(pattern, present_chs, &tries);

        // Read the word list
        let all_words: Vec<String> = words_from_file(input_filename)?;
        let disallowed: HashSet<String> = words_from_file("disallowed").unwrap_or(HashSet::new());

        // Filter out proper names, words of the wrong length, and disallowed words
        let target_words: Vec<&str> = all_words
            .iter()
            .filter(|s| s.chars().all(|ch| ch.is_lowercase()))
            .filter(|s| s.len() == TARGET_WORD_LEN)
            .filter(|s| !disallowed.contains(s.as_str()))
            .map(|s| s.as_str())
            .collect();

        let candidates = candidate_words(&target_words, |w| {
            knowledge.check_word(w)
        });

        if candidates.len() < 100 {
            println!("Candidates ({}):", candidates.len());
            for w in candidates.iter() {
                println!("  {}", w);
            }
            if candidates.len() < 3 {
                return Ok(());
            }
        } else {
            println!("Candidates: {}", candidates.len());
        }

        let freq = char_freq(&candidates);

        // Find the words which give you the best character coverage
        let mut coverage = HashSet::new();
        coverage.extend(knowledge.get_absent().iter());
        coverage.extend(present_chs.chars());

        let word_scores = freq_scored_guesses(&target_words, &freq, &coverage);

        let top_score = word_scores[0].1;
        let mut guesses: Vec<&str> = word_scores
            .iter()
            .take_while(|(_, s)| *s == top_score)
            .map(|(w, _)| *w)
            .collect();
        guesses.sort_by_key(|w| {
            candidates
                .iter()
                .map(|c| hamming(w, c).unwrap())
                .min()
                .unwrap()
        });

        println!("Guesses:");
        for guess in guesses.iter().take(5) {
            println!("  {}", guess);
        }
    }
    Ok(())
}

fn candidate_words<'a, F: Fn(&str) -> bool>(words: &[&'a str], pred: F) -> Vec<&'a str> {
    words.iter().filter(|w| pred(w)).copied().collect()
}
