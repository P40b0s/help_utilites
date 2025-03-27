use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error 
{
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    DeserializeError(#[from] serde_json::Error),
    #[error(transparent)]
    TomlDeserializeError(#[from] toml::de::Error),
    #[error(transparent)]
    TomlSerializeError(#[from] toml::ser::Error),
    #[error(transparent)]
    #[cfg(feature="http")]
    HyperError(#[from] hyper::Error),
    #[error(transparent)]
    #[cfg(feature="http")]
    HyperHttpError(#[from] hyper::http::Error),
    #[error(transparent)]
    #[cfg(feature="http")]
    HttpClientLegacyError(#[from] hyper_util::client::legacy::Error),
    #[error("По данным параметрам заявки `{0}`")]
    NotFreeWorkers(String),
    #[error("Ошибка сервиса станций `{0}`")]
    StationServiceError(String),
    #[error("Ошибка подключения к сервису `{0}` при отправке сообщения")]
    SendError(String),
    #[error("Ошибка открытия фала `{0}` -> {1}")]
    FileOpenError(String, String),
    #[error("Ошибка входного формата при преобразовании `{0}` -> {1}")]
    DateParseError(String, String),
    // #[error(transparent)]
    // #[cfg(feature="http")]
    // ReqwestError(#[from] reqwest::Error),
    // #[error(transparent)]
    // #[cfg(feature="http")]
    // ReqwestMiddlewareError(#[from] reqwest_middleware::Error),
}