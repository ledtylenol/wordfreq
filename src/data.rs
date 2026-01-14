use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct WordFilter(HashSet<String>);
impl WordFilter {
    pub fn contains(&self, s: &str) -> bool {
        self.0.contains(s)
    }
}

#[derive(Serialize)]
pub struct WordProcessor {
    pub avglen: f64,
    pub ttr: f64,
    pub total_words: usize,
    pub rare_words: usize,
    pub unique_words: usize,
    pub words: Vec<WordData>,
}

impl WordProcessor {
    pub fn from_str(data: &str, filter: Option<WordFilter>) -> Self {
        let mut total_words = 0;
        let collect_to_hashmap = |mut acc: HashMap<_, _>, elem| {
            acc.entry(elem).and_modify(|e| *e += 1).or_insert(1);
            acc
        };
        let data = data
            .split(|c: char| {
                c.is_whitespace()
                    || c == ','
                    || c == '.'
                    || c == '"'
                    || c == '!'
                    || c == '?'
                    || c == '-'
                    || c == '—'
            })
            .map(|word| {
                total_words += 1;
                word.to_lowercase()
                    .chars()
                    .take_while(|char| char.is_alphabetic())
                    .collect::<String>()
            })
            .filter(|s| !s.is_empty());
        let data = match filter {
            Some(f) => data
                .filter(|data| !f.contains(data))
                .fold(HashMap::new(), collect_to_hashmap),
            None => data.fold(HashMap::new(), collect_to_hashmap),
        };
        let mut words = data
            .into_iter()
            .map(|(word, count)| WordData { word, count })
            .collect::<Vec<WordData>>();
        words.sort_by(|a, b| b.count.cmp(&a.count));

        let avglen =
            words.iter().map(|data| data.word.len()).sum::<usize>() as f64 / words.len() as f64;
        let ttr = words.len() as f64 / total_words as f64;
        let rare_words = words.iter().filter(|word| word.count == 1).count();
        Self {
            //store the length for json purposes
            unique_words: words.len(),
            words,
            avglen,
            total_words,
            ttr,
            rare_words,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct WordData {
    pub word: String,
    pub count: usize,
}
