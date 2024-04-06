#[derive(Debug, thiserror::Error)]
pub enum ViteError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("manifest not found: {path}")]
    Manifest { path: String },
}
