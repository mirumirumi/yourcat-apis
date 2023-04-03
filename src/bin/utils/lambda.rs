use anyhow::Result;
use http::{
    header::{self, HeaderName},
    HeaderValue,
};
use lambda_http::{
    ext::PayloadError, http::Method, request::RequestContext, service_fn, tower::util::ServiceFn,
    Body, Error, Request, RequestExt, Response,
};
use lambda_runtime::Context;
use std::{env, future::Future, str::FromStr};
use tower_http::cors::{Cors, CorsLayer};
use tracing::{info, Level};

pub fn init_app<H, F>(lambda_handler: H) -> Cors<ServiceFn<H>>
where
    H: Fn(Request) -> F,
    F: Future<Output = Result<Response<Body>, Error>>,
{
    let level = env::var("LOG_LEVEL").unwrap_or_else(|_| "debug".to_string());
    let level = Level::from_str(&level).unwrap();

    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_file(true)
        .with_line_number(true)
        .json()
        .init();

    let cors_layer = CorsLayer::new()
        .allow_origin(
            env::var("API_ALLOW_ORIGIN")
                .expect("'API_ALLOW_ORIGIN' env var is not set.")
                .parse::<HeaderValue>()
                .unwrap(),
        )
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

    lambda_http::tower::ServiceBuilder::new()
        .layer(cors_layer)
        .service(service_fn(lambda_handler))
}

pub fn log_incoming_event(
    request: Request,
    context: Context,
    payload: Result<Option<String>, PayloadError>,
) {
    // All original `request_context` (not all of them are logged now)
    /*
    let request_context = request.request_context();
    println!(
        "{:#?}",
        serde_json::to_string(&request_context)
            .unwrap_or_else(|_| "No request context".to_string())
    );
    */

    // Note that in the case of `{proxy+}`,
    // the requested endpoint is not known at all unless the path parameter is logged

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
            payload_from_req_body = ?request.body(),
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
            payload_from_req_body = ?request.body(),
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
}
