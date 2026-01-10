use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Read,
    path::PathBuf,
    time::Instant,
};

use clap::Parser;
use serde::Deserialize;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Commands {
    /// File to analyze
    #[arg(value_name = "FILE")]
    file: PathBuf,

    /// Whether to include stopwords
    #[arg(short = 'a', long)]
    analyze_stopwords: bool,

    /// List the top N words
    #[arg(short = 't', long, value_name = "N", value_parser = 1..10000)]
    top: Option<usize>,
    /// Lists the bottom N words
    /// WARNING: sorting is non deterministic, so multiple rare words will be random on every call
    #[arg(long, value_name = "N", value_parser = 1..10000)]
    bottom: Option<usize>,

    ///Show various statistics about diversity
    #[arg(short = 'd', long)]
    diversity: bool,
}

struct WordProcessor {
    pub words: Vec<WordData>,
    pub avglen: f64,
    pub ttr: f64,
    pub total_words: usize,
    pub rare_words: usize,
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
            words,
            avglen,
            total_words,
            ttr,
            rare_words,
        }
    }
}
struct WordData {
    pub word: String,
    pub count: usize,
}

#[derive(Deserialize)]
struct WordFilter(HashSet<String>);

impl WordFilter {
    fn contains(&self, s: &str) -> bool {
        self.0.contains(s)
    }
}

fn main() {
    let commands = Commands::parse();
    let now = Instant::now();
    let mut data = String::new();

    let set = if !commands.analyze_stopwords {
        let filter_words = include_str!("../stop_words.json");
        Some(serde_json::from_str(filter_words).unwrap())
    } else {
        None
    };
    let filename = commands.file;

    let mut f = File::open(filename).expect("filesystem error");

    //TODO: handle by line? prevent allocating too much memory
    f.read_to_string(&mut data).expect("could not read file");
    let processor = WordProcessor::from_str(&data, set);
    if let Some(num) = commands.top {
        println!();
        if num > data.len() {
            println!("the given number exceeds the total word count. continuing anyway");
        }
        for (i, WordData { word, count }) in processor.words.iter().take(num).enumerate() {
            println!("top {} word: {word:?} with {count} appearances", i + 1);
        }
    }
    if let Some(num) = commands.bottom {
        println!();
        if num > data.len() {
            println!("the given number exceeds the total word count. continuing anyway");
        }
        for (i, WordData { word, count }) in processor.words.iter().rev().take(num).enumerate() {
            if *count > 1 {
                println!("bottom {} word: {word:?} with {count} appearances", i + 1);
            } else {
                println!("bottom {} word: {word:?}", i + 1);
            }
        }
    }

    if commands.diversity {
        println!();
        println!(
            "Diversitate:\nTotal cuvinte: {total}\nCuvinte unice: {unic} ({procent:.1}%)\nToken-Type Ratio: {ratio} ({diversitate})\n",
            total = processor.total_words,
            unic = processor.words.len(),
            procent = processor.ttr * 100.0,
            ratio = processor.ttr,
            //TODO:
            diversitate = "todo"
        );
        //should never panic
        let max = processor
            .words
            .iter()
            .max_by(|a, b| a.word.len().cmp(&b.word.len()))
            .unwrap();
        println!(
            "Lungimea medie cuvant: {len:.2}\nCel mai lung cuvant: \"{cuv}\" ({caractere} caractere)\n",
            len = processor.avglen,
            cuv = max.word,
            caractere = max.word.len()
        );
        println!(
            "Cuvinte rare (1 aparitie): {count}, ({percent:.1}% din vocabular)",
            count = processor.rare_words,
            percent = 100.0 * processor.rare_words as f64 / processor.words.len() as f64
        )
    }
    println!("processing finished after {} ms", now.elapsed().as_millis());
}
