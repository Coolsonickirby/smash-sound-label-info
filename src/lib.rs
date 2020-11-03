//! A Rust library for working with `soundlabelinfo.sli` files from Smash Ultimate. This allows for
//! modifying various properties associated with  background music.
//! 
/// ```rust
/// # fn main() -> binread::BinResult<()> {
/// use sound_label_info::SliFile;
/// 
/// let mut file = SliFile::open("soundlabelinfo.sli")?;
/// 
/// for entry in file.entries() {
///     println!("tone_name: {:#X}", entry.tone_name);
/// }
/// 
/// for entry in file.entries_mut() {
///     entry.tone_id = 0;
/// }
/// 
/// file.save("soundlabelinfo_out.sli")?;
/// # Ok(())
/// # }
/// ```

use binread::{BinRead, BinReaderExt, derive_binread};
use binwrite::{BinWrite, WriterOption};

use std::fs::File;
use std::path::Path;
use std::io::{self, Write, BufReader, BufWriter};

#[cfg(feature = "derive_serde")]
use serde::{Serialize, Deserialize};

mod hash40;

/// Type alias for Hash40
pub type Hash40 = u64;

pub use binread::{BinResult as Result, Error};

/// ```rust
/// # fn main() -> binread::BinResult<()> {
/// use sound_label_info::SliFile;
/// 
/// let mut file = SliFile::open("soundlabelinfo.sli")?;
/// 
/// for entry in file.entries() {
///     println!("tone_name: {:#X}", entry.tone_name);
/// }
/// 
/// for entry in file.entries_mut() {
///     entry.tone_id = 0;
/// }
/// 
/// file.save("soundlabelinfo.sli")?;
/// # Ok(())
/// # }
/// ```
#[derive_binread]
#[cfg_attr(feature = "derive_serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
#[br(magic = b"SLI\x00")]
pub struct SliFile (
    u32,
    
    #[br(temp)]
    u32,

    #[br(count = self_1)]
    Vec<Entry>,
);

impl BinWrite for SliFile {
    fn write_options<W: Write>(&self, writer: &mut W, options: &WriterOption) -> io::Result<()> {
        (
            "SLI\x00",
            self.0,
            self.1.len() as u32,
            &self.1
        ).write_options(writer, options)
    }
}

/// An entry representing a single tone
#[cfg_attr(feature = "derive_serde", derive(Serialize, Deserialize))]
#[derive(BinRead, BinWrite, Debug)]
pub struct Entry {
    #[serde(with = "serde_hash40")]
    pub tone_name: Hash40,
    pub nus3bank_id: u32,
    pub tone_id: u32,
}

#[cfg(feature = "derive_serde")]
pub fn set_labels<P: AsRef<Path>>(path: P) -> Result<()> {
    fn inner(path: &Path) -> Result<()> {
        let contents = std::fs::read_to_string(path)?;
        let labels = contents.split("\n")
            .map(|string| (hash40::hash40(string.trim()), string.to_owned()))
            .collect();

        *serde_hash40::LABELS.lock().unwrap() = labels;

        Ok(())
    }

    inner(path.as_ref())
}

#[cfg(feature = "derive_serde")]
mod serde_hash40 {
    use std::{
        sync::Mutex,
        collections::HashMap,
    };

    lazy_static::lazy_static! {
        pub static ref LABELS: Mutex<HashMap<Hash40, String>> = Mutex::new(HashMap::new());
    }

    use super::{hash40::hash40, Hash40};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn deserialize<'de, D, E>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de, Error = E>,
        E: serde::de::Error,
    {
        let s: String = Deserialize::deserialize(deserializer)?;

        if s.starts_with("0x") {
            u64::from_str_radix(s.trim_start_matches("0x"), 16)
                .map_err(|_| D::Error::custom(format!("{} is an invalid Hash40", s)))
        } else {
            Ok(hash40(&s))
        }
    }

    pub fn serialize<S>(hash40: &Hash40, serializer: S) -> Result<S::Ok, S::Error> 
        where S: Serializer,
    {
        match LABELS.lock().unwrap().get(hash40) {
            Some(label) => {
                serializer.serialize_str(&label)
            }
            None => {
                serializer.serialize_str(&format!("{:#x}", hash40))
            }
        }
    }
}


impl SliFile {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        BufReader::new(File::open(path)?).read_le()
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.write(&mut BufWriter::new(File::create(path)?))
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.write_options(writer, &binwrite::writer_option_new!(endian: binwrite::Endian::Little))
            .map_err(Into::into)
    }

    pub fn new(version: u32, entries: Vec<Entry>) -> Self {
        SliFile(version, entries)
    }

    pub fn entries(&self) -> &Vec<Entry> {
        &self.1
    }

    pub fn entries_mut(&mut self) -> &mut Vec<Entry> {
        &mut self.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_trip() {
        let original = std::fs::read("soundlabelinfo.sli").unwrap();
        let sound_label_info = SliFile::open("soundlabelinfo.sli").unwrap();

        // println!("{:#X?}", sound_label_info);

        let mut round_trip = Vec::new();
        sound_label_info.write(&mut round_trip).unwrap();

        assert_eq!(original, round_trip);
        //sound_label_info.save("sound_label_info_out.bin").unwrap();
    }
}
