use std::{
    collections::{HashMap, HashSet},
    env,
    fs::{File, read_to_string},
    io::Read,
};

use serde::Deserialize;

struct WordProcessor {
    pub words: Vec<WordData>,
    pub avglen: usize,
    pub ttr: f64,
    pub total_words: usize,
}

impl WordProcessor {
    pub fn diversity(&self) -> f64 {
        self.total_words as f64 / self.words.len() as f64
    }
    pub fn from_str(data: &str, filter: Option<WordFilter>) -> Self {
        let mut total_words = 0;
        let collect_to_hashmap = |mut acc: HashMap<_, _>, elem| {
            acc.entry(elem).and_modify(|e| *e += 1).or_insert(1);
            acc
        };
        let data = data
            .split(|c: char| c.is_whitespace() || c == ',' || c == '.' || c == '"')
            .filter(|s| !s.is_empty())
            .map(|word| {
                total_words += 1;
                word.to_lowercase()
                    .chars()
                    .filter(|char| char.is_alphabetic())
                    .collect::<String>()
            });
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

        let avglen = words.iter().map(|data| data.word.len()).sum::<usize>() / words.len();
        let ttr = words.len() as f64 / total_words as f64;
        Self {
            words,
            avglen,
            total_words,
            ttr,
        }
    }
}
struct WordData {
    pub word: String,
    pub count: usize,
}

impl WordData {
    fn new(word: String, count: usize) -> Self {
        Self { word, count }
    }
}
#[derive(Deserialize)]
struct WordFilter(HashSet<String>);

impl WordFilter {
    fn insert(&mut self, s: impl Into<String>) -> bool {
        self.0.insert(s.into())
    }
    fn contains(&self, s: &str) -> bool {
        self.0.contains(s)
    }
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let num = args.len();
    let mut data = String::new();

    let set = if !args.iter().any(|arg| arg == "--filter-stopwords") {
        None
    } else {
        let filter_words = read_to_string("./stop_words.json").expect("cannot find stopword file");
        Some(serde_json::from_str(&filter_words).unwrap())
    };
    println!("{args:?}, {num}");
    let filename = args.get(1).expect("invalid filename");

    let mut f = File::open(filename).expect("eroare de filesystem");

    f.read_to_string(&mut data).expect("could not read file");
    let processor = WordProcessor::from_str(&data, set);
    for args in args.windows(2) {
        let (Some(arg1), Some(arg2)) = (args.first(), args.get(1)) else {
            continue;
        };
        match (arg1.as_str(), arg2.as_str()) {
            ("--top", num) => {
                let num = num
                    .parse::<usize>()
                    .expect("--top must be followed by a valid number");
                if num > data.len() {
                    println!("the given number exceeds the total word count. continuing anyway");
                }
                for (i, WordData { word, count }) in processor.words.iter().take(num).enumerate() {
                    println!("top {} word: {word} with {count} appearances", i + 1);
                }
            }
            ("--diversity", _) | (_, "--diversity") => {
                print!(
                    "Diversitate:\nTotal cuvinte: {total}\nCuvinte unice: {unic} ({procent:.1})\nToken-Type Ratio: {ratio} ({diversitate})",
                    total = processor.total_words,
                    unic = processor.words.len(),
                    procent = processor.ttr * 100.0,
                    ratio = processor.ttr,
                    diversitate = "todo"
                )
            }
            _ => (),
        }
    }
}
