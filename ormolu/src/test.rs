#![allow(unused, dead_code)]

#[cfg(test)]
mod table_macro {
    use crate::*;
    use chrono::{Local, NaiveDate, NaiveDateTime};
    use ormolu_macros::Table;

    #[derive(Table)]
    #[name = "public.customer"]
    pub struct Customer {
        id: PrimaryKey<Self, i32>,
        first_name: String,
        last_name: String,
        email: Unique<String>,
        phone_number: Option<Unique<String>>,
        created_at: chrono::NaiveDateTime,
        updated_at: chrono::NaiveDateTime,
    }

    #[derive(Table)]
    #[name = "private.product"]
    pub struct Product {
        id: PrimaryKey<Self, i32>,
        name: String,
        description: Option<String>,
        //TODO: #[check(value > 0)]
        // price > 0
        price: f64,
        // stock_quantity >= 0
        stock_quantity: i32,
        category: Option<String>,
        created_at: chrono::NaiveDateTime,
        updated_at: chrono::NaiveDateTime,
    }

    #[test]
    fn parse_table_name() {
        assert_eq!(Customer::object_name(), "customer");
        assert_eq!(Product::object_name(), "product");
    }

    #[test]
    fn parse_schema_name() {
        assert_eq!(Customer::schema_name(), "public");
        assert_eq!(Product::schema_name(), "private");
    }

    #[test]
    fn create_primary_key_getter() {
        // assert_eq!(Customer::primary_key_field_name(), "id");
        let now_local = Local::now(); // DateTime<Local>
        let naive_now: NaiveDateTime = now_local.naive_local(); // Convert to NaiveDateTime

        let user = Customer {
            id: PrimaryKey::from(131),
            first_name: String::new(),
            last_name: String::new(),
            email: Unique::from(String::new()),
            phone_number: None,
            created_at: naive_now,
            updated_at: naive_now,
        };

        let id = PrimaryKey::from(131);

        assert_eq!(user.primary_key(), &id)
    }

    #[test]
    fn fields_getter() {
        let columns = Customer::database_columns();

        let expected = [
            "id",
            "first_name",
            "last_name",
            "email",
            "phone_number",
            "created_at",
            "updated_at",
        ];

        assert_eq!(columns, expected);
    }

    #[test]
    fn check_ordinal() {
        let ordinal = Customer::ordinal("id").expect("msg");
        let column_name = Customer::column(ordinal).expect("msg");
        assert_eq!(column_name, "id")
    }

    #[tokio::test]
    async fn expand_unique_getter() {
        // Unique<T>
        let method_exists = Customer::get_by_email("me@company.com").await.expect("");
        assert!(method_exists.is_some() || method_exists.is_none());
    }

    #[tokio::test]
    async fn expand_option_unique_getter() {
        let method_exists2 = Customer::get_by_phone_number("me@company.com".to_string())
            .await
            .expect("");
        assert!(method_exists2.is_some() || method_exists2.is_none())
    }

    // TODO: support trimming {table_name}_id ie order_id -> id
    // for the rust code
}

#[cfg(test)]
mod query_builder {
    use crate::{query::*, *};

    #[derive(Table)]
    #[schema = "public"]
    #[object = "customer"]
    pub struct Customer {
        id: PrimaryKey<Self, i32>,
        first_name: String,
        last_name: String,
        email: Unique<String>,
        phone_number: Option<Unique<String>>,
        created_at: chrono::NaiveDateTime,
        updated_at: chrono::NaiveDateTime,
    }

    #[derive(Table)]
    #[schema = "public"]
    #[object = "order"]
    pub struct Order {
        id: PrimaryKey<Self, i32>,
        customer_id: ForeignKey<Customer, 1, i32>,
        order_date: chrono::NaiveDateTime,
        total_amount: f64,
        status: String,
        name: String,
        shipping_address: String,
        created_at: chrono::NaiveDateTime,
        updated_at: chrono::NaiveDateTime,
    }

    #[test]
    fn example_builder() {
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

    #[derive(Table)]
    #[schema = "public"]
    #[object = "customer"]
    pub struct Customer {
        id: PrimaryKey<Self, i32>,
        first_name: String,
        last_name: String,
        email: Unique<String>,
        phone_number: Option<Unique<String>>,
        created_at: chrono::NaiveDateTime,
        updated_at: chrono::NaiveDateTime,
    }

    #[derive(Table)]
    #[schema = "public"]
    #[object = "order"]
    pub struct Order {
        id: PrimaryKey<Self, i32>,
        customer_id: ForeignKey<Customer, 1, i32>,
        order_date: chrono::NaiveDateTime,
        total_amount: f64,
        status: String,
        test: bool,
        name: String,
        shipping_address: String,
        created_at: chrono::NaiveDateTime,
        updated_at: chrono::NaiveDateTime,
    }

    #[test]
    fn multiple_field_filter_with_or() {
        let state: QueryState<Order> = QueryState::new_select();
        let orders = QuerySet::new(state);
        let f = orders
            // TODO: Can we be more clever with order.test. Possibly just return
            // a bool instead of Field<bool>
            .filter(|order| !order.name.contains("test") || !order.test)
            .filter(|order| order.id != PrimaryKey::from(2) && order.name.contains("john"));

        assert_eq!(f.to_string(), "SELECT id, customer_id, order_date, total_amount, status, name, test, shipping_address, created_at, updated_at FROM order WHERE name NOT LIKE %test% AND id != 2 AND name LIKE %john%;".to_string())
    }

    #[test]
    fn multiple_field_filter() {
        #[derive(Table)]
        #[schema = "public"]
        #[object = "customer"]
        pub struct Customer {
            id: PrimaryKey<Self, i32>,
            first_name: String,
            last_name: String,
            email: Unique<String>,
            phone_number: Option<Unique<String>>,
            created_at: chrono::NaiveDateTime,
            updated_at: chrono::NaiveDateTime,
        }

        #[derive(Table)]
        #[schema = "public"]
        #[object = "order"]
        pub struct Order {
            id: PrimaryKey<Self, i32>,
            customer_id: ForeignKey<Customer, 1, i32>,
            order_date: chrono::NaiveDateTime,
            total_amount: f64,
            status: String,
            name: String,
            shipping_address: String,
            created_at: chrono::NaiveDateTime,
            updated_at: chrono::NaiveDateTime,
        }

        let state: QueryState<Order> = QueryState::new_select();
        let orders = QuerySet::new(state);
        let f = orders
            .filter(|order| !order.name.contains("test"))
            .filter(|order| order.id != 2.into() && order.name.contains("john"));

        // TODO: fix the 2.into() above ^
        assert_eq!(f.to_string(), "SELECT id, customer_id, order_date, total_amount, status, name, shipping_address, created_at, updated_at FROM order WHERE name NOT LIKE %test% AND id != 2 AND name LIKE %john%;".to_string())
    }

    #[test]
    fn single_and_field_filter() {
        #[derive(Table)]
        #[schema = "public"]
        #[object = "customer"]
        pub struct Customer {
            id: PrimaryKey<Self, i32>,
            first_name: String,
            last_name: String,
            email: Unique<String>,
            phone_number: Option<Unique<String>>,
            created_at: chrono::NaiveDateTime,
            updated_at: chrono::NaiveDateTime,
        }

        #[derive(Table)]
        #[schema = "public"]
        #[object = "order"]
        pub struct Order {
            id: PrimaryKey<Self, i32>,
            customer_id: ForeignKey<Customer, 1, i32>,
            order_date: chrono::NaiveDateTime,
            total_amount: f64,
            status: String,
            name: String,
            shipping_address: String,
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
            !order.name.contains("test")
                && order.id != PrimaryKey::from(2)
                && order.name.contains("john")
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
    use std::marker::PhantomData;

    use crate::{query::*, *};

    #[test]
    fn into_iter_for_loop() {
        // generated as comp time, no need for generic
        // pub struct CustomerId(i32);

        // impl CustomerId {
        //     // column on the original model struct
        //     fn column_name() -> &'static str {
        //         "customer_id"
        //     }
        // }

        pub struct FancyCustomer {
            id: i32,
        }

        const CUSTOMER_ID: usize = 1;

        // #[derive(Table)]
        // #[gild(name = "public.order")]
        // pub struct FancyOrder {
        //     #[gild(column = "order_id")]
        //     id: PrimaryKey<Self, i32>,
        //     customer_id: ForeignKey<FancyCustomer, CUSTOMER_ID, i32>,
        //     order_date: chrono::NaiveDateTime,
        //     total_amount: f64,
        //     status: String,
        //     name: String,
        //     shipping_address: String,
        //     created_at: chrono::NaiveDateTime,
        //     updated_at: chrono::NaiveDateTime,
        // }

        #[derive(Table)]
        #[name = "public.order"]
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

        // TODO: no need for inner str ref if remote name the same as local field name
        pub enum OrderField {
            Id(&'static str),
            CustomerId(&'static str),
            OrderDate(&'static str),
            TotalAmount(&'static str),
            Status(&'static str),
            Name(&'static str),
            ShippingAddress(&'static str),
            CreatedAt(&'static str),
            UpdatedAt(&'static str),
        }

        pub struct OrderFields;

        impl OrderFields {
            pub const ID: OrderField = OrderField::Id("id");
            pub const CUSTOMER_ID: OrderField = OrderField::CustomerId("customer_id");
            // pub const OrderDate(&'static str),
            // pub const TotalAmount(&'static str),
            // pub const Status(&'static str),
            // pub const Name(&'static str),
            // pub const ShippingAddress(&'static str),
            // pub const CreatedAt(&'static str),
            // pub const UpdatedAt(&'static str),
        }

        // impl Order {
        //     pub type Fields = OrderFields;
        // }

        #[derive(Table)]
        #[name = "public.customer"]
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
    use futures::StreamExt;

    #[derive(Table)]
    #[schema = "public"]
    #[object = "customer"]
    pub struct Customer {
        id: PrimaryKey<Self, i32>,
        first_name: String,
        last_name: String,
        email: Unique<String>,
        phone_number: Option<Unique<String>>,
        created_at: chrono::NaiveDateTime,
        updated_at: chrono::NaiveDateTime,
    }

    #[derive(Table)]
    #[schema = "public"]
    #[object = "order"]
    pub struct Order {
        id: PrimaryKey<Self, i32>,
        customer_id: ForeignKey<Customer, 1, i32>,
        order_date: chrono::NaiveDateTime,
        total_amount: f64,
        status: String,
        name: String,
        shipping_address: String,
        test: bool,
        created_at: chrono::NaiveDateTime,
        updated_at: chrono::NaiveDateTime,
    }
    #[test]
    fn subtype() {
        // #[derive(Table)]
        pub struct FrontendOrder {}
    }

    #[tokio::test]
    async fn one_to_many() {
        let Ok(Some(customer)) = Customer::get_by_id(1.into()).await else {
            panic!("failed to get cust")
        };

        let customers_orders = customer.orders();
        let mut real_cust_orders = customers_orders.filter(|o| !o.test);

        // for order in real_cust_orders {}

        // AsyncIterator
        while let Some(order) = real_cust_orders.next().await {}

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

        // let john_customer = Customer {
        //     id: 432,
        //     ..Default::default()
        // };

        // john_customer.orders()

        // for order in john_customer.orders().into_async_iter() {
        //     let cust = order.customer();
        // }

        // assert_eq!(f.to_string(), "SELECT id, customer_id, order_date, total_amount, status, name, shipping_address, created_at, updated_at FROM order WHERE customer_id = 432;".to_string())
    }
}

#[cfg(test)]
mod database {
    use std::marker::PhantomData;

    use crate::{query::*, *};
    use ormolu_interfaces::GetConnectionPool;
    use sqlx::FromRow;

    #[derive(Table)]
    #[name = "public.resource_type"]
    pub struct ResourceType {
        id: PrimaryKey<Self, i32>,
        name: String,
        created: chrono::NaiveDateTime,
        modified: chrono::NaiveDateTime,
    }

    #[tokio::test]
    async fn get_connection_pool() {
        let pool = ResourceType::get_connection_pool().await;
    }

    #[tokio::test]
    async fn fetch_via_model() {
        let pk = PrimaryKey::from(1);

        if let Ok(Some(resource_type)) = ResourceType::get_by_id(pk).await {
            assert_eq!(resource_type.id, pk);
            assert_eq!(resource_type.name, String::from("test"))
        } else {
            panic!("failed to get data from model")
        }

        // let pool = ResourceType::get_connection_pool().await;

        // let x = sqlx::query_as::<_, ResourceType>("SELECT * FROM resource_type WHERE id = ?;")
        //     .bind(1)
        //     .fetch_optional(&pool)
        //     .await;

        // let mut q: QueryState<ResourceType> = sql_command::query::QueryState::new_select();
        // // q.where_conditions.push(Where::eq(, value));

        // fn get_by_id(id: i32) -> impl Future<Output = Result<Option<ResourceType>, OrmoluError>> {
        //     async move {
        //         let mut q: ormolu_interfaces::sql_command::query::QueryState<ResourceType> =
        //             ormolu_interfaces::sql_command::query::QueryState::new_select();

        //         let v = id.to_string();

        //         let pool = ResourceType::get_connection_pool().await;

        //         Ok(
        //             sqlx::query_as::<_, ResourceType>("SELECT * FROM resource_type WHERE id = ?;")
        //                 .bind(1)
        //                 .fetch_optional(&pool)
        //                 .await?,
        //         )
        //     }
        // }
    }
}

#[cfg(test)]
mod get_pool {
    use crate::{query::*, *};
    use sqlx::postgres::PgPoolOptions;
    use std::env;

    #[test]
    fn override_get_pool() {
        #[derive(Table)]
        #[name = "public.order"]
        pub struct Order {
            id: PrimaryKey<Self, i32>,
            customer_id: ForeignKey<Customer, 1, i32>,
            order_date: chrono::NaiveDateTime,
            total_amount: f64,
            status: String,
            name: String,
            shipping_address: String,
            created_at: chrono::NaiveDateTime,
            updated_at: chrono::NaiveDateTime,
        }

        #[derive(Table)]
        #[name = "public.customer"]
        pub struct Customer {
            id: PrimaryKey<Self, i32>,
            first_name: String,
            last_name: String,
            email: Unique<String>,
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

    #[derive(Table)]
    #[schema = "public"]
    #[object = "customer"]
    pub struct Customer {
        id: PrimaryKey<Self, i32>,
        first_name: String,
        last_name: String,
        email: Unique<String>,
        phone_number: Option<Unique<String>>,
        created_at: chrono::NaiveDateTime,
        updated_at: chrono::NaiveDateTime,
    }

    #[derive(Table)]
    #[schema = "public"]
    #[object = "order"]
    pub struct Order {
        id: PrimaryKey<Self, i32>,
        customer_id: ForeignKey<Customer, 1, i32>,
        order_date: chrono::NaiveDateTime,
        total_amount: f64,
        status: String,
        name: String,
        shipping_address: String,
        created_at: chrono::NaiveDateTime,
        updated_at: chrono::NaiveDateTime,
    }

    // TODO: each custom select specific fields from x
    // becomes a sql view which CAN be type safe and generated
    #[test]
    fn view() {
        let joins: Vec<Join> = vec![];
        let selects: Vec<_> = vec![];
        let wheres = vec![];

        pub enum PartialOrder {
            Id {
                id: PrimaryKey<Self, i32>,
            },
            IdCustomerId {
                id: PrimaryKey<Self, i32>,
                customer_id: ForeignKey<Customer, 1, i32>,
            },
        }

        // query!({
        //     select: [first_name, email, phone_number],
        //     from: Customer
        // });

        struct CustomerContact {
            first_name: String,
            email: Unique<String>, // remove unique? YES - within this context its no longer Unique
            phone_number: Option<String>,
        }

        // TODO: NOTE: passing in an existing thing will have it type checked against, but
        // it not existing will have it create its own type

        // query_as!(CustomerContact,
        // {
        //     select: [first_name, email, phone_number],
        //     from: Customer
        // });

        //let {id} = partial!(Order, ["id"])

        // let part_order = PartialOrder::
        // JOIN customer c ON
        // join!(Customer.id, Product.customer_id)

        // SELECT
        //     o.order_id,
        //     o.order_date,
        //     o.total_amount,
        //     c.customer_id,
        //     c.first_name,
        //     c.last_name,
        //     c.email,
        //     c.phone
        // FROM
        //     orders o
        // JOIN
        //     customers c ON o.customer_id = c.customer_id
        // ORDER BY
        //     o.order_date DESC;

        create_select!(
            CustomerOrder // [
                          //     Order.id,
                          //     Order.created_at,
                          //     Order.total_amount,
                          //     Order.category,
                          //     Customer.first_name,
                          //     Customer.last_name,
                          //     Customer.email,
                          //     Customer.phone
                          // ],
                          // join!(Customer.id, Order.customer_id)
        );

        // let a = MyToken {
        //     field: String::new(),
        // };

        let query = [selects, wheres, joins];

        pub enum Join {
            Inner { source: String, dest: String },
            Left {},
        }
        // #[derive(View)]
        // pub struct FrontendCustomer {};
    }
}

#[cfg(test)]
mod convert_schema_to_source {

    use crate::Col;
    use ormolu_interfaces::*;
    use ormolu_macros::Table;

    #[test]
    fn advanced() {
        let schema = r#"
            CREATE TABLE person (
                --id SERIAL PRIMARY KEY,                       -- Primary key, auto-incrementing integer
                id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY
                username VARCHAR(50) NOT NULL UNIQUE,       -- Non-null, unique constraint
                email VARCHAR(255) UNIQUE,                   -- Nullable (by default), unique constraint
                age INT CHECK (age >= 0),                    -- Nullable int with a minimum value constraint (>=0)
                score NUMERIC(5,2) DEFAULT 0.0 CHECK (score >= 0), -- Numeric with precision, default and check constraint
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),  -- Timestamp, non-null, default current time
                bio TEXT,                                    -- Nullable text field
                is_active BOOLEAN NOT NULL DEFAULT TRUE,    -- Boolean with a default value
                country_code CHAR(2) NOT NULL CHECK (country_code ~ '^[A-Z]{2}$'), -- Two-letter uppercase country code constraint
                referral_code VARCHAR(10) UNIQUE NULL       -- Nullable unique field
            );
            "#;

        // let a: Identity<PrimaryKey<String, String>> = Identity(PrimaryKey::new("".to_string());

        // let f = i32::MAX;
        pub struct Person {
            id: Identity<PrimaryKey<Self, i32>>,
            partner: ForeignKey<Person, 0, i32>,
            pub username: Unique<VarChar<50>>,
            email: Option<Unique<VarChar<255>>>,
            age: Option<i32>, // CHECK (age >= 0),                    -- Nullable int with a minimum value constraint (>=0)
            score: f32, // NUMERIC(5,2) DEFAULT 0.0 CHECK (score >= 0), -- Numeric with precision, default and check constraint
            // #[gild(default = "NOW()")]
            created_at: chrono::NaiveDateTime,
            bio: Option<String>,
            // #[gild(default = true)]
            is_active: bool,
            country_code: Char<2>, // CHECK (country_code ~ '^[A-Z]{2}$'), -- Two-letter uppercase country code constraint
            referral_code: Option<Unique<VarChar<10>>>,
        }

        fn get_person() -> Person {
            todo!()
        }

        let mut john = get_person();
        john.username = VarChar::<50>::try_from("Alice").unwrap().into();
    }
}

#[cfg(test)]
mod stored_procedure {
    use ormolu_interfaces::StoredProcedure;
    use ormolu_macros::StoredProcedure as Sproc;

    #[test]
    fn create_exec() {
        #[derive(Sproc)]
        #[name = "public.reset_login_sessions"]
        pub struct ResetLoginSessions {
            name: String,
            // since: chrono::NaiveDate,
        }
        // TODO: pattern to pass db connection write before into_async so before
        // we actually hit db we can say hey don't use the default connect using the connection

        pub struct ResetResponse {
            connections_reset: i32,
        }
        // could also do module
        // /dbo/sprocs.rs
        // db::dbo::reset_login_sessions()

        // db::schema::{stored_proc|entity}

        let a: ResetResponse = ResetLoginSessions::exec(String::new()).unwrap();

        let pending_proc_call = ResetLoginSessions {
            name: String::new(),
        };
        pending_proc_call.execute::<ResetResponse>();

        // impl StoredProcedure for ResetLoginSessions {
        //     type ReturnValue = i32;
        // }
    }
}
// json like macro for sql syntax
//
//
// query!({
//      select: ["id", "name"],
//      from: "customer"
// })
