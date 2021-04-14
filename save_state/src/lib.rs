use bincode::Error as bincodeError;
use std::convert::From;
use std::io::{Cursor, Error as ioError, Read, Write};

#[macro_export]
macro_rules! impl_savable {
    ($struct_name: ident, $object_size: expr) => {
        impl ::save_state::Savable for $struct_name {
            fn save<W: ::std::io::Write>(
                &self,
                writer: &mut W,
            ) -> Result<(), ::save_state::SaveError> {
                ::bincode::serialize_into(writer, self)?;
                Ok(())
            }

            fn load<R: ::std::io::Read>(
                &mut self,
                reader: &mut R,
            ) -> Result<(), ::save_state::SaveError> {
                let obj = ::bincode::deserialize_from(reader)?;

                let _ = ::std::mem::replace(self, obj);
                Ok(())
            }

            fn object_size() -> u64 {
                $object_size
            }

            fn current_save_size(&self) -> Result<u64, ::save_state::SaveError> {
                ::bincode::serialized_size(self).map_err(|e| e.into())
            }
        }
    };
}

pub trait Savable {
    fn save<W: Write>(&self, writer: &mut W) -> Result<(), SaveError>;
    fn load<R: Read>(&mut self, reader: &mut R) -> Result<(), SaveError>;
    /// When saving, the size, will be expanded to reach max size.
    /// When loading, the size must be equal or less than the max_size.
    ///
    /// The purpose of this number is for extra padding that accounts for future
    /// features that might be added to the object, and should not be changed for
    /// a long time.
    fn object_size() -> u64;
    /// The size of the object if saved now, note that this might change, for example
    /// due to the length of string objects or data inside the object.
    fn current_save_size(&self) -> Result<u64, SaveError>;
}

pub fn save_object<T: Savable>(object: &T) -> Result<Vec<u8>, SaveError> {
    let object_size = T::object_size() as usize;
    let mut result = Vec::with_capacity(object_size);
    object.save(&mut result)?;

    if result.len() > object_size {
        Err(SaveError::SaveSizeExceedLimit)
    } else {
        let size_diff = object_size - result.len();
        for _ in 0..size_diff {
            // add only the lowest 8 bits
            result.push(0);
        }

        assert_eq!(result.len(), object_size);

        Ok(result)
    }
}

pub fn load_object<T: Savable>(object: &mut T, data: &[u8]) -> Result<(), SaveError> {
    let object_size = T::object_size();

    if data.len() != object_size as usize {
        return Err(SaveError::LoadSizeDoesNotMatch);
    }

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
    #[error("Save Size exceed limit")]
    SaveSizeExceedLimit,
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
