use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, ErrorKind, Read, Result, Seek, SeekFrom, Write};
use std::path::Path;

// third-party dependencies
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use crc::crc32;
use serde::{Deserialize, Serialize};

// types
pub type ByteString = Vec<u8>;
pub type ByteStr = [u8];

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyValuePair {
    pub key: ByteString,
    pub value: ByteString,
}

#[derive(Debug)]
pub struct Fdb {
    f: File,
    pub index: HashMap<ByteString, u64>,
}

impl Fdb {
    pub fn open(path: &Path) -> Result<Self> {
        let f = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(path)?;
        let index = HashMap::new();

        Ok(Fdb { f, index })
    }

    pub fn load(&mut self) -> Result<()> {
        let mut buf = BufReader::new(&mut self.f);

        loop {
            // TODO: test Seek
            let position = buf.seek(SeekFrom::Current(0))?;
            let maybe_kv = Fdb::process_record(&mut buf);

            let kv = match maybe_kv {
                Ok(kv) => kv,
                Err(err) => match err.kind() {
                    ErrorKind::UnexpectedEof => {
                        break;
                    }
                    _ => return Err(err),
                },
            };

            self.index.insert(kv.key, position);
        }

        Ok(())
    }

    fn process_record<R: Read>(f: &mut R) -> Result<KeyValuePair> {
        let saved_checksum = f.read_u32::<LittleEndian>()?;
        let key_len = f.read_u32::<LittleEndian>()?;
        let val_len = f.read_u32::<LittleEndian>()?;
        let data_len = key_len + val_len;

        let mut data = ByteString::with_capacity(data_len as usize);
        {
            f.by_ref().take(data_len as u64).read_to_end(&mut data)?;
        }

        assert_eq!(data.len(), data_len as usize);

        let checksum = crc32::checksum_ieee(&data);
        if checksum != saved_checksum {
            panic!(
                "data corruption encountered: {:08x} != {:08x}",
                checksum, saved_checksum
            );
        }

        let value = data.split_off(key_len as usize);
        let key = data;

        Ok(KeyValuePair { key, value })
    }

    /////////////////////
    /// GET //////////
    /// ///////////////

    pub fn get(&mut self, key: &ByteStr) -> Result<Option<ByteString>> {
        let pos = match self.index.get(key) {
            Some(p) => *p,
            None => return Ok(None),
        };

        let kv = self.get_at(pos)?;

        if kv.value.len() > 0 {
            Ok(Some(kv.value))
        } else {
            Ok(None)
        }
    }

    fn get_at(&mut self, pos: u64) -> Result<KeyValuePair> {
        let mut file = BufReader::new(&mut self.f);
        file.seek(SeekFrom::Start(pos))?;

        let kv = Fdb::process_record(&mut file)?;
        Ok(kv)
    }

    /////////////////////
    /// INSERT //////////
    /// ///////////////

    pub fn insert(&mut self, key: &ByteStr, value: &ByteStr) -> Result<()> {
        let pos = self.insert_but_ignore_index(key, value)?;

        // to_vec converts ByteStr to ByteString
        self.index.insert(key.to_vec(), pos);

        Ok(())
    }

    fn insert_but_ignore_index(&mut self, key: &ByteStr, value: &ByteStr) -> Result<u64> {
        let mut file = BufWriter::new(&mut self.f);

        let key_len = key.len();
        let val_len = value.len();
        let mut temp = ByteString::with_capacity(key_len + val_len);

        for byte in key {
            temp.push(*byte);
        }

        for byte in value {
            temp.push(*byte);
        }

        let checksum = crc32::checksum_ieee(&temp);

        // TODO: test Seek
        let next_byte = SeekFrom::End(0);
        let current_pos = file.seek(SeekFrom::Current(0))?;
        file.seek(next_byte)?;

        file.write_u32::<LittleEndian>(checksum)?;
        file.write_u32::<LittleEndian>(key_len as u32)?;
        file.write_u32::<LittleEndian>(val_len as u32)?;
        file.write_all(&mut temp)?;

        Ok(current_pos)
    }

    /////////////////////
    /// UPDATE //////////
    /// ///////////////
    #[inline]
    pub fn update(&mut self, key: &ByteStr, value: &ByteStr) -> Result<()> {
        self.insert(key, value)
    }

    /////////////////////
    /// DELETE //////////
    /// ///////////////
    #[inline]
    pub fn delete(&mut self, key: &ByteStr) -> Result<()> {
        self.insert(key, b"")
    }
}
