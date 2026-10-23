use std::io::{Result, BufWriter, Seek, SeekFrom};
use std::fs::{File, OpenOptions};
use crate::arcadia::actors::{ActorId};
use crate::arcadia::storage;

pub enum Message
{
 Death { id: ActorId },
 Birth { id: ActorId, parent: ActorId, home: u64  }
}

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

 const DEATH :u16 = 1;
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
}