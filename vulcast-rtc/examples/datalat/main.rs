use serde::Serialize;
use std::sync::Arc;

use clap::Parser;
use graphql_ws::GraphQLWebSocket;
use http::Uri;
use tokio::net::TcpStream;
use tokio_tungstenite::Connector;

use crate::graphql_signaller::GraphQLSignaller;
use vulcast_rtc::broadcaster::Broadcaster;

mod graphql_signaller;
mod signal_schema;

#[derive(Serialize)]
struct SessionToken {
    token: String,
}

#[derive(Parser)]
pub struct Opts {
    /// Listening address for signal endpoint (domain required).
    #[clap(long, default_value = "wss://localhost:8443")]
    pub signal_addr: String,
    /// Pre-authorized access token for Vulcast.
    #[clap(short, long)]
    pub vulcast_token: String,
    /// Pre-authorized access token for Client.
    #[clap(short, long)]
    pub client_token: String,
    // Disable TLS.
    #[clap(long)]
    pub no_tls: bool,
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
    let (socket, response) = tokio_tungstenite::client_async_tls_with_config(
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
        create_signalling_connection(&opts.signal_addr, opts.vulcast_token, opts.no_tls).await?;
    let client_conn =
        create_signalling_connection(&opts.signal_addr, opts.client_token, opts.no_tls).await?;
    let vulcast_gql_signaller = Arc::new(GraphQLSignaller::new(vulcast_conn.clone()));
    let client_gql_signaller = Arc::new(GraphQLSignaller::new(client_conn.clone()));
    let vulcast_broadcaster = Broadcaster::new(vulcast_gql_signaller.clone());
    let client_broadcaster = Broadcaster::new(client_gql_signaller.clone());

    vulcast_gql_signaller.shutdown().recv().await.unwrap();
    client_gql_signaller.shutdown().recv().await.unwrap();

    Ok(())
}
