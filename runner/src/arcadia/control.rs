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

pub struct Control
{
 threshold: u32,
 checker: Normal<f32>,
 giveaway: u32,
 giver: Normal<f32>
}

impl Default for Control
{
 fn default() -> Self
 {
  Control
    {
     threshold: 0,
     checker: Normal::new(0.0, 1.0).unwrap(),
     giveaway : 0,
     giver: Normal::new(0.0, 1.0).unwrap()
    }
 }
}

impl Control
{
 pub fn tick(&mut self, body: &mut Body)
 {
  let mut rng = rand::thread_rng();
  if self.checker.sample(&mut rng) < body.get_credits() as f32
    {
    let giveaway = self.giver.sample(&mut rng);
    if giveaway > 0.0
       {
       let mut startup : Vec<u8> = vec![];
       self.save_1(&mut startup).unwrap();
       body.birth(giveaway as u32, startup);
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
   self.threshold = source.read_u32::<LittleEndian>()?;
   self.checker = Normal::new(self.threshold as f32, (self.threshold as f32)/10.0).unwrap();
   self.giveaway = source.read_u32::<LittleEndian>()?;
   self.giver = Normal::new(self.giveaway as f32, (self.giveaway as f32)/10.0).unwrap();
   log::debug!("Control loaded, threshold {}", self.threshold);
   Ok(())
 }

 pub fn save_1<W:Write>(&self, target: &mut W) -> Result<()>
 {
   target.write_u32::<LittleEndian>(self.threshold)?;
   target.write_u32::<LittleEndian>(self.giveaway)?;
   Ok(())
 }
}