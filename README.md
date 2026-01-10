Word frequency calculator


Usage: wordfreq \[file] \[additional flags]

By default the command displays nothing. Every output is determined by the flags you introduce

# FLAGS:
> -t --top N

  Display the top N words by frequency
> -b --bottom N

  Display the bottom N words by frequency (in most cases all of them will be rare words)

> -d --diversity 

  Display various statistics about the text
> -a --analyze-stopwords

  Include stopwords (and, of, was, is, were etc) when parsing

## WARNING:

Due to hashmap usage the order is **NON DETERMINISTIC**! 2 words of the same frequency will not have the same ordering on different executions
