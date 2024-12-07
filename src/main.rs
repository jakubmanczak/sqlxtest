use std::{
    net::{Ipv4Addr, SocketAddrV4},
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    println!(
        "Is it now {:?}",
        std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );

    let pool = match sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:dbfile.db")
        .await
    {
        Ok(p) => {
            println!("db pool works..");
            p
        }
        Err(e) => panic!("fuck!!! {e}"),
    };

    match sqlx::migrate!("./migrations").run(&pool).await {
        Ok(_) => println!("migrations ran!"),
        Err(e) => panic!("fuck!!! {e}"),
    };

    let app = Router::new()
        .route("/", get(okay))
        .route("/get", get(get_records))
        .route("/post", post(post_record))
        .with_state(pool);
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 2004))
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}

#[derive(Serialize, Deserialize)]
struct Record {
    id: Option<i64>,
    text: Option<String>,
    num: Option<i64>,
}

async fn okay() -> () {
    ()
}

async fn get_records(State(pool): State<Pool<Sqlite>>) -> Response {
    let records = sqlx::query_as!(Record, "SELECT id, text, num FROM records")
        .fetch_all(&pool)
        .await
        .expect("failed to fetch em!");

    Json(records).into_response()
}

async fn post_record(State(pool): State<Pool<Sqlite>>, Json(body): Json<Record>) -> Response {
    let id = body.id.unwrap_or(
        std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
    );
    let text = body.text.unwrap_or("".into());
    let num = body.num.unwrap_or(0);

    match sqlx::query!("INSERT INTO records VALUES (?, ?, ?)", id, text, num)
        .execute(&pool)
        .await
    {
        Ok(_) => Json(Record {
            id: Some(id),
            text: Some(text),
            num: Some(num),
        })
        .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
