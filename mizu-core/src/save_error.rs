use crate::SAVE_STATE_VERSION;
use save_state::Error as saveStateError;
use std::io::Error as ioError;

#[derive(thiserror::Error, Debug)]
pub enum SaveError {
    #[error("SaveStateError: {0}")]
    SaveStateError(save_state::Error),
    #[error("This is not a valid save_state file")]
    InvalidSaveStateHeader,
    #[error(
        "The save_state file does not match the emulator version, got ({0}), needed {}",
        SAVE_STATE_VERSION
    )]
    UnmatchedSaveErrorVersion(usize),
    #[error("This save_state file is not for this cartridge")]
    InvalidCartridgeHash,
    #[error("Save file could not be opened/created")]
    SaveFileError,
}

impl From<save_state::Error> for SaveError {
    fn from(e: save_state::Error) -> Self {
        Self::SaveStateError(e)
    }
}

impl From<ioError> for SaveError {
    fn from(e: ioError) -> Self {
        Self::SaveStateError(saveStateError::IoError(e))
    }
}
