use serde::Serialize;
use std::{io::Read, sync::Arc};

use clap::{AppSettings, Clap};
use futures::StreamExt;
use graphql_ws::GraphQLWebSocket;
use http::Uri;
use tokio::net::TcpStream;
use tokio_tungstenite::Connector;

use signal_schema as schema;
use vulcast_rtc::broadcaster::{Broadcaster, Handlers};

mod signal_schema;

macro_rules! enclose {
    ( ($( $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
        Some(Connector::Rustls(Arc::new(client_config))),
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

    let broadcaster = Broadcaster::new(Handlers {
        server_rtp_capabilities: Box::new(enclose! { (client) move || {
            let client = client.clone();
            Box::pin(async move {
                    client
                        .query_unchecked::<schema::ServerRtpCapabilities>(
                            schema::server_rtp_capabilities::Variables,
                        )
                        .await
                        .server_rtp_capabilities
            })
        }}),
        create_webrtc_transport: Box::new(enclose! { (client) move || {
            let client = client.clone();
            Box::pin(async move {
                client.query_unchecked::<schema::CreateWebrtcTransport>(
                    schema::create_webrtc_transport::Variables
                )
                .await
                .create_webrtc_transport
            })
        }}),
        on_rtp_capabilities: Box::new(enclose! { (client) move |rtp_capabilities| {
            let client = client.clone();
            Box::pin(async move {
                client.query_unchecked::<schema::ClientRtpCapabilities>(
                    schema::client_rtp_capabilities::Variables{
                        rtp_capabilities
                    }
                )
                .await
                .rtp_capabilities;
            })
        }}),
        on_produce: Box::new(
            enclose! { (client) move |transport_id, kind, rtp_parameters| {
                let client = client.clone();
                Box::pin(async move {
                    client.query_unchecked::<schema::Produce>(
                        schema::produce::Variables{
                            transport_id,
                            kind,
                            rtp_parameters
                        }
                    )
                    .await
                    .produce
                })
            }},
        ),
        on_connect_webrtc_transport: Box::new(
            enclose! { (client) move |transport_id, dtls_parameters| {
                let client = client.clone();
                Box::pin(async move {
                    client.query_unchecked::<schema::ConnectWebrtcTransport>(
                        schema::connect_webrtc_transport::Variables{
                            transport_id,
                            dtls_parameters
                        }
                    )
                    .await
                    .connect_webrtc_transport;
                })
            }},
        ),
        consume_data: Box::new(enclose! { (client) move |transport_id, data_producer_id| {
            let client = client.clone();
            Box::pin(async move {
                client.query_unchecked::<schema::ConsumeData>(
                    schema::consume_data::Variables{
                        transport_id,
                        data_producer_id
                    }
                )
                .await
                .consume_data
            })
        }}),
    });

    let data_producer_available = client.subscribe::<signal_schema::DataProducerAvailable>(
        signal_schema::data_producer_available::Variables,
    );
    let mut data_producer_available_stream = data_producer_available.execute();
    tokio::spawn(enclose! { (broadcaster) async move {
        while let Some(Ok(response)) = data_producer_available_stream.next().await {
            let data_producer_id = response.data.unwrap().data_producer_available;
            println!("{:?}: data producer available", &data_producer_id);
            let data_consumer = broadcaster.consume_data(data_producer_id);
            let id = data_consumer.id();
            tokio::spawn( async move {
                let id = id.clone();
                println!("{:?}: data consumer started", id);
                let mut stream = data_consumer.stream();
                while let Some(msg) = stream.next().await {
                    let str = String::from_utf8_lossy(msg.as_slice());
                    println!("{:?}: {:?} ({})", id, msg, str);
                }
                println!("{:?}: data consumer terminated", id);
            });
        }
    }});

    broadcaster.produce_fake_media();

    println!("Press Enter to instantly die...");
    let _ = std::io::stdin().read(&mut [0u8]).unwrap();

    Ok(())
}
