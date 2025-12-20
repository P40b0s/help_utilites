use std::{any::TypeId, cell::LazyCell, net::{IpAddr, Ipv4Addr, SocketAddr}, result, sync::Arc, time::Duration};
use http_body_util::Empty;
pub use http_body_util::{BodyExt, Full};
pub use hyper::{body::Bytes, header::*, Request, Response, StatusCode, Uri};
use hyper_util::{client::legacy::Client, rt::{TokioExecutor, TokioIo}};
use rand::Rng;
use std::collections::HashMap;
use serde::Serialize;
use tokio::{net::TcpSocket, sync::Mutex};
pub use tokio::net::TcpStream;
//use rustls::RootCertStore;
use hyper_rustls::ConfigBuilderExt;
use crate::{error::Error, retry};
pub type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

async fn connect(addr: SocketAddr) -> Result<TcpStream, Error>
{
    //let client_stream = TcpStream::connect(&addr).await;
    let client_stream = match tokio::time::timeout(Duration::from_millis(500),  TcpStream::connect(&addr)).await
    {
        Ok(connected) => connected,
        Err(_) => Err(std::io::Error::other("Connection timeout"))
    };
    if client_stream.is_err()
    {
        logger::error!("Ошибка подключения к {} -> {}", &addr, client_stream.err().unwrap());
        return Err(Error::SendError(addr.to_string()));
    }
    Ok(client_stream.unwrap())
}
async fn get_body_timeout(uri: Uri)  -> Result<Bytes, Error>
{
    let req =  Request::builder()
    .method("GET")
    .uri(&uri)
    .header(HOST, "pravo.gov.ru")
    .header(USER_AGENT, "Mozilla/5.0 (X11; Linux x86_64; rv:127.0) Gecko/20100101 Firefox/127.0")
    .header(ACCEPT, "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
    .header(ACCEPT_ENCODING, "gzip, deflate")
    .header(ACCEPT_LANGUAGE, "ru-RU,ru;q=0.8,en-US;q=0.5,en;q=0.3")
    //.header(CONNECTION, "keep-alive")
    .header(REFERER, ["http:://pravo.gov.ru", uri.path()].concat())
    .header(UPGRADE_INSECURE_REQUESTS, 1)
    .header("Priority", "u=1")
    .body(to_body(Bytes::new()))
    .unwrap();
    let response = match tokio::time::timeout(Duration::from_millis(100),  get_body(req)).await
    {
        Ok(response) => response,
        Err(_) => Err(Error::SendError("Connection timeout".to_owned()))
    };
    response
}
async fn get_body_retry(uri: Uri)  -> Result<Bytes, Error>
{
    retry::retry(5, 100, 400, || get_body_timeout(uri.clone())).await
    //tokio_retry::Retry::spawn(retry_strategy, || get_body_timeout(uri.clone())).await
}
async fn get_body(req: Request<BoxBody>) -> Result<Bytes, Error>
{
    let host = req.uri().authority().unwrap().as_str().replace("localhost", "127.0.0.1");
    logger::info!("Отправка запроса на {}, headers: {:?}", req.uri(), req.headers());
    let addr: SocketAddr = host.parse().unwrap();
    // let retry_strategy =  tokio_retry::strategy::ExponentialBackoff::from_millis(100)
    // .map(jitter) // add jitter to delays
    // .take(5);
   
    // let client_stream = tokio_retry::Retry::spawn(retry_strategy, || connect(addr)).await;
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
        Ok(body)
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
    .header(CONTENT_TYPE, "text/html; charset=utf-8")
    .body(to_body(Bytes::from(err))).unwrap()
}
pub fn error_empty_response(code: StatusCode) -> Response<BoxBody>
{
    Response::builder()
    .status(code)
    .body(to_body(Bytes::new())).unwrap()
}
pub fn ok_response(msg: String) -> Response<BoxBody>
{
    Response::builder()
    .status(StatusCode::OK)
    .header(CONTENT_TYPE, "text/html; charset=utf-8")
    .body(to_body(Bytes::from(msg))).unwrap()
}
pub fn json_response<S: Serialize>(obj: &S) -> Response<BoxBody>
{
    let result = serde_json::to_string(obj).unwrap();
    Response::builder()
    .status(StatusCode::OK)
    .header(CONTENT_TYPE, "application/json")
    .body(to_body(Bytes::from(result))).unwrap()
}

pub fn unauthorized_response() -> Response<BoxBody>
{
    Response::builder()
    .status(StatusCode::UNAUTHORIZED)
    .header(CONTENT_TYPE, "text/html; charset=utf-8")
    .body(to_body(Bytes::from_static(b"Unauthorized")))
    .unwrap()
}

#[derive(Debug, Clone)]
pub struct HyperClient
{
    uri: Uri,
    headers: hashbrown::HashMap<HeaderName, String>,
    timeout_from: u64,
    timeout_to: u64,
    retry_count: u8
}

impl HyperClient
{
    pub fn new(uri: Uri) -> Self
    {
        let _ = rustls::crypto::ring::default_provider().install_default();

        Self
        { 
            uri, 
            headers: hashbrown::HashMap::new(),
            timeout_from: 5000,
            timeout_to: 30000,
            retry_count: 7
        }
    }
    ///выберется рандомное время из данного рэнджа
    pub fn new_with_timeout(uri: Uri, from: u64, to: u64, retry_count: u8) -> Self
    {
        let _ = rustls::crypto::ring::default_provider().install_default();
        Self
        { 
            uri, 
            headers: hashbrown::HashMap::new(),
            timeout_from: from,
            timeout_to: to,
            retry_count
        }
    }
    pub fn get_uri(&self) -> &Uri
    {
        &self.uri
    }
    pub fn with_header<S: AsRef<str> + ToString>(mut self, name: HeaderName, value: S) -> Self
    {
        self.headers.insert(name, value.to_string());
        self
    }
    pub fn with_headers<S: AsRef<str> + ToString>(mut self, headers: Vec<(HeaderName, S)>) -> Self
    {
        self.headers = headers.into_iter().map(|m| (m.0, m.1.to_string())).collect::<HashMap<HeaderName, String>>();
        self
    }
    pub fn add_path(mut self, path: &str) -> Self
    {
        let mut uri = self.uri.to_string();
        if !uri.ends_with("/")
        {
            uri.push('/');
        }
        if path.starts_with("/")
        {
            self.uri = [&uri, &path[1..]].concat().parse().unwrap();
        }
        else
        {
            self.uri = [&uri, path].concat().parse().unwrap();
        }
        self
    }
    pub async fn get_with_params<S: AsRef<str> + ToString>(&self, params: &[(S, S)]) -> Result<(StatusCode, Bytes), Error>
    {
        self.get_body_retry(params, "GET", None::<bool>).await
    }
    pub async fn get(&self) -> Result<(StatusCode, Bytes), Error>
    {
        let params: Vec<(String, String)> = Vec::new();
        self.get_body_retry(&params, "GET", None::<bool>).await
    }
    pub async fn get_with_body<B: Serialize + Clone>(&self, body: B) -> Result<(StatusCode, Bytes), Error>
    {
        let v: Vec<(&str, &str)> = Vec::new();
        self.get_body_retry(&v, "GET", Some(body)).await
    }
    pub async fn post_with_params<S: AsRef<str> + ToString>(&self, params: &[(S, S)]) -> Result<(StatusCode, Bytes), Error>
    {
        self.get_body_retry(params, "POST", None::<bool>).await
    }
    pub async fn post_with_body<B: Serialize + Clone>(&self, body: B) -> Result<(StatusCode, Bytes), Error>
    {
        let v: Vec<(&str, &str)> = Vec::new();
        self.get_body_retry(&v, "POST", Some(body)).await
    }
    pub async fn patch_with_params<S: AsRef<str> + ToString>(&self, params: &[(S, S)]) -> Result<(StatusCode, Bytes), Error>
    {
        self.get_body_retry(params, "PATCH", None::<bool>).await
    }
    pub async fn patch_with_body<B: Serialize + Clone>(&self, body: B) -> Result<(StatusCode, Bytes), Error>
    {
        let v: Vec<(&str, &str)> = Vec::new();
        self.get_body_retry(&v, "PATCH", Some(body)).await
    }
    pub async fn put_with_params<S: AsRef<str> + ToString>(&self, params: &[(S, S)]) -> Result<(StatusCode, Bytes), Error>
    {
        self.get_body_retry(params, "PUT", None::<bool>).await
    }
    pub async fn put_with_body<B: Serialize + Clone>(&self, body: B) -> Result<(StatusCode, Bytes), Error>
    {
        let v: Vec<(&str, &str)> = Vec::new();
        self.get_body_retry(&v, "PUT", Some(body)).await
    }
    pub async fn delete<S: AsRef<str> + ToString>(&self, params: &[(S, S)]) -> Result<(StatusCode, Bytes), Error>
    {
        self.get_body_retry(params, "DELETE", None::<bool>).await
    }
    pub async fn delete_with_body<S: AsRef<str> + ToString, B: Serialize + Clone>(&self, body: B) -> Result<(StatusCode, Bytes), Error>
    {
        let v: Vec<(&str, &str)> = Vec::new();
        self.get_body_retry(&v, "DELETE", Some(body)).await
    }
    fn apply_params_to_uri<S: AsRef<str> + ToString>(&self, params: &[(S, S)]) -> Uri
    {
        let params_len = params.len();
        if params_len == 0
        {
            self.uri.clone()
        }
        else
        {
            let mut uri = self.uri.to_string();
            if uri.ends_with("/")
            {
                uri.remove(uri.len()-1);
            }
            uri.push('?');
            
            for (i, (k, v)) in params.into_iter().enumerate()
            {
                let key_value = [k.as_ref(), "=", encoding::encode(v.as_ref()).as_ref()].concat();
                uri.push_str(&key_value);
                if i < params_len -1
                {
                    uri.push('&');
                }
            }
            uri.parse().unwrap()
        }
    }

    async fn get_body_timeout<S: AsRef<str> + ToString>(&self, params: &[(S, S)], method: &str, body: Option<Bytes>)  -> Result<(StatusCode, Bytes), Error>
    {
        let mut req =  Request::builder()
        .method(method)
        .uri(self.apply_params_to_uri(params));
        let headers = req.headers_mut().unwrap();
        for (k, v) in &self.headers
        {
            headers.insert(k, v.parse().unwrap());
        }
        let guard = COOK.lock().await;
        if let Some(c) = guard.get(self.get_uri().host().unwrap())
        {
            headers.insert(COOKIE, c.parse().unwrap());
        }
        drop(guard);
        let body = if let Some(b) = body
        {
            to_body(b.clone().into())
        }
        else
        {
            to_body(Bytes::new())
        };
        let req: Request<http_body_util::combinators::BoxBody<Bytes, hyper::Error>> = req
        .body(body)
        .unwrap();
        let response = match tokio::time::timeout(Self::rnd_duration(self.timeout_from, self.timeout_to),  Self::get_body_tls(req)).await
        {
            Ok(response) => response,
            Err(_) => Err(Error::SendError("Connection timeout".to_owned()))
        };
        response
    }

    fn rnd_duration(timeout_from: u64, timeout_to: u64) -> Duration
    {
        Duration::from_millis(rand::thread_rng().gen_range(timeout_from..timeout_to))
    }
    async fn get_body_retry<S: AsRef<str> + ToString, B: Serialize + Clone>(&self, params: &[(S, S)], method: &str, body: Option<B>) -> Result<(StatusCode, Bytes), Error>
    {
        let body: Option<Bytes> = body.and_then(|b| Some(Bytes::from(serde_json::to_string(&b).unwrap())));
        retry::retry(self.retry_count, self.timeout_from, self.timeout_to, || self.get_body_timeout(params, method, body.clone())).await
    }
    // async fn get_body(req: Request<BoxBody>) -> Result<(StatusCode, Bytes), Error>
    // {
    //     let host = req.uri().authority().unwrap().as_str().replace("localhost", "127.0.0.1");
    //     logger::debug!("Отправка запроса на {}, headers: {:?}", req.uri(), req.headers());
    //     //let addr: SocketAddr = host.parse().unwrap();
    //     let https = req.uri().scheme().and_then(|s| Some(s.as_str() == "https"));
    //     let addr: SocketAddr = if https.is_some() && *https.as_ref().unwrap() == true
    //     {
    //         SocketAddr::new(IpAddr::V4(host.parse::<Ipv4Addr>().unwrap()), 443)
    //     }
    //     else
    //     {
    //         host.parse().unwrap()
    //     };
    //     let client_stream = TcpStream::connect(&addr).await;
    //     if client_stream.is_err()
    //     {
    //         logger::error!("Ошибка подключения к сервису {} -> {}", &addr, client_stream.err().unwrap());
    //         return Err(Error::SendError(addr.to_string()));
    //     }
    //     let io = TokioIo::new(client_stream.unwrap());
   
    //     let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
    //     tokio::task::spawn(async move 
    //         {
    //             if let Err(err) = conn.await 
    //             {
    //                 logger::error!("Ошибка подключения: {:?}", err);
    //             }
    //         });
    //     let send = sender.send_request(req).await?;
    //     let status = send.status();
    //     let body = send.collect().await?.to_bytes();
    //     logger::debug!("От {} получен ответ со статусом -> {}", &addr, &status);
    //     Ok((status, body))
    // }

    async fn get_body_tls(req: Request<BoxBody>) -> Result<(StatusCode, Bytes), Error>
    {
        //let host = req.uri().authority().unwrap().as_str().replace("localhost", "127.0.0.1");
        let tls = rustls::ClientConfig::builder()
        .with_native_roots()?
        .with_no_client_auth();
        let https = hyper_rustls::HttpsConnectorBuilder::new()
        .with_tls_config(tls)
        .https_or_http()
        .enable_http1()
        .build();
        let client: Client<_, Empty<Bytes>> = Client::builder(TokioExecutor::new()).build(https);
        let fut = async move 
        {
            let mut req = req;
            logger::debug!("Отправка запроса на {}, headers: {:?}", req.uri(), req.headers());
            let mut response = client
            .get(req.uri().clone())
            .await?;
            if let Some(c) = response.headers().get("set-cookie")
            {
                let mut guard = COOK.lock().await;
                //были установлены для данного хоста но обновились
                if let Some(old) = guard.insert(req.uri().host().unwrap().to_owned(), c.to_str().unwrap().to_owned())
                {
                    //если отличаются от предыдущих, обновляем и делаем повторный запрос
                    if c.to_str().unwrap() != &old
                    {
                        req.headers_mut().insert(COOKIE, c.clone());
                        logger::debug!("Поменялись куки, деаем поторный запрос на {}, headers: {:?}", req.uri(), req.headers());
                    }
                  
                }
                //установлены впервые
                else 
                {
                    req.headers_mut().insert(COOKIE, c.clone());
                    logger::debug!("Установлены новые куки, делаем поторный запрос на {}, headers: {:?}", req.uri(), req.headers());
                    
                }
                response = client
                    .get(req.uri().clone())
                    .await?;
            }
            //перенаправление запроса при статусе 301
            if let Some(c) = response.headers().get("location")
            {
                logger::debug!("От сервера получен ответ с перенаправлением запроса на {}", c.to_str().unwrap());
                if let Ok(new_uri) = c.to_str().unwrap().parse::<Uri>()
                {
                    *req.uri_mut() = new_uri;
                    response = client
                    .get(req.uri().clone())
                    .await?;
                }
            }
            logger::debug!("От сервера получен ответ со статусом {}, headers: {:?}", response.status(), response.headers());
            let status = response.status();
            let body = response
                .into_body()
                .collect()
                .await?
                .to_bytes();
            Ok((status, body))
        };
    
        fut.await
    }
}

static COOK: std::sync::LazyLock<Arc<Mutex<HashMap<String, String>>>> = std::sync::LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));


#[cfg(test)]
mod tests
{
    use hyper::{body::Bytes, header::{HeaderName, HeaderValue, ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, CONTENT_TYPE, HOST, ORIGIN, REFERER, UPGRADE_INSECURE_REQUESTS, USER_AGENT}, HeaderMap, Request, Uri};

    use super::{to_body, BoxBody};

    // #[tokio::test]
    // async fn test_cli()
    // {
    //     logger::StructLogger::initialize_logger();
    //     let cli = super::HttpClient::new("http://95.173.157.133:8000/api/ebpi/redactions");
    //     for i in 0..10
    //     {
    //         let result = cli.get_with_params(Some(&[("t", "%7B%22hash%22%3A%2266c4df9cc6da662d2ee557c6f2e21cf2f84c3ba3d8bcdfeb43d1925ef3149b24%22%2C%22ttl%22%3A3%7D")])).await;
    //         if result.is_err()
    //         {
    //             logger::error!("{:?}", result);
    //         }
    //         else 
    //         {
    //             logger::info!("{:?}", result.as_ref().unwrap());
    //         }
    //     }
        
    // }
    #[tokio::test]
    async fn test_hyper_cli()
    {
        let _ = logger::StructLogger::new_default();
        let uri: Uri = "http://95.173.157.133:8000/api/ebpi/redactions?t=%7B%22hash%22%3A%2266c4df9cc6da662d2ee557c6f2e21cf2f84c3ba3d8bcdfeb43d1925ef3149b24%22%2C%22ttl%22%3A3%7D".parse().unwrap();
      
        for i in 0..10
        {
            //let r = empty_get_request(uri.clone());
            let result = super::get_body_retry(uri.clone()).await;
            if result.is_err()
            {
                logger::error!("{}->{:?}", i, result);
            }
            else {
                logger::info!("{:?}", result.as_ref().unwrap());
            }
        }
        
    }

    #[tokio::test]
    async fn test_hyper_cli_new()
    {
        let _ = logger::StructLogger::new_default();
        let uri: Uri = "http://95.173.157.133:8000/api/ebpi/redactions/".parse().unwrap();
        let hyper_client = super::HyperClient::new(uri)
        .with_headers(headers());
        for i in 0..10
        {
            //let r = empty_get_request(uri.clone());
            let result = hyper_client.get_with_params(&[("t", "%7B%22hash%22%3A%2266c4df9cc6da662d2ee557c6f2e21cf2f84c3ba3d8bcdfeb43d1925ef3149b24%22%2C%22ttl%22%3A3%7D")]).await;
            if result.is_err()
            {
                logger::error!("{}->{:?}", i, result);
            }
            else {
                logger::info!("{:?}", result.as_ref().unwrap());
            }
        }
    }

    #[tokio::test]
    async fn test_hyper_cli_tls()
    {
        let _ = logger::StructLogger::new_default();
        let uri: Uri = "https://fake-json-api.mock.beeceptor.com/companies".parse().unwrap();
        let hyper_client = super::HyperClient::new(uri)
        .with_headers(headers2());
        for i in 0..10
        {
            //let r = empty_get_request(uri.clone());
            let result = hyper_client.get().await;
            if result.is_err()
            {
                logger::error!("{}->{:?}", i, result);
            }
            else {
                logger::info!("{:?}", result.as_ref().unwrap());
            }
        }
    }
    #[tokio::test]
    async fn test_hyper_cli_tls_retirn301()
    {
        let _ = logger::StructLogger::new_default();
                      //https://egov-buryatia.ru/npa_template?date_doc_from=2024-01-01&date_doc_to=2024-12-31&TIP_DOC=%D0%97%D0%B0%D0%BA%D0%BE%D0%BD&ORGAN_VLASTI=%D0%9D%D0%B0%D1%80%D0%BE%D0%B4%D0%BD%D1%8B%D0%B9+%D0%A5%D1%83%D1%80%D0%B0%D0%BB&PAGEN_1=1
        let uri: Uri = "https://egov-buryatia.ru/npa_template?date_doc_from=2024-01-01&date_doc_to=2024-12-31&TIP_DOC=%D0%97%D0%B0%D0%BA%D0%BE%D0%BD&ORGAN_VLASTI=%D0%9D%D0%B0%D1%80%D0%BE%D0%B4%D0%BD%D1%8B%D0%B9+%D0%A5%D1%83%D1%80%D0%B0%D0%BB&PAGEN_1=1".parse().unwrap();
        let client = super::HyperClient::new(uri).with_headers(vec![
            (ACCEPT_ENCODING, "gzip, deflate, br, zstd"),
            (ACCEPT, "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"),
            (HOST, "egov-buryatia.ru"),
            (USER_AGENT, "Mozilla/5.0 (X11; Linux x86_64; rv:134.0) Gecko/20100101 Firefox/134.0"),
            (HeaderName::from_static("sec-fetch-dest"), "document"),
            (HeaderName::from_static("sec-fetch-mode"), "navigate"),
            (UPGRADE_INSECURE_REQUESTS, "1"),
            (ACCEPT_LANGUAGE, "ru-RU,ru;q=0.8,en-US;q=0.5,en;q=0.3"),
            (CONTENT_TYPE, "application/octet-stream"),
            (ORIGIN, "https://egov-buryatia.ru"),
            (HeaderName::from_static("x-requested-with"), "XMLHttpRequest")
        ]);
        let res = client.get().await;
        logger::info!("{:?}", res);
    }
    fn headers2() -> Vec<(HeaderName, String)>
    {
        let mut h= Vec::new();
        h.push((HOST, "fake-json-api.mock.beeceptor.com".to_owned()));
        h.push((USER_AGENT, "Mozilla/5.0 (X11; Linux x86_64; rv:127.0) Gecko/20100101 Firefox/127.0".to_owned()));
        h.push((ACCEPT, "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8".to_owned()));
        h.push((ACCEPT_ENCODING, "gzip, deflate".to_owned()));
        h.push((ACCEPT_LANGUAGE, "ru-RU,ru;q=0.8,en-US;q=0.5,en;q=0.3".to_owned()));
        //h.push((REFERER, "http:://pravo.gov.ru".to_owned()));
        //h.push((UPGRADE_INSECURE_REQUESTS, "1".to_owned()));
        h

    }
    fn headers() -> Vec<(HeaderName, String)>
    {
        let mut h= Vec::new();
        h.push((HOST, "pravo.gov.ru".to_owned()));
        h.push((USER_AGENT, "Mozilla/5.0 (X11; Linux x86_64; rv:127.0) Gecko/20100101 Firefox/127.0".to_owned()));
        h.push((ACCEPT, "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8".to_owned()));
        h.push((ACCEPT_ENCODING, "gzip, deflate".to_owned()));
        h.push((ACCEPT_LANGUAGE, "ru-RU,ru;q=0.8,en-US;q=0.5,en;q=0.3".to_owned()));
        h.push((REFERER, "http:://pravo.gov.ru".to_owned()));
        h.push((UPGRADE_INSECURE_REQUESTS, "1".to_owned()));
        h

    }

    fn empty_get_request(uri: Uri) -> Request<BoxBody>
    {
        let host = [uri.host().unwrap(), uri.port().unwrap().as_str()].concat();
        Request::builder()
        .method("GET")
        .uri(&uri)
        .header(HOST, "pravo.gov.ru")
        .header(USER_AGENT, "Mozilla/5.0 (X11; Linux x86_64; rv:127.0) Gecko/20100101 Firefox/127.0")
        .header(ACCEPT, "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
        .header(ACCEPT_ENCODING, "gzip, deflate")
        .header(ACCEPT_LANGUAGE, "ru-RU,ru;q=0.8,en-US;q=0.5,en;q=0.3")
        //.header(CONNECTION, "keep-alive")
        .header(REFERER, ["http:://pravo.gov.ru", uri.path()].concat())
        .header(UPGRADE_INSECURE_REQUESTS, 1)
        .header("Priority", "u=1")
        .body(to_body(Bytes::new()))
        .unwrap()
    }
}


mod encoding
{
    use std::borrow::Cow;
    use std::fmt;
    use std::io;
    use std::str;

    /// Wrapper type that implements `Display`. Encodes on the fly, without allocating.
    /// Percent-encodes every byte except alphanumerics and `-`, `_`, `.`, `~`. Assumes UTF-8 encoding.
    ///
    /// ```rust
    /// use urlencoding::Encoded;
    /// format!("{}", Encoded("hello!"));
    /// ```
    #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
    #[repr(transparent)]
    pub struct Encoded<Str>(pub Str);

    impl<Str: AsRef<[u8]>> Encoded<Str> {
        /// Long way of writing `Encoded(data)`
        ///
        /// Takes any string-like type or a slice of bytes, either owned or borrowed.
        #[inline(always)]
        pub fn new(string: Str) -> Self {
            Self(string)
        }

        #[inline(always)]
        pub fn to_str(&self) -> Cow<str> {
            encode_binary(self.0.as_ref())
        }

        /// Perform urlencoding to a string
        #[inline]
        #[allow(clippy::inherent_to_string_shadow_display)]
        pub fn to_string(&self) -> String {
            self.to_str().into_owned()
        }

        /// Perform urlencoding into a writer
        #[inline]
        pub fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
            encode_into(self.0.as_ref(), false, |s| writer.write_all(s.as_bytes()))?;
            Ok(())
        }

        /// Perform urlencoding into a string
        #[inline]
        pub fn append_to(&self, string: &mut String) {
            append_string(self.0.as_ref(), string, false);
        }
    }

    impl<'a> Encoded<&'a str> {
        /// Same as new, but hints a more specific type, so you can avoid errors about `AsRef<[u8]>` not implemented
        /// on references-to-references.
        #[inline(always)]
        pub fn str(string: &'a str) -> Self {
            Self(string)
        }
    }

    impl<String: AsRef<[u8]>> fmt::Display for Encoded<String> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            encode_into(self.0.as_ref(), false, |s| f.write_str(s))?;
            Ok(())
        }
    }

    /// Percent-encodes every byte except alphanumerics and `-`, `_`, `.`, `~`. Assumes UTF-8 encoding.
    ///
    /// Call `.into_owned()` if you need a `String`
    #[inline(always)]
    pub fn encode(data: &str) -> Cow<str> {
        encode_binary(data.as_bytes())
    }

    /// Percent-encodes every byte except alphanumerics and `-`, `_`, `.`, `~`.
    #[inline]
    pub fn encode_binary(data: &[u8]) -> Cow<str> {
        // add maybe extra capacity, but try not to exceed allocator's bucket size
        let mut escaped = String::with_capacity(data.len() | 15);
        let unmodified = append_string(data, &mut escaped, true);
        if unmodified {
            return Cow::Borrowed(unsafe {
                // encode_into has checked it's ASCII
                str::from_utf8_unchecked(data)
            });
        }
        Cow::Owned(escaped)
    }

    fn append_string(data: &[u8], escaped: &mut String, may_skip: bool) -> bool {
        encode_into(data, may_skip, |s| {
            escaped.push_str(s);
            Ok::<_, std::convert::Infallible>(())
        }).unwrap()
    }

    fn encode_into<E>(mut data: &[u8], may_skip_write: bool, mut push_str: impl FnMut(&str) -> Result<(), E>) -> Result<bool, E> {
        let mut pushed = false;
        loop {
            // Fast path to skip over safe chars at the beginning of the remaining string
            let ascii_len = data.iter()
                .take_while(|&&c| matches!(c, b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' |  b'-' | b'.' | b'_' | b'~')).count();

            let (safe, rest) = if ascii_len >= data.len() {
                if !pushed && may_skip_write {
                    return Ok(true);
                }
                (data, &[][..]) // redundatnt to optimize out a panic in split_at
            } else {
                data.split_at(ascii_len)
            };
            pushed = true;
            if !safe.is_empty() {
                push_str(unsafe { str::from_utf8_unchecked(safe) })?;
            }
            if rest.is_empty() {
                break;
            }

            match rest.split_first() {
                Some((byte, rest)) => {
                    let enc = &[b'%', to_hex_digit(byte >> 4), to_hex_digit(byte & 15)];
                    push_str(unsafe { str::from_utf8_unchecked(enc) })?;
                    data = rest;
                }
                None => break,
            };
        }
        Ok(false)
    }

    #[inline]
    fn to_hex_digit(digit: u8) -> u8 {
        match digit {
            0..=9 => b'0' + digit,
            10..=255 => b'A' - 10 + digit,
        }
    }

}

mod decoding
{
    use std::borrow::Cow;
    use std::string::FromUtf8Error;

    #[inline]
    pub(crate) fn from_hex_digit(digit: u8) -> Option<u8> {
        match digit {
            b'0'..=b'9' => Some(digit - b'0'),
            b'A'..=b'F' => Some(digit - b'A' + 10),
            b'a'..=b'f' => Some(digit - b'a' + 10),
            _ => None,
        }
    }

    /// Decode percent-encoded string assuming UTF-8 encoding.
    ///
    /// If you need a `String`, call `.into_owned()` (not `.to_owned()`).
    ///
    /// Unencoded `+` is preserved literally, and _not_ changed to a space.
    pub fn decode(data: &str) -> Result<Cow<str>, FromUtf8Error> {
        match decode_binary(data.as_bytes()) {
            Cow::Borrowed(_) => Ok(Cow::Borrowed(data)),
            Cow::Owned(s) => Ok(Cow::Owned(String::from_utf8(s)?)),
        }
    }

    /// Decode percent-encoded string as binary data, in any encoding.
    ///
    /// Unencoded `+` is preserved literally, and _not_ changed to a space.
    pub fn decode_binary(data: &[u8]) -> Cow<[u8]> {
        let offset = data.iter().take_while(|&&c| c != b'%').count();
        if offset >= data.len() {
            return Cow::Borrowed(data)
        }

        let mut decoded: Vec<u8> = Vec::with_capacity(data.len());
        let mut out = NeverRealloc(&mut decoded);

        let (ascii, mut data) = data.split_at(offset);
        out.extend_from_slice(ascii);

        loop {
            let mut parts = data.splitn(2, |&c| c == b'%');
            // first the decoded non-% part
            let non_escaped_part = parts.next().unwrap();
            let rest = parts.next();
            if rest.is_none() && out.0.is_empty() {
                // if empty there were no '%' in the string
                return data.into();
            }
            out.extend_from_slice(non_escaped_part);

            // then decode one %xx
            match rest {
                Some(rest) => match rest.get(0..2) {
                    Some(&[first, second]) => match from_hex_digit(first) {
                        Some(first_val) => match from_hex_digit(second) {
                            Some(second_val) => {
                                out.push((first_val << 4) | second_val);
                                data = &rest[2..];
                            },
                            None => {
                                out.extend_from_slice(&[b'%', first]);
                                data = &rest[1..];
                            },
                        },
                        None => {
                            out.push(b'%');
                            data = rest;
                        },
                    },
                    _ => {
                        // too short
                        out.push(b'%');
                        out.extend_from_slice(rest);
                        break;
                    },
                },
                None => break,
            }
        }
        Cow::Owned(decoded)
    }


    struct NeverRealloc<'a, T>(pub &'a mut Vec<T>);

    impl<T> NeverRealloc<'_, T> 
    {
        #[inline]
        pub fn push(&mut self, val: T) 
        {
            // these branches only exist to remove redundant reallocation code
            // (the capacity is always sufficient)
            if self.0.len() != self.0.capacity() 
            {
                self.0.push(val);
            }
        }
        #[inline]
        pub fn extend_from_slice(&mut self, val: &[T]) where T: Clone 
        {
            if self.0.capacity() - self.0.len() >= val.len() 
            {
                self.0.extend_from_slice(val);
            } 
        }
    }
}