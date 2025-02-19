# tren

A toy transaction engine, to play with banking concepts

## Assumptions

 * The default store is an in-memory store, which assumes we have enough memory available to fit the data. In a real case scenario, it would be a DB, drastically reducing memory usage
 * Also, the access pattern is "optimized" (~"hopefully good enough") for the exercise, meaning e.g. since there is no interaction between accounts each account can keep its own separate list of transactions
   * this is a list because at the beginnin I have foreseen the possibility to "rewind" transactions after resolving a dispute. However turning back to a HashMap is trivial 
