mod env;
mod handler;
mod server;
mod state;

use {actix_rt::System, env_logger, server::start, std::io::Result};

fn main() -> Result<()> {
    env_logger::init();
    System::new().block_on(async { start().await })
}
