use std::path::{Display, Path};
use blake2::{Blake2b512, Digest};

pub struct Hasher{}
impl Hasher
{
    pub fn hash_from_slice<S: AsRef<[u8]>>(data: S) -> String
    {
        Self::hashing(data)
    }
    
    pub fn hash_from_path<P: AsRef<Path> + std::fmt::Display>(path: P) -> Option<String>
    {
        crate::io::read_file_to_binary(&path).and_then(|f|
        {
            let hash = Self::hashing(&f);
            logger::debug!("Создан хэш {} для файла {}", &hash, &path.to_string());
            Ok(hash)
        }).ok()
    }
    
    ///Создание хэша base64 из массива строк
    pub fn hash_from_string(args: &[&str]) -> String
    {
        let normalize_string = normalize(args);
        let args_bytes = normalize_string.as_bytes();
        Self::hashing(args_bytes)
    }
    pub fn from_bytes_to_base64<S: AsRef<[u8]>>(v : S) -> String
    {
        let str =  base64::display::Base64Display::with_config(v.as_ref(), base64::STANDARD);
        str.to_string()
    }
    
    fn hashing<S: AsRef<[u8]>>(data: S) -> String
    {
        let mut hasher = Blake2b512::new();
        hasher.update(data);
        let hash = hasher.finalize();
        let hash_vec: &[u8] = &hash[..];
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




fn normalize(args: &[&str]) -> String
{
    let mut for_encode : String = String::new();
    for o in args
    {
        let normalize = o.replace(" ", "").to_lowercase();
        for_encode.push_str(&normalize)
    }
    for_encode
}




#[cfg(test)]
mod test
{
    use logger::debug;
    use serde::{Deserialize, Serialize};

    #[test]
    pub fn date_output() 
    {
        logger::StructLogger::initialize_logger();
        debug!("{} ", "");  
    }


}