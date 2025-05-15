use clap::*;
mod test;
// use std::env;
// use syn::Data;
// mod generate;
// pub mod postgres;
// use crate::generate::generate;
// mod test;
// pub mod traits;

#[derive(Parser)]
#[command(
    name = "ormolu-cli",
    about = "Command-line app to interact with databases and rust source"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate rust source code from the structures in the database
    Generate {
        /// Directory to output the generated files. Default: current directory
        #[arg(short, long)]
        output_path: Option<String>,

        #[arg(short, long)]
        database_url: Option<String>,
    },
    /// Generate sql migration files that can be run to alter database structures to match rust source code
    Alter {
        /// Directory to output the generated files. Default: current directory
        #[arg(short, long)]
        output_path: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    // let cli = Cli::parse();
    // match cli.command {
    //     Commands::Generate {
    //         output_path,
    //         database_url,
    //     } => match database_url {
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
    //     },
    //     Commands::Alter { output_path: _ } => {
    //         unimplemented!()
    //     }
    // }
}
