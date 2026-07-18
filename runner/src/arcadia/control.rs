use std::io::{Result, Read, Write};
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};
use rand_distr::{Normal, Distribution};
use crate::arcadia::actors::Body;

pub struct Control
{
 threshold: u32,
 checker: Normal<f32>
}

impl Default for Control
{
 fn default() -> Self
 {
  Control { threshold: 0, checker: Normal::new(0.0, 1.0).unwrap() }
 }
}

impl Control
{
 pub fn tick(&mut self, body: &mut Body)
 {
  let mut rng = rand::thread_rng();
  if self.checker.sample(&mut rng) < body.get_credits() as f32
    { println!("Gotcha! {}", body.get_credits()) }
 }

 pub fn load_1<R:Read>(&mut self, source: &mut R) -> Result<()>
 {
   self.threshold = source.read_u32::<LittleEndian>()?;
   self.checker = Normal::new(self.threshold as f32, (self.threshold as f32)/10.0).unwrap();
   log::debug!("Control loaded, threshold {}", self.threshold);
   Ok(())
 }

 pub fn save_1<W:Write>(&self, target: &mut W) -> Result<()>
 {
   target.write_u32::<LittleEndian>(self.threshold)?;
   Ok(())
 }
}