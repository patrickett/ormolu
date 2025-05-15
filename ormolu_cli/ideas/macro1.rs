use sqlx::Result;

struct Email {
    domain: String,
    username: String,
}

#[ormolu(table_name = "employee")]
pub struct Employee {
    #[primary_key]
    id: i32,
    first_name: String,
    last_name: String,
    admin: bool,
    active: bool,
    display_name: String,
    password: String,
    #[foreign_key(Email)]
    email_id: i32,
    email_confirmed: Option<NaiveDateTime>,
    #[ormolu(column_name = "createdAt")]
    created: NaiveDateTime,
    #[ormolu(column_name = "lastModified")]
    modified: NaiveDateTime,
}

pub trait Ormolu<PrimaryKey> {
    fn primary_key(&self) -> PrimaryKey;
    fn find_by_primary_key(pk: PrimaryKey) -> impl Future<Output = Result<Self>>;
}

/// because:
/// #[foreign_key(Email)]
impl Employee {
    fn get_email(&self) -> impl Future<Output = Result<Email>>;
}

/// because:
/// #[foreign_key(Email), prefetch]
impl Employee {
    fn email(&self) -> Result<Email>;
}

fn test_me() {
    let sellers = Product::find_first(|product| product.price.equal(3.50))
        .map_query(|product| product.seller)
        .where_col(|seller| seller.name.ilike("%Nessie%"))
        .run(&client)
        .await?;
}
