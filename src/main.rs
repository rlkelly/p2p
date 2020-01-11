use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::net::TcpListener;
use tokio::time;

use music_snobster::handlers::{process, Service};
use music_snobster::handlers::scheduler::ping_all_peers;
use music_snobster::args::get_args;
use music_snobster::tui::run_tui;

// TODO: handle requests
//   - SEND FILE
//   - REQUEST FILE
//   - DOWNLOAD FILE

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = get_args();
    println!("{:?}", config);
    let state = Arc::new(Mutex::new(Service::new(config.clone())));
    let mut listener = TcpListener::bind(format!("127.0.0.1:{}", config.port)).await?;

    // text interface
    if config.tui {
        let tui_state = Arc::clone(&state);
        tokio::spawn(async move {
            run_tui(tui_state);
            std::process::exit(0);
        });
    }

    // regularly scheduled background tasks
    let scheduler_state = Arc::clone(&state);
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_millis(10_000));
        let sstate = Arc::clone(&scheduler_state);
        loop {
            let ss = Arc::clone(&sstate);
            interval.tick().await;
            ping_all_peers(ss).await;
        }
    });

    // process incoming requests
    loop {
        let (stream, addr) = listener.accept().await?;
        let state = Arc::clone(&state);

        tokio::spawn(async move {
            if let Err(e) = process(state, stream, addr).await {
                println!("an error occured; error = {:?}", e);
            }
        });
    }
}
