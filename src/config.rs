use lazy_static::lazy_static;
use std::path::PathBuf;
use clap::Clap;
use serde::{
    Serialize,
    Deserialize,
};

lazy_static! {
    pub static ref CONFIG_PATH: PathBuf = {
        let mut path = dirs::config_dir().unwrap();
        path.push("dftodo");
        path
    };

    pub static ref CONFIG_FILE_PATH: PathBuf = {
        let mut path = dirs::config_dir().unwrap();
        path.push("dftodo");
        path.push("config");
        path.set_extension("json");
        path
    };

    pub static ref DEFAULT_DATA_PATH: PathBuf = {
        let mut path = dirs::data_dir().unwrap();
        path.push("dftodo");
        path
    };
}

pub const DEFAULT_DATA_FILE_NAME: &str = "stack";

#[derive(Clap, Debug)]
#[clap(name = "DFTodo", 
       version = "0.1.0",
       about = "A simple depth-first stack todo manager with a cli")]
pub struct DFTodoArgs {
    #[clap(subcommand)]
    pub action: DFTodoAction,
}

#[derive(clap::Subcommand, Debug)]
pub enum DFTodoAction {
    #[clap(about = "Display top option in current todo stack")]
    Top,
    #[clap(about = "Push new item onto top of current stack")]
    Push(DFTodoItem),
    #[clap(about = "Pop top item off of current stack")]
    Pop,
}

#[derive(Clap, Debug)]
pub struct DFTodoItem {
    #[clap(about = "Todo item description",
           index = 1)]
    pub item: String,
}


#[derive(Serialize, Deserialize)]
pub struct Config {
    pub data_path: PathBuf,
    pub file_name: String,
}
