use std::{fmt, fs::OpenOptions, io::{BufWriter, Write}, path::{Path, PathBuf}, str::FromStr};
use logger::{error, warn};
use serde::de::DeserializeOwned;
pub use serde::{de, Deserialize, Deserializer, Serialize};
pub use serde_json;
#[cfg(feature="dates")]
use crate::Date;

///Сериализация объекта в строковый формат
pub fn serialize_to_file<T>(json : T, file_name : &str, directory: Option<&str>) -> Result<(), String> where T : Clone + Serialize
{
    let mut work_dir = PathBuf::default();
    if directory.is_some()
    {
        let p = PathBuf::from(directory.unwrap());
        work_dir = p;
        work_dir.push(file_name);
    }
    else
    {
        work_dir = std::env::current_dir().unwrap();
        work_dir.push(file_name);
    }
    let write = OpenOptions::new()
    .write(true)
    .create(true)
    .open(work_dir);
    if write.is_err()
    {
        let e = format!("{}", write.err().unwrap());
        error!("{}",&e);
        return Err(e);
    }
    let pretty = serde_json::to_string_pretty(&json);
    if let Ok(pretty) = pretty
    {
        let mut f = BufWriter::new(write.unwrap());
        let _write = f.write_all(pretty.as_bytes());
        return Ok(());
    }
    else
    {
        let e = format!("Ошибка загрузки файла {} -> {}", write.err().unwrap(), pretty.err().unwrap());
        error!("{}", &e);
        return Err(e);
    }
}



#[derive(Clone)]
pub enum Serializer
{
    Json,
    Toml
}
///Сериализация объекта в файл с помощью указанного сериализатора
pub fn serialize<T, P: AsRef<Path>>(json : T, file_path : P, path_as_absolute: bool, serializer: Serializer) -> Result<(), crate::error::Error> where T : Clone + Serialize 
{
    let path = if !path_as_absolute
    {
        Path::new(&std::env::current_dir().unwrap()).join(file_path)
    }
    else
    {
        file_path.as_ref().to_path_buf()
    };
    let write = OpenOptions::new()
    .write(true)
    .create(true)
    .truncate(true)
    .open(&path)?;
    let ser = match serializer
    {
        Serializer::Toml => toml::to_string(&json)?,
        Serializer::Json => serde_json::to_string_pretty(&json)?
    };
    let mut f = BufWriter::new(write);
    let _write = f.write_all(ser.as_bytes());
    return Ok(());
   
}


/// десериализация файла в нужный объект с помощью указанного десериализатора
pub fn deserialize<'de, T, P: AsRef<Path>>(file_path: P, path_as_absolute: bool, serializer: Serializer) -> Result<T, crate::error::Error> where T : Clone + DeserializeOwned
{
    let path = if !path_as_absolute
    {
        Path::new(&std::env::current_dir().unwrap()).join(file_path)
    }
    else
    {
        file_path.as_ref().to_path_buf()
    };
    let file = std::fs::read_to_string(&path)?;
    let result: T = match serializer
    {
        Serializer::Toml => toml::from_str(&file)?,
        Serializer::Json => serde_json::from_str(&file)?
    };
    Ok(result)
}




pub fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
    }
}

pub fn null_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("null") | Some("NULL") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
    }
}
#[cfg(feature="dates")]
pub fn deserialize_date<'de, D>(deserializer: D) -> Result<Date, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let s: String = serde::de::Deserialize::deserialize(deserializer)?;
    Date::parse(&s).ok_or(serde::de::Error::custom(format!("Формат даты `{}` не поддерживается", &s)))
}


#[cfg(feature="dates")]
pub fn deserialize_option_date<'de, D>(deserializer: D) -> Result<Option<Date>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt.as_deref() 
    {
        None | Some("null") | Some("NULL") => Ok(None),
        Some(s) => Ok(Date::parse(s))
    }
    
}



