use anyhow::{Context, Result};
use axum::{
    body::Body as AxumBody,
    extract::{Extension, Path, Query},
    response::{Json, Result as AxumResult},
    routing::{get, post},
    Router,
};
use http::{
    header::{self, HeaderName},
    Request as HttpRequest, Response as HttpResponse,
};
use hyper::body::to_bytes;
use lambda_http::{
    http::Method, request::RequestContext, run, service_fn, tower::ServiceBuilder, Body, Error,
    Request as LambdaRequest, RequestExt, Response,
};
// use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{env, str::FromStr};
use tower::ServiceExt;
use tower_http::cors::{Any, CorsLayer};
use tracing::{
    // debug,
    // error,
    info,
    Level,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let level = env::var("LOG_LEVEL").unwrap_or_else(|_| "debug".to_string());
    let level = Level::from_str(&level).unwrap();

    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_file(true)
        .with_line_number(true)
        .json()
        .init();

    run(service_fn(lambda_handler)).await
}

async fn lambda_handler(request: LambdaRequest) -> Result<Response<Body>, Error> {
    let context = request.lambda_context();
    let payload = request.payload::<String>();

    // もとの request_contex すべて（いまログに出しているのは全てではない）
    // let request_context = request.request_context();
    // println!(
    //     "{:#?}",
    //     serde_json::to_string(&request_context)
    //         .unwrap_or_else(|_| "No request context".to_string())
    // );

    // {proxy+} だとパスパラメータ出さないとリクエストルート全くわからない！！

    if let RequestContext::ApiGatewayV1(request_context) = request.request_context() {
        info!(
            request_id = %context.request_id,
            http_version = ?request.version(),
            method = ?request.method(),
            resource_path = ?request_context.resource_path,
            path = ?request_context.path,
            fullpath = %request.uri(),
            headers = ?request.headers(),
            payload = ?payload,
            // payload = ?request.body(),
            domain_name = ?request_context.domain_name,
            stage_name = ?request_context.stage,
            apigw_request_id = ?request_context.request_id,
            source_ip = ?request_context.identity.source_ip,
            ua = ?request_context.identity.user_agent,
            log_stream = %context.env_config.log_stream,
            log_group = %context.env_config.log_group,
            xray_trace_id = %context.xray_trace_id.unwrap_or_else(|| "None".to_string()),
            client_context = ?context.client_context,
            invoked_function_arn = %context.invoked_function_arn,
            function_name = %context.env_config.function_name,
            version = %context.env_config.version,
            memory = ?context.env_config.memory,
            identity = ?context.identity,
            deadline = ?context.deadline,
            "Lambda incoming event:",
        );
    } else {
        info!(
            request_id = %context.request_id,
            http_version = ?request.version(),
            method = ?request.method(),
            fullpath = %request.uri(),
            headers = ?request.headers(),
            payload = ?payload,
            // payload = ?request.body(),
            log_stream = %context.env_config.log_stream,
            log_group = %context.env_config.log_group,
            xray_trace_id = %context.xray_trace_id.unwrap_or_else(|| "None".to_string()),
            client_context = ?context.client_context,
            invoked_function_arn = %context.invoked_function_arn,
            function_name = %context.env_config.function_name,
            version = %context.env_config.version,
            memory = ?context.env_config.memory,
            identity = ?context.identity,
            deadline = ?context.deadline,
            "Lambda incoming event:",
        );
    }

    let cors_layer = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(vec![
            header::AUTHORIZATION,
            "Cognito-Auth-Header".parse::<HeaderName>().unwrap(),
            header::CONTENT_TYPE,
            "X-Amz-Date".parse::<HeaderName>().unwrap(),
            "X-Amz-Security-Token".parse::<HeaderName>().unwrap(),
            "X-Api-Key".parse::<HeaderName>().unwrap(),
        ])
        .allow_methods(vec![
            Method::POST,
            Method::GET,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ]);

    let router = Router::new()
        .route("/get-image", get(get_image))
        .route("/get-is-cat", get(get_is_cat))
        .route("/post-photo", post(post_photo));

    let app = ServiceBuilder::new()
        .layer(cors_layer.clone())
        .service(router.clone());

    // `lambda_http::Request` を `http::Request` に変換
    let (parts, body) = request.into_parts();

    let http_request = HttpRequest::from_parts(parts, Body::from(body));

    // `axum::Router` にリクエストを渡してレスポンスを取得
    let http_response = app.oneshot(http_request).await?;

    // `http::Response` を `lambda_http::Response` に変換して返す
    let (parts, body) = HttpResponse::from(http_response).into_parts();
    let bytes = to_bytes(body).await?;
    let response = Response::from_parts(parts, Body::from(bytes.as_ref().to_vec()));

    Ok(response)
}

async fn get_image() -> Json<Value> {
    Json(json!({ "msg": "I am get_image /" }))
}

async fn get_is_cat() -> Json<Value> {
    Json(json!({ "msg": "I am get_is_cat /" }))
}

async fn post_photo() -> Json<Value> {
    Json(json!({ "msg": "I am post_photo /" }))
}
