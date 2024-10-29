use std::{fs::File, io::Read, path::{Path, PathBuf}};
use crate::error::Error;
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





#[cfg(feature="encoding")]
pub enum FileEncoding
{
    Utf8,
    Windows1251
}
#[cfg(feature="encoding")]
///Если не указано явно, сначала пробует открыть файл в utf-8 если возникнет ошибка то пробует перевести кодировку в windows-1251
/// и открыть, если в открытом файле не находит букву а... за это вот стыдно, но перебирать несколько киррилических символов неохота
/// да и слишком такое на удачу... то ставит метку что есть ошибка в определении кодировки
pub async fn open_file<P: AsRef<Path> + ToString>(path: P, encoding: Option<FileEncoding>) -> Result<String, Error>
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
    async fn test_open_win1251_file()
    {
        let _ = logger::StructLogger::new_default();
        let file = "/hard/xar/projects/test_data/copy_from_in_test_data/in3/15943916/envelope.ltr";
        let file = super::open_file(file, None).await;
        info!("{}", file.unwrap());
    }
    #[tokio::test]
    async fn test_open_utf8_file()
    {
        let _ = logger::StructLogger::new_default();
        let file = "/hard/xar/projects/test_data/copy_from_in_test_data/in3/15943916/document.xml";
        let file = super::open_file(file, None).await;
        info!("{}", file.unwrap());
    }
}
