use std::collections::HashSet;

use clap::Parser;
use strsim::hamming;

use wordlib::{char_freq, freq_scored_guesses, Knowledge, words_from_file};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The word list to draw guesses from
    #[clap(short, default_value = "/usr/share/dict/words", env = "WORDLIST")]
    wordlist: String,
    /// The list of known disallowed words in the word list
    #[clap(short, default_value = "disallowed")]
    disallowed: String,
    /// A pattern describing the known positions of letters, and the length of the word.
    /// Use '.' for unknown positions. E.g. "f...." for a five-letter word known to start with 'f'.
    pattern: String,
    /// Characters that are known to appear in the word, whether or not their positions are known.
    /// Order doesn't matter.
    #[clap(default_value = "")]
    chars: String,
    /// Previous guesses. Used to derive information about what letters are not present, and what
    /// positions letters are not in.
    tried: Vec<String>,
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    let target_word_len = args.pattern.len();

    let tries: Vec<&str> = args.tried.iter().map(|s| s.as_str()).collect();
    let knowledge = Knowledge::from_tries(
        args.pattern.as_str(),
        args.chars.as_str(),
        tries.as_slice()
    );

    // Read the word list
    let all_words: Vec<String> = words_from_file(args.wordlist.as_str())?;
    let disallowed: HashSet<String> = words_from_file(args.disallowed.as_str())
        .unwrap_or(HashSet::new());

    // Filter out proper names, words of the wrong length, and disallowed words
    let target_words: Vec<&str> = all_words
        .iter()
        .filter(|s| s.chars().all(|ch| ch.is_lowercase()))
        .filter(|s| s.len() == target_word_len)
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
    coverage.extend(args.chars.chars());

    let word_scores = freq_scored_guesses(&target_words, &freq, &coverage);

    let top_score = word_scores[0].1;
    let mut guesses: Vec<&str> = word_scores
        .iter()
        .take_while(|(_, s)| *s == top_score)
        .map(|(w, _)| *w)
        .collect();

    //Sort guesses by shortest hamming distance to the candidate set; hopefully this reveals positional info.
    guesses.sort_by_cached_key(|w| {
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

    Ok(())
}

fn candidate_words<'a, F: Fn(&str) -> bool>(words: &[&'a str], pred: F) -> Vec<&'a str> {
    words.iter().filter(|w| pred(w)).copied().collect()
}
