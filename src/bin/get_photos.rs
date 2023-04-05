use anyhow::Result;
use lambda_http::{run, Body, Error, Request, RequestExt, Response};
use lazy_static::lazy_static;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::{env, str};
use tracing::info;

mod utils {
    pub mod lambda;
    pub mod responses;
}

use utils::{lambda, responses::*};

const KEY: &str = "scanned_data.json";
const SHOW_IMAGE_COUNT: usize = 100;

#[derive(Deserialize, Serialize)]
struct Photo {
    extension: String,
    size: Size,
    file_id: String,
    timestamp: String,
}

#[derive(Deserialize, Serialize)]
struct Size {
    width: f64,
    height: f64,
}

#[rustfmt::skip]
lazy_static! {
    static ref CACHE_BUCKET_NAME: String = env::var("CACHE_BUCKET_NAME").expect("'CACHE_BUCKET_NAME' env var is not set.");
}

async fn lambda_handler(request: Request) -> Result<Response<Body>, Error> {
    let context = request.lambda_context();
    lambda::log_incoming_event(&request, context);

    // TODO: Use `struct` to define globally -> たぶん main に書けばいい！！！けどその場合は引数の取り回しが課題だね。実行時間で比較だな
    // 前GPTに教えてもらったコードを試す。同期で定義できるクライアントがあるならそれでいいけど、ダメならあの複雑なやつをうまく抽象化したい。
    let config = aws_config::load_from_env().await;
    let s3 = aws_sdk_s3::Client::new(&config);

    let res = s3
        .get_object()
        .bucket(&*CACHE_BUCKET_NAME)
        .key(KEY)
        .send()
        .await
        .expect("An error has occurred when calling `s3.get_object()` .");

    let data = res.body.collect().await?.into_bytes();
    let photos = str::from_utf8(&data)?;
    let photos: Vec<Photo> = serde_json::from_str(photos)?;

    let mut rng = rand::thread_rng();
    let photos: Vec<&Photo> = photos
        .choose_multiple(
            &mut rng,
            if SHOW_IMAGE_COUNT <= photos.len() {
                SHOW_IMAGE_COUNT
            } else {
                photos.len()
            },
        )
        .collect();

    let result = serde_json::to_string(&photos)?;

    info!(result);
    _200(result)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(lambda::init_app(lambda_handler)).await
}
