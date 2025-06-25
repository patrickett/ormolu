mod postgres;
pub use postgres::*;
// use crate::postgres::generate_postgres;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};
use url::Url;

pub trait Generate {
    type TableData;

    fn generate(url: &Url) -> std::io::Result<String> {
        let rt = tokio::runtime::Runtime::new()?;
        let data = rt.block_on(Self::get_table_data(url));

        Ok(Self::create_output(data).expect("failed to create output"))
    }

    fn get_table_data(url: &Url) -> impl Future<Output = Vec<Self::TableData>>;
    fn create_output(data: Vec<Self::TableData>) -> Result<String, &'static str>;
}

pub fn generate(output_path: PathBuf, database_url: String) {
    let output_path = Path::new(&output_path);

    // check if path exists
    fs::create_dir_all(output_path).expect("failed to create directory");

    // check that path is not a file
    let metadata = output_path.metadata().expect("output metadata");

    if !metadata.is_dir() {
        panic!("Output provided is not a directory")
    }

    // read database_url
    let url = Url::parse(&database_url).expect("parse url");

    let source_code = match url.scheme() {
        "postgres" | "postgresql" => PostgreSQL::generate(&url),
        _ => unimplemented!("{} is not supported", url.scheme()),
    }
    .expect("failed to generate source code");

    let file_path = output_path.join("db.rs");
    let mut file = fs::File::create(&file_path).expect("create file");
    file.write_all(source_code.as_bytes())
        .expect("write source code");

    let _output = Command::new("rustfmt")
        .arg(file_path)
        .output()
        .expect("rustfmt");
}
