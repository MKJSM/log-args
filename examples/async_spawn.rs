use log_args::params;
use tokio::sync::mpsc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

// This struct has fields that will be moved into an async block
#[derive(Clone, Debug)]
struct Client {
    client_id: String,
    company_id: String,
}

impl Client {
    // Use no_fields to disable automatic field logging when using tokio::spawn with async move
    // This is necessary when fields will be captured by an async move block
    #[params(custom(service = "client_service"))]
    async fn handle_connection(&self) {
        // We can still manually log fields when needed
        info!(client_id = ?self.client_id, company_id = ?self.company_id, "Starting connection handler");

        let (tx, mut rx) = mpsc::channel(10);
        let client_id = self.client_id.clone();
        let company_id = self.company_id.clone();

        // This spawns an async task that will move client_id and company_id
        let task = tokio::spawn(async move {
            // Inside here, self would be moved if we didn't use no_fields
            while let Some(msg) = rx.recv().await {
                info!("Received message: {}", msg);
            }

            // We can still log the values we explicitly cloned
            info!(client_id = ?client_id, company_id = ?company_id, "Worker finished");
        });

        // Send some messages
        let _ = tx.send("Hello").await;
        let _ = tx.send("World").await;

        // Wait for task to complete
        drop(tx);
        let _ = task.await;

        // We can still access self here because no_fields prevented automatic captures
        info!("Connection handler for {} completed", self.client_id);
    }
}

#[tokio::main]
async fn main() {
    // Setup tracing subscriber
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Create a client and handle connection
    let client = Client {
        client_id: "client-123".to_string(),
        company_id: "company-456".to_string(),
    };

    client.handle_connection().await;
}
