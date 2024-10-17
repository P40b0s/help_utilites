use std::{fs::File, io::Read, path::{Path, PathBuf}};

use logger::error;

pub fn read_file_to_binary<P: AsRef<Path>>(file_path: P) -> std::io::Result<Vec<u8>>
{
    let f = File::open(file_path)?;
    let mut f = f;
    let mut buffer = Vec::new();
    let _ = f.read_to_end(&mut buffer)?;
    Ok(buffer)
}

///Получение списка директорий
pub fn get_dirs<P: AsRef<Path>>(path: P) -> Option<Vec<String>>
{
    let paths = std::fs::read_dir(path);
    if paths.is_err()
    {
        error!("😳 Ошибка чтения директории -> {}", paths.err().unwrap());
        return None;
    }
    let mut dirs = vec![];
    for d in paths.unwrap()
    {
        let dir = d.unwrap().file_name().to_str().unwrap().to_owned();
        dirs.push(dir);
    }
    return Some(dirs);
}