# tren

A toy transaction engine, to play with banking concepts

## run

``` bash
> cargo run -- --help

Usage: tren <file_path>

Arguments:
  <file_path>  Filename to operate on (e.g. 'transactions.csv')

Options:
  -h, --help     Print help
  -V, --version  Print version
```

``` bash
cargo run -- src/tests/base_transactions.csv
```

## test

``` bash
# unsurprisingly
cargo test
```

## lint

``` bash
cargo clippy -- -W clippy::pedantic -D warnings
```

## CI

Project will be tested on push through GitHub Actions. You can pre-test your changes using

``` bash
cargo fmt --all -- --check
cargo clippy -- -W clippy::pedantic -D warnings
cargo test
```

or, with some more setup using [GitHub CLI](https://github.com/cli/cli/tree/trunk) and [act](https://nektosact.com/introduction.html) (or standalone `act` if available for your system)

``` bash
gh act
```

## Assumptions

 * The csv is correct, meaning e.g. that dispute rows have an empty amount
   * the program will exit on plain wrong rows (e.g. too many or too few columns)
 * The default store is an in-memory store, which assumes we have enough memory available to fit the data. In a real case scenario, it would be some kind of DB, drastically reducing memory usage
 * Also, the access pattern is "optimized" (~"hopefully good enough") for the exercise, meaning e.g. since there is no interaction between accounts each account can keep its own separate list of transactions
   * this is a list because at the beginning I have foreseen the possibility to "rewind" transactions after resolving a dispute. This also gives an easy way to preserve local chronological order. However turning back to a HashMap, ordered set or similar is trivial if the list length becomes suboptimal
 * It is assumed a precision of 4 digits after decimals, but the input is permissive. However, the output will be rounded to the 4th digit
 * It is assumed that a transaction that has been skipped (e.g. a withdrawal with insufficient funds) cannot be disputed
 * It is assumed that only deposits and withdrawals can be disputed (and subsequently resolved or charged back)
 * transactions that do not make sense are just skipped, including but not limited to
   * dispute transactions targeting a non existent transaction
   * dispute transactions from a client targeting a different client
   * dispute transactions targeting another dispute transaction
