use std::{path::PathBuf, net::{SocketAddr, IpAddr}};

use guoql::{context::Context, schema::schema};
use warp::Filter;

macro_rules! or {
    ($e1: expr, $($e: expr),+ $(,)?) => {
        $e1$(.or($e))+
    };
}

use clap::Parser;

#[derive(Parser)]
struct Args {
    /// 数据文件路径。
    #[clap(default_value = "./guoql.db")]
    db: PathBuf,

    /// 监听地址。
    #[clap(long, default_value = "127.0.0.1")]
    host: IpAddr,

    /// 监听端口。
    #[clap(long, default_value = "8080")]
    port: u16,

    /// 打开调试日志。
    #[clap(short, long)]
    debug: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let mut builder = env_logger::Builder::new();
    if args.debug {
        builder.filter_level(log::LevelFilter::Debug);
    } else {
        builder.filter_module("guoql", log::LevelFilter::Info);
    }
    builder.init();

    let context = Context::new(args.db).await;

    let routes = or![
        warp::path!().and(warp::get().map(|| {
            warp::http::Response::builder()
                .status(302)
                .header("Location", "/graphql")
                .body(warp::hyper::Body::empty())
        })),
        warp::path!("graphql").and(or![
            warp::post().and(juniper_warp::make_graphql_filter(
                schema(),
                warp::any().map(move || context.clone()).boxed(),
            )),
            warp::get().and(juniper_warp::playground_filter("/graphql", None)),
        ]),
    ]
    .with(warp::log("warp::server"));

    let addr = SocketAddr::new(args.host, args.port);
    log::info!("Listening on {addr}");
    warp::serve(routes).run(addr).await;
}
