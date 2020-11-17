# digital-duration-nom

A tiny library that parses and prints string representations of
durations that are commonly used in many sports (e.g., in running
1:23:45 typically means one hour, twenty-three minutes, forty five
seconds).

This crate started (and largely remains) a place for me to put some
code that I am using in a few CLI tools, all of which already use nom.
It uses the NewType pattern to wrap std::time::Duration, which works
for me, but is certainly a compromise.

### Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
digital-duration-nom = "0.2.0"
```

### Dependencies

[Nom](https://github.com/Geal/nom) is currently a dependency, because this
code was refactored out of software that was already using nom and I don't
really expect this code to be useful to anyone other than myself.  As such,
I don't see any benefit to getting rid of the nom dependency, although it
could easily be done if there were a reason to.

## Public Domain

digital-duration-nom has been released into the public domain, per the
[UNLICENSE](UNLICENSE).
