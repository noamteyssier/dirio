# dirio

A simple tool for monitoring disk usage over the runtime of a command.

This expects `du` to be in your `$PATH` and evaluates it at a specified interval while a test command is running.

It tracks and reports a TSV of the following:

1. elapsed time
2. observed disk usage
3. disk usage delta (from initial disk usage)
4. disk usage peak (max observed disk usage over the runtime)

## Installation

```bash
cargo install dirio
```

## Usage

This monitors the bytes of a directory over the runtime of a command.
By default that directory is `./` or the current working directory.
You can specify some other directory with the `-p` flag.

```bash
# Monitor disk usage and output to stdout
dirio "fasterq-dump SRR000001 --split-files --include-technical"

# Monitor disk usage and output to some log file
dirio -o dirio.log.tsv "fasterq-dump SRR000001 --split-files --include-technical"
```
