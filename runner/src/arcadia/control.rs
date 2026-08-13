use std::io::{Result, Read, Write};
use std::collections::VecDeque;
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

pub struct CreditSensor
{
 precision: u32,
 selector: Normal<f32>
}

impl Default for CreditSensor
{
 fn default() -> Self
 {
  CreditSensor { precision: 0, selector: Normal::new(0.0, 1.0).unwrap() }
 }
}

impl CreditSensor
{
 fn load_1(&mut self, source: &mut dyn Read) -> Result<()>
 {
  self.precision = source.read_u32::<LittleEndian>()?;
  self.selector = Normal::new(1.0, (self.precision as f32)/1000.0).unwrap();
  Ok( () )
 }

 fn save_1(&self, target: &mut dyn Write) -> Result<()>
 {
  target.write_u32::<LittleEndian>(self.precision)?;
  Ok(())
 }

 pub fn tick(&mut self, values: &mut Values, body: &mut Body)
 {
  let mut rng = rand::thread_rng();
  values.credits = self.selector.sample(&mut rng) * ( body.get_credits() as f32);
 }
}

pub struct BirthSignal
{
 scale: f32,
 threshold: f32,
 variation: u32,
 selector: Normal<f32>
}

impl Default for BirthSignal
{
 fn default() -> Self
 {
  BirthSignal { scale: 0.0, threshold: 0.0, variation:0, selector: Normal::new(0.0, 1.0).unwrap() }
 }
}

impl BirthSignal
{
 fn load_1(&mut self, source: &mut dyn Read) -> Result<()>
 {
  self.scale = source.read_f32::<LittleEndian>()?;
  self.threshold = source.read_f32::<LittleEndian>()?;
  self.variation = source.read_u32::<LittleEndian>()?;
  self.selector = Normal::new(0.0, (self.variation as f32)/1000.0).unwrap();
  Ok( () )
 }

 fn save_1(&self, target: &mut dyn Write) -> Result<()>
 {
  target.write_f32::<LittleEndian>(self.scale)?;
  target.write_f32::<LittleEndian>(self.threshold)?;
  target.write_u32::<LittleEndian>(self.variation)?;
  Ok(())
 }

 pub fn tick(&mut self, values: &mut Values, _body: &mut Body)
 {
  let mut rng = rand::thread_rng();
  let value = ( values.credits / self.scale) + self.threshold;
  values.birth = self.selector.sample(&mut rng) < value;
 }
}

#[derive(Default)]
pub struct BirthCredit
{
 giveaway: Sampler
}

impl BirthCredit
{
 fn load_1(&mut self, source: &mut dyn Read) -> Result<()>
 {
  self.giveaway.set( source.read_u32::<LittleEndian>()? );
  Ok(())
 }
 fn save_1(&self, target: &mut dyn Write) -> Result<()>
 {
  target.write_u32::<LittleEndian>(self.giveaway.nominal)?;
  Ok(())
 }
 pub fn tick(&mut self, values: &mut Values, body: &mut Body)
 {
  let mut giveaway = self.giveaway.sample();
  giveaway = body.take_credits(giveaway);
  if giveaway > 0
     { values.birthcredits.push_back( Seed::new(self.giveaway.sample()) ) }
 }
}

pub struct Seed
{
 credits: u32
}

impl Seed
{
 pub fn new(credits: u32) -> Seed
 {
  Seed { credits: credits }
 }

 pub fn load_1(source: &mut dyn Read) -> Result<Self>
 {
  let credits = source.read_u32::<LittleEndian>()?;
  Ok( Seed { credits: credits } )
 }

 pub fn save_1(&self, target: &mut dyn Write) -> Result<()>
 {
  target.write_u32::<LittleEndian>(self.credits)?;
  Ok( () )
 }
}

#[derive(Default)]
pub struct Values
{
 credits: f32,
 birth: bool,
 birthcredits: VecDeque<Seed>
}

impl Values
{
 fn load_1(&mut self, source: &mut dyn Read) -> Result<()>
 {
  self.credits = source.read_f32::<LittleEndian>()?;
  self.birth = (source.read_u8()?) != 0;
  let countbc = source.read_u32::<LittleEndian>()? as usize;
  for _ in 0..countbc
    { self.birthcredits.push_back( Seed::load_1(source)? ); }
  Ok( () )
 }

 fn save_1(&self, target: &mut dyn Write) -> Result<()>
 {
  target.write_f32::<LittleEndian>(self.credits)?;
  target.write_u8(self.birth as u8)?;
  target.write_u32::<LittleEndian>(self.birthcredits.len() as u32)?;
  for seed in self.birthcredits.iter()
      { seed.save_1(target)?; }
  Ok(())
 }
}

#[derive(Default)]
pub struct Control
{
 creditsensor: CreditSensor,
 birthsignal: BirthSignal,
 birthcredit: BirthCredit,
 values: Values,
 threshold: Sampler,
 giveaway: Sampler,
 birth: bool,
 to_child: Option<u32>
}

impl Control
{
 pub fn tick(&mut self, body: &mut Body)
 {
  self.creditsensor.tick(&mut self.values, body);
  self.birthsignal.tick(&mut self.values, body);
  self.birthcredit.tick(&mut self.values, body);
  if self.threshold.sample() < body.get_credits()
      { self.birth = true; }
  if self.birth
    { self.to_child = Some(self.giveaway.sample()); self.birth = false; }
  if let Some(giveaway) = self.to_child
    {
       let mut startup : Vec<u8> = vec![];
       self.save_1(&mut startup).unwrap();
       body.birth(giveaway, startup);
       self.to_child = None;
   }
 }

 pub fn new(&mut self, startup: Vec<u8>)
 {
  let mut reader = &startup[..];
  self.load_1(&mut reader).unwrap();
 }

 pub fn load_1<R:Read>(&mut self, source: &mut R) -> Result<()>
 {
   self.creditsensor.load_1(source)?;
   self.birthsignal.load_1(source)?;
   self.birthcredit.load_1(source)?;
   self.values.load_1(source)?;
   self.threshold.set( source.read_u32::<LittleEndian>()? );
   self.giveaway.set( source.read_u32::<LittleEndian>()? );
   log::debug!("Control loaded, threshold {}", self.threshold.nominal);
   Ok(())
 }

 pub fn save_1<W:Write>(&self, target: &mut W) -> Result<()>
 {
   self.creditsensor.save_1(target)?;
   self.birthsignal.save_1(target)?;
   self.birthcredit.save_1(target)?;
   self.values.save_1(target)?;
   target.write_u32::<LittleEndian>(self.threshold.nominal)?;
   target.write_u32::<LittleEndian>(self.giveaway.nominal)?;
   Ok(())
 }
}