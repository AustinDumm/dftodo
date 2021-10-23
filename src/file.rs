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

pub trait DFTodoStackFile: Read + Write + DFTodoOpen + DFTodoCreate {}
impl<T> DFTodoStackFile for T where T: Read + Write + DFTodoOpen + DFTodoCreate {}

pub trait DFTodoOpen {
    fn open<P: AsRef<Path>>(path: P) -> io::Result<Self>
        where Self: Sized;
}

pub trait DFTodoCreate {
    fn create<P: AsRef<Path>>(path: P) -> io::Result<Self>
        where Self: Sized;
}

impl DFTodoOpen for File {
    fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        File::open(path)
    }
}

impl DFTodoCreate for File {
    fn create<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        File::create(path)
    }
}

pub fn get_active_stack_file(append: bool) -> Result<File, &'static str> {
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

pub fn write_top_item(stack_file: File, item: DFTodoItem) -> Result<(), &'static str> {
    let mut file = LineWriter::new(stack_file);
    file.write_all((item.item + "\n").as_bytes()).map_err(|_| {"Error writing to file"})?;

    Ok(())
}

pub fn get_top_item(stack_file: File) -> Option<String> {
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
    }

    impl Write for MockFile {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.data.push_str(&String::from_utf8(buf.to_vec()).unwrap());
            Ok(buf.len())
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
}

