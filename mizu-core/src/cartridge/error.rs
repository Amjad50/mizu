use super::mappers::MapperType;
use std::convert::From;
use std::fmt::Debug;
use std::io::{Error as ioError, ErrorKind as ioErrorKind};

/// An error that may occur when loading a new Cartridge file.
#[derive(thiserror::Error, Debug)]
pub enum CartridgeError {
    /// File error happened while reading the ROM file.
    #[error("File error: {0}")]
    FileError(ioError),
    /// The ROM file does not have valid extension.
    #[error("The file ends with an invalid extension, should end with '.gb' or '.gbc'")]
    ExtensionError,
    /// The rom file does not contain a valid Nintendo logo data at `0x104`.
    #[error("The rom file does not contain a valid Nintendo logo data at 0x104")]
    InvalidNintendoLogo,
    /// The game title contain invalid UTF-8 characters.
    #[error("The game title contain invalid UTF-8 characters")]
    InvalidGameTitle,
    /// Rom file contain unsupported cartridge type.
    #[error("Rom file contain unsupported cartridge type")]
    InvalidCartridgeType,
    /// The rom file header contain invalid `rom_size` value.
    #[error("The rom size index {0} is invalid")]
    InvalidRomSizeIndex(u8),
    /// The rom file header contain invalid `ram_size` value.
    #[error("The ram size index {0} is invalid")]
    InvalidRamSizeIndex(u8),
    /// The rom file size does not match the rom size provided in the rom header.
    #[error("The file size does not match the rom size {0} bytes indicated inside the header")]
    InvalidRomSize(usize),
    /// The cartridge type suggest the cartridge has ram, but it is not present.
    #[error("The cartridge type suggest the cartridge has ram, but it is not present")]
    RamNotPresentError,
    /// The cartridge type suggest the cartridge does not have ram, but it is present
    #[error("The cartridge type suggest the cartridge does not have ram, but it is present")]
    NotNeededRamPresentError,
    /// The header of the cartridge checksum does not match the provided value in the header.
    #[error("The header of the cartridge checksum {got} does not match the expected {expected}")]
    InvalidChecksum { expected: u8, got: u8 },
    /// The mapper type is not supported by the emulator.
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
