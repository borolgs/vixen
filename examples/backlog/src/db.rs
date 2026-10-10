use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::mpsc,
    thread,
};

use anyhow::{Context, anyhow};
use rusqlite::Connection;
use tokio::sync::oneshot;

type Call = Box<dyn FnOnce(&mut Connection) + Send>;

#[derive(Debug, Clone)]
pub struct Db {
    tx: mpsc::Sender<Call>,
}

impl Db {
    pub fn open_in_memory() -> rusqlite::Result<Self> {
        Self::new(Connection::open_in_memory()?)
    }

    pub fn new(conn: Connection) -> rusqlite::Result<Self> {
        Ok(Self::spawn(conn))
    }

    fn spawn(mut conn: Connection) -> Self {
        let (tx, rx) = mpsc::channel::<Call>();

        thread::spawn(move || {
            while let Ok(call) = rx.recv() {
                if catch_unwind(AssertUnwindSafe(|| call(&mut conn))).is_err()
                    && !conn.is_autocommit()
                {
                    let _ = conn.execute_batch("ROLLBACK");
                }
            }
        });

        Self { tx }
    }

    /// Run `f` on the connection thread and wait for its result.
    pub async fn call<F, R>(&self, f: F) -> anyhow::Result<R>
    where
        F: FnOnce(&mut Connection) -> anyhow::Result<R> + Send + 'static,
        R: Send + 'static,
    {
        let (tx, rx) = oneshot::channel();

        self.tx
            .send(Box::new(move |conn| {
                let _ = tx.send(f(conn));
            }))
            .map_err(|_| anyhow!("db connection thread has stopped"))?;

        rx.await
            .context("db call panicked on the connection thread")?
    }
}
