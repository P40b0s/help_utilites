use std::future::Future;
use std::thread::sleep;
use std::time::Duration;
use logger::warn;
use rand::{thread_rng, Rng};
use rand::seq::SliceRandom;

///Повторное выполнение функции  
/// __attempts__ количество повторов  (0 бесконечный повтор)  
/// __delay__ задержка между повторами в миллисекундах  
pub async fn retry<F, Fu, V, E>(mut attempts: u8, delay_from: u64, delay_to: u64, f: F) -> Result<V, E>
where F: Fn() -> Fu,
      Fu: Future<Output=Result<V, E>> 
{
    loop 
    {
        match f().await 
        {
            Ok(v) => return Ok(v),
            Err(e) if attempts == 1 => return Err(e),
            _ => 
            {
                if attempts != 0
                {
                    attempts -= 1;
                    warn!("Повторная попытка выполнения retry осталось {} попыток", attempts);
                }
                else 
                {
                    warn!("Повторная попытка выполнения retry осталось ∞ попыток");
                }
                let delay = rand::rng().random_range(delay_from..delay_to);
                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
            }
        };
    }
}

///retry operation with `attempts` and random range `delay_from` in ms - `delay_to` in ms
pub fn retry_sync<F, V, E>(mut attempts: u8, delay_from: u64, delay_to: u64, f: F) -> Result<V, E>
where F: Fn() -> Result<V, E>
{
    loop 
    {
        match f()
        {
            Ok(v) => return Ok(v),
            Err(e) if attempts == 1 => return Err(e),
            _ => 
            {
                if attempts != 0
                {
                    attempts -= 1;
                    warn!("Повторная попытка выполнения retry осталось {} попыток", attempts);
                }
                else 
                {
                    warn!("Повторная попытка выполнения retry осталось ∞ попыток");
                }
                let delay = rand::rng().random_range(delay_from..delay_to);
                sleep(Duration::from_millis(delay));
            }
        };
    }
}