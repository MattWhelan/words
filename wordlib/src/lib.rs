use std::collections::{HashMap, HashSet};
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

struct PosInfo {
    hit: HashSet<usize>,
    miss: HashSet<usize>,
}

impl Default for PosInfo {
    fn default() -> Self {
        PosInfo {
            hit: HashSet::new(),
            miss: HashSet::new(),
        }
    }
}

pub struct Knowledge {
    length: usize,
    matches: HashMap<char, PosInfo>,
    absent: HashSet<char>,
}

impl Knowledge {
    pub fn new(length: usize) -> Knowledge {
        Knowledge {
            length,
            matches: HashMap::new(),
            absent: HashSet::new(),
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
                    let pos = ret.matches.entry(ch).or_default();
                    if !pos.hit.contains(&i) {
                        ret.add_miss(ch, i);
                    }
                } else {
                    ret.absent.insert(ch);
                }
            }
        }

        ret
    }

    pub fn fits(&self, ch: &char, at: usize) -> bool {
        if let Some(pos) = self.matches.get(ch) {
            if pos.hit.contains(&at) {
                true
            } else {
                if pos.miss.contains(&at) {
                    false
                } else {
                    // No known positional match covers this position
                    !self.matches.iter()
                        .any(|(_, pos)| pos.hit.contains(&at))
                }
            }
        } else {
            if self.absent.contains(ch) {
                false
            } else {
                // No known positional match covers this position
                !self.matches.iter()
                    .any(|(_, pos)| pos.hit.contains(&at))
            }
        }
    }

    pub fn check_word(&self, w: &str) -> bool {
        if w.len() == self.length {
            if w.chars()
                .enumerate()
                .all(|(i, ch)| self.fits(&ch, i)) {
                let word_chars: HashSet<char> = w.chars().collect();
                self.matches.keys().all(|ch| word_chars.contains(ch))
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn add_hit(&mut self, ch: char, at: usize) {
        let pos = self.matches.entry(ch).or_default();
        pos.hit.insert(at);
    }

    pub fn add_miss(&mut self, ch: char, at: usize) {
        let pos = self.matches.entry(ch).or_default();
        pos.miss.insert(at);
        if pos.miss.len() == self.length - 1 {
            let hit = (0..self.length).filter(|i| !pos.miss.contains(i)).next().unwrap();
            pos.hit.insert(hit);
        }
    }

    pub fn add_absent(&mut self, ch: char) {
        self.absent.insert(ch);
    }

    pub fn get_absent(&self) -> &HashSet<char> {
        &self.absent
    }

    pub fn get_covered(&self) -> HashSet<char> {
        if self.length == self.matches.len() {
            ('a'..='z').collect()
        } else {
            self.matches.keys()
                .chain(self.absent.iter())
                .copied()
                .collect()
        }
    }
}
