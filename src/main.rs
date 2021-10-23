mod config;
mod file;

use clap::Clap;

use std::fs::File;

use crate::config::{
    DFTodoArgs,
    DFTodoAction,
    DFTodoItem,
};

use crate::file::{
    get_active_stack_file,
    get_top_item,
    write_top_item,
    remove_top_item,
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
    let file: File = get_active_stack_file(true)?;
    let top_item = get_top_item(file);
    match top_item {
        Some(item) => println!("{}", item),
        None => println!("ERROR: No item found"),
    }

    Ok(())
}

fn push_item(item: DFTodoItem) -> Result<(), &'static str> {
    let file: File = get_active_stack_file(true)?;

    write_top_item(file, item)
}

fn pop_item() -> Result<(), &'static str> {
    let file: File = get_active_stack_file(true)?;
    remove_top_item(file)
}

