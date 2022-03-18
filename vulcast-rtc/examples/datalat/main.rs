use serde::Serialize;
use statrs::statistics::{Data, Distribution, Max, Min, OrderStatistics};
use std::{collections::HashMap, convert::TryInto, sync::Arc, time::Duration};
use tokio_stream::StreamExt;

use clap::Parser;
use graphql_ws::GraphQLWebSocket;
use http::Uri;
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::Connector;

use crate::graphql_signaller::GraphQLSignaller;
use vulcast_rtc::broadcaster::Broadcaster;

mod graphql_signaller;
mod signal_schema;

#[derive(Serialize)]
struct SessionToken {
    token: String,
}

#[derive(Debug, Parser)]
pub struct Opts {
    /// Listening address for signal endpoint (domain required).
    #[clap(long, default_value = "wss://localhost:8443")]
    pub signal_addr: String,
    /// Pre-authorized access token for Vulcast.
    #[clap(long)]
    pub vulcast_token: String,
    /// Pre-authorized access token for Client.
    #[clap(long)]
    pub client_token: String,
    /// Disable TLS.
    #[clap(long)]
    pub no_tls: bool,

    /// Send count
    #[clap(short, long)]
    pub count: u32,
    /// Send interval in milliseconds
    #[clap(short, long)]
    pub interval: u64,
}

async fn create_signalling_connection(
    signal_addr: &str,
    token: String,
    no_tls: bool,
) -> Result<GraphQLWebSocket, Box<dyn std::error::Error>> {
    struct PromiscuousServerVerifier;
    impl rustls::client::ServerCertVerifier for PromiscuousServerVerifier {
        fn verify_server_cert(
            &self,
            _end_entity: &rustls::Certificate,
            _intermediates: &[rustls::Certificate],
            _server_name: &rustls::ServerName,
            _scts: &mut dyn Iterator<Item = &[u8]>,
            _ocsp_response: &[u8],
            _now: std::time::SystemTime,
        ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
            // here be dragons
            Ok(rustls::client::ServerCertVerified::assertion())
        }
    }
    let client_config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(Arc::new(PromiscuousServerVerifier))
        .with_no_client_auth();

    let uri: Uri = signal_addr.parse()?;

    let host = uri.host().unwrap();
    let port = uri.port_u16().unwrap();
    let stream = TcpStream::connect((host, port)).await?;

    let req = http::Request::builder()
        .uri(uri)
        .header("Sec-WebSocket-Protocol", "graphql-ws")
        .body(())?;
    let (socket, _response) = tokio_tungstenite::client_async_tls_with_config(
        req,
        stream,
        None,
        Some(if no_tls {
            Connector::Plain
        } else {
            Connector::Rustls(Arc::new(client_config))
        }),
    )
    .await?;

    Ok(GraphQLWebSocket::new(
        socket,
        Some(serde_json::to_value(SessionToken { token })?),
    ))
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let opts: Opts = Opts::parse();

    let vulcast_conn =
        create_signalling_connection(&opts.signal_addr, opts.vulcast_token.clone(), opts.no_tls)
            .await?;
    let client_conn =
        create_signalling_connection(&opts.signal_addr, opts.client_token.clone(), opts.no_tls)
            .await?;
    let vulcast_gql_signaller = Arc::new(GraphQLSignaller::new(vulcast_conn.clone()));
    let client_gql_signaller = Arc::new(GraphQLSignaller::new(client_conn.clone()));
    let vulcast_broadcaster = Broadcaster::new(vulcast_gql_signaller.clone()).await;
    let client_broadcaster = Broadcaster::new(client_gql_signaller.clone()).await;
    let mut client_data_producer = client_broadcaster.produce_data().await;
    let mut vulcast_data_consumer = vulcast_broadcaster
        .consume_data(client_data_producer.id().clone())
        .await
        .unwrap();

    #[derive(Debug)]
    struct State {
        send_time: HashMap<u32, std::time::Instant>,
        lat: Vec<f64>,
        iat: Vec<f64>,
        ooo: u32,
        count: u32
    }
    let state = Arc::new(Mutex::new(State {
        send_time: HashMap::new(),
        lat: vec![],
        iat: vec![],
        ooo: 0,
        count: 0
    }));

    println!("{:#?}", opts);

    let j1 = tokio::spawn({
        let state = state.clone();
        let count = opts.count;
        let interval = opts.interval;
        async move {
            tokio::time::sleep(Duration::from_secs(1)).await;
            for i in (0..count).rev() {
                let mut state = state.lock().await;
                let State { send_time, .. } = &mut *state;
                client_data_producer.send(i.to_le_bytes().to_vec()).unwrap();
                send_time.insert(i, std::time::Instant::now());
                drop(state);

                if interval != 0 {
                    tokio::time::sleep(Duration::from_millis(interval)).await;
                }
            }
        }
    });
    let j2 = tokio::spawn({
        let state = state.clone();
        async move {
            let mut last_id = None;
            let mut last_arrival: Option<std::time::Instant> = None;
            while let Some(message) = vulcast_data_consumer.next().await {
                let i = u32::from_le_bytes(message.try_into().unwrap());

                let mut state = state.lock().await;
                let State {
                    send_time,
                    lat,
                    iat,
                    ooo,
                    count
                } = &mut *state;

                let now = std::time::Instant::now();

                // latency
                let start = *send_time.get(&i).unwrap();
                let int = now - start;
                lat.push(int.as_micros() as f64);

                // inter-arrival time
                if let Some(last) = last_arrival {
                    let del = now - last;
                    iat.push(del.as_micros() as f64);
                }
                last_arrival = Some(now);

                if let Some(last_id) = last_id {
                    if i > last_id {
                        *ooo += 1;
                    }
                }
                last_id = Some(i);
                *count += 1;

                drop(state);

                if i == 0 {
                    break;
                }
            }
        }
    });
    let _ = tokio::join!(j1, j2);

    let mut state = Arc::try_unwrap(state).unwrap().into_inner();
    let State { lat, iat, ooo, count,.. } = &mut state;
    let mut lat = Data::new(lat);
    let mut iat = Data::new(iat);

    println!("Results");
    println!("----------------------------------------");
    let dropped = opts.count - *count;
    println!(
        "dropped: {} ({:.2}%)",
        dropped,
        (dropped as f64 / opts.count as f64) * 100.0
    );
    println!(
        "ooo: {} ({:.2}%)",
        ooo,
        (*ooo as f64 / opts.count as f64) * 100.0
    );
    println!();

    println!("lat min: {} us", lat.min());
    println!("lat min: {} us", lat.min());
    println!("lat max: {} us", lat.max());
    println!("lat avg: {} us", lat.mean().unwrap());
    println!("lat p50: {} us", lat.percentile(50));
    println!("lat p95: {} us", lat.percentile(95));
    println!("lat p99: {} us", lat.percentile(99));
    println!("lat stdev: {} us", lat.std_dev().unwrap());
    println!();

    println!("iat min: {} us", iat.min());
    println!("iat max: {} us", iat.max());
    println!("iat avg: {} us", iat.mean().unwrap());
    println!("iat p50: {} us", iat.percentile(50));
    println!("iat p95: {} us", iat.percentile(95));
    println!("iat p99: {} us", iat.percentile(99));
    println!("iat stddev: {} us", iat.std_dev().unwrap());

    Ok(())
}
