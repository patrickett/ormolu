use std::{
    collections::HashSet,
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
    process::Command,
};

use convert_case::{Case, Casing};
use proc_macro2::*;
use quote::*;
use sqlx::{PgPool, Row};
use url::Url;

const START_TAG: &str = "// ormolu begin";
const END_TAG: &str = "// ormolu end";

#[derive(Debug)]
struct PgColumn {
    name: String,
    // type_info: PgTypeInfo,
    type_info: String,
    ordinal: i32,
    is_nullable: bool,
}

#[derive(Debug)]
struct Table {
    name: String,
    columns: Vec<PgColumn>,
}

async fn list_tables(pool: &PgPool, schema: &str) -> Result<Vec<Table>, sqlx::Error> {
    // Query to get table names
    let tables_query = "
        SELECT table_name
        FROM information_schema.tables
        WHERE table_schema = $1
          AND table_type = 'BASE TABLE';
    ";
    let table_rows = sqlx::query(tables_query)
        .bind(schema)
        .fetch_all(pool)
        .await?;

    // Vector to hold table names and their columns
    let mut tables = Vec::new();

    // Query to get columns for a table
    let columns_query = "
        SELECT column_name, data_type, ordinal_position, is_nullable
        FROM information_schema.columns
        WHERE table_schema = $1
          AND table_name = $2
        ORDER BY ordinal_position;
    ";

    // Loop through each table and get its columns
    for table_row in table_rows {
        let table_name: String = table_row.get("table_name");

        let column_rows = sqlx::query(columns_query)
            .bind(schema)
            .bind(&table_name)
            .fetch_all(pool)
            .await?;

        let columns: Vec<PgColumn> = column_rows
            .into_iter()
            .map(|row| {
                let name: String = row.get("column_name");
                let ordinal: i32 = row.get("ordinal_position");
                let type_info: String = row.get("data_type");

                let is_nullable: bool = row.get::<String, _>("is_nullable") == "YES";

                // let type_info = sqlx::postgres::PgTypeInfo(t_info);

                PgColumn {
                    is_nullable,
                    name,
                    type_info,
                    ordinal,
                }
            })
            .collect();
        println!("found {} columns for table {}", columns.len(), table_name);

        tables.push(Table {
            name: table_name,
            columns,
        });
    }

    Ok(tables)
}

fn map_sql_type_to_rust(
    sql_type: &str,
    is_nullable: bool,
    required_imports: &mut HashSet<String>,
) -> syn::Type {
    let base_type: syn::Type = match sql_type {
        "integer" => syn::parse_str("i32").unwrap(),
        "text" => syn::parse_str("String").unwrap(),
        "boolean" => syn::parse_str("bool").unwrap(),
        "date" => {
            required_imports.insert("chrono::NaiveDate".to_string());
            syn::parse_str("chrono::NaiveDate").unwrap()
        }
        "timestamp without time zone" | "timestamp with time zone" => {
            required_imports.insert("chrono::NaiveDateTime".to_string());
            syn::parse_str("chrono::NaiveDateTime").unwrap()
        }
        "real" => syn::parse_str("f32").unwrap(),
        "double precision" => syn::parse_str("f64").unwrap(),
        _ => syn::parse_str("String").unwrap(), // Default to String for unknown types
    };

    if is_nullable {
        syn::parse_str(&quote! { Option<#base_type> }.to_string()).unwrap()
    } else {
        base_type
    }
}

fn generate_mod_statements(modules: &HashSet<String>) -> proc_macro2::TokenStream {
    let mod_statements: Vec<_> = modules
        .iter()
        .map(|module| {
            let mod_ident = Ident::new(module, proc_macro2::Span::call_site());
            quote! { pub mod #mod_ident; }
        })
        .collect();

    quote! {
        #(#mod_statements)*
    }
}

fn generate_imports(required_imports: &HashSet<String>) -> proc_macro2::TokenStream {
    let imports: Vec<_> = required_imports
        .iter()
        .map(|import| {
            // Split the import path by "::"
            let parts: Vec<Ident> = import
                .split("::")
                .map(|part| Ident::new(part, proc_macro2::Span::call_site()))
                .collect();

            // Generate the `use` statement
            quote! { use #(#parts)::*; }
        })
        .collect();

    quote! {
        #(#imports)*
    }
}

fn generate_rust_struct(
    table: &Table,
    required_imports: &mut HashSet<String>,
) -> proc_macro2::TokenStream {
    let struct_name = Ident::new(
        &table.name.to_case(Case::Pascal),
        proc_macro2::Span::call_site(),
    );
    let fields: Vec<_> = table
        .columns
        .iter()
        .map(|column| {
            let field_name = Ident::new(&column.name, proc_macro2::Span::call_site());
            let field_type =
                map_sql_type_to_rust(&column.type_info, column.is_nullable, required_imports);
            quote! { pub #field_name: #field_type }
        })
        .collect();

    quote! {
        #[derive(Debug)]
        pub struct #struct_name {
            #(#fields),*
        }
    }
}

fn format_rust_file(file_path: &str) {
    let output = Command::new("rustfmt")
        .arg(file_path)
        .output()
        .expect("Failed to execute rustfmt");

    if !output.status.success() {
        eprintln!(
            "rustfmt failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    println!("formatted {}", file_path)
}

fn create_file(table: &Table, output_path: &Path) -> String {
    let file_full_path = format!(
        "{}/{}.rs",
        output_path.to_str().expect(""),
        table.name.to_case(Case::Snake)
    );

    let file_path = Path::new(&file_full_path);

    if file_path.exists() {
        "".to_string()
    } else {
        let mut file = File::create(file_full_path.clone()).expect("Could not create file");
        let mut required_imports = HashSet::new();
        let struct_tokens = generate_rust_struct(table, &mut required_imports);
        let imports = generate_imports(&required_imports);

        // TODO: track what files were created, created a mod.rs and import them
        // TODO: relations

        // TODO: far future, option to remove dep on chrono

        let code = format!("{}\n{}", imports.to_string(), struct_tokens.to_string());
        let ormolu_code = format!("{}\n{}\n{}\n", START_TAG, code, END_TAG);
        file.write_all(ormolu_code.as_bytes())
            .expect("Could not write to file");

        drop(file);
        format_rust_file(file_full_path.as_str());

        file_full_path
    }
}

pub async fn generate_postgres(url: &Url, output_path: &Path) {
    println!("Connecting to Postgres ...");

    // connect to database
    let pg_pool = sqlx::PgPool::connect(&url.to_string()).await;
    let Ok(pool) = pg_pool else {
        panic!(
            "cannot connect with given database_url: {}",
            &url.to_string()
        )
    };

    let current_schema = url
        .query_pairs()
        .find(|(key, _)| key == "currentSchema")
        .map(|(_, value)| value)
        .unwrap_or(std::borrow::Cow::Borrowed(&"public"));

    println!("searching in schema: {}", &current_schema);

    // Extract the database name from the path
    let path_segments: Vec<&str> = url.path_segments().map_or_else(Vec::new, |c| c.collect());

    // The database name is the last segment in the path
    let db_name = path_segments
        .last()
        .expect("No database name found in the URL");

    // Call the function to list tables and their structures
    let tables = list_tables(&pool, &current_schema).await.unwrap();
    println!("Closing connection to Postgres.");

    let mod_rs_path = format!(
        "{}/mod.rs",
        output_path
            .to_path_buf()
            .into_os_string()
            .into_string()
            .expect("")
    );

    let mut modules: HashSet<String> = HashSet::new();

    let mod_rs_file = Path::new(&mod_rs_path);

    for table in tables {
        modules.insert(table.name.clone());
        create_file(&table, output_path);
        println!("created file {}.rs", table.name);
    }

    let mod_statements = generate_mod_statements(&modules);

    if mod_rs_file.exists() {
        println!("mod.rs already exists checking with ormolu section");
        let mut file_content = String::new();
        File::open(mod_rs_file)
            .expect("Could not open file")
            .read_to_string(&mut file_content)
            .expect("Could not read file");

        if let Some(start_index) = file_content.find(START_TAG) {
            if let Some(end_index) = file_content.find(END_TAG) {
                let new_content = format!(
                    "{}\n{}\n{}",
                    &file_content[..start_index + START_TAG.len()],
                    mod_statements.to_string(),
                    &file_content[end_index..]
                );

                let mut file = OpenOptions::new()
                    .write(true)
                    .truncate(false)
                    .open(mod_rs_path.clone())
                    .expect("Could not open file for writing");

                file.write(new_content.as_bytes())
                    .expect("Could not write to file");
                format_rust_file(mod_rs_file.to_str().expect("msg"))
            } else {
                panic!("End tag not found in file");
            }
        } else {
            panic!("Start tag not found in file");
        }
    } else {
        let mut file = File::create(mod_rs_file).expect("Could not create file");

        let content = format!("{}\n{}\n{}", START_TAG, mod_statements.to_string(), END_TAG);

        file.write_all(content.as_bytes())
            .expect("failed to mod statements in mod.rs");
        format_rust_file(mod_rs_path.as_str());
    }

    // let db_names = get_databases(&pool).await.unwrap();

    // convert each table to file
    // if file name for table already exists
    // check for begin to end section
    // replace file from begin to end section
}

// Database             (gen db struct)
//      Table[]         (gen table struct and item struct  (struct User && struct UserTable) )
//          Column[]    (gen Vec<Column>) which is just each fields datatype and metadata
