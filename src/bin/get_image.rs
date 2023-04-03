use anyhow::{Context, Result};
use lambda_http::{run, Body, Error, Request, RequestExt, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use thiserror::Error;
use tracing::{debug, error, info};

mod utils {
    pub mod lambda;
    pub mod responses;
}

use utils::{lambda, responses::*};

async fn lambda_handler(request: Request) -> Result<Response<Body>, Error> {
    let context = request.lambda_context();
    let payload = request.payload::<String>();

    lambda::log_incoming_event(request, context, payload);

    _200("/get-image")
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(lambda::init_app(lambda_handler)).await
}
