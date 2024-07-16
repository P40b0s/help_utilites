use std::{fmt, fs::OpenOptions, io::{BufWriter, Write}, path::PathBuf, str::FromStr};
use logger::error;
pub use serde::{de, Deserialize, Deserializer, Serialize};
pub use serde_json;

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
        let e = format!("Ошибка десериализации файла {} -> {}", write.err().unwrap(), pretty.err().unwrap());
        error!("{}", &e);
        return Err(e);
    }
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



