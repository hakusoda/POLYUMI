use env_logger::Env;
use log::info;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> std::io::Result<()> {
	env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
	
	info!("starting polyumi service");

	polyumi_frontend::setup_frontend()
		.await
}