mod args;
mod database;

use args::{Command, AddArgs};
use structopt::StructOpt;

fn main() {
    let args = args::Arguments::from_args();
    match args.command {
        Command::Add(add_args) => add_tags(&add_args),
        Command::Init => initialize_database(),
    }
}

fn add_tags(args: &AddArgs) {
    let mut database = database::Database::load_from_current_dir().unwrap();
    for file in &args.files {
        database.add_tag(&file, &args.tag);
    }
    println!("{:#?}", database);
    database.save();
}

fn initialize_database() {
    let current_dir = std::env::current_dir().unwrap();
    let db = database::Database::init(&current_dir).unwrap();
    db.save();
}
