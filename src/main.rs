use std::env;

use dotenv::dotenv;

mod handlers;
mod routes;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let routes = routes::routes();

    let port = env::var("PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse()
        .unwrap();
    println!("Server started at http://localhost:{}", port);
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}
