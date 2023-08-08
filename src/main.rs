use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use std::time::Duration;
use tokio::time::sleep;

use autometrics::{
    autometrics,
    objectives::{Objective, ObjectiveLatency, ObjectivePercentile},
    prometheus_exporter,
};

/// This is a Service-Level Objective (SLO) we're defining for our API.
/// It's a combination of a success rate and latency.
/// We're saying that 99.9% of requests should be successful
/// and 90% of requests should be responded to within 250ms.
const API_SLO: Objective = Objective::new("api")
    .success_rate(ObjectivePercentile::P99_9)
    .latency(ObjectiveLatency::Ms250, ObjectivePercentile::P90);

#[tokio::main]
async fn main() {
    // Set up the exporter to collect metrics
    prometheus_exporter::init();

    let app = Router::new()
        .route("/", get(index))
        .route("/slow", get(slow_function))
        .route("/error", get(error_function))
        .route(
            "/metrics",
            get(|| async { prometheus_exporter::encode_http_response() }),
        );

    let addr = "[::]:3000".parse().unwrap();
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// our main handler function that is fine
#[autometrics(objective = API_SLO)]
pub async fn index() -> impl IntoResponse {
    return "Hello, World!";
}

// our slow function that is slow
#[autometrics(objective = API_SLO)]
pub async fn slow_function() -> impl IntoResponse {
    sleep(Duration::from_millis(1000)).await;
    return "Hello, World again!";
}

// our error function that errors
#[autometrics(objective = API_SLO)]
pub async fn error_function() -> Result<String, StatusCode> {
    return Err(StatusCode::INTERNAL_SERVER_ERROR);
}
