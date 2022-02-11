use std::collections::HashSet;
use std::time;

use solver::{EntropyStrategy, GuessStrategy, valid_words};
use wordlib::{Knowledge, words_from_file};

use rand::seq::SliceRandom;

#[test]
fn average_guesses() {
    const LEN: usize = 5;
    let all_words: Vec<String> = words_from_file("/usr/share/dict/words").unwrap();
    let target_words: Vec<&str> = valid_words(LEN, &all_words, HashSet::new());

    let guesser = EntropyStrategy::new(&target_words);
    // let guesser = FreqStrategy::new(&target_words);

    let mut total_attempts: u64 = 0;
    let mut failures = 0;

    let mut rng = &mut rand::thread_rng();
    let puzzles: Vec<&str> = target_words.choose_multiple(&mut rng, 500).copied().collect();

    let start = time::Instant::now();
    for answer in puzzles.iter().copied() {
        let mut knowledge = Knowledge::new(LEN);

        let mut num_attempts: Option<u64> = None;
        for attempt in 1..7 {
            let guess = guesser.next_guess(&knowledge);

            if guess == answer {
                num_attempts = Some(attempt);
                break;
            }
            knowledge.learn(guess, answer);
        }
        if let Some(x) = num_attempts {
            total_attempts += x;
        } else {
            total_attempts += 7;
            failures += 1;
        }
    }
    let end = time::Instant::now();

    let word_count = puzzles.len() as f32;
    let average_attempts: f32 = total_attempts as f32 / word_count;
    println!("{} words. Average attempts {}. {} failures.", word_count, average_attempts, failures);

    let duration = end.duration_since(start);
    let guess_time = duration.div_f64(total_attempts as f64);

    println!("{} attempts in {}ms. {} µs/attempt", total_attempts, duration.as_millis(), guess_time.as_micros());

    assert!(average_attempts < 4.1);
    assert!(failures as f32 / word_count < 0.001);
}
