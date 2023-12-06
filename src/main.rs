use autometrics::{
    autometrics,
    objectives::{Objective, ObjectiveLatency, ObjectivePercentile},
    prometheus_exporter,
};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use std::env;
use std::error::Error;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time::sleep;

/// This is a Service-Level Objective (SLO) we're defining for our API.
/// It's a combination of a success rate and latency.
/// We're saying that 99.9% of requests should be successful
/// and 90% of requests should be responded to within 250ms.
const API_SLO: Objective = Objective::new("api")
    .success_rate(ObjectivePercentile::P99_9)
    .latency(ObjectiveLatency::Ms250, ObjectivePercentile::P90);

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Set up the exporter to collect metrics
    prometheus_exporter::init();

    let app = Router::new()
        .route("/", get(index))
        .route("/slow", get(slow_function))
        .route("/error", get(error_function))
        .route("/metrics", get(metrics));

    let addr = env::var("LISTEN_ADDRESS").unwrap_or_else(|_| "127.0.0.1:3000".to_string());

    let listener = TcpListener::bind(&addr).await?;
    let local_addr = listener.local_addr()?;

    eprintln!("Listening on: {local_addr}",);
    eprintln!();
    eprintln!("The following endpoints are available: ");
    eprintln!();
    eprintln!("- http://{local_addr}/        | static 200 response",);
    eprintln!("- http://{local_addr}/slow    | same but it is delayed with 1 second",);
    eprintln!("- http://{local_addr}/error   | static 500 response",);
    eprintln!("- http://{local_addr}/metrics | Prometheus endpoint containing the metrics",);
    eprintln!();
    eprintln!("To see the metrics in Explorer run: `am start {local_addr}`");

    // Start accepting and handling requests
    axum::serve(listener, app).await?;

    Ok(())
}

// our main handler function that is fine
#[autometrics(objective = API_SLO)]
async fn index() -> impl IntoResponse {
    "Hello, World!"
}

// our slow function that is slow
#[autometrics(objective = API_SLO)]
async fn slow_function() -> impl IntoResponse {
    sleep(Duration::from_millis(1000)).await;
    "Hello, World again!"
}

// our error function that errors
#[autometrics(objective = API_SLO)]
async fn error_function() -> Result<String, StatusCode> {
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

async fn metrics() -> Result<String, StatusCode> {
    prometheus_exporter::encode_to_string().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
