use anyhow::{Error, Result};
use base64::{self, engine::general_purpose, Engine};
use http::{
    header::{self, HeaderName},
    HeaderValue,
};
use image::{self, ImageFormat};
use lambda_http::{
    http::Method, lambda_runtime::Context, request::RequestContext, service_fn,
    tower::util::ServiceFn, Body, Error as LambdaError, Request, RequestExt, Response,
};
use regex::Regex;
use std::{env, future::Future, io::Cursor, str::FromStr};
use tower_http::cors::{Cors, CorsLayer};
use tracing::{info, Level};

pub fn init_app<H, F>(lambda_handler: H) -> Cors<ServiceFn<H>>
where
    H: Fn(Request) -> F,
    F: Future<Output = Result<Response<Body>, LambdaError>>,
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
                .expect("\"API_ALLOW_ORIGIN\" env var is not set.")
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

pub fn log_incoming_event(request: &Request, context: Context) {
    if let RequestContext::ApiGatewayV1(request_context) = request.request_context() {
        info!(
            request_id = %context.request_id,
            request_time = %request_context.request_time.unwrap_or_else(|| "No request time".to_string()),
            http_version = ?request.version(),
            method = ?request.method(),
            resource_path = %request_context.resource_path.unwrap_or_else(|| "No resource path".to_string()),
            fullpath = %request.uri(),
            path = %request_context.path.unwrap_or_else(|| "No path".to_string()),
            path_params = ?request.path_parameters(),
            query_params = ?request.query_string_parameters(),
            headers = ?request.headers(),
            body = ?request.body(),
            api_id = %request_context.apiid.unwrap_or_else(|| "No api id".to_string()),
            domain_name = %request_context.domain_name.unwrap_or_else(|| "No domain name".to_string()),
            stage = %request_context.stage.unwrap_or_else(|| "No stage".to_string()),
            stage_variables = ?request.stage_variables(),
            apigw_request_id = %request_context.request_id.unwrap_or_else(|| "No API Gateway request id".to_string()),
            apigw_resource_id = %request_context.resource_id.unwrap_or_else(|| "No API Gateway resource id".to_string()),
            source_ip = %request_context.identity.source_ip.unwrap_or_else(|| "No source ip".to_string()),
            ua = %request_context.identity.user_agent.unwrap_or_else(|| "No user agent".to_string()),
            authorizer = ?request_context.authorizer, // Structuring?
            log_stream = %context.env_config.log_stream,
            log_group = %context.env_config.log_group,
            xray_trace_id = %context.xray_trace_id.unwrap_or_else(|| "None".to_string()),
            client_context = ?context.client_context,
            invoked_function_arn = %context.invoked_function_arn,
            function_name = %context.env_config.function_name,
            version = %context.env_config.version,
            memory = ?context.env_config.memory,
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
            body = ?request.body(),
            log_stream = %context.env_config.log_stream,
            log_group = %context.env_config.log_group,
            xray_trace_id = %context.xray_trace_id.unwrap_or_else(|| "None".to_string()),
            client_context = ?context.client_context,
            invoked_function_arn = %context.invoked_function_arn,
            function_name = %context.env_config.function_name,
            version = %context.env_config.version,
            memory = ?context.env_config.memory,
            deadline = ?context.deadline,
            "Lambda incoming event:",
        );
    }
}

#[derive(PartialEq, Eq)]
#[allow(dead_code)]
pub enum Extension {
    JPG,
    PNG,
}

#[allow(dead_code)]
pub fn save_image_in_temp(
    input_b64: &str,
    file_name: &str,
    want_ext: Extension,
    size: Option<(u32, u32)>,
) -> Result<()> {
    let prefix = Regex::new(r"^data:image/(.*?);base64,").unwrap();

    let input_ext = if let Some(captures) = prefix.captures(input_b64) {
        if let Some(input_ext) = captures.get(1) {
            input_ext.as_str()
        } else {
            return Err(Error::msg(
                "This base64 data does not include the extension of images.",
            ));
        }
    } else {
        return Err(Error::msg("Invalid base64 format of the image."));
    };

    let b64_payload = prefix.replace(input_b64, "").into_owned();
    let image_bytes = &general_purpose::STANDARD.decode(b64_payload.as_bytes())?;

    let mut image = image::load(
        Cursor::new(image_bytes),
        ImageFormat::from_extension(input_ext)
            .expect("Failed `ImageFormat::from_extension()` with `input_ext`"),
    )?;

    if let Some((width, height)) = size {
        image = image.resize(width, height, image::imageops::FilterType::CatmullRom);
    }

    let temp_path = format!(
        "/tmp/{}.{}",
        file_name,
        match want_ext {
            Extension::JPG => "jpg",
            Extension::PNG => "png",
        }
    );

    image.save(temp_path).unwrap();

    Ok(())
}
