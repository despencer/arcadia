use std::io::{Result, BufWriter, Seek, SeekFrom};
use std::fs::{File, OpenOptions};
use byteorder::{WriteBytesExt, LittleEndian};

pub enum Message
{
 Death { id: u64 },
 Birth { id: u64, parent: u64, home: u64  }
}

pub struct Writer
{
 writer: BufWriter<File>
}

impl Writer
{
 pub fn new(filename: String) -> Result<Self>
 {
  let mut file = OpenOptions::new().create(true).write(true).open(filename)?;
  let pos = file.seek(SeekFrom::End(0))?;
  let mut writer = BufWriter::new(file);
  if pos == 0
     { writer.write_u16::<LittleEndian>(1)?; }
  Ok( Writer { writer } )
 }

 const DEATH :u16 = 1;
 const BIRTH :u16 = 2;

 pub fn write(&mut self, tick :u64, message: Message) -> Result<()>
 {
  let channel : u16 = 0;
  self.writer.write_u16::<LittleEndian>(channel)?;
  self.writer.write_u64::<LittleEndian>(tick)?;
  match message
     {
     Message::Death { id } =>
        {
        self.writer.write_u16::<LittleEndian>(Self::DEATH)?;
        self.writer.write_u8(1)?;
        self.writer.write_u16::<LittleEndian>(8)?;
        self.writer.write_u64::<LittleEndian>(id)?;
        },
     Message::Birth { id, parent, home } => 
        {
        self.writer.write_u16::<LittleEndian>(Self::BIRTH)?;
        self.writer.write_u8(1)?;
        self.writer.write_u16::<LittleEndian>(24)?;
        self.writer.write_u64::<LittleEndian>(id)?;
        self.writer.write_u64::<LittleEndian>(parent)?;
        self.writer.write_u64::<LittleEndian>(home)?;
        }
     }
  Ok( () )
 }
}