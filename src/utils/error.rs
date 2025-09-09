/// src/utils/error.rs
/// This file defines the custom error types for the application.

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("An unknown error occurred")]
    Unknown,
    // Add more specific error types here as needed
}
