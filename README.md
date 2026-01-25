# Word frequency calculator

## Author
- **Name:** \[omitted] (LedTylenol)
- **Group** 3.1
- **Email:** \[omitted]
- **Year:** 2025-2026

## Description

Basic command line NLP tool, it takes a text as input and parses it, then returns the processed information depending on the flags it was called with.

## Technologies

- **Language** Rust 1.91.1
- Libraries:
  - [Clap](https://crates.io/crates/clap)    - Command Line Argument Parser, for a clean and easy to use command parser
  - [Rand](https://crates.io/crates/rand)    - For cloud generation randomness
  - [Anyhow](https://crates.io/crates/anyhow)  - Simplified error handling since any possible error is unrecoverable
  - [Serde](https://crates.io/crates/serde) and [Serde_json](https://crates.io/crates/serde_json) - Data deserialization
- Tools: Git, Cargo

## System Requirements

On windows: [Visual C++ Redistributable](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist#latest-microsoft-visual-c-redistributable-version)

On every platform: [cargo](https://rustup.rs)

## Installation

```bash
# Clone the repository
git clone https://github.com/ledtylenol/wordfreq.git
cd wordfreq
# Install using cargo
cargo install --path . --locked
```
## Running the app

run `wordfreq --help` for more information
# Base command:
`wordfreq <PATH>` **OR** `wordfreq --compare <PATH1> <PATH2>`

### Possible flags:

- `--top <N>`
  - List the top N words
- `--concordance <TEXT>`
  - Search for context related to TEXT
  - Optional command: `--max <N>`
    - Maximum occurrences to be listed
- `--out <PATH>`
  - Serialize to PATH as JSON
- `--diversity`
  - List various statistics about the diversity of the text
- `--n-grams <2|3>`
  - Print bigrams (2) or trigrams (3) instead of words
- `--cloud`
  - Print a word cloud
  - Optional command: `--width <N>`
    - Word cloud char limit (default 40)
- `--custom-filter <PATH>`
  - Optional JSON word list to use instead of the default one
- `--analyze-stopwords`
  - Include stopwords in analysis


## WARNING:

Due to hashmap usage the order is **NON DETERMINISTIC**! 2 words of the same frequency will not have the same ordering on different executions
