use std::collections::HashSet;

use clap::Parser;
use solver::{EntropyStrategy, GuessStrategy};

use wordlib::{Knowledge, words_from_file};

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
    let target_words: Vec<&str> = solver::valid_words(target_word_len, &all_words, disallowed);

    let guesser = EntropyStrategy::new(&target_words);
    let guess = guesser.next_guess(&knowledge);

    println!("Next guess: {}", guess);

    Ok(())
}

