#pragma once

#include <cstdint>

class Broadcaster;
namespace mediasoupclient {
class DataConsumer;
}

struct SignalHandler {
  // Get router RTP capabilities. Returns RtpCapabilitiesFinalized.
  char *(*server_rtp_capabilities)(const void *ctx);
  // Get WebRTC transport. Returns WebRtcTransportOptions.
  char *(*create_webrtc_transport)(const void *ctx);

  // Called when RTP capabilities are available from client.
  void (*on_rtp_capabilities)(const void *ctx, const char *rtp_capabilities);
  // Called when client wants to produce. Expects ProducerId.
  char *(*on_produce)(const void *ctx, const char *transport_id,
                            const char *kind, const char *rtp_parameters);
  // Called when client wants to connect WebRTC transport.
  void (*on_connect_webrtc_transport)(const void *ctx, const char *transport_id,
                                      const char *dtls_parameters);
  // Called when client wants to consume data.
  void (*on_consume_data)(const void *ctx, const char *transport_id,
                          const char *data_producer_id);

  // Called when new message is available from a DataConsumer.
  void (*on_message)(const void *ctx, const char *data_consumer_id, const char *data,
                     std::size_t len);
};

void init(const char *argv0);

Broadcaster *create_broadcaster(const void *ctx, SignalHandler signal_handler);
void stop_broadcaster(Broadcaster *broadcaster);

mediasoupclient::DataConsumer *
create_data_consumer(Broadcaster *b, const char *data_consumer_id,
                     const char *data_producer_id);
void stop_data_consumer(mediasoupclient::DataConsumer *consumer);

enum RtcLogLevel { LS_VERBOSE, LS_INFO, LS_WARNING, LS_ERROR, LS_NONE };
enum MediasoupLogLevel { LOG_NONE, LOG_ERROR, LOG_WARN, LOG_DEBUG, LOG_TRACE };

void set_mediasoup_log_level(MediasoupLogLevel level);
void set_rtc_log_level(RtcLogLevel level);
