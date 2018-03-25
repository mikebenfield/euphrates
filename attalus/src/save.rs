/// Functions for saving and loading

const MAX_BYTES: u64 = 0x10000000; // 256 MiB

use std;
use std::path::Path;
use std::io::{Read, Write};
use std::fs::File;

use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use failure::Error;

use bincode;

type Result<T> = std::result::Result<T, Error>;

pub fn serialize<T>(t: &T) -> Result<Vec<u8>>
where
    T: ?Sized + Serialize,
{
    bincode::config()
        .limit(MAX_BYTES)
        .serialize(t)
        .map_err(|e| format_err!("Serialization error {}", e))
}

pub fn serialize_into<W, T>(w: W, t: &T) -> Result<()>
where
    W: Write,
    T: ?Sized + Serialize,
{
    bincode::config()
        .limit(MAX_BYTES) // 256 MiB
        .serialize_into(w, t)
        .map_err(|e| {
            format_err!("Serialization error {}", e)
        })
}

pub fn serialize_at<P, T>(path: P, t: &T) -> Result<()>
where
    P: AsRef<Path>,
    T: ?Sized + Serialize,
{
    let file = File::create(path)?;
    serialize_into(&file, t)
}

pub fn deserialize<'a, T>(bytes: &'a [u8]) -> Result<T>
where T: Deserialize<'a>
{
    bincode::config()
        .limit(MAX_BYTES)
        .deserialize(bytes)
        .map_err(|e| format_err!("Deserialization error {}", e))
}

pub fn deserialize_from<R, T>(reader: R) -> Result<T>
where
    R: Read,
    T: DeserializeOwned,
{
    bincode::config()
        .limit(MAX_BYTES)
        .deserialize_from(reader)
        .map_err(|e| format_err!("Deserialization error {}", e))
}


pub fn deserialize_at<P, T>(path: P) -> Result<T>
where
    P: AsRef<Path>,
    T: DeserializeOwned,
{
    let file = File::open(path)?;
    deserialize_from(&file)
}
