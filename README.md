# sports-metrics

A tiny library that handles string representations of durations that are
commonly used in some sports.

This crate is simply a place for me to put some code that I am using in two
slightly different CLI tools, both of which are currently private projects.
I struggled to come up with a name that was not overly specific, and wound
up with one that is probably too generic.  For now, a better name might be
trivial-extensions-to-duration-that-ctm-finds-useful.

### Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
sports-metrics = "0.1"
```

### Dependencies

[Nom](https://github.com/Geal/nom) is a dependencie, not because parsing
durations is by hand is hard.  One of the first uses for sports-metrics is to
extract finish times from race results, and those times are often embedded
in text that is amenable to parsing via nom.  As such, the duration parsers
are written in nom so they can be combined with other parsers.

## Public Domain

sports-metrics has been released into the public domain, per the [UNLICENSE](UNLICENSE).
