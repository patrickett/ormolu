#![allow(unused, dead_code)]

#[cfg(test)]
mod gilded_macro {
    use crate::*;

    #[test]
    fn parse_header_attributes() {
        #[allow(dead_code)]
        #[derive(Gilded)]
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
        #[allow(dead_code)]
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

        #[allow(dead_code)]
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

        let expected: Vec<String> = vec![
            "id".into(),
            "first_name".into(),
            "last_name".into(),
            "email".into(),
            "phone_number".into(),
            "created_at".into(),
            "updated_at".into(),
        ];

        assert_eq!(columns, expected)
    }

    #[test]
    fn unique_field_find_method() {
        use ormolu_macros::Gilded;

        #[allow(dead_code)]
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
        #[allow(dead_code)]
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
        #[allow(dead_code)]
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
        #[allow(dead_code)]
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

// #[gild(references = "customer(customer_id)")]
// #[gild(references = Customer(customer_id))]
// #[gild(references = (Customer, "customer_id"))]
// #[gild(references!(Customer, "customer_id"))]
// customer_id: i32,

#[cfg(test)]
mod query_builder {

    use ormolu_interfaces::query::Where;

    use crate::sql_command::{self};

    #[test]
    fn example_builder() {
        let mut q =
            sql_command::query::QueryState::new("user", vec!["name".to_string(), "id".to_string()]);
        q.limit = Some(1);
        q.offset = Some(0);
        // q.set_select(["name".into(), "id".into()]);
        q.r#where.push(Where::neq("id", "4".into()));

        assert!(!q.is_many());

        assert_eq!(
            q.to_string(),
            "SELECT name, id FROM user WHERE id != 4 LIMIT 1 OFFSET 0".to_string()
        )
    }
}

// Customer.orders().order_by().fetch() -> Future<Vec<Order>> // can be created at the same time but the other way

#[cfg(test)]
mod filter_proxy_iter_dsl {
    use crate::{query::*, *};

    #[test]
    fn field_filter() {
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
        #[allow(dead_code)]
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

        impl Findable for Order {}
        impl Findable for Customer {}

        impl Customer {
            // TODO: orders prob should not return a QuerySet<Order>
            // QuerySet is only when we are getting data, we should have
            // to do customer.orders().find().filter()
            fn orders(&self) -> QuerySet<Order> {
                todo!()
            }
        }

        let orders: QuerySet<Order> = QuerySet::default();
        let f = orders.filter(|order| {
            !order.name.contains("test") && order.id != 2 && order.name.contains("john")
        });

        assert_eq!(f.to_string(), "SELECT id, customer_id, order_date, total_amount, status, name, shipping_address, created_at, updated_at FROM order WHERE name NOT LIKE %test% AND id NOT = 2 AND name LIKE %john%".to_string())
    }
}
