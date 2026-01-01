use thiserror::Error;

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Asset not found: {0}")]
    AssetNotFound(String),
    #[error("Failed to create surface")]
    SurfaceFailure,
    #[error("Graphics error: {0}")]
    SkiaError(String),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
    #[error("Recursion depth limit exceeded")]
    RecursionLimit,
}
