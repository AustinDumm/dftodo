
use std::fs::{
    self,
    File,
    OpenOptions
};
use std::path::PathBuf;

use std::io::{
    Write,
    BufRead,
    BufReader,
    LineWriter,
};

use lazy_static::lazy_static;
use clap::Clap;
use serde::{
    Serialize,
    Deserialize,
};
use serde_json;

lazy_static! {
    static ref CONFIG_PATH: PathBuf = {
        let mut path = dirs::config_dir().unwrap();
        path.push("dftodo");
        path
    };

    static ref CONFIG_FILE_PATH: PathBuf = {
        let mut path = dirs::config_dir().unwrap();
        path.push("dftodo");
        path.push("config");
        path.set_extension("json");
        path
    };

    static ref DEFAULT_DATA_PATH: PathBuf = {
        let mut path = dirs::data_dir().unwrap();
        path.push("dftodo");
        path
    };
}

const DEFAULT_DATA_FILE_NAME: &str = "stack";

#[derive(Clap, Debug)]
#[clap(name = "DFTodo", 
       version = "0.1.0",
       about = "A simple depth-first stack todo manager with a cli")]
struct DFTodoArgs {
    #[clap(subcommand)]
    action: DFTodoAction,
}

#[derive(clap::Subcommand, Debug)]
enum DFTodoAction {
    #[clap(about = "Display top option in current todo stack")]
    Top,
    #[clap(about = "Push new item onto top of current stack")]
    Push(DFTodoItem),
    #[clap(about = "Pop top item off of current stack")]
    Pop,
}

#[derive(Clap, Debug)]
struct DFTodoItem {
    #[clap(about = "Todo item description",
           index = 1)]
    item: String,
}


#[derive(Serialize, Deserialize)]
struct Config {
    data_path: PathBuf,
    file_name: String,
}

fn main() -> Result<(), &'static str> {
    let args = DFTodoArgs::parse();

    match args.action {
        DFTodoAction::Top => print_top(),
        DFTodoAction::Push(item) => push_item(item),
        DFTodoAction::Pop => Err("Pop not yet implemented"),
    }
}

fn print_top() -> Result<(), &'static str> {
    let file = get_active_stack_file()?;
    let top_item = get_top_item(file);
    match top_item {
        Some(item) => println!("{}", item),
        None => println!("ERROR: No item found"),
    }

    Ok(())
}

fn push_item(item: DFTodoItem) -> Result<(), &'static str> {
    let file = get_active_stack_file()?;

    let mut file = LineWriter::new(file);
    file.write_all((item.item + "\n").as_bytes()).unwrap();

    Ok(())
}

fn get_active_stack_file() -> Result<File, &'static str> {
    let config_file = get_config_file()?;
    let (stack_file_directory, stack_file_name) = get_stack_directory(config_file)?;
    get_stack_file(stack_file_directory, stack_file_name)
}

fn get_config_file() -> Result<File, &'static str> {
    if !CONFIG_FILE_PATH.as_path().exists() {
        fs::create_dir_all(CONFIG_PATH.as_path()).unwrap();

        let file = File::create(CONFIG_FILE_PATH.as_path()).unwrap();
        let config =
            Config { 
                data_path: DEFAULT_DATA_PATH.clone(),
                file_name: DEFAULT_DATA_FILE_NAME.to_string(),
            };

        serde_json::to_writer(file, &config).unwrap()
    }

    File::open(CONFIG_FILE_PATH.as_path()).map_err(|_| { "Failed to open configuration file" })
}

fn get_stack_directory(config_file: File) -> Result<(PathBuf, String), &'static str> {
    let config: Config = serde_json::from_reader(config_file).unwrap();
    Ok((config.data_path, config.file_name))
}

fn get_stack_file(file_path: PathBuf, file_name: String) -> Result<File, &'static str> {
    let path = file_path.as_path();
    let mut file_path = file_path.clone();
    file_path.push(file_name);

    if !path.exists() {
        fs::create_dir_all(path).unwrap();
        File::create(file_path.clone()).unwrap();
    }

    OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .create(true)
        .open(file_path).map_err(|_| { "Failed to open stack file" })
}

fn get_top_item(stack_file: File) -> Option<String> {
    let iterator = BufReader::new(stack_file).lines();

    match iterator.last() {
        None => Some("No item found in stack".to_string()),
        Some(Ok(item)) => Some(item),
        Some(Err(_)) => None,
    }
}

