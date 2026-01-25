use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use backend::{config::AppConfig, create_app, AppState};
use std::sync::Arc;
use tower::ServiceExt; // for `oneshot`

#[tokio::test]
async fn test_health_check() {
    // 1. Setup config and state
    let config = AppConfig {
        pr_fetch_days: 10,
        max_github_api_pages: 1,
        metrics_days_to_display: 7,
        metrics_window_size: 7,
        cache_ttl_seconds: 60,
        cache_max_capacity: 100,
        popular_repos: vec![],
        github_token: None,
    };
    let state = Arc::new(AppState::new(config).expect("Failed to create state"));

    // 2. Create app
    let app = create_app(state);

    // 3. Send request
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // 4. Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(body_json["status"], "ok");
    assert_eq!(body_json["service"], "repoflow-backend");
}

#[tokio::test]
async fn test_get_popular_repos() {
    use backend::config::RepoId;

    // 1. Setup config with specific popular repos
    let popular_repo = RepoId {
        owner: "test_owner".to_string(),
        repo: "test_repo".to_string(),
    };
    let config = AppConfig {
        pr_fetch_days: 10,
        max_github_api_pages: 1,
        metrics_days_to_display: 7,
        metrics_window_size: 7,
        cache_ttl_seconds: 60,
        cache_max_capacity: 100,
        popular_repos: vec![popular_repo.clone()],
        github_token: None,
    };
    let state = Arc::new(AppState::new(config).expect("Failed to create state"));

    let app = create_app(state);

    // 2. Send request
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/repos/popular")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // 3. Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: Vec<RepoId> = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(body_json.len(), 1);
    assert_eq!(body_json[0], popular_repo);
}

#[test]
fn test_repo_metrics_response_contract() {
    // This test ensures the backend serialization matches the Frontend's expected JSON structure.
    // If this test fails, it means we might have broken the API contract with the frontend.
    use backend::metrics::{FlowMetricsResponse, RepoMetricsResponse, SummaryMetrics};

    let response = RepoMetricsResponse {
        summary: SummaryMetrics {
            current_opened: 10,
            current_merged: 5,
            current_spread: 5,
            merge_rate: 50,
            is_widening: false,
        },
        time_series: vec![FlowMetricsResponse {
            date: "2024-01-01".to_string(),
            opened: 2,
            merged: 1,
            spread: 1,
        }],
    };

    let json = serde_json::to_value(&response).unwrap();

    // Verify fields exist and have correct types/names
    assert_eq!(json["summary"]["current_opened"], 10);
    assert_eq!(json["summary"]["current_merged"], 5);
    assert_eq!(json["summary"]["current_spread"], 5);
    assert_eq!(json["summary"]["merge_rate"], 50);
    assert_eq!(json["summary"]["is_widening"], false);

    let series = &json["time_series"][0];
    assert_eq!(series["date"], "2024-01-01");
    assert_eq!(series["opened"], 2);
    assert_eq!(series["merged"], 1);
    assert_eq!(series["spread"], 1);
}
