use anyhow::Result;
use lambda_http::{Body, Error, Response};

fn base_response_builder() -> http::response::Builder {
    Response::builder().header("Content-Type", "application/json")
}

pub fn _200(body: impl Into<Body>) -> Result<Response<Body>, Error> {
    Ok(base_response_builder().status(200).body(body.into())?)
}

pub fn _201(body: impl Into<Body>) -> Result<Response<Body>, Error> {
    Ok(base_response_builder().status(201).body(body.into())?)
}

// Since there is little need to write messages indicating the contents of errors,
// and for precise error handling, it is necessary to obtain structured data from the response content,
// there is room to consider dropping the msg portion altogether.

pub fn _400(msg: &str) -> Result<Response<Body>, Error> {
    // Invalid Request

    Ok(base_response_builder()
        .status(400)
        .body(format!("Invalid Request: {}", msg).into())?)
}

pub fn _401(msg: &str) -> Result<Response<Body>, Error> {
    // Unauthorized

    Ok(base_response_builder()
        .status(401)
        .body(format!("Unauthorized: {}", msg).into())?)
}

pub fn _403(msg: &str) -> Result<Response<Body>, Error> {
    // Forbidden

    Ok(base_response_builder()
        .status(403)
        .body(format!("Forbidden: {}", msg).into())?)
}

pub fn _404(msg: &str) -> Result<Response<Body>, Error> {
    // Not Found

    Ok(base_response_builder()
        .status(404)
        .body(format!("Not Found: {}", msg).into())?)
}

pub fn _409(body: impl Into<Body>) -> Result<Response<Body>, Error> {
    // Conflict

    Ok(base_response_builder().status(409).body(body.into())?)
}

pub fn _415(body: impl Into<Body>) -> Result<Response<Body>, Error> {
    // Unsupported Media Type

    Ok(base_response_builder().status(415).body(body.into())?)
}
