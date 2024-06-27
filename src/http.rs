use std::{any::TypeId, net::SocketAddr, time::Duration};
use hashbrown::HashMap;
pub use http_body_util::{BodyExt, Full};
pub use hyper::{body::Bytes, header::{self, ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, CONNECTION, CONTENT_TYPE, HOST, USER_AGENT}, Request, Response, StatusCode, Uri};
pub use hyper_util::rt::TokioIo;
use serde::{Deserialize, Serialize};
use tokio::net::TcpSocket;
pub use tokio::net::TcpStream;
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
pub async fn get_body(req: Request<BoxBody>) -> Result<Bytes, Error>
{
    let host = req.uri().authority().unwrap().as_str().replace("localhost", "127.0.0.1");
    logger::info!("Отправка запроса на {}, headers: {:?}", req.uri(), req.headers());
    let addr: SocketAddr = host.parse().unwrap();
    let client_stream = match tokio::time::timeout(
        Duration::from_secs(20),
        tokio::net::TcpStream::connect(&addr)
    )
    .await
    {
        Ok(ok) => ok.map_err(|e| Error::SendError(format!("Ошибка соединения с сервером : {}", e))),
        Err(e) => Err(Error::SendError(format!("таймаут соединения с сервером : {}", e))),
    }?;
    //let client_stream = TcpStream::connect(&addr).await;
    // if client_stream.is_err()
    // {
    //     logger::error!("Ошибка подключения к сервису {} -> {}", &addr, client_stream.err().unwrap());
    //     return Err(Error::SendError(addr.to_string()));
    // }
    let io = TokioIo::new(client_stream);
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
    .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
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
    .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
    .body(to_body(Bytes::from(msg))).unwrap()
}
pub fn json_response<S: Serialize>(obj: &S) -> Response<BoxBody>
{
    let result = serde_json::to_string(obj).unwrap();
    Response::builder()
    .status(StatusCode::OK)
    .header(header::CONTENT_TYPE, "application/json")
    .body(to_body(Bytes::from(result))).unwrap()
}

pub fn unauthorized_response() -> Response<BoxBody>
{
    Response::builder()
    .status(StatusCode::UNAUTHORIZED)
    .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
    .body(to_body(Bytes::from_static(b"Unauthorized")))
    .unwrap()
}

/// Запрос формируем сами, так быстрее будет, в него же включаем передаваемое значение если нужно
pub async fn request_with_retry(req: Request<BoxBody>) -> Result<Bytes, Error>
{
    let host = if let Some(h) = req.uri().authority()
    {
       
        // if let Some(p) = req.uri().port()
        // {
        //     [h.as_str(), p.as_str()].concat()
        // }
        // else
        // {
            h.as_str().to_owned()
        // }
    }
    else
    {
        return Err(Error::SendError(format!("В запросе {} не найден адрес сервера", req.uri().to_string())));
    };
    logger::info!("Отправка запроса на {}, headers: {:?}", req.uri(),  req.headers());
    logger::info!("tcp address: {}", &host);
    let client_stream = TcpStream::connect(&host).await;
    //let client_stream = TcpStream::connect("95.173.157.133:8000").await;
    if client_stream.is_err()
    {
        logger::error!("Ошибка подключения к сервису {} -> {}", host.clone(), client_stream.err().unwrap());
        return Err(Error::SendError(host.clone()));
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
    let send = sender.send_request(req);
    match tokio::time::timeout(Duration::from_secs(2), send).await 
    {
        Ok(result) => match result 
        {
            Ok(r) => 
            if r.status() == StatusCode::OK
            {
                let body = r.collect().await?.to_bytes();
                //if TypeId::of::<R>() == TypeId::of::<Bytes>
                //let response: R = serde_json::from_slice(&body)?;
                return Ok(body);
            }
            else
            {
                logger::error!("Ошибка получения инфомации от сервиса {} -> {}", &host, r.status());
                return Err(Error::SendError(format!("Ошибка получения инфомации от сервиса {} -> {}", &host, r.status())));
            },
            Err(e) => return Err(Error::HyperError(e))
        },
        Err(_) =>
        {
            let e = format!("Нет ответа от сервера {} > 2 секунд", &host);
            logger::warn!("{}", &e);
            return Err(Error::SendError(e));
        }
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

    impl<T> NeverRealloc<'_, T> {
        #[inline]
        pub fn push(&mut self, val: T) {
            // these branches only exist to remove redundant reallocation code
            // (the capacity is always sufficient)
            if self.0.len() != self.0.capacity() {
                self.0.push(val);
            }
        }
        #[inline]
        pub fn extend_from_slice(&mut self, val: &[T]) where T: Clone {
            if self.0.capacity() - self.0.len() >= val.len() {
                self.0.extend_from_slice(val);
            }
        }
    }
}