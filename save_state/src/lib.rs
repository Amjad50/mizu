use bincode::Error as bincodeError;
use std::convert::From;
use std::io::{Cursor, Error as ioError, Read, Write};

#[macro_export]
macro_rules! impl_savable {
    ($struct_name: ident) => {
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

            fn current_save_size(&self) -> Result<u64, ::save_state::SaveError> {
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
