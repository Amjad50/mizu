pub use save_state_derive::*;
use serde_cbor;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use paste::paste;
use serde_cbor::Error as serdeCborError;
use std::convert::From;
use std::io::{
    Cursor, Error as ioError, ErrorKind as ioErrorKind, Read, Result as ioResult, Write,
};

pub type Result<T> = std::result::Result<T, Error>;

pub fn serialize_into<W, T>(writer: W, value: &T) -> Result<()>
where
    W: std::io::Write,
    T: serde::Serialize,
{
    serde_cbor::to_writer(writer, value).map_err(|e| e.into())
}

pub fn deserialize_from<R, T>(reader: R) -> Result<T>
where
    R: std::io::Read,
    T: serde::de::DeserializeOwned,
{
    let mut deserializer = serde_cbor::de::Deserializer::from_reader(reader);
    let value = serde::de::Deserialize::deserialize(&mut deserializer)?;
    Ok(value)
}

pub fn serialized_size<T>(value: &T) -> Result<u64>
where
    T: serde::Serialize,
{
    let mut counter = Counter::default();
    serde_cbor::to_writer(&mut counter, value)?;
    Ok(counter.counter)
}

/// a simple help that implements `io::Write`, which helps get the size of
/// a Savable object
#[derive(Default)]
struct Counter {
    counter: u64,
}

impl Counter {
    #[inline]
    fn add(&mut self, c: usize) -> ioResult<()> {
        // for some reason, using `checked_add` is exponentially slower, this is good
        let (counter, overflow) = self.counter.overflowing_add(c as u64);
        self.counter = counter;

        if overflow {
            Err(ioError::new(
                ioErrorKind::InvalidInput,
                "write length exceed u64 limit",
            ))
        } else {
            Ok(())
        }
    }
}

impl Write for Counter {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> ioResult<usize> {
        self.add(buf.len())?;
        Ok(buf.len())
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> ioResult<usize> {
        let len = bufs.iter().map(|b| b.len()).sum();
        self.add(len)?;
        Ok(len)
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> ioResult<()> {
        self.add(buf.len())?;
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> ioResult<()> {
        Ok(())
    }
}

pub trait Savable {
    fn save<W: Write>(&self, writer: &mut W) -> Result<()>;
    fn load<R: Read>(&mut self, reader: &mut R) -> Result<()>;
    /// The size of the object if saved now, note that this might change, for example
    /// due to the length of string objects or data inside the object.
    #[inline]
    fn save_size(&self) -> Result<u64> {
        let mut counter = Counter::default();
        self.save(&mut counter)?;
        Ok(counter.counter)
    }
}

pub fn save_object<T: Savable>(object: &T) -> Result<Vec<u8>> {
    let mut result = Vec::new();
    object.save(&mut result)?;

    Ok(result)
}

pub fn load_object<T: Savable>(object: &mut T, data: &[u8]) -> Result<()> {
    let mut cursor = Cursor::new(data);
    object.load(&mut cursor)?;

    let (remaining_data_len, overflow) = (data.len() as u64).overflowing_sub(cursor.position());
    assert!(!overflow);

    if remaining_data_len > 0 {
        Err(Error::TrailingData(remaining_data_len))
    } else {
        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Io Eror: {0}")]
    IoError(ioError),
    #[error("Bincode Error: {0}")]
    SerdeCobrError(serdeCborError),
    #[error("After loading an object, some data still remained ({0} bytes)")]
    TrailingData(u64),
    #[error("Enum could not be loaded correctly due to corrupted data ({0})")]
    InvalidEnumVariant(usize),
}

impl From<ioError> for Error {
    fn from(e: ioError) -> Self {
        Error::IoError(e)
    }
}

impl From<serdeCborError> for Error {
    fn from(e: serdeCborError) -> Self {
        Error::SerdeCobrError(e)
    }
}

macro_rules! impl_primitive {
    ($struct_name: ident $(, $g: tt)? ) => {
        impl Savable for $struct_name {
            #[inline]
            fn save<W: ::std::io::Write>(&self, writer: &mut W) -> Result<()> {
                paste!(writer.[<write_ $struct_name>]$($g<LittleEndian>)?(*self)?);
                Ok(())
            }

            #[inline]
            fn load<R: ::std::io::Read>(&mut self, reader: &mut R) -> Result<()> {
                *self = paste!(reader.[<read_ $struct_name>]$($g<LittleEndian>)?()?);
                Ok(())
            }

            #[inline]
            fn save_size(&self) -> Result<u64> {
               Ok(::std::mem::size_of::<Self>() as u64)
            }
        }
    };
}

// this is used here to implement some std types
macro_rules! impl_savable_with_serde {
    ($struct_name: ident $(<$($generics: ident),+>)?) => {
        impl $(<$($generics: serde::Serialize + serde::de::DeserializeOwned),+>)? Savable for $struct_name $(<$($generics),+>)?{
            #[inline]
            fn save<W: ::std::io::Write>(&self, writer: &mut W) -> Result<()> {
                serialize_into(writer, self)?;
                Ok(())
            }

            #[inline]
            fn load<R: ::std::io::Read>(&mut self, reader: &mut R) -> Result<()> {
                let obj = deserialize_from(reader)?;
                let _ = ::std::mem::replace(self, obj);
                Ok(())
            }
        }
    };
}

macro_rules! impl_for_tuple {
    ($($id: tt $tuple_element: ident),+) => {
        impl<$($tuple_element),+> Savable for ($($tuple_element),+)
        where $($tuple_element: Savable),+
        {
            #[inline]
            fn save<W: ::std::io::Write>(&self, mut writer: &mut W) -> Result<()> {
                $(<$tuple_element as Savable>::save(&self.$id, &mut writer)?;)+
                Ok(())
            }

            #[inline]
            fn load<R: ::std::io::Read>(&mut self, mut reader: &mut R) -> Result<()> {
                $(<$tuple_element as Savable>::load(&mut self.$id, &mut reader)?;)+
                Ok(())
            }
        }
    };
}

impl_primitive!(u8);
impl_primitive!(u16, ::);
impl_primitive!(u32, ::);
impl_primitive!(u64, ::);
impl_primitive!(i8);
impl_primitive!(i16, ::);
impl_primitive!(i32, ::);
impl_primitive!(i64, ::);
impl_primitive!(f32, ::);
impl_primitive!(f64, ::);
impl_savable_with_serde!(bool);
impl_savable_with_serde!(char);
impl_savable_with_serde!(String);
impl_savable_with_serde!(Vec<T>);

impl_for_tuple!(0 A0, 1 A1);
impl_for_tuple!(0 A0, 1 A1, 2 A2);
impl_for_tuple!(0 A0, 1 A1, 2 A2, 3 A3);
impl_for_tuple!(0 A0, 1 A1, 2 A2, 3 A3, 4 A4);
impl_for_tuple!(0 A0, 1 A1, 2 A2, 3 A3, 4 A4, 5 A5);
impl_for_tuple!(0 A0, 1 A1, 2 A2, 3 A3, 4 A4, 5 A5, 6 A6);
impl_for_tuple!(0 A0, 1 A1, 2 A2, 3 A3, 4 A4, 5 A5, 6 A6, 7 A7);
impl_for_tuple!(0 A0, 1 A1, 2 A2, 3 A3, 4 A4, 5 A5, 6 A6, 7 A7, 8 A8);

impl Savable for usize {
    fn save<W: ::std::io::Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u64::<LittleEndian>(*self as u64)?;
        Ok(())
    }

    fn load<R: ::std::io::Read>(&mut self, reader: &mut R) -> Result<()> {
        *self = reader.read_u64::<LittleEndian>()? as usize;
        Ok(())
    }

    fn save_size(&self) -> Result<u64> {
        Ok(::std::mem::size_of::<u64>() as u64)
    }
}

impl Savable for isize {
    fn save<W: ::std::io::Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_i64::<LittleEndian>(*self as i64)?;
        Ok(())
    }

    fn load<R: ::std::io::Read>(&mut self, reader: &mut R) -> Result<()> {
        *self = reader.read_i64::<LittleEndian>()? as isize;
        Ok(())
    }

    fn save_size(&self) -> Result<u64> {
        Ok(::std::mem::size_of::<i64>() as u64)
    }
}

// TODO: wait for `min_specialization` feature and implement
//  `u8` separatly as it would be faster
impl<T, const N: usize> Savable for [T; N]
where
    T: Savable,
{
    fn save<W: ::std::io::Write>(&self, mut writer: &mut W) -> Result<()> {
        for element in self {
            element.save(&mut writer)?;
        }
        Ok(())
    }

    fn load<R: ::std::io::Read>(&mut self, mut reader: &mut R) -> Result<()> {
        for element in self {
            element.load(&mut reader)?;
        }
        Ok(())
    }
}

impl<T> Savable for Option<T>
where
    T: Savable + Default,
{
    fn save<W: Write>(&self, mut writer: &mut W) -> Result<()> {
        match self {
            Some(s) => {
                true.save(&mut writer)?;
                s.save(&mut writer)?;
            }
            None => false.save(&mut writer)?,
        }
        Ok(())
    }

    fn load<R: Read>(&mut self, mut reader: &mut R) -> Result<()> {
        let mut value = false;
        value.load(&mut reader)?;

        if value {
            match self {
                Some(s) => {
                    s.load(&mut reader)?;
                }
                None => {
                    let mut s = T::default();
                    s.load(&mut reader)?;
                    self.replace(s);
                }
            }
        } else {
            *self = None;
        }

        Ok(())
    }
}

impl<T> Savable for std::marker::PhantomData<T> {
    fn save<W: Write>(&self, _writer: &mut W) -> Result<()> {
        Ok(())
    }

    fn load<R: Read>(&mut self, _reader: &mut R) -> Result<()> {
        Ok(())
    }
}

impl Savable for () {
    fn save<W: Write>(&self, _writer: &mut W) -> Result<()> {
        Ok(())
    }

    fn load<R: Read>(&mut self, _reader: &mut R) -> Result<()> {
        Ok(())
    }
}
