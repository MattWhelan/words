use std::collections::{HashMap, HashSet};

pub fn show_freq(freq: &HashMap<char, usize>) {
    let mut by_freq = Vec::from_iter(freq.iter());
    by_freq.sort_by_key(|(_, v)| *v);
    by_freq.reverse();
    println!("Letter | Count");
    for (letter, count) in by_freq {
        println!("{:7}| {}", letter, count);
    }
    println!()
}

pub fn coverage_guesses<'a>(
    words: &'a [&str],
    freq: &HashMap<char, usize>,
    covered: &mut HashSet<char>,
) -> Vec<&'a str> {
    //Score each word by freq-points
    let best = words
        .iter()
        .max_by_key(|w| word_score(w, freq, covered))
        .unwrap();
    covered.extend(best.chars());

    let mut ret = if covered.len() < freq.len() {
        coverage_guesses(words, freq, covered)
    } else {
        Vec::new()
    };
    ret.insert(0, best);

    ret
}

pub fn freq_scored_guesses<'a>(
    words: &'a [&str],
    freq: &HashMap<char, usize>,
    covered: &HashSet<char>,
) -> Vec<(&'a str, usize)> {
    //Score each word by freq-points
    let mut words: Vec<(&str, usize)> = words
        .iter()
        .map(|&w| (w, word_score(w, freq, covered)))
        .collect();
    words.sort_by_key(|(_, s)| *s);
    words.reverse();

    words
}

fn word_score(word: &str, freq: &HashMap<char, usize>, covered: &HashSet<char>) -> usize {
    let uniques: HashSet<char> = word.chars().collect();
    uniques
        .into_iter()
        .map(|ch| {
            if covered.contains(&ch) {
                0
            } else {
                *freq.get(&ch).unwrap_or(&0)
            }
        })
        .sum()
}

pub fn char_freq(words: &[&str]) -> HashMap<char, usize> {
    words
        .iter()
        .flat_map(|w| w.chars())
        .fold(HashMap::new(), |mut acc, ch| {
            *acc.entry(ch).or_default() += 1;
            acc
        })
}
