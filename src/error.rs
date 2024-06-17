use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error 
{
    #[error(transparent)]
    DeserializeError(#[from] serde_json::Error),
    #[error(transparent)]
    #[cfg(feature="http")]
    HyperError(#[from] hyper::Error),
    #[error(transparent)]
    #[cfg(feature="http")]
    HyperHttpError(#[from] hyper::http::Error),
    #[error("По данным параметрам заявки `{0}`")]
    NotFreeWorkers(String),
    #[error("Ошибка сервиса станций `{0}`")]
    StationServiceError(String),
    #[error("Ошибка подключения к сервису `{0}` при отправке сообщения")]
    SendError(String),
}