use serde::Serialize;
use tokio::sync::mpsc;

use crate::app;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Tauri(#[from] tauri::Error),

    #[error(transparent)]
    Send(#[from] mpsc::error::SendError<super::handle::DialogMsg>),

    #[error(transparent)]
    WindowSend(#[from] mpsc::error::SendError<app::Event>),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
