const KX: u32 = 965120573;
const KY: u32 = 486042514;
const KZ: u32 = 563820594;
const KW: u32 = 75647390;

pub struct SimpleRand 
{
    x: u32, y: u32, z: u32, w: u32
}

impl SimpleRand
{
    pub fn new(seed: u32) -> SimpleRand 
    {
        SimpleRand
        {
            x: KX^seed, y: KY^seed,
            z: KZ, w: KW
        }
    }

    // Xorshift 128
    pub fn rand(&mut self) -> u32 
    {
        let t = self.x^self.x.wrapping_shl(11);
        self.x = self.y;
        self.y = self.z;
        self.z = self.w;
        self.w ^= self.w.wrapping_shr(19)^t^t.wrapping_shr(8);
        return self.w;
    }
    ///shuffle array items
    pub fn shuffle<T>(&mut self, a: &mut [T]) 
    {
        if a.len() == 0 {return;}
        let mut i = a.len()-1;
        while i > 0 
        {
            let j = (self.rand() as usize) % (i+1);
            a.swap(i,j);
            i -=1 ;
        }
    }

    pub fn rand_range(&mut self, a: i32, b: i32) -> i32 
    {
        let m = (b-a+1) as u32;
        return a + (self.rand() % m) as i32;
    }

    pub fn rand_float(&mut self) -> f64 
    {
        (self.rand() as f64)/(<u32>::max_value() as f64)
    }
}

#[cfg(test)]
mod tests
{
    use crate::utils::SimpleRand;
    #[test]
    fn test_rand()
    {
        let mut rng = SimpleRand::new(6);
        let v: Vec<i32> = (0..100).map(|_| rng.rand_range(1000,9999)).collect();
        println!("{:?}",v);
    }
}
