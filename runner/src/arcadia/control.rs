use std::io::{Result, Read, Write, Error, ErrorKind};
use std::collections::HashMap;
use rand_distr::{Normal, Distribution};
use crate::arcadia::actors::Body;
use crate::arcadia::values::{Seed, Values};
use crate::arcadia::storage::{Reader,Writer};

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

pub struct Variator
{
 precision: u32,
 selector: Normal<f32>
}

impl Default for Variator
{
 fn default() -> Self
 {
  Variator { precision: 0, selector: Normal::new(0.0, 1.0).unwrap() }
 }
}

impl Variator
{
 pub fn set(&mut self, value: u32)
 {
  self.precision = value;
  self.selector = Normal::new(1.0, (self.precision as f32)/1000.0).unwrap();
 }

 pub fn variate(&self, value: f32) -> f32
 {
  let mut rng = rand::thread_rng();
  self.selector.sample(&mut rng) * value
 }

 pub fn variate_u32(&self, value: u32) -> u32
 {
  let r = self.variate(value as f32);
  if r <= 0.0
     { return 0; }
  if r >= 2_000_000_000.0
     { return 2_000_000_000; }
  r as u32
 }
}

trait Unit
{
 fn utype(&self) -> u16;
 fn version(&self) -> u8;
 fn load(&mut self, version: u8, reader: &mut Reader) -> Result<()>;
 fn save(&self, target: &mut Writer) -> Result<()>;
 fn tick(&self, units: &Units, values: &mut Values, body: &mut Body);
 fn blueprints(&self) -> BluePrint;
 fn make(&mut self, _bp: &BluePrint) -> Result<()>;
}

#[derive(Default)]
pub struct CreditSensor
{
 variator: Variator
}

impl Unit for CreditSensor
{
 fn utype(&self) -> u16 { Units::CREDIT_SENSOR }
 fn version(&self) -> u8 { 1 }

 fn load(&mut self, _version: u8, reader: &mut Reader) -> Result<()>
 {
  self.variator.set( reader.u32()? );
  Ok( () )
 }

 fn save(&self, target: &mut Writer) -> Result<()>
 {
  target.u32(self.variator.precision)?;
  Ok(())
 }

 fn tick(&self, _units: &Units, values: &mut Values, body: &mut Body)
 {
  values.credits = self.variator.variate(body.get_credits() as f32);
 }

 fn blueprints(&self) -> BluePrint
 {
  BluePrint::new(Units::CREDIT_SENSOR, vec![ BluePrint::from(self.variator.precision) ])
 }

 fn make(&mut self, bp: &BluePrint) -> Result<()>
 {
  self.variator.set( bp.get_u32(0)? );
  Ok(())
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

impl Unit for BirthSignal
{
 fn utype(&self) -> u16 { Units::BIRTH_SIGNAL }
 fn version(&self) -> u8 { 1 }

 fn load(&mut self, _version: u8, reader: &mut Reader) -> Result<()>
 {
  self.scale = reader.f32()?;
  self.threshold = reader.f32()?;
  self.variation = reader.u32()?;
  self.selector = Normal::new(0.0, (self.variation as f32)/1000.0).unwrap();
  Ok( () )
 }

 fn save(&self, target: &mut Writer) -> Result<()>
 {
  target.f32(self.scale)?;
  target.f32(self.threshold)?;
  target.u32(self.variation)?;
  Ok(())
 }

 fn tick(&self, _units: &Units, values: &mut Values, _body: &mut Body)
 {
  let mut rng = rand::thread_rng();
  let value = ( values.credits / self.scale) + self.threshold;
  values.birth = self.selector.sample(&mut rng) < value;
 }

 fn blueprints(&self) -> BluePrint
 {
  BluePrint::new(Units::BIRTH_SIGNAL, vec![ BluePrint::from(self.scale), BluePrint::from(self.threshold), BluePrint::from(self.variation) ] )
 }

 fn make(&mut self, bp: &BluePrint) -> Result<()>
 {
  self.threshold = bp.get_f32(1)?;
  self.variation = bp.get_u32(2)?;
  self.selector = Normal::new(0.0, (self.variation as f32)/1000.0).unwrap();
  Ok( () )
 }
}

#[derive(Default)]
pub struct BirthCredit
{
 giveaway: Sampler
}

impl Unit for BirthCredit
{
 fn utype(&self) -> u16 { Units::BIRTH_CREDIT }
 fn version(&self) -> u8 { 1 }

 fn load(&mut self, _version: u8, reader: &mut Reader) -> Result<()>
 {
  self.giveaway.set( reader.u32()? );
  Ok(())
 }
 fn save(&self, target: &mut Writer) -> Result<()>
 {
  target.u32(self.giveaway.nominal)?;
  Ok(())
 }
 fn tick(&self, _units: &Units, values: &mut Values, body: &mut Body)
 {
  let mut giveaway = self.giveaway.sample();
  giveaway = body.take_credits(giveaway);
  if giveaway > 0
     { values.birthcredits.push_back( Seed::new(giveaway) ) }
 }
 fn blueprints(&self) -> BluePrint
 {
  BluePrint::new(Units::BIRTH_CREDIT, vec![ BluePrint::from(self.giveaway.nominal) ])
 }
 fn make(&mut self, bp: &BluePrint) -> Result<()>
 {
  self.giveaway.set( bp.get_u32(0)? );
  Ok( () )
 }
}

#[derive(Default)]
pub struct ChildMaker
{
 variator: Variator
}

impl Unit for ChildMaker
{
 fn utype(&self) -> u16 { Units::CHILD_MAKER }
 fn version(&self) -> u8 { 1 }

 fn load(&mut self, _version: u8, reader: &mut Reader) -> Result<()>
 {
  self.variator.set( reader.u32()? );
  Ok(()) 
 }
 fn save(&self, target: &mut Writer) -> Result<()>
 {
  target.u32(self.variator.precision)?;
  Ok(())
 }
 fn tick(&self, units: &Units, values: &mut Values, _body: &mut Body)
 {
  while values.birthcredits.len() > 0
     {
     let mut seed = values.birthcredits.pop_front().unwrap();
     seed.blueprints = units.blueprints();
     seed.blueprints.variate(&self.variator);
     values.seeds.push_back(seed);
     }
 }
 fn blueprints(&self) -> BluePrint
 {
  BluePrint::new(Units::CHILD_MAKER, vec![ BluePrint::from(self.variator.precision) ])
 }
 fn make(&mut self, bp: &BluePrint) -> Result<()>
 {
  self.variator.set( bp.get_u32(0)? );
  Ok( () )
 }

}

#[derive(Default)]
pub struct Spawner
{
}

impl Unit for Spawner
{
 fn utype(&self) -> u16 { Units::SPAWNER }
 fn version(&self) -> u8 { 1 }

 fn load(&mut self, _version: u8, _reader: &mut Reader) -> Result<()>
 {
  Ok(()) 
 }
 fn save(&self, _target: &mut Writer) -> Result<()>
 {
  Ok(())
 }
 fn tick(&self, _units: &Units, values: &mut Values, body: &mut Body)
 {
  while values.seeds.len() > 0
     {
     let seed = values.seeds.pop_front().unwrap();
     body.birth(seed);
     }
 }
 fn blueprints(&self) -> BluePrint
 {
  BluePrint::new(Units::SPAWNER, vec![ ])
 }
 fn make(&mut self, _bp: &BluePrint) -> Result<()>
 {
  Ok(())
 }
}

pub enum BluePrint
{
 FValue { value: f32},
 UValue { value: u32},
 Collection { unit: u16, value: Vec<BluePrint> }
}

impl Default for BluePrint
{
 fn default() -> Self
   { Self::Collection { unit: Units::COMPOUND, value: vec![] } }
}

impl BluePrint
{
 const FVALUE: u8 = 1;
 const UVALUE: u8 = 2;
 const COLLECTION: u8 = 3;

 pub fn load(source: &mut Reader) -> Result<Self>
 {
  let utype = source.u8()?;
  let bp = match utype
        {
        Self::FVALUE => Self::FValue { value: source.f32()? },
        Self::UVALUE => Self::UValue { value: source.u32()? },
        Self::COLLECTION =>
              {
              let unit = source.u16()?;
              let mut value : Vec<BluePrint> = Vec::new();
              let ucount = source.u32()?;
              for _ in 0..ucount
                 { value.push( BluePrint::load(source)? ); }
              Self::Collection { unit: unit, value: value }
              },
        _ => return Err(Error::new(ErrorKind::InvalidData, "Unknown unit"))
        };

  Ok(bp)
 }
 pub fn save(&self, target: &mut Writer) -> Result<()>
 {
  match self
    {
    Self::FValue {value} =>
           { target.u8(Self::FVALUE)?; target.f32(*value)? },
    Self::UValue {value} =>
           { target.u8(Self::UVALUE)?; target.u32(*value)? },
    Self::Collection {unit, value} =>
           { target.u8(Self::COLLECTION)?;
             target.u16(*unit)?;
             target.u32(value.len() as u32)?;
             for bp in value.iter()
                { bp.save(target)?; } }
    }
  Ok( () )
 }
 pub fn variate(&mut self, variator: &Variator)
 {
  match self
    {
    Self::FValue {value} => *self = Self::FValue { value: variator.variate(*value) },
    Self::UValue {value} => *self = Self::UValue { value: variator.variate_u32(*value) },
    Self::Collection {value, ..} =>
          for bp in value
            { bp.variate(variator); }
    }
 }
}

impl From<u32> for BluePrint
{
 fn from(value: u32) -> Self { Self::UValue { value: value } }
}

impl From<f32> for BluePrint
{
 fn from(value: f32) -> Self { Self::FValue { value: value } }
}

impl BluePrint
{
 fn new(unit: u16, value: Vec<BluePrint>) -> Self { Self::Collection { unit:unit, value: value } }

 pub fn get_collection(&self) -> Result<&[BluePrint]>
 {
  match self
     {
     Self::Collection {value, ..} => Ok(&value),
     _ => Err(Error::new(ErrorKind::InvalidData, "Collection required"))
     }
 }

 pub fn get_unit(&self) -> Result<u16>
 {
  match self
     {
     Self::Collection {unit, ..} => Ok(*unit),
     _ => Err(Error::new(ErrorKind::InvalidData, "Collection required"))
     }
 }

 pub fn get_f32(&self, index: usize) -> Result<f32>
 {
  let item = &(self.get_collection()?)[index];
  match item
     {
     BluePrint::FValue { value } => Ok( *value ),
     _ => Err(Error::new(ErrorKind::InvalidData, "F32 required"))
     }
 }

 pub fn get_u32(&self, index: usize) -> Result<u32>
 {
  let item = &(self.get_collection()?)[index];
  match item
     {
     BluePrint::UValue { value } => Ok( *value ),
     _ => Err(Error::new(ErrorKind::InvalidData, "U32 required"))
     }
 }

}

type UnitCreator = fn() -> Box<dyn Unit>;

pub struct Units
{
 factory: HashMap<u16, UnitCreator>,
 units: Vec<Box<dyn Unit>>,
}

impl Default for Units
{
 fn default() -> Self
  {
  let mut factory: HashMap<u16, UnitCreator> = HashMap::new();
  factory.insert(Self::CREDIT_SENSOR, || Box::new(CreditSensor::default()) );
  factory.insert(Self::BIRTH_SIGNAL, || Box::new(BirthSignal::default()) );
  factory.insert(Self::BIRTH_CREDIT, || Box::new(BirthCredit::default()) );
  factory.insert(Self::CHILD_MAKER, || Box::new(ChildMaker::default()) );
  factory.insert(Self::SPAWNER, || Box::new(Spawner::default()) );
  Units { factory, units: Vec::new() }
  }
}

impl Units
{
 pub fn tick(&self, values: &mut Values, body: &mut Body)
 {
  for u in self.units.iter()
     { u.tick(self, values, body); }
 }

 const COMPOUND :u16 = 1;
 const CREDIT_SENSOR :u16 = 2;
 const BIRTH_SIGNAL :u16 = 3;
 const BIRTH_CREDIT :u16 = 4;
 const CHILD_MAKER :u16 = 5;
 const SPAWNER :u16 = 6;

 pub fn load(&mut self, reader: &mut Reader) -> Result<()>
 {
  for _ in 0..reader.count()?
     {
     let mut unit = self.factory.get(&reader.u16()?).expect("Unknown unit")();
     let version = reader.u8()?;
     if version > unit.version()
         { return Err(Error::new(ErrorKind::InvalidData, "Unknown unit version")); }
     unit.load(version, reader)?;
     self.units.push(unit);
     }
   Ok(())
 }

 pub fn new(&mut self, bp: &BluePrint) -> Result<()>
 {
  for ubp in bp.get_collection()?
      {
      let mut aunit = self.factory.get(&ubp.get_unit()?).expect("Unknown unit")();
      aunit.make(ubp)?;
      self.units.push(aunit);
      }

  Ok(())
 }

 pub fn save(&self, target: &mut Writer) -> Result<()>
 {
  target.count(self.units.len())?;
  for unit in self.units.iter()
      {
      target.u16(unit.utype())?;
      target.u8(unit.version())?;
      unit.save(target)?;
      }
   Ok(())
 }

 fn blueprints(&self) -> BluePrint
 {
  let mut value : Vec<BluePrint> = Vec::new();
  for unit in self.units.iter()
     { value.push( unit.blueprints() ); }
  BluePrint::Collection { unit: Self::COMPOUND, value: value }
 }

}

#[derive(Default)]
pub struct Control
{
 units: Units,
 values: Values,
}

impl Control
{
 fn version(&self) -> u8 { 1 }

 pub fn tick(&mut self, body: &mut Body)
 {
  self.units.tick(&mut self.values, body);
 }

 pub fn new(&mut self, bp: &BluePrint)
 {
  self.units.new(bp).unwrap();
 }

 pub fn load_1(&mut self, source: &mut dyn Read) -> Result<()>
 {
   let mut reader = Reader::new(source);
   if reader.u8()? > self.version()
         { return Err(Error::new(ErrorKind::InvalidData, "Unknown control version")); }
   self.units.load(&mut reader)?;
   self.values.load(&mut reader)?;
   Ok(())
 }

 pub fn save_1(&self, target: &mut dyn Write) -> Result<()>
 {
   let mut writer = Writer::new(target);
   writer.u8(self.version())?;
   self.units.save(&mut writer)?;
   self.values.save(&mut writer)?;
   Ok(())
 }
}