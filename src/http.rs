use std::net::SocketAddr;
use hashbrown::HashMap;
use http_body_util::{BodyExt, Full};
use hyper::{body::Bytes, header::{self, CONTENT_TYPE, HOST}, Request, Response, StatusCode, Uri};
use hyper_util::rt::TokioIo;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use crate::error::Error;
pub type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;


pub async fn post<I: Serialize, O>(uri: Uri, obj: &I) -> Result<O, Error> where for<'de> O: Deserialize<'de>
{
    let host = uri.authority().unwrap().as_str().replace("localhost", "127.0.0.1");
    let req = Request::builder()
    .method("POST")
    .uri(&uri)
    .header(HOST, "localhost")
    .header(CONTENT_TYPE, "application/json")
    .body(to_body(Bytes::from(serde_json::to_string(&obj).unwrap())))
    .unwrap();
    logger::info!("Отправка запроса на {}, headers: {:?}", req.uri(), req.headers());
    let addr: SocketAddr = host.parse().unwrap();
    let client_stream = TcpStream::connect(&addr).await;
    if client_stream.is_err()
    {
        logger::error!("Ошибка подключения к сервису {} -> {}", &addr, client_stream.err().unwrap());
        return Err(Error::SendError(addr.to_string()));
    }
    let io = TokioIo::new(client_stream.unwrap());
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
    tokio::task::spawn(async move 
        {
            if let Err(err) = conn.await 
            {
                logger::error!("Ошибка подключения: {:?}", err);
            }
        });
    let send = sender.send_request(req).await?;
    let body = send.collect().await?.to_bytes();
    let response: O = serde_json::from_slice(&body)?;

    Ok(response)
}

pub async fn post_with_params<O>(uri: Uri) -> Result<O, Error> where for<'de> O: Deserialize<'de>
{
    let host = uri.authority().unwrap().as_str().replace("localhost", "127.0.0.1");
    let req = Request::builder()
    .method("POST")
    .uri(&uri)
    .header(HOST, "localhost")
    .body(to_body(Bytes::new()))
    .unwrap();
    logger::info!("Отправка запроса на {}, headers: {:?}", req.uri(), req.headers());
    let addr: SocketAddr = host.parse().unwrap();
    let client_stream = TcpStream::connect(&addr).await;
    if client_stream.is_err()
    {
        logger::error!("Ошибка подключения к сервису {} -> {}", &addr, client_stream.err().unwrap());
        return Err(Error::SendError(addr.to_string()));
    }
    let io = TokioIo::new(client_stream.unwrap());
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
    tokio::task::spawn(async move 
        {
            if let Err(err) = conn.await 
            {
                logger::error!("Ошибка подключения: {:?}", err);
            }
        });
    let send = sender.send_request(req).await?;
    let body = send.collect().await?.to_bytes();
    let response: O = serde_json::from_slice(&body)?;

    Ok(response)
}

pub async fn get<O>(uri: Uri) -> Result<O, Error> where for<'de> O: Deserialize<'de>
{
    let host = uri.authority().unwrap().as_str().replace("localhost", "127.0.0.1");
    let req = Request::builder()
    .method("GET")
    .uri(&uri)
    .header(HOST, "localhost")
    .body(to_body(Bytes::new()))
    .unwrap();
    logger::info!("Отправка запроса на {}, headers: {:?}", req.uri(), req.headers());
    let addr: SocketAddr = host.parse().unwrap();
    let client_stream = TcpStream::connect(&addr).await;
    if client_stream.is_err()
    {
        logger::error!("Ошибка подключения к сервису {} -> {}", &addr, client_stream.err().unwrap());
        return Err(Error::SendError(addr.to_string()));
    }
    let io = TokioIo::new(client_stream.unwrap());
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
    tokio::task::spawn(async move 
        {
            if let Err(err) = conn.await 
            {
                logger::error!("Ошибка подключения: {:?}", err);
            }
        });
    let send = sender.send_request(req).await?;
    if send.status() == StatusCode::OK
    {
        let body = send.collect().await?.to_bytes();
        //logger::debug!("{}", String::from_utf8_lossy(&body));
        let response: O = serde_json::from_slice(&body)?;
        Ok(response)
    }
    else
    {
        logger::error!("Ошибка получения инфомации от сервиса {} -> {}", &addr, send.status());
        return Err(Error::SendError(format!("Ошибка получения инфомации от сервиса {} -> {}", &addr, send.status())));
    }
}

///Получение словаря запросу по url в формате ключ\значение
pub fn get_query(uri: &Uri) -> Option<HashMap<String, String>>
{
    let params: Option<HashMap<String, String>> = uri
    .query()
    .map(|v| 
    {
        url::form_urlencoded::parse(v.as_bytes())
            .into_owned()
            .collect()
    });
    params
}


pub fn to_body(bytes: Bytes) -> BoxBody
{
    Full::new(bytes)
        .map_err(|never| match never {})
    .boxed()
}  
pub fn empty_response(code: StatusCode) -> Response<BoxBody>
{
    Response::builder()
    .status(code)
    .body(to_body(Bytes::new())).unwrap()
}

pub fn error_response(err: String, code: StatusCode) -> Response<BoxBody>
{
    Response::builder()
    .status(code)
    .body(to_body(Bytes::from(err))).unwrap()
}
pub fn error_empty_response(code: StatusCode) -> Response<BoxBody>
{
    Response::builder()
    .status(code)
    .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
    .body(to_body(Bytes::new())).unwrap()
}
pub fn ok_response(msg: String) -> Response<BoxBody>
{
    Response::builder()
    .status(StatusCode::OK)
    //.header(ACCESS_CONTROL_ALLOW_HEADERS, "User-Id")
    .body(to_body(Bytes::from(msg))).unwrap()
}
pub fn json_response<S: Serialize>(obj: &S) -> Response<BoxBody>
{
    let result = serde_json::to_string(obj).unwrap();
    Response::builder()
    .status(StatusCode::OK)
    .body(to_body(Bytes::from(result))).unwrap()
}

pub fn unauthorized_response() -> Response<BoxBody>
{
    Response::builder()
    .status(StatusCode::UNAUTHORIZED)
    .body(to_body(Bytes::from_static(b"Unauthorized")))
    .unwrap()
}