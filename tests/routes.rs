use axum::body::Body;
use axum::extract::Json as AxumJson;
use axum::http::{Request, StatusCode};
use axum::routing::post;
use axum::{Json, Router as AxumRouter};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use srvcs_nthroot::{api::Deps, health, router, telemetry};
use tower::ServiceExt;

const DEAD_URL: &str = "http://127.0.0.1:1";

/// Spawn a *computing* mock `srvcs-root`: reads `{"value": v, "n": n}` and
/// returns `{"result": v.powf(1 / n)}` — the real nth root as an f64. The
/// nthroot alias is genuinely driven by this answer rather than a canned value.
async fn spawn_root() -> String {
    let app = AxumRouter::new().route(
        "/",
        post(|AxumJson(body): AxumJson<Value>| async move {
            let value = body.get("value").and_then(Value::as_f64).unwrap_or(0.0);
            let n = body.get("n").and_then(Value::as_f64).unwrap_or(1.0);
            Json(json!({ "result": value.powf(1.0 / n) }))
        }),
    );
    serve(app).await
}

/// Spawn a mock returning a fixed status + body (used for error-path tests).
async fn spawn_fixed(status: StatusCode, body: Value) -> String {
    let app = AxumRouter::new().route(
        "/",
        post(move || {
            let body = body.clone();
            async move { (status, Json(body)) }
        }),
    );
    serve(app).await
}

async fn serve(app: AxumRouter) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

fn app(root_url: &str) -> axum::Router {
    router(
        telemetry::metrics_handle_for_tests(),
        Deps {
            root_url: root_url.to_string(),
        },
    )
}

async fn nthroot(root_url: &str, value: f64, n: f64) -> (StatusCode, Value) {
    let res = app(root_url)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/")
                .header("content-type", "application/json")
                .body(Body::from(json!({ "value": value, "n": n }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

async fn status_of(uri: &str) -> StatusCode {
    app(DEAD_URL)
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap()
        .status()
}

fn result_f64(body: &Value) -> f64 {
    body["result"].as_f64().expect("result is a float")
}

// --- Standard endpoints. ---

#[tokio::test]
async fn healthz_ok() {
    assert_eq!(status_of("/healthz").await, StatusCode::OK);
}

#[tokio::test]
async fn readyz_reflects_state() {
    health::set_ready(true);
    assert_eq!(status_of("/readyz").await, StatusCode::OK);
}

#[tokio::test]
async fn metrics_ok() {
    assert_eq!(status_of("/metrics").await, StatusCode::OK);
}

#[tokio::test]
async fn openapi_ok() {
    assert_eq!(status_of("/openapi.json").await, StatusCode::OK);
}

#[tokio::test]
async fn generates_request_id_when_absent() {
    let res = app(DEAD_URL)
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(
        res.headers().contains_key("x-request-id"),
        "response must carry a generated x-request-id"
    );
}

#[tokio::test]
async fn index_reports_identity() {
    let res = app(DEAD_URL)
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["service"], "srvcs-nthroot");
    assert_eq!(
        body["concern"],
        "arithmetic: nth root of value (alias of root)"
    );
    assert_eq!(body["depends_on"], json!(["srvcs-root"]));
}

// --- Correctness cases, against the computing mock. ---

#[tokio::test]
async fn nthroot_27_3_is_3() {
    let root = spawn_root().await;
    let (status, body) = nthroot(&root, 27.0, 3.0).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["value"], 27.0);
    assert_eq!(body["n"], 3.0);
    // 27 ** (1/3) == 3.0
    assert!((result_f64(&body) - 3.0).abs() < 1e-9);
}

#[tokio::test]
async fn nthroot_16_2_is_4() {
    let root = spawn_root().await;
    let (status, body) = nthroot(&root, 16.0, 2.0).await;
    assert_eq!(status, StatusCode::OK);
    // 16 ** (1/2) == 4.0
    assert!((result_f64(&body) - 4.0).abs() < 1e-9);
}

#[tokio::test]
async fn nthroot_fractional_result() {
    let root = spawn_root().await;
    let (status, body) = nthroot(&root, 2.0, 2.0).await;
    assert_eq!(status, StatusCode::OK);
    // 2 ** (1/2) == sqrt(2)
    let expected = 2.0_f64.powf(0.5);
    assert!((result_f64(&body) - expected).abs() < 1e-9);
}

#[tokio::test]
async fn nthroot_value_one_is_one() {
    let root = spawn_root().await;
    let (status, body) = nthroot(&root, 1.0, 5.0).await;
    assert_eq!(status, StatusCode::OK);
    // 1 ** (1/n) == 1.0 for any n
    assert!((result_f64(&body) - 1.0).abs() < 1e-9);
}

// --- Error / degraded paths. ---

#[tokio::test]
async fn degrades_when_root_unreachable() {
    let (status, body) = nthroot(DEAD_URL, 27.0, 3.0).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["dependency"], "srvcs-root");
}

#[tokio::test]
async fn forwards_422_from_root() {
    let root = spawn_fixed(
        StatusCode::UNPROCESSABLE_ENTITY,
        json!({ "error": "value is not a number" }),
    )
    .await;
    let (status, _) = nthroot(&root, 27.0, 3.0).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}
