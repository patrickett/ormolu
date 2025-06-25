use std::{env, path::PathBuf};

use clap::*;
use generate::generate;
// mod db;
mod generate;

#[derive(Parser)]
#[command(
    name = "ormolu-cli",
    about = "Command-line app to interact with databases and rust source"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn default_database_url() -> String {
    dotenvy::dotenv().expect("failed to get .env");

    match env::var("DATABASE_URL") {
        Ok(val) => val,
        Err(_) => panic!("could not get DATABASE_URL"),
    }
}

fn default_output_path() -> PathBuf {
    if let Ok(pb) = env::current_dir() {
        return pb;
    }
    PathBuf::from(".")
}

#[derive(Subcommand)]
enum Commands {
    /// Generate rust source code from the structures in the database
    Generate {
        /// Directory to output the generated files. Default: current directory
        #[arg(short, long)]
        output_path: Option<PathBuf>,

        #[arg(short, long)]
        database_url: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate {
            output_path,
            database_url,
        } => {
            let output_path = output_path.unwrap_or_else(default_output_path);
            let database_url = database_url.unwrap_or_else(default_database_url);

            generate(output_path, database_url)
        } //match database_url {
          //         Some(db_url) => {
          //             let output_path = output_path.unwrap_or(
          //                 env::current_dir()
          //                     .unwrap()
          //                     .into_os_string()
          //                     .into_string()
          //                     .unwrap(),
          //             );

          //             // generate(output_path, db_url).await
          //         }
          //         None => {
          //             // check dotenv
          //             todo!()
          //         }
          // },
          //     Commands::Alter { output_path: _ } => {
          //         unimplemented!()
          // }
    }
}
