mod config;
mod file;

use std::iter::Peekable;
use clap::Clap;

use std::io::{
    Write,
    BufRead,
    BufReader,
    LineWriter,
};

use crate::config::{
    DFTodoArgs,
    DFTodoAction,
    DFTodoItem,
};

use crate::file::{
    get_active_stack_file,
    get_top_item,
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

