use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs::File;
use std::io;
use std::io::Read;

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

pub fn words_from_file<T: FromIterator<String>>(filename: &str) -> io::Result<T> {
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents.lines()
        .map(|s| s.to_string())
        .collect())
}

#[derive(Clone, Hash, Debug)]
pub struct Knowledge {
    /// Represents the length and known character positions
    pattern: Vec<Option<char>>,
    /// All characters known to be present
    present: BTreeSet<char>,
    /// Characters known absent from the word
    absent: BTreeSet<char>,
    /// Characters present but not hits
    misses: Vec<BTreeSet<char>>,
}

impl Knowledge {
    pub fn new(length: usize) -> Knowledge {
        Knowledge {
            pattern: vec![None; length],
            present: BTreeSet::new(),
            absent: BTreeSet::new(),
            misses: vec![BTreeSet::new(); length]
        }
    }

    pub fn from_tries(pattern: &str, present_chs: &str, tries: &[&str]) -> Knowledge {
        let length = pattern.len();
        let mut ret = Knowledge::new(length);
        for (i, ch) in pattern.chars().enumerate() {
            if ch != '.' {
                ret.add_hit(ch, i);
            }
        }

        let present: HashSet<char> = present_chs.chars().collect();

        for &w in tries {
            for (i, ch) in w.chars().enumerate() {
                if present.contains(&ch) {
                    match &ret.pattern[i] {
                        None => ret.add_miss(ch, i),
                        Some(hit) => {
                            if *hit != ch {
                                ret.add_miss(ch, i);
                            }
                        }
                    }
                } else {
                    ret.absent.insert(ch);
                }
            }
        }

        ret
    }

    pub fn is_empty(&self) -> bool {
        self.pattern.iter().all(|x| x.is_none()) &&
            self.absent.is_empty() &&
            self.present.is_empty() &&
            self.misses.iter().all(|s| s.is_empty())
    }

    pub fn fits(&self, ch: &char, at: usize) -> bool {
        if let Some(known) = self.pattern[at] {
            return known == *ch
        } else if self.present.contains(&ch){
            !self.misses[at].contains(&ch)
        } else {
            !self.absent.contains(&ch)
        }
    }

    pub fn check_word(&self, w: &str) -> bool {
        if w.len() == self.pattern.len() {
            if w.chars()
                .enumerate()
                .all(|(i, ch)| self.fits(&ch, i)) {
                //Each character fits (is plausible), but does w cover all the things we know?
                let word_chars: BTreeSet<char> = w.chars().collect();
                word_chars.is_superset(&self.present)
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn filter<'a>(&self, words: &[&'a str]) -> Vec<&'a str> {
        words.iter().filter(|w| self.check_word(w)).copied().collect()
    }

    fn add_hit(&mut self, ch: char, at: usize) {
        self.pattern[at] = Some(ch);
        self.present.insert(ch);
    }

    fn add_miss(&mut self, ch: char, at: usize) {
        self.misses[at].insert(ch);
        self.present.insert(ch);
    }

    fn add_absent(&mut self, ch: char) {
        self.absent.insert(ch);
    }

    pub fn get_covered(&self) -> HashSet<char> {
        self.present.iter().chain(self.absent.iter()).copied().collect()
    }

    pub fn learn(&mut self, guess: &str, answer: &str) {
        let answer_set: HashSet<char> = answer.chars().collect();
        guess.chars().zip(answer.chars())
            .enumerate()
            .for_each(|(i, (ch, answer_char))| {
                if ch == answer_char {
                    self.add_hit(ch, i);
                } else if answer_set.contains(&ch) {
                    self.add_miss(ch, i);
                } else {
                    self.add_absent(ch);
                }
            });
    }
}
