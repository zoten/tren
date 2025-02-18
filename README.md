# tren

A toy transaction engine, to play with banking concepts

## Assumptions

 * deposits/withdrawal cannot fail (e.g. there's no transactional outbox pattern between in-state memory and any storage)
 * the CSV data is considered coming from a trusted source. This means the software expects withdrawals and deposits to be coherent (e.g. if we get a deposit and a subsequent withdrawal we expect the latest amount to be less than the former one, or the event shouldn't have happened)
