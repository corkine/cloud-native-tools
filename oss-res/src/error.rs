#[derive(Debug)]
pub enum TransferError {
    IoError(std::io::Error),
    JsonParseError(serde_json::Error),
    OssError(String),
    Other(String),
}

impl From<std::io::Error> for TransferError {
    fn from(error: std::io::Error) -> Self {
        TransferError::IoError(error)
    }
}

impl std::fmt::Display for TransferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransferError::IoError(e) => write!(f, "IO Error: {}", e),
            TransferError::Other(s) => write!(f, "Other Error: {}", s),
            TransferError::OssError(e) => write!(f, "OSS Error: {}", e),
            TransferError::JsonParseError(e) => write!(f, "JSON Parse Error: {}", e),
        }
    }
}