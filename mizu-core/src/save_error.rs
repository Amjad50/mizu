use crate::SAVE_STATE_VERSION;
use save_state::Error as saveStateError;
use std::io::Error as ioError;

/// An error that may occur while saving/loading GameBoy state.
#[derive(thiserror::Error, Debug)]
pub enum SaveError {
    /// An error happend while serializing or deserializing one of the objects
    /// in the save state stream.
    #[error("SaveStateError: {0}")]
    SaveStateError(save_state::Error),
    /// The provided stream/file does not have a valid save state header.
    #[error("This is not a valid save_state file")]
    InvalidSaveStateHeader,
    /// The save state version mismatches the version supported/compatible with
    /// by this version of the emulator.
    ///
    /// Check [`SAVE_STATE_VERSION`] for the current version.
    #[error(
        "The save_state file does not match the emulator version, got ({0}), needed {}",
        SAVE_STATE_VERSION
    )]
    UnmatchedSaveErrorVersion(usize),
    /// The save state stream/file provided is not for the currently running
    /// cartridge.
    #[error("This save_state file is not for this cartridge")]
    InvalidCartridgeHash,
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
