use thiserror::Error;

#[derive(Error, Debug)]
pub enum InclusionServiceError {
    #[error("Blob index not found")]
    MissingBlobIndex,
    
    #[error("Failed to convert keccak hash to array")]
    KeccakHashConversion,
    
    #[error("Failed to verify row root inclusion multiproof")]
    RowRootVerificationFailed,
    
    #[error("Failed to convert shares to blob: {0}")]
    ShareConversionError(String),
    
    #[error("Failed to create inclusion proof input: {0}")]
    GeneralError(String),
}