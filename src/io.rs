use std::{fs::File, io::Read, path::Path};

pub fn read_file_to_binary<P: AsRef<Path>>(file_path: P) -> std::io::Result<Vec<u8>>
{
    let f = File::open(file_path)?;
    let mut f = f;
    let mut buffer = Vec::new();
    let _ = f.read_to_end(&mut buffer)?;
    Ok(buffer)
}