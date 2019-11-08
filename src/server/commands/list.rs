use crate::server::commands::Cmd;
use crate::server::error::FTPError;
use crate::server::reply::{Reply, ReplyCode};
use crate::server::CommandArgs;
use crate::storage;
use futures::future::Future;
use futures::sink::Sink;
use tokio;

pub struct List;

impl<S, U> Cmd<S, U> for List
where
    U: Send + Sync + 'static,
    S: 'static + storage::StorageBackend<U> + Sync + Send,
    S::File: tokio_io::AsyncRead + Send,
    S::Metadata: storage::Metadata,
{
    fn execute(&self, args: &CommandArgs<S, U>) -> Result<Reply, FTPError> {
        // TODO: Map this error so we can give more meaningful error messages.
        let mut session = args.session.lock()?;
        let tx = match session.data_cmd_tx.take() {
            Some(tx) => tx,
            None => {
                return Ok(Reply::new(ReplyCode::CantOpenDataConnection, "No data connection established"));
            }
        };
        spawn!(tx.send(args.cmd.clone()));
        Ok(Reply::new(ReplyCode::FileStatusOkay, "Sending directory list"))
    }
}