# Ormolu

## Name

From [Wikipedia](https://en.wikipedia.org/wiki/Ormolu): Ormolu is the gilding technique of applying finely ground, high-carat gold–mercury amalgam to an object of bronze, and objects finished in this way.

## Status

**Current status: IN DEVELOPMENT - ORMOLU is not ready for production usage. The API
is still evolving and documentation is lacking.**

Currently I am only focusing on supporting PostgreSQL.
I will accept PR's for adding other database support.

### Database as a source of truth

Ormolu expects that what exists in the database is the proper intended way things
should be. So the `ormolu-cli` will generate all the types to interact with the tables
in the database for you.

### Usage

```bash
ormolu generate -d "postgres://username:password@host/database?currentSchema=my_schema" -o ./src/db/
```

### Example

```Rust
use ormolu::*;

#[derive(Gilded)]
#[gild(table = "customer")]
pub struct Customer {
    #[gild(primary_key, column = "customer_id")]
    id: i32,
    first_name: String,
    last_name: String,
    #[gild(unique)]
    email: String,
    phone_number: Option<String>,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
}

#[derive(Gilded)]
#[gild(table = "order")]
pub struct Order {
    #[gild(primary_key, column = "order_id")]
    id: i32,
    #[gild(references = Customer)] // column not required since both structs have 'customer_id'
    customer_id: i32,
    order_date: chrono::NaiveDateTime,
    total_amount: f64,
    status: String,
    shipping_address: String,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
}
```

```rust
// find_by_email will automatically be created since 'email' is made unique in the database.
let john = Customer::find_by_email("jdoe@example.com".to_string()).await?;

let johns_orders: Vec<Order> = john.orders().order_by_asc(|o| o.order_date).fetch().await?;

// TODO: syntax
let johns_orders: Vec<Order> = john
    .orders(|orders| orders.order_by_asc(|o| o.order_date))
    .fetch()
    .await?;

if let Some(newest_order) = johns_orders.first() {
    // This is just demonstrating the relation capability, we have the customer above.
    let cust = newest_order.customer().fetch().await?;
    println!(
        "{} newest order's total was {}",
        cust.first_name, newest_order.total_amount
    );
}
```

For more complete examples check out [examples](ormolu/examples)

```Rust
let order = Order::find_first(|order| order.total.greater_than(10.0));
```

### Attributes

```rust
#[gild(primary_key)]
```

This makes the struct `sqlx::FromRow`

### Todos

- Is there a way we can try to perform a fetch or something but if it fails check
  if the db schema has changed and if so throw an error that alerts that we are out of sync
- prepared queries https://orm.drizzle.team/docs/rqb#multiple-placeholders (query.prepare())
- if each column in the table has a default value we can implement a Derive default impl
