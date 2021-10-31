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
        if !path.as_ref().parent().unwrap().exists() {
            fs::create_dir_all(path.as_ref().parent().unwrap()).unwrap();
        }

        OpenOptions::new()
            .read(true)
            .write(true)
            .append(append)
            .truncate(!append)
            .create(true)
            .open(path)
 }
}

pub fn get_active_stack_file_path<F>(config_file_path: &Path,
                                     default_data_path: PathBuf) -> Result<PathBuf, &'static str>
where F: DFTodoStackFile {
    let config_file: F = get_config_file(config_file_path,
                                         default_data_path)?;
    get_stack_directory(config_file)
}

pub fn get_active_stack_file<F>(append: bool,
                                config_file_path: &Path,
                                default_data_path: PathBuf) -> Result<F, &'static str>
where F: DFTodoStackFile {
    let stack_file_directory = get_active_stack_file_path::<F>(config_file_path, default_data_path)?;
    get_stack_file(stack_file_directory, append)
}

fn get_config_file<F>(config_file_path: &Path,
                      default_data_path: PathBuf) -> Result<F, &'static str>
where F: DFTodoStackFile {
    let mut file = F::create(config_file_path, true).map_err(|_| { "Failed to open configuration file" })?;
    if !config_file_path.exists() {
        let config =
            Config { 
                data_path: default_data_path.clone(),
            };

        serde_json::to_writer(&mut file, &config).unwrap()
    }

    Ok(file)
}

fn get_stack_directory<F>(config_file: F) -> Result<PathBuf, &'static str>
where F: DFTodoStackFile {
    let config: Config = serde_json::from_reader(config_file).unwrap();
    Ok(config.data_path)
}

fn get_stack_file<F>(file_path: PathBuf, append: bool) -> Result<F, &'static str>
where F: DFTodoStackFile {
    let path = file_path.as_path();

    F::create(path, append)
        .map_err(|_| { "Error opening file" })
}

pub fn write_top_item<F>(stack_file: &mut F, item: DFTodoItem) -> Result<(), &'static str>
where F: DFTodoStackFile {
    let mut file = LineWriter::new(stack_file);
    file.write_all((item.item + "\n").as_bytes()).map_err(|_| {"Error writing to file"})
}

pub fn remove_top_item(stack_file_path: PathBuf) -> Result<(), &'static str> {
    // Because this is reading the file as truncated, it is taking in no data then replacing it
    // with nothing as well. We need to read the full data in as non truncated then open a new that
    // is truncated
    let read_file: File = get_stack_file(stack_file_path.clone(), true)?;
    let line_iter = BufReader::new(read_file).lines().peekable();
    let content = collect_all_but_last(line_iter);
    let mut write_file: File = get_stack_file(stack_file_path, false)?;
    write_file.write_all(content.as_bytes()).map_err(|_| { "Failed to write to file" })
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
        path: String,
        data: String,
        append: bool,
        read_index: usize,
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
            for (i, character) in self.data.as_bytes().iter().skip(self.read_index).enumerate() {
                if i == buf.len() {
                    break;
                }

                buf[i] = *character;
                written_bytes = i + 1;
                self.read_index += 1;
            }

            Ok(written_bytes)
        }
    }

    impl DFTodoCreate for MockFile {
        fn create<P: AsRef<Path>>(path: P, append: bool) -> io::Result<Self>
        where Self: Sized {
            Ok(MockFile { path: path.as_ref().to_str().unwrap().to_string(), data: String::new(), append, read_index: 0 })
        }
    }

    #[test]
    fn creates_correct_stack_file_path() -> Result<(), &'static str> {
        let path_buf = get_active_stack_file_path::<MockFile>(Path::new("dir/conf.json"), [r"path", "stack.txt"].iter().collect())?;
        assert_eq!(path_buf, [r"path", "stack.txt"].iter().collect::<PathBuf>());

        Ok(())
    }

    #[test]
    fn creates_correct_stack_file() -> Result<(), &'static str> {
        let stack_file: MockFile = get_active_stack_file(true, Path::new("dir/conf.json"), [r"path", "stack.txt"].iter().collect())?;
        assert_eq!(stack_file.path, String::from("path/stack.txt"));
        assert_eq!(stack_file.append, true);
        assert_eq!(stack_file.data, "");

        Ok(())
    }

    #[test]
    fn creates_correct_config_file() -> Result<(), &'static str> {
        let mock_file: MockFile = get_config_file(Path::new("dir/conf.json"), [r"path", "stack.txt"].iter().collect())?;
        assert_eq!(mock_file.path, String::from("dir/conf.json"));
        assert_eq!(mock_file.append, true);
        assert_eq!(mock_file.data, "{\"data_path\":\"path/stack.txt\"}");

        Ok(())
    }

    #[test]
    fn does_write_top_item_to_empty() -> Result<(), &'static str> {
        let item_text = "This is the new item";
        let mut mock_file = MockFile { path: "path".to_string(), data: "".to_string(), append: true, read_index: 0 };
        let item = DFTodoItem { item: item_text.to_string() };

        write_top_item(&mut mock_file, item)?;

        assert_eq!(mock_file.path, "path".to_string());
        assert_eq!(mock_file.data, (item_text.to_string() + "\n"));
        assert_eq!(mock_file.append, true);
        assert_eq!(mock_file.read_index, 0);

        Ok(())
    }
}

