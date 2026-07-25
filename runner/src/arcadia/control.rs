use std::io::{Result, Read, Write};
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};
use rand_distr::{Normal, Distribution};
use crate::arcadia::actors::Body;

pub struct Sampler
{
 nominal: u32,
 selector: Normal<f32>
}

impl Default for Sampler
{
 fn default() -> Self
 {
  Sampler { nominal: 0, selector: Normal::new(0.0, 1.0).unwrap() }
 }
}

impl Sampler
{
 pub fn set(&mut self, value: u32)
 {
  self.nominal = value;
  self.selector = Normal::new(value as f32, (value as f32)/10.0).unwrap();
 }

 pub fn sample(&self) -> u32
 {
  let mut rng = rand::thread_rng();
  let r = self.selector.sample(&mut rng);
  if r <= 0.0
     { return 0; }
  r as u32
 }
}

#[derive(Default)]
pub struct Control
{
 threshold: Sampler,
 giveaway: Sampler
}

impl Control
{
 pub fn tick(&mut self, body: &mut Body)
 {
  if self.threshold.sample() < body.get_credits()
  {
    let giveaway = self.giveaway.sample();
    if giveaway > 0
       {
       let mut startup : Vec<u8> = vec![];
       self.save_1(&mut startup).unwrap();
       body.birth(giveaway, startup);
       }
  }
 }

 pub fn new(&mut self, startup: Vec<u8>)
 {
  let mut reader = &startup[..];
  self.load_1(&mut reader).unwrap();
 }

 pub fn load_1<R:Read>(&mut self, source: &mut R) -> Result<()>
 {
   self.threshold.set( source.read_u32::<LittleEndian>()? );
   self.giveaway.set( source.read_u32::<LittleEndian>()? );
   log::debug!("Control loaded, threshold {}", self.threshold.nominal);
   Ok(())
 }

 pub fn save_1<W:Write>(&self, target: &mut W) -> Result<()>
 {
   target.write_u32::<LittleEndian>(self.threshold.nominal)?;
   target.write_u32::<LittleEndian>(self.giveaway.nominal)?;
   Ok(())
 }
}