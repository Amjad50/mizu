use super::mappers::MapperType;
use std::convert::From;
use std::error::Error;
use std::fmt::Display;
use std::io::Error as ioError;

#[derive(Debug)]
pub enum CartridgeError {
    FileError(ioError),
    ExtensionError,
    InvalidFileSize(usize),
    InvalidNintendoLogo,
    InvalidGameTitle,
    InvalidCartridgeType,
    InvalidRomSizeIndex(u8),
    InvalidRamSizeIndex(u8),
    InvalidRomSize(usize),
    RamNotPresentError,
    NotNeededRamPresentError,
    InvalidChecksum { expected: u8, got: u8 },
    MapperNotImplemented(MapperType),
}

impl CartridgeError {
    fn message(&self) -> String {
        match self {
            CartridgeError::FileError(err) => format!("File error: {}", err),
            CartridgeError::ExtensionError => {
                "The file ends with an invalid extension, should end with '.gb'".to_string()
            }
            CartridgeError::InvalidCartridgeType => {
                "Rom file contain unsupported cartridge type".to_string()
            }
            CartridgeError::InvalidFileSize(size) => {
                format!("The rom file size {} bytes, is invalid", size)
            }
            CartridgeError::InvalidNintendoLogo => {
                "The rom file does not contain a valid nintendo logo data at 0x104".to_string()
            }
            CartridgeError::InvalidGameTitle => {
                "The game title contain invalid UTF-8 characters".to_string()
            }
            CartridgeError::InvalidRomSizeIndex(index) => {
                format!("The rom size index {} is invalid", index)
            }
            CartridgeError::InvalidRamSizeIndex(index) => {
                format!("The ram size index {} is invalid", index)
            }
            CartridgeError::InvalidRomSize(size) => format!(
                "The file size does not match the rom size {} bytes indicated inside the header",
                size
            ),
            CartridgeError::RamNotPresentError => {
                "The cartridge type suggest the cartridge has ram, but it is not present"
                    .to_string()
            }
            CartridgeError::NotNeededRamPresentError => {
                "The cartridge type suggest the cartridge does not have ram, but it is present"
                    .to_string()
            }
            CartridgeError::InvalidChecksum { expected, got } => format!(
                "The header of the cartridge check sum {} does not match the expected {}",
                got, expected
            ),
            CartridgeError::MapperNotImplemented(mapper) => {
                format!("The mapper {:?} is not yet implemented", mapper)
            }
        }
    }
}

impl Error for CartridgeError {}

impl Display for CartridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl From<ioError> for CartridgeError {
    fn from(from: ioError) -> Self {
        Self::FileError(from)
    }
}
