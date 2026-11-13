use std::io::{Result, BufWriter, Seek, SeekFrom};
use std::fs::{File, OpenOptions};
use crate::arcadia::storage::Stored;
use crate::arcadia::storage;
use crate::arcadia::interface::Universe;
use crate::arcadia::interface;

/*pub enum Message
{
 Death { id: Id<Actor> },
 Birth { id: Id<Actor>, parent: Id<Actor>, home: u64  }
}*/

pub struct Writer
{
 target: BufWriter<File>,
}

impl Writer
{
 pub fn new(filename: String) -> Result<Self>
 {
  let mut file = OpenOptions::new().create(true).write(true).open(filename)?;
  let pos = file.seek(SeekFrom::End(0))?;
  let mut target = BufWriter::new(file);
  if pos == 0
     {
     let mut swriter = storage::Writer::new(&mut target);
     swriter.u16(1)?;
     }
  Ok( Self { target: target } )
 }

 pub fn write(&mut self, tick :u64, message: &Universe) -> Result<()>
 {
  if self.filter(message)
     {
     let channel : u16 = 0;
     let mut writer = storage::Writer::new(&mut self.target);
     writer.u16(channel)?;
     writer.u64(tick)?;
     message.save(&mut writer)?;
     }
  Ok(())
 }

 fn filter(&self, message: &Universe) -> bool
 {
  match message
     {
     Universe::Actor {actor} =>
          match actor
            {
            interface::Actor::Make {..} => return true,
            interface::Actor::Drop {..} => return true,
            _ => {}
            },
//     _ => {}
     }
  false
 }
}

/* const DEATH :u16 = 1;
 const BIRTH :u16 = 2;

 pub fn write(&mut self, tick :u64, message: Message) -> Result<()>
 {
  let channel : u16 = 0;
  let mut writer = storage::Writer::new(&mut self.target);
  writer.u16(channel)?;
  writer.u64(tick)?;
  match message
     {
     Message::Death { id } =>
        {
        writer.u16(Self::DEATH)?;
        writer.u8(1)?;
        writer.u16(8)?;
        id.save(&mut writer)?;
        },
     Message::Birth { id, parent, home } => 
        {
        writer.u16(Self::BIRTH)?;
        writer.u8(1)?;
        writer.u16(24)?;
        id.save(&mut writer)?;
        parent.save(&mut writer)?;
        writer.u64(home)?;
        }
     }
  Ok( () )
 }
}*/
