use crate::server::commands::Cmd;
use crate::server::error::FTPError;
use crate::server::reply::{Reply, ReplyCode};
use crate::server::CommandArgs;
use crate::storage;

pub struct Cdc;

impl<S, U> Cmd<S, U> for Cdc
where
    U: Send + Sync + 'static,
    S: 'static + storage::StorageBackend<U> + Sync + Send,
    S::File: tokio_io::AsyncRead + Send,
    S::Metadata: storage::Metadata,
{
    fn execute(&self, _args: &CommandArgs<S, U>) -> Result<Reply, FTPError> {
        Ok(Reply::new(ReplyCode::CommandNotImplemented, "Not implemented."))
    }
}