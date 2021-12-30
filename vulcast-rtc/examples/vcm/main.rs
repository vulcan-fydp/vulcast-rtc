use serde::Serialize;
use std::sync::Arc;

use clap::{AppSettings, Clap};
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

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    /// Listening address for signal endpoint (domain required).
    #[clap(long, default_value = "wss://localhost:8443")]
    pub signal_addr: String,
    /// Pre-authorized access token.
    #[clap(short, long)]
    pub token: String,
    // Disable TLS.
    #[clap(long)]
    pub no_tls: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    vulcast_rtc::set_native_log_level(vulcast_rtc::LogLevel::Verbose);

    let opts: Opts = Opts::parse();

    struct PromiscuousServerVerifier;
    impl rustls::ServerCertVerifier for PromiscuousServerVerifier {
        fn verify_server_cert(
            &self,
            _roots: &rustls::RootCertStore,
            _presented_certs: &[rustls::Certificate],
            _dns_name: webpki::DNSNameRef,
            _ocsp_response: &[u8],
        ) -> Result<rustls::ServerCertVerified, rustls::TLSError> {
            // here be dragons
            Ok(rustls::ServerCertVerified::assertion())
        }
    }
    let mut client_config = rustls::ClientConfig::default();
    client_config
        .dangerous()
        .set_certificate_verifier(Arc::new(PromiscuousServerVerifier));

    let uri: Uri = opts.signal_addr.parse()?;
    println!("connecting to {}", &uri);

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
        Some(if opts.no_tls {
            Connector::Plain
        } else {
            Connector::Rustls(Arc::new(client_config))
        }),
    )
    .await?;

    println!("response http {}:", response.status());
    for (ref header, value) in response.headers() {
        println!("- {}={:?}", header, value);
    }

    let client = GraphQLWebSocket::new();
    client.connect(
        socket,
        Some(serde_json::to_value(SessionToken { token: opts.token })?),
    );

    let graphql_signaller = Arc::new(GraphQLSignaller::new(client.clone()));
    let broadcaster = Broadcaster::new(graphql_signaller.clone());

    let _vcm_capturer = broadcaster
        .produce_video_from_vcm_capturer(Some(-1), 1920, 1080, 30)
        .await;

    let _audio_producer = broadcaster.produce_audio_from_default_alsa().await;

    let _ = graphql_signaller.shutdown().recv().await;

    Ok(())
}
