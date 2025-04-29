use std::path::Path;
use blake3::{Hash, Hasher as B3Hasher};
use base64ct::{Base64, Encoding};
use crate::error::Error;


pub struct Hasher
{
    hash: Hash
}
impl Hasher
{
    pub fn from_slice<S: AsRef<[u8]>>(data: S) -> Self
    {
        Self::hashing(data)
    }

    #[cfg(feature="async-io")]
    pub async fn from_path_async<P: AsRef<Path>>(path: P) -> Result<Self, Error>
    {
        let file = crate::io::read_file_to_binary_async(&path).await?;
        let hash = Self::hashing(&file);
        Ok(hash)
    }
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error>
    {
        let file = crate::io::read_file_to_binary(&path)?;
        let hash = Self::hashing(&file);
        Ok(hash)
    }
    
    ///replace whitespaces, to_lower, blake3b to base64 string
    pub fn from_strings<I: IntoIterator<Item = S>, S: AsRef<str>>(args: I) -> Self
    {
        let normalize_string = normalize(args);
        let args_bytes = normalize_string.as_bytes();
        Self::hashing(args_bytes)
    }
    ///replace whitespaces, to_lower, blake3b to base64 string
    pub fn from_string<S: AsRef<str>>(val: S) -> Self
    {
        let normalize_string = normalize(&[val]);
        let args_bytes = normalize_string.as_bytes();
        Self::hashing(args_bytes)
    }
    ///vec<u8> to base64 string
    pub fn from_bytes_to_base64<S: AsRef<[u8]>>(v : S) -> String
    {
        Base64::encode_string(v.as_ref())
    }
    ///base64 string to vec<u8>
    pub fn from_base64_to_bytes<S: AsRef<str>>(v : S) -> Result<Vec<u8>, Error>
    {
        Base64::decode_vec(v.as_ref()).map_err(|e| Error::Base64Error(e))
    }
    pub fn as_base64(&self) -> String
    {
        Base64::encode_string(self.hash.as_bytes())
    }
    pub fn as_hex(&self) -> String
    {
        self.hash.to_hex().to_string()
    }
    pub fn as_bytes(&self) -> &[u8]
    {
        self.hash.as_bytes()
    }
    fn hashing<S: AsRef<[u8]>>(data: S) -> Self
    {
        let mut hasher = B3Hasher::new();
        hasher.update(data.as_ref());
        let hash = hasher.finalize();
        Self
        {
            hash
        }
    }
}


fn normalize<'a, I: IntoIterator<Item = S>, S: AsRef<str>>(args: I) -> String
{
    args.into_iter()
        .map(|m| m.as_ref().replace(" ", "").to_lowercase())
        .collect::<String>()
}




#[cfg(test)]
mod test
{
    use logger::debug;
    #[test]
    pub fn date_output() 
    {
        let _ = logger::StructLogger::new_default();
        let s = &["1 ываываыва ыаваыва ыва ыва23", "45ыва ыва ыва ываываыва6", "78ацуацуаца ывацуац уацуац вацуа цуацуа цуа 9"];
        let b64 = super::Hasher::from_strings(s).as_base64();
        let hex = super::Hasher::from_strings(s).as_hex();
        debug!("b64:{} hex:{}", &b64, &hex); 
        assert_eq!(b64, "2sgXOJ7sqyqKkIQNCEEuXr98lIAX+k4ixzfK1srAcCc=".to_owned());
        assert_eq!(hex, "dac817389eecab2a8a90840d08412e5ebf7c948017fa4e22c737cad6cac07027".to_owned());
    }


}