use actix_web::{get, middleware, web, App, HttpRequest, HttpResponse, HttpServer, Result};
use sqlx::sqlite::SqlitePool;
use serde::Serialize;
use time::{Duration, OffsetDateTime};
use std::env;

// static pixel data 1x1 transparent gif
const PIXEL: &[u8] = &[
    0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x00, 0x00, 0xFF, 0xFF, 0xFF,
    0x00, 0x00, 0x00, 0x21, 0xF9, 0x04, 0x01, 0x00, 0x00, 0x00, 0x00, 0x2C, 0x00, 0x00, 0x00, 0x00,
    0x01, 0x00, 0x01, 0x00, 0x00, 0x02, 0x02, 0x44, 0x01, 0x00, 0x3B,
];

// db connection pool shared across request handlers
#[derive(Debug)]
struct AppState {
    db: SqlitePool,
}

#[derive(Serialize)]
struct Stats {
    total_views: i64,
    views_today: i64,
    views_this_week: i64,
    recent_views: Vec<ViewLog>,
}

#[derive(Serialize)]
struct ViewLog {
    timestamp: OffsetDateTime,
    camo_id: String,
    user_agent: String,
}

// stats endpoint to show data
#[get("/stats")]
async fn get_stats(state: web::Data<AppState>) -> Result<HttpResponse> {
    let now = OffsetDateTime::now_utc();
    let day_ago = now - Duration::days(1);
    let week_ago = now - Duration::days(7);

    // get total views
    let total_views: i64 = sqlx::query!(
        "SELECT COUNT(*) AS count FROM pixel_hits" 
    )
        .fetch_one(&state.db)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
        .count
        .into();

    // get views today
    let views_today: i64 = sqlx::query!(
        "SELECT COUNT(*) AS count FROM pixel_hits WHERE timestamp > ?",
        day_ago
    )
        .fetch_one(&state.db)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
        .count
        .into();

    // get views this week
    let views_this_week: i64 = sqlx::query!(
        "SELECT COUNT(*) AS count FROM pixel_hits WHERE timestamp > ?",
        week_ago
    )
        .fetch_one(&state.db)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
        .count
        .into();

    // get recent views
    let recent_views = sqlx::query!(
        r#"
        SELECT timestamp, camo_id, user_agent
        FROM pixel_hits
        ORDER BY timestamp DESC
        LIMIT 10
        "#
    )
        .fetch_all(&state.db)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
        .into_iter()
        .map(|row| ViewLog {
            timestamp: row.timestamp,
            camo_id: row.camo_id.unwrap_or_default(),
            user_agent: row.user_agent.unwrap_or_default(),
        })
        .collect();

    let stats = Stats {
        total_views,
        views_today,
        views_this_week,
        recent_views,
    };

    Ok(HttpResponse::Ok().json(stats))
}

// pixel endpoint
#[get("/pixel.gif")]
async fn pixel(req: HttpRequest, state: web::Data<AppState>) -> Result<HttpResponse> {

    println!("All headers:");
    for (header_name, header_value) in req.headers() {
        println!("{}: {:?}", header_name, header_value);
    }

    let user_agent = req
        .headers()
        .get("User-Agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let is_github_request = user_agent.contains("github-camo");

    if !is_github_request {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let camo_id = if user_agent.starts_with("github-camo") {
        user_agent
            .trim_start_matches("github-camo (")
            .trim_end_matches(")")
            .to_string()
    } else {
        // should never happen
        "non-github".to_string()
    };

    let ip = req
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("unknown")
        .to_string();

    let timestamp = OffsetDateTime::now_utc();

    // store view
    sqlx::query!(
        r#"
        INSERT INTO pixel_hits (ip_address, user_agent, camo_id, timestamp)
        VALUES (?, ?, ?, ?)
        "#,
        ip,
        user_agent,
        camo_id,
        timestamp
    )
        .execute(&state.db)
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    Ok(HttpResponse::Ok()
        .insert_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
        .content_type("image/gif")
        .insert_header(("Pragma", "no-cache"))
        .insert_header(("Expires", "0"))
        .body(PIXEL.to_vec()))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Hello, world!");
    // get database url from environment variable or use default
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:pulse.db".to_string());
    let pool = SqlitePool::connect(&database_url)
        .await
        .expect("Failed to create pool");

    // doing sqlx migrations so no need:
    // create tables if they don't exist
    // sqlx::query(include_str!("../migrations/<initial-migration>.sql"))
    //    .execute(&pool)
    //    .await
    //    .expect("Failed to create database tables");

    // start http server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState { db: pool.clone() }))
            .wrap(middleware::Logger::default())
            .service(pixel)
            .service(get_stats)
    })
    .bind(
        (env::var("HOST").unwrap_or("0.0.0.0".to_string()), 
        env::var("PORT").unwrap_or("8080".to_string()).parse::<u16>().unwrap()))?
    .run()
    .await
}
