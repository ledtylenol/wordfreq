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
    pub bigrams: Vec<WordData>,
    pub trigrams: Vec<WordData>,
}

impl WordProcessor {
    pub fn from_str(analyze_text: &str, filter: Option<WordFilter>) -> Self {
        let mut total_words = 0;
        // closure to accumulate to a hashmap
        let collect_to_hashmap = |mut acc: HashMap<_, _>, elem| {
            acc.entry(elem).and_modify(|e| *e += 1).or_insert(1);
            acc
        };

        let data = analyze_text
            .split(|c: char| {
                // common split particles
                c.is_whitespace()
                    || c == ','
                    || c == '.'
                    || c == '"'
                    || c == '!'
                    || c == '?'
                    || c == '-'
                    || c == '—'
            })
            .map(|word| word.to_lowercase())
            .filter(|s| {
                // only get the words that are alphabetic
                let f = !s.is_empty() && s.chars().all(|c| c.is_alphabetic());
                if let Some(filter) = filter.as_ref() {
                    f && !filter.contains(s)
                } else {
                    f
                }
            })
            .inspect(|_| {
                // increment the word count
                total_words += 1;
            })
            .collect::<Vec<_>>();

        let split = analyze_text
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
            .map(|word| word.to_lowercase().chars().collect::<String>())
            .filter(|s| !s.is_empty() && { s.chars().all(|c| c.is_alphabetic()) });
        // tuple of (i, i+1)
        let bigrams = split.clone().zip(split.clone().skip(1)).collect::<Vec<_>>();
        let mut trigrams = bigrams
            .clone()
            .into_iter()
            .zip(split.skip(2))
            .filter(|((a, b), c)| {
                // filter trigrams with more than 1 stopword
                filter.is_none()
                    || filter.as_ref().is_some_and(|filter| {
                        !(filter.contains(a) || filter.contains(b) || filter.contains(c))
                    })
            })
            .map(|((a, b), c)| format!("{a} {b} {c}"))
            .fold(HashMap::new(), collect_to_hashmap)
            .into_iter()
            .map(|(text, count)| WordData { text, count })
            .collect::<Vec<_>>();
        let mut bigrams = bigrams
            .iter()
            .filter(|(a, b)| {
                // filter bigrams with more than 1 stopword
                filter.is_none()
                    || filter
                        .as_ref()
                        .is_some_and(|filter| !(filter.contains(a) || filter.contains(b)))
            })
            .map(|(a, b)| format!("{a} {b}"))
            .fold(HashMap::new(), collect_to_hashmap)
            .into_iter()
            .map(|(text, count)| WordData { text, count })
            .collect::<Vec<_>>();
        let data = data.into_iter().fold(HashMap::new(), collect_to_hashmap);
        let mut words = data
            .into_iter()
            .map(|(text, count)| WordData { text, count })
            .collect::<Vec<WordData>>();
        words.sort_by(|a, b| b.count.cmp(&a.count));
        bigrams.sort_by(|a, b| b.count.cmp(&a.count));
        trigrams.sort_by(|a, b| b.count.cmp(&a.count));

        let avglen =
            words.iter().map(|data| data.text.len()).sum::<usize>() as f64 / words.len() as f64;
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
            bigrams,
            trigrams,
        }
    }
    pub fn get_variation_string(&self) -> String {
        if self.ttr < 0.05 {
            "low variation".into()
        } else if self.ttr < 0.15 {
            "medium variation".into()
        } else if self.ttr < 0.3 {
            "medium-high variation".into()
        } else {
            "high variation".into()
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct WordData {
    pub text: String,
    pub count: usize,
}
