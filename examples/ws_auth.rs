//! A websocket client to receive personal trade status.
//!
//! ```bash
//! cargo run --example ws_auth <api_key> <api_secret>
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

use maicoin_max::v2::ws::{AuthRequest, PrivFeedType, ServerPushEvent, BASE_URL};
use maicoin_max::Credentials;

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
            ServerPushEvent::PrivTradeFeed(feed) => {
                feed.trades
                    .into_iter()
                    .for_each(move |order| println!("{:?}", order));
            }
            event => error!("unexpected feed: {:?}", event),
        }
    } else {
        error!("failed to parse server event: {}", raw);
    }
}

fn main() -> Result<()> {
    init_log();

    let mut args = std::env::args().collect::<Vec<String>>().into_iter();
    args.next();
    let api_key = args.next().unwrap();
    let api_secret = args.next().unwrap();

    let credentials = Credentials::new(api_key, api_secret);

    task::block_on(async {
        // Connect to the server.
        let mut stream = connect_async(BASE_URL).await?.0.fuse();

        // subscribe
        let req = {
            let auth_req = AuthRequest::new(&credentials, None, Some(vec![PrivFeedType::Trade]));
            serde_json::to_string(&auth_req)?
        };
        stream.send(Message::text(req)).await?;
        if let Some(Ok(Message::Text(resp))) = stream.next().await {
            match serde_json::from_str::<ServerPushEvent>(dbg!(resp.as_str()))? {
                ServerPushEvent::Error(err) => bail!("error while submitting ticker: {:?}", err),
                ServerPushEvent::AuthResp(_) => {
                    info!("auth success")
                }
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
