use std::{fs::File, io::Read, path::{Path, PathBuf}};
use logger::error;
#[cfg(feature="async-io")]
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub fn read_file_to_binary<P: AsRef<Path>>(file_path: P) -> std::io::Result<Vec<u8>>
{
    let f = File::open(file_path)?;
    let mut f = f;
    let mut buffer = Vec::new();
    let _ = f.read_to_end(&mut buffer)?;
    Ok(buffer)
}
#[cfg(feature="async-io")]
pub async fn read_file_to_binary_async<P: AsRef<Path>>(file_path: P) -> std::io::Result<Vec<u8>>
{
    let f = tokio::fs::File::open(file_path).await?;
    let mut f = f;
    let mut buffer = Vec::new();
    let _ = f.read_to_end(&mut buffer).await?;
    Ok(buffer)
}
///совпадение имени файла по маске
/// ```
/// coincidence_by_mask("file.txt",  "f*.txt")
/// ```
/// 
pub fn coincidence_by_mask(file_name: &str, mask: &str) -> bool
{
    if let Some((start, end)) = mask.split_once("*")
    {
        if start.is_empty()
        {
            file_name.ends_with(end)
        }
        else if end.is_empty() 
        {
            file_name.starts_with(start)
        }
        else 
        {
            file_name.starts_with(start) && file_name.ends_with(end)
        }
    }
    else 
    {
        file_name == mask
    }
}


#[deprecated = "Получает список всех файлов и директорий, для директорий необходимо использовать `get_only_dirs`"]
///Получение списка всех файлов и директорий
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
///Получение списка всех директорий
pub fn get_only_dirs<P: AsRef<Path>>(path: P) -> Option<Vec<PathBuf>>
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
        let dir = d.unwrap().path();
        if dir.is_dir()
        {
            dirs.push(dir);
        }
    }
    return Some(dirs);
}
///рекурсивное копирование директорий
pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<(), crate::error::Error> 
{
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? 
    {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() 
        {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } 
        else 
        {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

///search files in directory by mask `f*.txt`, `*.txt`, `file*`
pub async fn get_files_by_mask(target_dir: impl AsRef<Path>, mask: &str) -> Result<Vec<PathBuf>, crate::error::Error> 
{
    let target_dir = target_dir.as_ref();
    let mut list  = tokio::fs::read_dir(target_dir).await?;
    let mut output = Vec::new();
    while let Some(entry) = list.next_entry().await?
    {
        let ty = entry.file_type().await?;
        if ty.is_dir()
        {
            return get_files_by_mask(target_dir, mask).await;
        }
        else 
        {
            if let Some(name) = entry.file_name().to_str()
            {
                if coincidence_by_mask(name, mask)
                {
                    output.push(entry.path());
                }
            }
        }
    }
    Ok(output)
}
#[cfg(feature="async-io")]
pub async fn get_dirs_async<P: AsRef<Path>>(path: P) -> Result<Vec<String>, crate::error::Error>
{
    let mut paths = tokio::fs::read_dir(path).await?;
    let mut dirs = vec![];
    while let Some(entry) = paths.next_entry().await?
    {
        let dir = entry.file_name().to_str().unwrap().to_owned();
        dirs.push(dir);
    }
    Ok(dirs)
}



#[cfg(feature="encoding")]
pub enum FileEncoding
{
    Utf8,
    Windows1251
}
#[cfg(feature="encoding")]
///Если не указано явно, сначала пробует открыть файл в utf-8 если возникнет ошибка то пробует перевести кодировку в windows-1251
pub async fn open_file_with_encoding<P: AsRef<Path> + ToString>(path: P, encoding: Option<FileEncoding>) -> Result<String, Error>
{
    use encoding::{all::WINDOWS_1251, DecoderTrap, Encoding};
    use tokio::io::AsyncReadExt;

    let mut bytes = Vec::new();
    //let mut ok_encoding = true;
    let mut file = tokio::fs::File::open(&path).await?;
    let _ = file.read_to_end(&mut bytes).await?;
    //Если указан конкретный энкодинг то сюда
    if let Some(e) = encoding
    {
        match e
        {
            FileEncoding::Utf8 => return enc_utf_8(&bytes, &path),
            FileEncoding::Windows1251 => return enc_win1251(&bytes, &path),
        }
    }
    else 
    {
        //если не указан то пробуем utf-8, если ошибка то пробуем windows-1251
        return enc_utf_8(&bytes, &path);
    }
    
    fn enc_utf_8<P: AsRef<Path> + ToString>(bytes: &[u8], path: &P) -> Result<String, Error>
    {
        let utf8 = std::str::from_utf8(&bytes);
        if utf8.is_err()
        {
            return enc_win1251(bytes, path);
        }
        else 
        {
            let utf8 = utf8.unwrap();
            logger::info!("Файл {} открыт в кодировке utf8", path.to_string());
            return Ok(utf8.to_owned());
        }
    }
    fn enc_win1251<P: AsRef<Path> + ToString>(bytes: &[u8], path: &P) -> Result<String, Error>
    {
        let result = WINDOWS_1251.decode(&bytes, DecoderTrap::Strict);
        if result.is_err()
        {
            return Err(Error::FileOpenError(path.to_string(), result.err().unwrap().into_owned()));
        }
        let result = result.unwrap();
        logger::info!("Файл {} открыт в кодировке windows-1251", path.to_string());
        return Ok(result);
    }
}

#[cfg(test)]
mod tests
{
    use logger::info;

    #[tokio::test]
    #[cfg(feature="encoding")]
    async fn test_open_win1251_file()
    {
        let _ = logger::StructLogger::new_default();
        let file = "/hard/xar/projects/test_data/copy_from_in_test_data/in3/15943916/envelope.ltr";
        let file = super::open_file(file, None).await;
        info!("{}", file.unwrap());
    }
    #[tokio::test]
    #[cfg(feature="encoding")]
    async fn test_open_utf8_file()
    {
        let _ = logger::StructLogger::new_default();
        let file = "/hard/xar/projects/test_data/copy_from_in_test_data/in3/15943916/document.xml";
        let file = super::open_file(file, None).await;
        info!("{}", file.unwrap());
    }
    #[test]
    fn testsearch_by_mask()
    {
        let _ = logger::StructLogger::new_default();
        assert!(super::coincidence_by_mask("file.txt",  "f*.txt"));
        assert!(super::coincidence_by_mask("file.txt",  "*le.txt"));
        assert!(super::coincidence_by_mask("file.txt",  "fi*.txt"));
        assert!(super::coincidence_by_mask("file.txt",  "file.t*"));
    }
}
