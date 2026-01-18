use anyhow::Result;
use rand::Rng;
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
};

use clap::{Args, Parser};

use crate::data::{WordData, WordFilter, WordProcessor};
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Commands {
    #[command(flatten)]
    file_args: FileArgs,

    /// Whether to include stopwords
    #[arg(short = 'a', long)]
    pub analyze_stopwords: bool,

    /// List the top N words
    #[arg(short = 't', long, value_name = "N", value_parser = 1..10000)]
    pub top: Option<i64>,

    /// Show various statistics about diversity
    #[arg(short = 'd', long)]
    pub diversity: bool,

    /// Path to write to
    #[arg(long, short = 'o')]
    pub out: Option<PathBuf>,

    /// Whether to print n_gram data
    #[arg(long, value_parser = 2..4, requires("top"))]
    pub n_grams: Option<i64>,

    /// Custom stopword filter to use instead of the default one
    #[arg(long)]
    pub custom_filter: Option<PathBuf>,

    /// Context search string
    #[arg(long)]
    pub concordance: Option<String>,

    /// Whether to print a word cloud
    #[arg(long)]
    pub cloud: bool,

    /// Custom word cloud width
    #[arg(long, short, requires = "cloud", default_value_t = 40)]
    pub width: usize,
}

#[derive(Args)]
#[group(required = true, multiple = false)]
pub struct FileArgs {
    /// Analyze one text
    #[arg(value_name = "PATH")]
    analyze: Option<PathBuf>,

    /// Compare the texts
    #[clap(long,value_name = "PATHS", value_delimiter = ' ', num_args = 2..3)]
    compare: Option<Vec<PathBuf>>,
}

impl Commands {
    pub fn top(&self, processor: &WordProcessor) {
        let Some(num) = self.top.map(|num| num as usize) else {
            return;
        };
        println!();
        if let Some(n_grams) = self.n_grams.map(|n| n as usize) {
            if n_grams == 2 {
                if num > processor.bigrams.len() {
                    println!("the given number exceeds the total word count. continuing anyway");
                }
                println!("top {num} bigrams:");
                for (i, WordData { text, count }) in processor.bigrams.iter().take(num).enumerate()
                {
                    let percent = 100.0 * *count as f64 / processor.unique_words as f64;
                    println!(
                        "    {}. {text:<10?} - {count} appearances ({percent:.2}%)",
                        i + 1
                    );
                }
            } else {
                if num > processor.trigrams.len() {
                    println!("the given number exceeds the total word count. continuing anyway");
                }
                println!("top {num} trigrams:");
                for (i, WordData { text, count }) in processor.trigrams.iter().take(num).enumerate()
                {
                    let percent = 100.0 * *count as f64 / processor.unique_words as f64;
                    println!(
                        "    {}. {text:<10?} - {count} appearances ({percent:.2}%)",
                        i + 1
                    );
                }
            }
        } else {
            if num > processor.words.len() {
                println!("the given number exceeds the total word count. continuing anyway");
            }
            println!("top {num} words:");
            for (i, WordData { text, count }) in processor.words.iter().take(num).enumerate() {
                let percent = 100.0 * *count as f64 / processor.unique_words as f64;
                println!(
                    "    {}. {text:<10?} - {count} appearances ({percent:.2}%)",
                    i + 1
                );
            }
        }
    }
    pub fn diversity(&self, processor: &WordProcessor) {
        if !self.diversity {
            return;
        }
        println!();
        println!(
            "Diversity:\nTotal words: {total}\nUnique words: {unic} ({procent:.1}%)\nToken-Type Ratio: {ratio} ({diversitate})\n",
            total = processor.total_words,
            unic = processor.words.len(),
            procent = processor.ttr * 100.0,
            ratio = processor.ttr,
            diversitate = processor.get_variation_string()
        );
        //should never panic
        let max = processor
            .words
            .iter()
            .max_by(|a, b| a.text.len().cmp(&b.text.len()))
            .unwrap();
        println!(
            "Average word length: {len:.2}\nLongest word: \"{cuv}\" ({caractere} characters)\n",
            len = processor.avglen,
            cuv = max.text,
            caractere = max.text.len()
        );
        println!(
            "Rare words (1 appearance): {count}, ({percent:.1}% of vocabilary)",
            count = processor.rare_words,
            percent = 100.0 * processor.rare_words as f64 / processor.words.len() as f64
        )
    }
    pub fn out(&self, processor: &WordProcessor) {
        let Some(out) = self.out.as_ref() else {
            return;
        };
        match serde_json::ser::to_string_pretty(&processor) {
            Ok(res) => {
                println!("success. writing to {out:?}");
                let _ = write_to_file(out, &res)
                    .inspect_err(|e| eprintln!("could not write to file: {e}"));
            }
            Err(e) => {
                eprintln!("could not write to file: {e}");
            }
        }
    }

    pub fn get_set(&self) -> Option<WordFilter> {
        if !self.analyze_stopwords {
            if let Some(path) = self.custom_filter.as_ref() {
                let mut text = String::new();
                let mut file = File::open(path).expect("could not open the specified filter file");
                file.read_to_string(&mut text)
                    .expect("could not read the specified filter file");
                Some(
                    serde_json::from_str(&text)
                        .expect("could not parse the words from the specified filter file"),
                )
            } else {
                Some(serde_json::from_str(include_str!("../stop_words.json")).unwrap())
            }
        } else {
            None
        }
    }

    pub fn handle_commands(&self) {
        if let Some(v) = self.file_args.compare.as_ref() {
            todo!("compare!!! {v:?}");
        } else if let Some(path) = &self.file_args.analyze {
            let mut f = File::open(path).expect("could not open the specified file");
            let mut data = String::new();
            f.read_to_string(&mut data).expect("could not read file");

            let processor = WordProcessor::from_str(&data, self.get_set());
            self.top(&processor);
            self.diversity(&processor);
            self.out(&processor);
            self.concordance(&data);
            self.cloud(&processor);
        };
    }

    pub fn concordance(&self, haystack: &str) {
        let Some(needle) = self.concordance.as_ref() else {
            return;
        };

        // turn the haystack into a vec of words
        let words = haystack.split_whitespace().collect::<Vec<_>>();

        let word_count = words.len();
        // save all indices wherein the word is found
        let v = words
            .iter()
            .enumerate()
            .filter(|(_, word)| word.to_lowercase().contains(needle))
            .map(|(i, _)| i)
            .collect::<Vec<_>>();

        // go through every index
        for &i in v.iter() {
            // compute whether there's more words to the left and right
            let overshoot = i + 2 > word_count;
            let undershoot = i < 1;
            // get the min and max indices
            let min_i = (i - 3).max(0);
            let max_i = (i + 3).min(word_count);
            if !undershoot {
                print!("...");
            }

            let mut first = true;
            // clippy sniped me here
            // (word) -> (i, word) -> (i, word)[0..max_i] -> (i,word)[min_i..max_i]
            for (j, word) in words.iter().enumerate().take(max_i).skip(min_i) {
                if first {
                    if j == i {
                        print!("*{}*", word);
                    } else {
                        print!("{}", word);
                    }

                    first = false;
                } else if j == i {
                    print!(" *{}*", word);
                } else {
                    print!(" {}", word);
                }
            }
            if !overshoot {
                print!("...");
            }
            println!();
        }
    }
    pub fn cloud(&self, processor: &WordProcessor) {
        if !self.cloud {
            return;
        }
        let width = self.width;
        let mut rng = rand::rng();

        //get all words
        let mut v = processor
            .words
            .iter()
            // take the first 30 (top 30 by count)
            .take(30)
            // don't care about count
            .map(|word_data| word_data.text.as_str())
            // get the index
            .enumerate()
            .collect::<Vec<_>>();
        let mut uppercase_count = 0;
        let mut buf = String::new();
        println!("Word cloud: (width {width})");
        println!();
        // keep looping until you have words no more
        while !v.is_empty() {
            // get a random index in range
            let mut r = rng.random_range(0..v.len());
            // keep trying to get non uppercase words until you run out (if you do)
            while uppercase_count > 3 && v[r].0 < 10 && !v.iter().all(|&(i, _)| i < 10) {
                r = rng.random_range(0..v.len());
            }
            // remove found element from vec
            let (i, text) = v.remove(r);
            // if you have exceeded the limit flush the buffer and print the result
            if buf.len() + text.len() >= width {
                uppercase_count = 0;
                // centered padding based on width
                println!("{buf:^dif$}", dif = width);
                buf.clear();
            }
            if i < 10 {
                buf.push_str(&format!("{} ", text.to_uppercase()));
                uppercase_count += 1;
            } else {
                buf.push_str(&format!("{} ", text));
            }
        }
        println!();
    }
}
//exactly what it says on the tin, take a path, some data, write data to file
fn write_to_file(path: &PathBuf, data: &str) -> Result<()> {
    let mut opts = OpenOptions::new();
    opts.write(true).truncate(true).create(true);
    let mut file = opts.open(path)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}
