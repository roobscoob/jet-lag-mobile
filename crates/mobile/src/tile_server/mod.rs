mod routes;

use std::path::PathBuf;
use std::sync::Arc;

use pmtiles::MmapBackend;
use pmtiles::async_reader::AsyncPmTilesReader;
use tokio::runtime::Runtime;
use tokio::sync::{RwLock, oneshot};

pub type PmTilesReader = AsyncPmTilesReader<MmapBackend>;

#[derive(Debug)]
pub enum TileServerError {
    IoError(std::io::Error),
    PmTilesError(String),
}

impl std::fmt::Display for TileServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TileServerError::IoError(e) => write!(f, "IO error: {e}"),
            TileServerError::PmTilesError(msg) => write!(f, "PMTiles error: {msg}"),
        }
    }
}

impl std::error::Error for TileServerError {}

impl From<std::io::Error> for TileServerError {
    fn from(e: std::io::Error) -> Self {
        TileServerError::IoError(e)
    }
}

pub struct TileServer {
    #[allow(dead_code)] // Kept alive to keep server running
    runtime: Runtime,
    port: u16,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl TileServer {
    pub fn start(pmtiles_path: PathBuf) -> Result<Self, TileServerError> {
        let runtime = Runtime::new()?;

        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let (result_tx, result_rx) = oneshot::channel::<Result<u16, TileServerError>>();

        runtime.spawn(async move {
            let result = Self::start_server(pmtiles_path, shutdown_rx).await;
            let _ = result_tx.send(result);
        });

        let port = runtime.block_on(async {
            result_rx.await.map_err(|_| {
                TileServerError::PmTilesError("Server task died unexpectedly".into())
            })?
        })?;

        Ok(Self {
            runtime,
            port,
            shutdown_tx: Some(shutdown_tx),
        })
    }

    async fn start_server(
        pmtiles_path: PathBuf,
        shutdown_rx: oneshot::Receiver<()>,
    ) -> Result<u16, TileServerError> {
        let backend = MmapBackend::try_from(pmtiles_path.as_path())
            .await
            .map_err(|e| {
                TileServerError::PmTilesError(format!(
                    "Failed to open PMTiles file at {pmtiles_path:?}: {e}"
                ))
            })?;

        let reader = AsyncPmTilesReader::try_from_source(backend)
            .await
            .map_err(|e| {
                TileServerError::PmTilesError(format!("Failed to read PMTiles archive: {e}"))
            })?;

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();

        let reader = Arc::new(RwLock::new(reader));
        let app = routes::create_router(reader, port);

        tokio::spawn(async move {
            let _ = axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    let _ = shutdown_rx.await;
                })
                .await;
        });

        Ok(port)
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

impl Drop for TileServer {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}
