use thiserror::Error;

#[derive(Error, Debug)]
pub enum PmtudError {
    #[error("error setting up an association: {0}")]
    PmtudLayer3TransportInitError(String),
    #[error("error recving response")]
    PmtudRecvError,
}
