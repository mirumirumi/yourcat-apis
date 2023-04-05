use anyhow::Result;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_lambda::types::InvocationType;
use aws_sdk_s3::primitives::ByteStream;
use chrono::Utc;
use chrono_tz::Asia::Tokyo;
use lambda_http::{run, Body, Error, Request, RequestExt, Response};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::io::Read;
use std::{env, fs::File};
use tracing::info;
use uuid::Uuid;

mod utils {
    pub mod lambda;
    pub mod responses;
}

use utils::{lambda, responses::*};

#[derive(Deserialize)]
struct ImageData {
    image_data: ImageDataInfo,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct ImageDataInfo {
    base64: String,
    width: i32,
    height: i32,
}

#[derive(Deserialize, Serialize)]
struct TableItem {
    file_id: String,
    extension: String,
    size: Size,
    timestamp: String,
}

#[derive(Deserialize, Serialize)]
struct Size {
    width: f64,
    height: f64,
}

impl From<HashMap<String, AttributeValue>> for TableItem {
    #[rustfmt::skip]
    fn from(map: HashMap<String, AttributeValue>) -> Self {
        let file_id = map.get("file_id").unwrap().as_s().unwrap().to_string();
        let extension = map.get("extension").unwrap().as_s().unwrap().to_string();
        let size_map = map.get("size").unwrap().as_m().unwrap();
        let width = size_map.get("width").unwrap().as_n().unwrap().parse::<f64>().unwrap();
        let height = size_map.get("height").unwrap().as_n().unwrap().parse::<f64>().unwrap();
        let timestamp = map.get("timestamp").unwrap().as_s().unwrap().to_string();
        TableItem { file_id, extension, size: Size { width, height }, timestamp }
    }
}

#[rustfmt::skip]
lazy_static! {
    static ref IMAGE_BUCKET_NAME: String = env::var("IMAGE_BUCKET_NAME").expect("'IMAGE_BUCKET_NAME' env var is not set.");
    static ref CACHE_BUCKET_NAME: String = env::var("CACHE_BUCKET_NAME").expect("'CACHE_BUCKET_NAME' env var is not set.");
    static ref IMAGE_TABLE_NAME: String = env::var("IMAGE_TABLE_NAME").expect("'IMAGE_TABLE_NAME' env var is not set.");
    static ref INVOKE_LAMBDA_NAME: String = env::var("INVOKE_LAMBDA_NAME").expect("'INVOKE_LAMBDA_NAME' env var is not set.");
}

async fn lambda_handler(request: Request) -> Result<Response<Body>, Error> {
    let context = request.lambda_context();
    lambda::log_incoming_event(&request, context);

    let config = aws_config::load_from_env().await;
    let s3 = aws_sdk_s3::Client::new(&config);
    let dynamodb = aws_sdk_dynamodb::Client::new(&config);
    let lambda = aws_sdk_lambda::Client::new(&config);

    let key = Uuid::new_v4().to_string();
    let ext = "jpg";
    let file_name = format!("{}.{}", key, ext);

    let payload = match request.body() {
        Body::Text(text) => text.clone(),
        _ => panic!("The image data is not included in request."),
    };

    let image_data: ImageData = serde_json::from_str(&payload)?;

    lambda::save_image_in_temp(
        &image_data.image_data.base64,
        &key,
        match ext {
            "jpg" => lambda::Extension::JPG,
            "jpeg" => lambda::Extension::JPG,
            "png" => lambda::Extension::PNG,
            _ => panic!("Invalid image file extension input."),
        },
        None,
    )?;

    let now = Utc::now().with_timezone(&Tokyo).to_rfc3339();
    let width = image_data.image_data.width;
    let height = image_data.image_data.height;
    let ext = format!(".{}", ext);

    // Save the image data to DB
    dynamodb
        .put_item()
        .table_name(&*IMAGE_TABLE_NAME)
        .item("file_id", AttributeValue::S(key))
        .item("extension", AttributeValue::S(ext.to_string()))
        .item(
            "size",
            AttributeValue::M({
                let mut map = HashMap::new();
                map.insert("width".to_string(), AttributeValue::N(width.to_string()));
                map.insert("height".to_string(), AttributeValue::N(height.to_string()));
                map
            }),
        )
        .item("timestamp", AttributeValue::S(now))
        .send()
        .await?;

    // Save the image file to S3
    let mut file = File::open(format!("/tmp/{}", file_name))?;
    s3.put_object()
        .bucket(&*IMAGE_BUCKET_NAME)
        .key(&file_name)
        .body({
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            ByteStream::from(buffer)
        })
        .content_type("image/jpeg")
        .cache_control("max-age=31536000")
        .send()
        .await?;

    // Don't remove file in `/tmp/` because the file name is UUID which guarantees uniqueness enough.

    // Tweet by the bot
    lambda
        .invoke()
        .function_name(&*INVOKE_LAMBDA_NAME)
        .invocation_type(InvocationType::Event)
        .payload(aws_sdk_lambda::primitives::Blob::new(serde_json::to_vec(
            &(json!({ "file_id": file_name })),
        )?))
        .send()
        .await?;

    // Scan entire table to update
    let res = dynamodb
        .scan()
        .table_name(&*IMAGE_TABLE_NAME)
        .send()
        .await?;

    let items: Vec<TableItem> = res
        .items()
        .unwrap()
        .iter()
        .map(|item| TableItem::from(item.clone()))
        .collect();

    s3.put_object()
        .bucket(&*CACHE_BUCKET_NAME)
        .key("scanned_data.json")
        .body(ByteStream::from(
            serde_json::to_string(&items)?.into_bytes(),
        ))
        .content_type("application/json")
        .send()
        .await?;

    let result = json!({
        "file_id": &file_name,
        "size": {
            "width": width,
            "height": height,
        },
    })
    .to_string();

    info!(result);
    _200(result)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(lambda::init_app(lambda_handler)).await
}
