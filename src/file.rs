use std::path::{
    PathBuf,
    Path,
};

use std::io::{
    self,
    BufRead,
    BufReader,
    LineWriter,
    Read,
    Write,
};

use std::iter::Peekable;

use std::fs::{
    self,
    File,
    OpenOptions
};

use crate::config::{
    CONFIG_FILE_PATH,
    CONFIG_PATH,
    DEFAULT_DATA_PATH,
    DEFAULT_DATA_FILE_NAME,

    Config,
    DFTodoItem,
};

pub trait DFTodoStackFile: Read + Write + DFTodoCreate {}
impl<T> DFTodoStackFile for T where T: Read + Write + DFTodoCreate {}

pub trait DFTodoCreate {
    fn create<P: AsRef<Path>>(path: P, append: bool) -> io::Result<Self>
        where Self: Sized;
}

impl DFTodoCreate for File {
    fn create<P: AsRef<Path>>(path: P, append: bool) -> io::Result<Self> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .append(append)
            .truncate(!append)
            .create(true)
            .open(path)
 }
}

pub fn get_active_stack_file<F>(append: bool) -> Result<F, &'static str>
where F: DFTodoStackFile {
    let config_file: F = get_config_file()?;
    let (stack_file_directory, stack_file_name) = get_stack_directory(config_file)?;
    get_stack_file(stack_file_directory, stack_file_name, append)
}

fn get_config_file<F>() -> Result<F, &'static str>
where F: DFTodoStackFile {
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

    F::create(CONFIG_FILE_PATH.as_path(), true).map_err(|_| { "Failed to open configuration file" })
}

fn get_stack_directory<F>(config_file: F) -> Result<(PathBuf, String), &'static str>
where F: DFTodoStackFile {
    let config: Config = serde_json::from_reader(config_file).unwrap();
    Ok((config.data_path, config.file_name))
}

fn get_stack_file<F>(mut file_path: PathBuf, file_name: String, append: bool) -> Result<F, &'static str>
where F: DFTodoStackFile {
    file_path.push(file_name);
    let path = file_path.as_path();

    F::create(path, append)
        .map_err(|_| { "Error opening file" })
}

pub fn write_top_item<F>(stack_file: F, item: DFTodoItem) -> Result<(), &'static str>
where F: DFTodoStackFile {
    let mut file = LineWriter::new(stack_file);
    file.write_all((item.item + "\n").as_bytes()).map_err(|_| {"Error writing to file"})
}

pub fn remove_top_item<F>(stack_file: F) -> Result<(), &'static str>
where F: DFTodoStackFile {
    let line_iter = BufReader::new(stack_file).lines().peekable();
    let content = collect_all_but_last(line_iter);
    let mut file: File = get_active_stack_file(false)?;
    file.write_all(content.as_bytes()).map_err(|_| { "Failed to write to file" })
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

pub fn get_top_item<F>(stack_file: F) -> Option<String>
where F: DFTodoStackFile {
    let iterator = BufReader::new(stack_file).lines();

    match iterator.last() {
        None => Some("No item found in stack".to_string()),
        Some(Ok(item)) => Some(item),
        Some(Err(_)) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    struct MockFile {
        data: String,
        append: bool,
    }

    impl Write for MockFile {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            if self.append {
                self.data.push_str(&String::from_utf8(buf.to_vec()).unwrap());
                Ok(buf.len())
            } else {
                self.data = String::from_utf8(buf.to_vec()).unwrap();
                Ok(buf.len())
            }
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl Read for MockFile {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let mut written_bytes: usize = 0;
            for (i, character) in self.data.as_bytes().iter().enumerate() {
                if i == buf.len() {
                    break;
                }

                buf[i] = *character;
                written_bytes = i;
            }

            Ok(written_bytes)
        }
    }

    impl DFTodoCreate for MockFile {
        fn create<P: AsRef<Path>>(_path: P, append: bool) -> io::Result<Self>
        where Self: Sized {
            Ok(MockFile { data: String::new(), append })
        }
    }
}

