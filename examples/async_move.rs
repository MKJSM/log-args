use log_args::params;
use std::time::Duration;
use tokio;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Debug, Clone)]
struct Session {
    user_id: String,
    session_id: String,
}

impl Session {
    fn new(user_id: &str, session_id: &str) -> Self {
        Self {
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
        }
    }

    #[params(fields(self.user_id, self.session_id), custom(service = "auth"))]
    async fn handle_request(&self) {
        info!("Handling request for user.");

        // This `tokio::spawn` with `async move` would have caused a "borrow of moved value"
        // error before the fix in the `#[params]` macro because `self` would be moved.
        let session_clone = self.clone();
        let handle = tokio::spawn(async move {
            // The `move` keyword moves ownership of `session_clone` into this new task.
            tokio::time::sleep(Duration::from_millis(100)).await;
            println!(
                "Async task finished for user_id: {}, session_id: {}",
                session_clone.user_id,
                session_clone.session_id
            );
        });

        handle.await.unwrap();

        // This log statement now works because `self` was not moved.
        // The `#[params]` macro correctly captured the values at the start of the function.
        info!("Finished handling request.");
    }
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let session = Session::new("user-123", "sess-abc");
    session.handle_request().await;
}
