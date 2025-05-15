use crate::postgres::generate_postgres;
use std::{fs, path::Path};
use url::Url;

pub async fn generate(output_path: String, database_url: String) {
    let output_path = Path::new(&output_path);

    // check if path exists
    fs::create_dir_all(output_path).expect("failed to create directory");

    // check that path is not a file
    let metadata = output_path.metadata().unwrap();

    if !metadata.is_dir() {
        panic!("Output provided is not a directory")
    }

    // read database_url
    let url = Url::parse(&database_url).unwrap();

    match url.scheme() {
        "postgres" | "postgresql" => generate_postgres(&url, output_path).await,
        _ => unimplemented!("{} is not supported", url.scheme()),
    }
}
