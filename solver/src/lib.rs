use std::collections::{HashMap, HashSet};

use strsim::hamming;

use wordlib::{char_freq, freq_scored_guesses, Knowledge};
use crate::LetterOutcome::{ABSENT, HIT, MISS};

pub trait GuessStrategy {
    fn next_guess(&self, knowledge: &Knowledge) -> &str;
}

pub struct FreqStrategy<'a> {
    words: &'a [&'a str],
}

impl<'a> FreqStrategy<'a> {
    pub fn new(words: &'a [&'a str]) -> FreqStrategy<'a> {
        FreqStrategy {
            words,
        }
    }
}

impl<'a> GuessStrategy for FreqStrategy<'a> {
    fn next_guess(&self, knowledge: &Knowledge) -> &'a str {
        let candidates = knowledge.filter(&self.words);

        if candidates.len() < 100 {
            println!("Candidates ({}):", candidates.len());
            for w in candidates.iter() {
                println!("  {}", w);
            }
            if candidates.len() < 3 {
                return &candidates[0];
            }
        } else {
            println!("Candidates: {}", candidates.len());
        }

        let freq = char_freq(&candidates);

        // Find the words which give you the best character coverage
        let mut coverage = HashSet::new();
        coverage.extend(knowledge.get_covered());

        let word_scores = freq_scored_guesses(self.words, &freq, &coverage);

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
        guesses[0]
    }
}

pub fn valid_words(target_word_len: usize, all_words: &[String], disallowed: HashSet<String>) -> Vec<&str> {
    all_words
        .iter()
        .filter(|s| s.chars().all(|ch| ch.is_lowercase()))
        .filter(|s| s.len() == target_word_len)
        .filter(|s| !disallowed.contains(s.as_str()))
        .map(|s| s.as_str())
        .collect()
}

pub struct EntropyStrategy<'a> {
    words: &'a [&'a str],
}

impl<'a> EntropyStrategy<'a> {
    pub fn new(words: &'a [&'a str]) -> EntropyStrategy<'a> {
        EntropyStrategy {
            words,
        }
    }

    fn entropy_of_guess(candidates: &[&str], w: &str) -> f32 {
        let num_candidates = candidates.len() as f32;

        let pattern_counts = candidates.iter()
            .map(|c| (LetterOutcome::pattern(w, c), 1i32))
            .fold(HashMap::with_capacity(300), |mut acc, (p, c)| {
                *acc.entry(p).or_default() += c;
                acc
            });

        let word_entropy = pattern_counts.values()
            .map(|c: &i32| {
                let p = *c as f32 / num_candidates;
                p * -p.log2()
            })
            .sum::<f32>();
        word_entropy
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
enum LetterOutcome {
    HIT,
    MISS,
    ABSENT,
}

impl LetterOutcome {
    fn _each() -> [LetterOutcome; 3] {
        [HIT, MISS, ABSENT]
    }

    fn pattern(guess: &str, target: &str) -> Vec<LetterOutcome> {
        // let target_chs: HashSet<char> = target.chars().collect();
        let mut ret: Vec<LetterOutcome> = Vec::with_capacity(guess.len());
        ret.extend(
        guess.chars()
            .zip(target.chars())
            .map(|(g, t)| {
                if g == t {
                    HIT
                } else if target.chars().any(|ch| ch == g) {
                    MISS
                } else {
                    ABSENT
                }
            })
        );
        ret
    }
}

impl<'a> GuessStrategy for EntropyStrategy<'a> {
    fn next_guess(&self, knowledge: &Knowledge) -> &'a str {
        let candidates = knowledge.filter(&self.words);

        println!("Candidates: {}", candidates.len());

        if candidates.len() < 3 {
            return &candidates[0];
        }

        if candidates.len() == self.words.len() {
            // This case takes forever, is common, and raise is the computed value
            let ret = "raise";
            let word_entropy = Self::entropy_of_guess(&candidates, ret);
            println!("{}: H = {}", ret, word_entropy);

            return "raise"
        }

        // For each word, find the guess result patterns that can occur given the candidates, and how frequent each pattern is
        let (guess, entropy) = self.words.iter()
            .map(|w| {
                let word_entropy = Self::entropy_of_guess(&candidates, w);

                (w, word_entropy)
            })
            .max_by(|(_, l), (_, r)| l.partial_cmp(r).unwrap())
            .unwrap();

        println!("{}: H = {}", guess, entropy);
        guess
    }
}

#[cfg(test)]
mod test {}