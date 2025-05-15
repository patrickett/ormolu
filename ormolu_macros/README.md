Depends on the internal crate, so it can use its types and traits

TODO:

- #[gild(ignore)] -- ignore field from database perspective just a local field
- #[gild(references = Customer, preload)] -- automatically fetch relation Error if not exist
  and change type signature to always be there so no .load() needed.
