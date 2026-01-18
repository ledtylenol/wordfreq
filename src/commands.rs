use anyhow::Result;
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
}

#[derive(Args)]
#[group(required = true, multiple = false)]
pub struct FileArgs {
    #[arg(value_name = "PATH")]
    analyze: PathBuf,

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
                let _ = write_to_file(&out, &res)
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
            println!("{v:?}");
        } else {
            let path = &self.file_args.analyze;
            let mut f = File::open(path).expect("could not open the specified file");
            let mut data = String::new();
            f.read_to_string(&mut data).expect("could not read file");

            let processor = WordProcessor::from_str(&data, self.get_set());
            self.top(&processor);
            self.diversity(&processor);
            self.out(&processor);
        }
    }
}
fn write_to_file(path: &PathBuf, data: &str) -> Result<()> {
    let mut opts = OpenOptions::new();
    opts.write(true).truncate(true).create(true);
    let mut file = opts.open(path)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}
