# Ormolu

Ormolu is a very light ORM like library.


## Name
From [Wikipedia](https://en.wikipedia.org/wiki/Ormolu): Ormolu is the gilding technique of applying finely ground, high-carat gold–mercury amalgam to an object of bronze, and objects finished in this way.

## Usage

### Database -> Rust source code
```bash
ormolu generate -d "postgres://username:password@host/database?currentSchema=my_schema" -o ./src/db/
```

