use anyhow::{Context, Result};
use lambda_http::{run, Body, Error, Request, RequestExt, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use thiserror::Error;
use tracing::{debug, error, info};

use yourcat::{init_app, log_lambda_event};

async fn lambda_handler(request: Request) -> Result<Response<Body>, Error> {
    let context = request.lambda_context();
    let payload = request.payload::<String>();

    log_lambda_event(request, context, payload);

    Ok(Response::builder().status(200).body("/get-is-cat".into())?)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(init_app(lambda_handler)).await
}
