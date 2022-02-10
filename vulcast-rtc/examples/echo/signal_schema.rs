use vulcast_rtc::types::*;

use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "examples/schema/signal_schema.gql",
    query_path = "examples/schema/signal_query.gql"
)]
pub struct ServerRtpCapabilities;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "examples/schema/signal_schema.gql",
    query_path = "examples/schema/signal_query.gql"
)]
pub struct DataProducerAvailable;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "examples/schema/signal_schema.gql",
    query_path = "examples/schema/signal_query.gql"
)]
pub struct CreateWebrtcTransport;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "examples/schema/signal_schema.gql",
    query_path = "examples/schema/signal_query.gql"
)]
pub struct ClientRtpCapabilities;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "examples/schema/signal_schema.gql",
    query_path = "examples/schema/signal_query.gql"
)]
pub struct Produce;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "examples/schema/signal_schema.gql",
    query_path = "examples/schema/signal_query.gql"
)]
pub struct ProduceData;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "examples/schema/signal_schema.gql",
    query_path = "examples/schema/signal_query.gql"
)]
pub struct ConnectWebrtcTransport;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "examples/schema/signal_schema.gql",
    query_path = "examples/schema/signal_query.gql"
)]
pub struct ConsumeData;
