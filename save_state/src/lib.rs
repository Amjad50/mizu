pub use bincode;
pub use save_state_derive::*;

use bincode::Error as bincodeError;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use paste::paste;
use std::convert::From;
use std::io::{Cursor, Error as ioError, Read, Write};

#[macro_export]
macro_rules! impl_savable {
    ($struct_name: ident) => {
        impl ::save_state::Savable for $struct_name {
            fn save<W: ::std::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::std::result::Result<(), ::save_state::SaveError> {
                ::bincode::serialize_into(writer, self)?;
                Ok(())
            }

            fn load<R: ::std::io::Read>(
                &mut self,
                reader: &mut R,
            ) -> ::std::result::Result<(), ::save_state::SaveError> {
                let obj = ::bincode::deserialize_from(reader)?;

                let _ = ::std::mem::replace(self, obj);
                Ok(())
            }

            fn current_save_size(&self) -> ::std::result::Result<u64, ::save_state::SaveError> {
                ::bincode::serialized_size(self).map_err(|e| e.into())
            }
        }
    };
}

pub trait Savable {
    fn save<W: Write>(&self, writer: &mut W) -> Result<(), SaveError>;
    fn load<R: Read>(&mut self, reader: &mut R) -> Result<(), SaveError>;
    /// The size of the object if saved now, note that this might change, for example
    /// due to the length of string objects or data inside the object.
    fn current_save_size(&self) -> Result<u64, SaveError>;
}

pub fn save_object<T: Savable>(object: &T) -> Result<Vec<u8>, SaveError> {
    let mut result = Vec::new();
    object.save(&mut result)?;

    Ok(result)
}

pub fn load_object<T: Savable>(object: &mut T, data: &[u8]) -> Result<(), SaveError> {
    let mut cursor = Cursor::new(data);
    object.load(&mut cursor)?;

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum SaveError {
    #[error("Io Eror: {0}")]
    IoError(ioError),
    #[error("Bincode Error: {0}")]
    BincodeError(bincodeError),
    #[error("Load size of the input data does not match `object_size`")]
    LoadSizeDoesNotMatch,
}

impl From<ioError> for SaveError {
    fn from(e: ioError) -> Self {
        SaveError::IoError(e)
    }
}

impl From<bincodeError> for SaveError {
    fn from(e: bincodeError) -> Self {
        SaveError::BincodeError(e)
    }
}

macro_rules! impl_primitive {
    ($struct_name: ident $(, $g: tt)? ) => {
        impl Savable for $struct_name {
            fn save<W: ::std::io::Write>(&self, writer: &mut W) -> Result<(), SaveError> {
                paste!(writer.[<write_ $struct_name>]$($g<LittleEndian>)?(*self)?);
                Ok(())
            }

            fn load<R: ::std::io::Read>(&mut self, reader: &mut R) -> Result<(), SaveError> {
                *self = paste!(reader.[<read_ $struct_name>]$($g<LittleEndian>)?()?);
                Ok(())
            }

            fn current_save_size(&self) -> Result<u64, SaveError> {
               Ok(::std::mem::size_of::<Self>() as u64)
            }
        }
    };
}

// this is used here to implement some std types
macro_rules! impl_savable {
    ($struct_name: ident $(<$($generics: ident),+>)?) => {
        impl $(<$($generics: serde::Serialize + serde::de::DeserializeOwned),+>)? Savable for $struct_name $(<$($generics),+>)?{
            fn save<W: ::std::io::Write>(&self, writer: &mut W) -> Result<(), SaveError> {
                ::bincode::serialize_into(writer, self)?;
                Ok(())
            }

            fn load<R: ::std::io::Read>(&mut self, reader: &mut R) -> Result<(), SaveError> {
                let obj = ::bincode::deserialize_from(reader)?;
                let _ = ::std::mem::replace(self, obj);
                Ok(())
            }

            fn current_save_size(&self) -> Result<u64, SaveError> {
                ::bincode::serialized_size(self).map_err(|e| e.into())
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
impl_savable!(bool);
impl_savable!(char);
impl_savable!(String);
impl_savable!(Vec<T>);

impl Savable for usize {
    fn save<W: ::std::io::Write>(&self, writer: &mut W) -> Result<(), SaveError> {
        writer.write_u64::<LittleEndian>(*self as u64)?;
        Ok(())
    }

    fn load<R: ::std::io::Read>(&mut self, reader: &mut R) -> Result<(), SaveError> {
        *self = reader.read_u64::<LittleEndian>()? as usize;
        Ok(())
    }

    fn current_save_size(&self) -> Result<u64, SaveError> {
        Ok(::std::mem::size_of::<Self>() as u64)
    }
}

impl Savable for isize {
    fn save<W: ::std::io::Write>(&self, writer: &mut W) -> Result<(), SaveError> {
        writer.write_i64::<LittleEndian>(*self as i64)?;
        Ok(())
    }

    fn load<R: ::std::io::Read>(&mut self, reader: &mut R) -> Result<(), SaveError> {
        *self = reader.read_i64::<LittleEndian>()? as isize;
        Ok(())
    }

    fn current_save_size(&self) -> Result<u64, SaveError> {
        Ok(::std::mem::size_of::<Self>() as u64)
    }
}

impl<const N: usize> Savable for [u8; N] {
    fn save<W: ::std::io::Write>(&self, writer: &mut W) -> Result<(), SaveError> {
        writer.write_all(self)?;
        Ok(())
    }

    fn load<R: ::std::io::Read>(&mut self, reader: &mut R) -> Result<(), SaveError> {
        reader.read_exact(self)?;
        Ok(())
    }

    fn current_save_size(&self) -> Result<u64, SaveError> {
        Ok(self.len() as u64)
    }
}
