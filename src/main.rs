use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
    time::Instant,
};

use anyhow::Result;
use clap::Parser;

use crate::{
    commands::Commands,
    data::{WordData, WordProcessor},
};

mod commands;
mod data;

fn write_to_file(path: &PathBuf, data: &str) -> Result<()> {
    let mut opts = OpenOptions::new();
    opts.write(true).truncate(true).create(true);
    let mut file = opts.open(path)?;
    file.write_all(data.as_bytes())?;
    Ok(())
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
            "Diversity:\nTotal words: {total}\nUnique words: {unic} ({procent:.1}%)\nToken-Type Ratio: {ratio} ({diversitate})\n",
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
            "Average word length: {len:.2}\nLongest word: \"{cuv}\" ({caractere} characters)\n",
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
    if let Some(out) = commands.out {
        if commands.write_words {
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
        } else {
            todo!("add non-word-export-mode-thing");
        }
    }
    println!("processing finished after {} ms", now.elapsed().as_millis());
}
