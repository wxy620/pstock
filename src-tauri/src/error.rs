

#[derive(Debug, thiserror::Error)]
pub enum SysError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("failed to parse as string: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error(transparent)]
    Anyhow(#[from]  anyhow::Error),
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),
    #[error(transparent)]
    TauriError(#[from] tauri::Error),
    #[error("sse error! {1}. code:{0}")]
    EstSseError(u32, String),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    ReqwestEventSourceError(#[from] reqwest_eventsource::Error),

    #[error("happen error, stop execute. code:{0}, message:{1}")]
    BREAK(u32, String)
}

#[derive(serde::Serialize)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
pub enum ErrorKind {
    Io(String),
    Utf8(String),
    Unknown(String),
}

impl serde::Serialize for SysError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let error_message = self.to_string();
        let error_kind = match self {
            Self::Io(_) => ErrorKind::Io(error_message),
            Self::Utf8(_) => ErrorKind::Utf8(error_message),
            _ => ErrorKind::Unknown(error_message),
        };
        error_kind.serialize(serializer)
    }
}



