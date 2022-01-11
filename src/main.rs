use std::collections::{HashMap, HashSet};
use std::env::args;
use std::fs::File;
use std::io::Read;
use std::process;

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
        let five_words: Vec<&str> = all_words.iter()
            .filter(|s| s.chars().all(|ch| ch.is_lowercase()))
            .filter(|s| s.len() == 5)
            .filter(|s| !disallowed.contains(**s))
            .copied()
            .collect();

        // Frequency analysis. Characters are worth points equal to their number of appearances in the word list
        let freq = char_freq(&five_words);
        // Find the words which give you the best character coverage
        let mut coverage= HashSet::new();
        let guesses = best_guesses(&five_words, &freq, &mut coverage);
        for guess in guesses {
            println!("{}", guess);
        }

        Ok(())
    }
}

fn best_guesses<'a>(words: &'a [&str], freq: &HashMap<char, usize>, covered: &mut HashSet<char>) -> Vec<&'a str> {
    //Score each word by freq-points
    let best = words.iter()
        .max_by_key(|w| word_score(w, freq, covered))
        .unwrap();
    covered.extend(best.chars());

    let mut ret = if covered.len() < freq.len() {
        best_guesses(words, freq, covered)
    } else {
        Vec::new()
    };
    ret.insert(0, best);

    ret
}

fn word_score(word: &str, freq: &HashMap<char, usize>, covered: &HashSet<char>) -> usize {
    let uniques: HashSet<char> = word.chars().collect();
    uniques.into_iter()
        .map(|ch| if covered.contains(&ch) {
            0
        } else {
            *freq.get(&ch).unwrap_or(&0)
        })
        .sum()
}

fn char_freq(words: &[&str]) -> HashMap<char, usize> {
    words.iter()
        .flat_map(|w| w.chars())
        .fold(HashMap::new(), |mut acc, ch| {
            *acc.entry(ch).or_default() += 1;
            acc
        })
}