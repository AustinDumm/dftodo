
use std::iter::Peekable;
use std::path::PathBuf;
use clap::Clap;

use std::fs::{
    self,
    File,
    OpenOptions
};

use std::io::{
    Write,
    BufRead,
    BufReader,
    LineWriter,
};

use serde_json;

mod config;

use config::{
    DFTodoArgs,
    DFTodoAction,
    DFTodoItem,

    Config,
    
    CONFIG_FILE_PATH,
    CONFIG_PATH,
    DEFAULT_DATA_FILE_NAME,
    DEFAULT_DATA_PATH,
};

fn main() -> Result<(), &'static str> {
    let args = DFTodoArgs::parse();

    match args.action {
        DFTodoAction::Top => print_top(),
        DFTodoAction::Push(item) => push_item(item),
        DFTodoAction::Pop => pop_item(),
    }
}

fn print_top() -> Result<(), &'static str> {
    let file = get_active_stack_file(true)?;
    let top_item = get_top_item(file);
    match top_item {
        Some(item) => println!("{}", item),
        None => println!("ERROR: No item found"),
    }

    Ok(())
}

fn push_item(item: DFTodoItem) -> Result<(), &'static str> {
    let file = get_active_stack_file(true)?;

    let mut file = LineWriter::new(file);
    file.write_all((item.item + "\n").as_bytes()).unwrap();

    Ok(())
}

fn pop_item() -> Result<(), &'static str> {
    let file = get_active_stack_file(true)?;
    let line_iter = BufReader::new(file).lines().peekable();
    let content = collect_all_but_last(line_iter);
    let mut file = get_active_stack_file(false)?;
    file.write(content.as_bytes()).unwrap();

    Ok(())
}

fn collect_all_but_last<I>(mut peekable: Peekable<I>) -> String
where I: Iterator<Item = std::io::Result<String>> {
    let mut collected: String = String::new();
    while let Some(Ok(item)) = peekable.next() {
        if peekable.peek().is_none() {
            break;
        }

        collected += &(item + "\n");
    }

    collected
}

fn get_active_stack_file(append: bool) -> Result<File, &'static str> {
    let config_file = get_config_file()?;
    let (stack_file_directory, stack_file_name) = get_stack_directory(config_file)?;
    get_stack_file(stack_file_directory, stack_file_name, append)
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

fn get_stack_file(file_path: PathBuf, file_name: String, append: bool) -> Result<File, &'static str> {
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
        .append(append)
        .truncate(!append)
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

