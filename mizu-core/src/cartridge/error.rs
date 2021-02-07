use super::mappers::MapperType;
use std::convert::From;
use std::fmt::Debug;
use std::io::{Error as ioError, ErrorKind as ioErrorKind};

#[derive(thiserror::Error, Debug)]
pub enum CartridgeError {
    #[error("File error: {0}")]
    FileError(ioError),
    #[error("The file ends with an invalid extension, should end with '.gb'")]
    ExtensionError,
    #[error("The rom file does not contain a valid nintendo logo data at 0x104")]
    InvalidNintendoLogo,
    #[error("The game title contain invalid UTF-8 characters")]
    InvalidGameTitle,
    #[error("Rom file contain unsupported cartridge type")]
    InvalidCartridgeType,
    #[error("The rom size index {0} is invalid")]
    InvalidRomSizeIndex(u8),
    #[error("The ram size index {0} is invalid")]
    InvalidRamSizeIndex(u8),
    #[error("The file size does not match the rom size {0} bytes indicated inside the header")]
    InvalidRomSize(usize),
    #[error("The cartridge type suggest the cartridge has ram, but it is not present")]
    RamNotPresentError,
    #[error("The cartridge type suggest the cartridge does not have ram, but it is present")]
    NotNeededRamPresentError,
    #[error("The header of the cartridge check sum {got} does not match the expected {expected}")]
    InvalidChecksum { expected: u8, got: u8 },
    #[error("The mapper {0:?} is not yet implemented")]
    MapperNotImplemented(MapperType),
}

impl From<ioError> for CartridgeError {
    fn from(from: ioError) -> Self {
        Self::FileError(from)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SramError {
    #[error("Could not load cartridge save file")]
    NoSramFileFound,
    #[error("There is a conflict in the size of SRAM save file in the Cartridge header and the file in disk")]
    SramFileSizeDoesNotMatch,
    #[error("Could not save cartridge save file")]
    FailedToSaveSramFile,
    #[error("Unknown error occured while trying to save/load cartridge save file")]
    Others,
}

impl From<ioError> for SramError {
    fn from(from: ioError) -> Self {
        match from.kind() {
            ioErrorKind::NotFound => Self::NoSramFileFound,
            ioErrorKind::PermissionDenied => Self::FailedToSaveSramFile,
            _ => Self::Others,
        }
    }
}
