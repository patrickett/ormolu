#![allow(unused, dead_code)]

#[cfg(test)]
mod gilded_macro {
    use crate::*;

    #[test]
    fn parse_header_attributes() {
        #[derive(sqlx::FromRow, Gilded)]
        #[gild(table = "user", schema = "public")]
        pub struct User {
            name: String,
        }

        let table_name = User::table_name();
        assert_eq!(table_name, "user");

        let schema_name = User::schema_name();
        assert_eq!(schema_name, "public")
    }

    #[test]
    fn create_primary_key_getter() {
        #[derive(Gilded)]
        #[gild(table = "user")]
        pub struct User {
            #[gild(primary_key)]
            id: i32,
            name: String,
        }

        assert_eq!(User::primary_key_field_name(), "id");

        let user = User {
            id: 131,
            name: String::from("John Doe"),
        };

        let id: i32 = 131;

        assert_eq!(user.primary_key(), &id)
    }

    #[test]
    fn fields_getter() {
        use ormolu_macros::Gilded;

        #[derive(Gilded)]
        #[gild(table = "customer")]
        pub struct Customer {
            #[gild(primary_key)]
            id: i32,
            first_name: String,
            last_name: String,
            #[gild(unique)]
            email: String,
            #[gild(unique)]
            phone_number: Option<String>,
            created_at: chrono::NaiveDateTime,
            updated_at: chrono::NaiveDateTime,
        }

        let columns = Customer::fields();

        let expected = [
            "id",
            "first_name",
            "last_name",
            "email",
            "phone_number",
            "created_at",
            "updated_at",
        ];

        assert_eq!(columns, expected)
    }

    #[test]
    fn unique_field_find_method() {
        use ormolu_macros::Gilded;

        #[derive(Gilded)]
        #[gild(table = "customer")]
        pub struct Customer {
            #[gild(primary_key)]
            id: i32,
            first_name: String,
            last_name: String,
            #[gild(unique)]
            email: String,
            #[gild(unique)]
            phone_number: Option<String>,
            created_at: chrono::NaiveDateTime,
            updated_at: chrono::NaiveDateTime,
        }

        // TODO: actually impl this
        let method_exists = Customer::find_by_email("me@company.com".to_string());
        assert!(method_exists);

        // TODO: remove the wrapper option
        let method_exists2 = Customer::find_by_phone_number(Some("me@company.com".to_string()));
        assert!(method_exists2)
    }

    #[test]
    fn advanced_example() {
        // CREATE TABLE customer (
        //  customer_id SERIAL PRIMARY KEY,
        //  first_name VARCHAR(100) NOT NULL,
        //  last_name VARCHAR(100) NOT NULL,
        //  email VARCHAR(255) UNIQUE NOT NULL,
        //  phone_number VARCHAR(20),
        //  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        //  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        // )

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

        // CREATE TABLE product (
        //  product_id SERIAL PRIMARY KEY,
        //  name VARCHAR(255) NOT NULL,
        //  description TEXT,
        //  price DECIMAL(10, 2) CHECK (price > 0),
        //  stock_quantity INT CHECK (stock_quantity >=0),
        //  category VARCHAR(100),
        //  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        //  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        // )
        //

        #[derive(Gilded)]
        #[gild(table = "product")]
        pub struct Product {
            #[gild(primary_key, column = "product_id")]
            id: i32,
            name: String,
            description: Option<String>,
            // price > 0
            price: f64,
            // stock_quantity >= 0
            stock_quantity: i32,
            category: Option<String>,
            created_at: chrono::NaiveDateTime,
            updated_at: chrono::NaiveDateTime,
        }

        // TODO: support trimming {table_name}_id ie order_id -> id
        // for the rust code

        // CREATE TABLE order (
        //  order_id SERIAL PRIMARY KEY,
        //  customer_id INT REFERENCES customer(customer_id) ON DELETE CASCADE,
        //  order_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        //  total_amount DECIMAL(10, 2) CHECK (total_amount >= 0),
        //  status VARCHAR(50) DEFAULT 'pending',
        //  shipping_address TEXT NOT NULL,
        //  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        //  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        // )

        #[derive(Gilded)]
        #[gild(table = "order")]
        pub struct Order {
            #[gild(primary_key, column = "order_id")]
            id: i32,
            #[gild(references = (Customer, "customer_id"))]
            // or #[gild(references = Customer)] since matching 'customer_id' col name
            customer_id: i32,
            order_date: chrono::NaiveDateTime,
            total_amount: f64,
            status: String,
            shipping_address: String,
            created_at: chrono::NaiveDateTime,
            updated_at: chrono::NaiveDateTime,
        }
    }
}

#[cfg(test)]
mod query_builder {
    use crate::{query::*, *};

    #[test]
    fn example_builder() {
        #[derive(Gilded, Default)]
        #[gild(table = "order")]
        pub struct Order {
            #[gild(primary_key, column = "order_id")]
            id: i32,
            // #[gild(references = (Customer, "customer_id"))]
            // or #[gild(references = Customer)] since matching 'customer_id' col name
            customer_id: i32,
            order_date: chrono::NaiveDateTime,
            total_amount: f64,
            status: String,
            name: String,
            shipping_address: String,
            created_at: chrono::NaiveDateTime,
            updated_at: chrono::NaiveDateTime,
        }

        let mut q: QueryState<Order> = sql_command::query::QueryState::new_select();
        q.limit = Some(1);
        q.offset = Some(0);
        // q.set_select(["name".into(), "id".into()]);
        q.where_conditions.push(Where::neq("id", "4".into()));

        assert_eq!(
            q.to_string(),
            "SELECT id, customer_id, order_date, total_amount, status, name, shipping_address, created_at, updated_at FROM order WHERE id != 4 LIMIT 1 OFFSET 0;".to_string()
        )
    }
}

#[cfg(test)]
mod filter_proxy_iter_dsl {
    use crate::{query::*, *};

    #[test]
    fn multiple_field_filter_with_or() {
        #[derive(Gilded, Default)]
        #[gild(table = "order")]
        pub struct Order {
            #[gild(primary_key, column = "order_id")]
            id: i32,
            #[gild(references = (Customer, "customer_id"))]
            // or #[gild(references = Customer)] since matching 'customer_id' col name
            customer_id: i32,
            order_date: chrono::NaiveDateTime,
            total_amount: f64,
            status: String,
            name: String,
            test: bool,
            shipping_address: String,
            created_at: chrono::NaiveDateTime,
            updated_at: chrono::NaiveDateTime,
        }

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

        let state: QueryState<Order> = QueryState::new_select();
        let orders = QuerySet::new(state);
        let f = orders
            // TODO: Can we be more clever with order.test. Possibly just return
            // a bool instead of Field<bool>
            .filter(|order| !order.name.contains("test") || !order.test)
            .filter(|order| order.id != 2 && order.name.contains("john"));

        assert_eq!(f.to_string(), "SELECT id, customer_id, order_date, total_amount, status, name, shipping_address, created_at, updated_at FROM order WHERE name NOT LIKE %test% AND id != 2 AND name LIKE %john%;".to_string())
    }

    #[test]
    fn multiple_field_filter() {
        #[derive(Gilded, Default)]
        #[gild(table = "order")]
        pub struct Order {
            #[gild(primary_key, column = "order_id")]
            id: i32,
            #[gild(references = (Customer, "customer_id"))]
            // or #[gild(references = Customer)] since matching 'customer_id' col name
            customer_id: i32,
            order_date: chrono::NaiveDateTime,
            total_amount: f64,
            status: String,
            name: String,
            shipping_address: String,
            created_at: chrono::NaiveDateTime,
            updated_at: chrono::NaiveDateTime,
        }

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

        let state: QueryState<Order> = QueryState::new_select();
        let orders = QuerySet::new(state);
        let f = orders
            .filter(|order| !order.name.contains("test"))
            .filter(|order| order.id != 2 && order.name.contains("john"));

        assert_eq!(f.to_string(), "SELECT id, customer_id, order_date, total_amount, status, name, shipping_address, created_at, updated_at FROM order WHERE name NOT LIKE %test% AND id != 2 AND name LIKE %john%;".to_string())
    }

    #[test]
    fn single_and_field_filter() {
        #[derive(Gilded, Default)]
        #[gild(table = "order")]
        pub struct Order {
            #[gild(primary_key, column = "order_id")]
            id: i32,
            #[gild(references = (Customer, "customer_id"))]
            // or #[gild(references = Customer)] since matching 'customer_id' col name
            customer_id: i32,
            order_date: chrono::NaiveDateTime,
            total_amount: f64,
            status: String,
            name: String,
            shipping_address: String,
            created_at: chrono::NaiveDateTime,
            updated_at: chrono::NaiveDateTime,
        }

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

        // NOTE: TODO just get a Vec<Order> for all fetch methods intially
        // handle no orders or too many manually
        impl Customer {
            // TODO: orders prob should not return a QuerySet<Order>
            // QuerySet is only when we are getting data, we should have
            // to do customer.orders().find().filter()
            //
            // customer.orders().filter(|o| o.name).find().one(&db) -> Order
            // Customer -> QuerySet<Order> ->
            // customer.orders().filter(|o| o.name).update().all(&db) -> Vec<Order>
            //
            // fn orders(&self) -> QuerySet<Order> {
            //     todo!()
            // }
        }

        let state: QueryState<Order> = QueryState::new_select();
        let orders = QuerySet::new(state);
        let f = orders.filter(|order| {
            !order.name.contains("test") && order.id != 2 && order.name.contains("john")
        });

        assert_eq!(f.to_string(), "SELECT id, customer_id, order_date, total_amount, status, name, shipping_address, created_at, updated_at FROM order WHERE name NOT LIKE %test% AND id != 2 AND name LIKE %john%;".to_string())
    }

    fn normal_filter() {
        let names = vec![
            String::from("Alice"),
            String::from("Bob"),
            String::from("Charlie"),
            String::from("Amanda"),
        ];

        // Filter names that start with 'A'
        let a_names: Vec<String> = names
            .into_iter()
            .filter(|name| name.starts_with('A'))
            .collect();

        // println!("{:?}", a_names);
    }

    fn map_ex() {
        struct User {
            pub name: String,
            pub age: u8,
            pub description: String,
        }

        let names = vec![User {
            name: String::new(),
            age: 0,
            description: String::new(),
        }];

        // Filter names that start with 'A'
        let a_names: Vec<(u8, String)> = names.into_iter().map(|u| (u.age, u.name)).collect();

        // println!("{:?}", a_names);
    }
}

#[cfg(test)]
mod iter_eval {
    use crate::{query::*, *};

    #[test]
    fn into_iter_for_loop() {
        #[derive(Gilded)]
        #[gild(table = "order")]
        pub struct Order {
            #[gild(primary_key, column = "order_id")]
            id: i32,
            #[gild(references = (Customer, "customer_id"))]
            customer_id: i32,
            order_date: chrono::NaiveDateTime,
            total_amount: f64,
            status: String,
            name: String,
            shipping_address: String,
            created_at: chrono::NaiveDateTime,
            updated_at: chrono::NaiveDateTime,
        }

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

        let state: QueryState<Order> = QueryState::new_select();
        let orders = QuerySet::new(state);
        let filtered_orders = orders
            .filter(|order| !order.name.contains("test"))
            .filter(|order| order.id != 2 && order.name.contains("john"));

        // for order in filtered_orders {}

        // TODO: id Field needs to be renamed to customer_id when made into request
        assert_eq!(filtered_orders.to_string(), "SELECT id, customer_id, order_date, total_amount, status, name, shipping_address, created_at, updated_at FROM order WHERE name NOT LIKE %test% AND customer_id != 2 AND name LIKE %john%;".to_string())
    }
}

#[cfg(test)]
mod relations {
    use crate::{query::*, *};

    #[test]
    fn one_to_many() {
        #[derive(Gilded)]
        #[gild(table = "order")]
        pub struct Order {
            #[gild(primary_key, column = "order_id")]
            id: i32,
            #[gild(references = (Customer, "customer_id"))]
            // TODO: Does changing customer_id: i32 -> customer_id: CustomerId<i32> change the semantics needed for the macro?
            customer_id: i32,
            order_date: chrono::NaiveDateTime,
            total_amount: f64,
            status: String,
            name: String,
            shipping_address: String,
            created_at: chrono::NaiveDateTime,
            updated_at: chrono::NaiveDateTime,
        }

        #[derive(Gilded, Default)]
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

        // many
        // impl Order {
        //     // to_singular,to_lower
        //     fn customer(&self) -> QuerySet<Customer> {
        //         let query_state: QueryState<Customer> = QueryState::new_select();
        //         let query_set = QuerySet::new(query_state);
        //         query_set.filter(|customer| customer.id == self.customer_id)
        //     }
        // }

        // // one
        // impl Customer {
        //     // to_plural,to_lower
        //     fn orders(&self) -> QuerySet<Order> {
        //         let query_state: QueryState<Order> = QueryState::new_select();
        //         let query_set = QuerySet::new(query_state);
        //         query_set.filter(|order| order.customer_id == self.id)
        //     }
        // }

        let john_customer = Customer {
            id: 432,
            ..Default::default()
        };

        // john_customer.orders()

        for order in john_customer.orders() {
            let cust = order.customer();
        }

        // assert_eq!(f.to_string(), "SELECT id, customer_id, order_date, total_amount, status, name, shipping_address, created_at, updated_at FROM order WHERE customer_id = 432;".to_string())
    }
}

#[cfg(test)]
mod get_pool {
    use std::env;

    use sqlx::postgres::PgPoolOptions;

    use crate::{query::*, *};

    #[test]
    fn override_get_pool() {
        #[derive(Gilded)]
        #[gild(table = "order")]
        pub struct Order {
            #[gild(primary_key, column = "order_id")]
            id: i32,
            #[gild(references = (Customer, "customer_id"))]
            // or #[gild(references = Customer)] since matching 'customer_id' col name
            customer_id: i32,
            order_date: chrono::NaiveDateTime,
            total_amount: f64,
            status: String,
            name: String,
            shipping_address: String,
            created_at: chrono::NaiveDateTime,
            updated_at: chrono::NaiveDateTime,
        }

        #[derive(Gilded)]
        #[gild(table = "customer", schema = "public")]
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

        // Customer::schema_name();

        // let state: QueryState<Order> = QueryState::new_select();
        // let orders = QuerySet::new(state);
        // let f = orders
        //     .filter(|order| !order.name.contains("test"))
        //     .filter(|order| order.id != 2 && order.name.contains("john"));
    }
}

#[cfg(test)]
mod select {
    use crate::{query::*, *};

    #[test]
    fn typed_select() {
        #[derive(Gilded)]
        #[gild(table = "order")]
        pub struct Order {
            #[gild(primary_key, column = "order_id")]
            id: i32,
            #[gild(references = (Customer, "customer_id"))]
            // or #[gild(references = Customer)] since matching 'customer_id' col name
            customer_id: i32,
            order_date: chrono::NaiveDateTime,
            total_amount: f64,
            status: String,
            name: String,
            shipping_address: String,
            created_at: chrono::NaiveDateTime,
            updated_at: chrono::NaiveDateTime,
        }

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

        fn select_order(s: OrderSelect) {}

        select_order(OrderSelect {
            id: true,
            shipping_address: true,
            ..Default::default()
        });

        // let state: QueryState<Order> = QueryState::new_select();
        // let orders = QuerySet::new(state);
        // let f = orders
        //     .filter(|order| !order.name.contains("test"))
        //     .filter(|order| order.id != 2 && order.name.contains("john"));
    }
}

// json like macro for sql syntax
//
//
// query!({
//      select: ["id", "name"],
//      from: "customer"
// })
