//! A websocket ticker client based on `async-tungstenite` and `async-std`.
//!
//! ```bash
//! cargo run --example ws_client <market_name>
//! ```

use std::time::Duration;

use anyhow::{bail, Result};
use async_std::task;
use async_stream::stream;
use async_tungstenite::async_std::connect_async;
use async_tungstenite::tungstenite::Message;
use env_logger::{Builder as EnvLoggerBuilder, Env as EnvLoggerEnv};
use futures::{pin_mut, select, sink::SinkExt, stream::StreamExt};
use log::*;

use maicoin_max::v2::ws::{ServerPushEvent, SubRequest, BASE_URL};

fn init_log() {
    let env = EnvLoggerEnv::new()
        .filter_or("LOG_LEVEL", "info")
        .write_style("LOG_STYLE");
    EnvLoggerBuilder::from_env(env).init();
}

fn handle_push(raw: String) {
    if let Ok(event) = serde_json::from_str::<ServerPushEvent>(raw.as_str()) {
        match event {
            ServerPushEvent::Error(err) => error!("error while receiving feed: {:?}", err),
            ServerPushEvent::PubTickerFeed(feed) => println!("{:?}", feed),
            event => error!("unexpected feed: {:?}", event),
        }
    } else {
        error!("failed to parse server event: {}", raw);
    }
}

fn main() -> Result<()> {
    init_log();

    let market = std::env::args()
        .nth(1)
        .expect("usage: ws_client <market_name>");
    task::block_on(async {
        // Connect to the server.
        let mut stream = connect_async(BASE_URL).await?.0.fuse();

        // subscribe
        let req = {
            let mut sub = SubRequest::new_sub(String::new());
            sub.subset().insert_ticker(market);
            serde_json::to_string(&sub)?
        };
        stream.send(Message::text(req)).await?;
        if let Some(Ok(Message::Text(resp))) = stream.next().await {
            match serde_json::from_str::<ServerPushEvent>(resp.as_str())? {
                ServerPushEvent::Error(err) => bail!("error while submitting ticker: {:?}", err),
                ServerPushEvent::SubResp(_) => {}
                event => bail!("unexpected response: {:?}", event),
            };
        } else {
            bail!("fail to get response for ticker submition");
        };

        // heartbeat ticker
        let ticker = stream! {
            loop {
                task::sleep(Duration::from_secs(30)).await;
                yield ();
            }
        };
        pin_mut!(ticker);

        loop {
            select! {
                _ = ticker.next() => {
                    if let Err(err) = stream.send(Message::Ping("heartbeat".into())).await {
                        error!("error while sending heartbeat: {:?}", err);
                    } else {
                        debug!("sending heartbeat to server");
                    }
                }
                recv = stream.next() => {
                    if let Some(Ok(recv_entry)) = recv {
                        match recv_entry {
                            Message::Text(feed) => handle_push(feed),
                            Message::Pong(_) => {}, // ignore heartbeat
                            x => error!("receiving unexpected push: {:?}", x),
                        }
                    } else {
                        info!("stream terminated");
                        break;
                    }
                }
            };
        }
        Ok(())
    })
}
