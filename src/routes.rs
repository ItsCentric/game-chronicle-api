use warp::Filter;

use crate::handlers;

pub fn routes() -> impl Filter<Extract = impl warp::Reply, Error = std::convert::Infallible> + Clone
{
    get_csv()
}

fn get_csv() -> impl Filter<Extract = impl warp::Reply, Error = std::convert::Infallible> + Clone {
    warp::path!("csv" / String)
        .and_then(handlers::get_endpoint_csv_dump)
        .recover(handlers::handle_rejection)
}
