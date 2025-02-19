# tren

A toy transaction engine, to play with banking concepts

## Assumptions

 * The default store is an in-memory store, which assumes we have enough memory available to fit the data. In a real case scenario, it would be a DB, drastically reducing memory usage
 * Also, the access pattern is "optimized" (~"hopefully good enough") for the exercise, meaning e.g. since there is no interaction between accounts each account can keep its own separate list of transactions
   * this is a list because at the beginning I have foreseen the possibility to "rewind" transactions after resolving a dispute. This also gives an easy way to preserve local chronological order. However turning back to a HashMap, ordered set or similar is trivial
 * It is assumed a precision of 4 digits after decimals, but the input is permissive. However, the output will be rounded to the 4th digit
