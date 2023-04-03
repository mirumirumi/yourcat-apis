use anyhow::Result;
use lambda_http::{Body, Error, Response};

// TODO: Common response headers (and support multiple)

pub fn _200(body: impl Into<Body>) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(body.into())?)
}

pub fn _201(body: impl Into<Body>) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(201)
        .header("Content-Type", "application/json")
        .body(body.into())?)
}

// Since there is little need to write messages indicating the contents of errors,
// and for precise error handling, it is necessary to obtain structured data from the response content,
// there is room to consider dropping the msg portion altogether.

pub fn _400(msg: &str) -> Result<Response<Body>, Error> {
    // Invalid Request

    Ok(Response::builder()
        .status(400)
        .header("Content-Type", "application/json")
        .body(format!("Invalid Request: {}", msg).into())?)
}

pub fn _401(msg: &str) -> Result<Response<Body>, Error> {
    // Unauthorized

    Ok(Response::builder()
        .status(401)
        .header("Content-Type", "application/json")
        .body(format!("Unauthorized: {}", msg).into())?)
}

pub fn _403(msg: &str) -> Result<Response<Body>, Error> {
    // Forbidden

    Ok(Response::builder()
        .status(403)
        .header("Content-Type", "application/json")
        .body(format!("Forbidden: {}", msg).into())?)
}

pub fn _404(msg: &str) -> Result<Response<Body>, Error> {
    // Not Found

    Ok(Response::builder()
        .status(404)
        .header("Content-Type", "application/json")
        .body(format!("Not Found: {}", msg).into())?)
}

pub fn _409(body: impl Into<Body>) -> Result<Response<Body>, Error> {
    // Conflict

    Ok(Response::builder()
        .status(409)
        .header("Content-Type", "application/json")
        .body(body.into())?)
}

pub fn _415(body: impl Into<Body>) -> Result<Response<Body>, Error> {
    // Unsupported Media Type

    Ok(Response::builder()
        .status(415)
        .header("Content-Type", "application/json")
        .body(body.into())?)
}
