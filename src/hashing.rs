use std::path::Path;
use blake2::{Blake2b512, Digest};
use base64ct::{Base64, Encoding};


///емкость буфера для  blake2b512 (to base64) = 88
//const BUF_SIZE: usize = 88;
pub struct Hasher{}
impl Hasher
{
    pub fn hash_from_slice<S: AsRef<[u8]>>(data: S) -> String
    {
        Self::hashing(data)
    }
    
    pub fn hash_from_path<P: AsRef<Path>>(path: P) -> Option<String>
    {
        crate::io::read_file_to_binary(&path).and_then(|f|
        {
            let hash = Self::hashing(&f);
            Ok(hash)
        }).ok()
    }
    
    ///Создание хэша base64 из массива строк
    pub fn hash_from_strings<I: IntoIterator<Item = S>, S: AsRef<str>>(args: I) -> String
    {
        let normalize_string = normalize(args);
        let args_bytes = normalize_string.as_bytes();
        Self::hashing(args_bytes)
    }
    pub fn hash_from_string<S: AsRef<str>>(val: S) -> String
    {
        let normalize_string = normalize(&[val]);
        let args_bytes = normalize_string.as_bytes();
        Self::hashing(args_bytes)
    }
    ///создание хеша из массива байтов
    pub fn from_bytes_to_base64<S: AsRef<[u8]>>(v : S) -> String
    {
        Base64::encode_string(v.as_ref())
    }
    fn hashing<S: AsRef<[u8]>>(data: S) -> String
    {
        let mut hasher = Blake2b512::new();
        hasher.update(data);
        let hash = hasher.finalize();
        let hash_vec: &[u8] = hash.as_ref();
        let hash_string = Self::from_bytes_to_base64(hash_vec);
        hash_string
        // //if let Ok(hash_string) = std::str::from_utf8(&hash_vec)
        // unsafe {
            
        //         let str = std::str::from_utf8_unchecked(&hash_vec);
        //         info!("Успешно сгенерирован хэш {}", &str);
        //         Some(str.to_owned())
           
            // if let Ok(hash_string) = std::str::from_utf8_unchecked(&hash_vec)
            // {
            //     info!("Успешно сгенерирован хэш {}", &hash_string);
            //     Some(hash_string.to_owned())
            // }
            // else
            // {
            //     error!("Ошибка входных параметров вектора от алгоритма blake2 {}", std::str::from_utf8(&hash_vec).err().unwrap().to_string());
            //     None
            // }
        // }
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
        let tt = super::Hasher::hash_from_strings(s);
        debug!("{}", &tt); 
        assert_eq!(tt, "OxDqYVEd8T//XVBGN3sD2lZ6mVMD3XLcDsKFNnOnD2m2WRu1vHpeFa4nSLsXHQQ6W1YrELQ/9xzF+kYmpY3xAw==".to_owned());
    }


}