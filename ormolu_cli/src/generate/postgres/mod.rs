#![allow(dead_code)]
use super::Generate;
use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{TokenStreamExt, quote};
use sqlx::prelude::FromRow;
use std::collections::HashMap;
use syn::{Ident, parse_str};
use url::Url;

pub struct PostgreSQL;

#[derive(FromRow, Debug)]
pub struct InfoSchemaColumn {
    table_catalog: String,
    table_schema: String,
    table_name: String,
    column_name: String,
    ordinal_position: i32,
    column_default: Option<String>,
    is_nullable: String,
    data_type: String,
    character_maximum_length: Option<i32>,
    udt_name: String, // TODO: actual datatype including custom types
    is_self_referencing: String,
    is_identity: String,
    is_updatable: String,
    constraint_type: Option<String>,
    referenced_table_schema: Option<String>,
    referenced_table: Option<String>,
    referenced_column: Option<String>,
    referenced_column_ordinal_position: Option<i32>,
}

impl Generate for PostgreSQL {
    type TableData = InfoSchemaColumn;

    async fn get_table_data(url: &Url) -> Vec<Self::TableData> {
        let Ok(pool) = sqlx::PgPool::connect(url.as_str()).await else {
            panic!("cannot connect via: {url}",)
        };

        sqlx::query_as::<_, InfoSchemaColumn>(
            "
            SELECT
                cols.*,
                tc.constraint_type,
                ccu.table_schema AS referenced_table_schema,
                ccu.table_name AS referenced_table,
                ccu.column_name AS referenced_column,
                ref_cols.ordinal_position AS referenced_column_ordinal_position
            FROM
                information_schema.columns AS cols
            LEFT JOIN information_schema.key_column_usage AS kcu
                ON cols.table_schema = kcu.table_schema
                AND cols.table_name = kcu.table_name
                AND cols.column_name = kcu.column_name
            LEFT JOIN information_schema.table_constraints AS tc
                ON kcu.constraint_name = tc.constraint_name
                AND kcu.table_schema = tc.table_schema
                AND kcu.table_name = tc.table_name
            LEFT JOIN information_schema.referential_constraints AS rc
                ON rc.constraint_name = tc.constraint_name
                AND rc.constraint_schema = tc.constraint_schema
            LEFT JOIN information_schema.constraint_column_usage AS ccu
                ON ccu.constraint_name = rc.unique_constraint_name
                AND ccu.constraint_schema = rc.unique_constraint_schema
            LEFT JOIN information_schema.columns AS ref_cols
                ON ref_cols.table_schema = ccu.table_schema
                AND ref_cols.table_name = ccu.table_name
                AND ref_cols.column_name = ccu.column_name
            WHERE
                cols.table_schema NOT IN ('pg_catalog', 'information_schema')
                AND cols.table_name NOT IN ('_sqlx_migrations')
                AND NOT (
                    tc.constraint_type = 'UNIQUE' AND
                    EXISTS (
                        SELECT 1
                        FROM information_schema.table_constraints AS t2
                        JOIN information_schema.key_column_usage AS k2
                          ON t2.constraint_name = k2.constraint_name
                          AND t2.table_schema = k2.table_schema
                          AND t2.table_name = k2.table_name
                        WHERE t2.constraint_type IN ('PRIMARY KEY', 'FOREIGN KEY')
                          AND k2.table_schema = cols.table_schema
                          AND k2.table_name = cols.table_name
                          AND k2.column_name = cols.column_name
                    )
                )
            ORDER BY
                cols.table_name DESC,
                cols.ordinal_position;
            ",
        )
        // .map(|row: sqlx::postgres::PgRow| {
        //     let table_catalog = row.get("column_name");
        //     let table_schema: String = row.get("column_name");
        //     let table_name: String = row.get("column_name");
        //     let column_name: String = row.get("column_name");
        //     let ordinal_position: i32 = row.get("column_name");
        //     let column_default: Option<String> = row.get("column_name");
        //     let is_nullable: bool = row.get("column_name");
        //     let data_type: String = row.get("column_name");
        //     let character_maximum_length: i32 = row.get("column_name");
        //     let udt_name_str: String = row.get("udt_name");
        //     let is_self_referencing: bool = row.get("column_name");
        //     let is_identity: bool = row.get("column_name");
        //     let is_updatable: bool = row.get("column_name");
        //     InfoSchemaColumn {
        //         table_catalog,
        //         table_schema,
        //         table_name,
        //         column_name,
        //         ordinal_position,
        //         column_default,
        //         is_nullable,
        //         data_type,
        //         character_maximum_length,
        //         udt_name,
        //         is_self_referencing,
        //         is_identity,
        //         is_updatable,
        //     }
        // })
        .fetch_all(&pool)
        .await
        .expect("sqlx")
    }

    fn create_output(data: Vec<Self::TableData>) -> Result<String, &'static str> {
        let mut map: HashMap<(String, String), Vec<InfoSchemaColumn>> = HashMap::new();

        for column in data {
            map.entry((column.table_schema.clone(), column.table_name.clone()))
                .or_default()
                .push(column);
        }

        let mut stream = TokenStream2::new();
        for ((table_schema, table_name), columns) in map {
            let struct_name: syn::Type =
                parse_str(table_name.to_case(Case::Pascal).as_str()).expect("struct_name");
            let fields: Vec<_> = columns.iter().map(typed_field).collect();

            let table = quote! {
              #[derive(Table)]
              #[gild(table = #table_name, schema = #table_schema)]
              pub struct #struct_name {
                  #(#fields),*
              }
            };

            stream.append_all(table);
        }

        // println!("{stream}");

        Ok(stream.to_string())
    }
}

// static USER_DEFINED: &str = "USER-DEFINED";

fn typed_field(col: &InfoSchemaColumn) -> TokenStream2 {
    let field_name = Ident::new(&col.column_name, Span::call_site());

    // if col.data_type.as_str() == USER_DEFINED {
    //     // panic!("custom user defined type")
    //     return quote! {};
    // }

    // TODO: do we actually need to parse type? prob not
    let mut field_type: String = match col.udt_name.as_str() {
        "text" | "_text" | "bytea" => "String".into(),
        "bool" => "bool".into(),
        "int2" => "i16".into(),
        "int4" => "i32".into(),
        "float4" => "f32".into(),
        "timestamp" => {
            // TODO: conditional if chrono enabled
            "chrono::NaiveDateTime".into()
        }
        "varchar" => {
            if let Some(max_len) = col.character_maximum_length {
                format!("VarChar<{max_len}>")
            } else {
                "String".into()
            }
        }
        _ => "String".into(), // udt_name => panic!("unknown udt_name: {udt_name}"),
    };

    if let Some(constraint) = &col.constraint_type {
        field_type = match constraint.as_str() {
            "PRIMARY KEY" => format!("PrimaryKey<Self, {field_type}>"),
            "FOREIGN KEY" => {
                let ord = col
                    .referenced_column_ordinal_position
                    .expect("foreign key needs ord pos");
                let ref_entity = col
                    .referenced_table
                    .as_ref()
                    .expect("ref table")
                    .to_case(Case::Pascal);

                format!("ForeignKey<{ref_entity}, {ord}, {field_type}>")
            }

            "UNIQUE" => format!("Unique<{field_type}>"),
            c => panic!("unknown constraint: {c}"),
        }
    }

    if col.is_identity.as_str() == "YES" {
        field_type = format!("Identity<{field_type}>");
    }

    if col.is_nullable.as_str() == "YES" {
        field_type = format!("Option<{field_type}>");
    }

    let field_type: syn::Type = parse_str(field_type.as_str()).expect("msg");

    quote! { pub #field_name: #field_type }
}

// _aclitem
// _bool
// _char
// _float4
// _float8
// _int2
// _name
// _oid
// _pg_statistic
// _regtype
// _text
// anyarray
// bool
// bytea
// char
// float4
// float8
// inet
// int2
// int2vector
// int4
// int8
// interval
// name
// numeric
// obit_label_category
// obit_publisher_kind
// oid
// oidvector
// pg_dependencies
// pg_lsn
// pg_mcv_list
// pg_ndistinct
// pg_node_tree
// regproc
// regtype
// text
// timestamp
// timestamptz
// url_scheme
// url_subdomain
// varchar
// xid
