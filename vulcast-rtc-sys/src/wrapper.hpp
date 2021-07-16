#pragma once

#include <cstddef>

class Broadcaster;
namespace mediasoupclient {
class DataConsumer;
class Producer;
} // namespace mediasoupclient

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
  char *(*consume_data)(const void *ctx, const char *transport_id,
                        const char *data_producer_id);

  // Called when new message is available from a DataConsumer.
  void (*on_data_consumer_message)(const void *ctx,
                                   const char *data_consumer_id,
                                   const char *data, size_t len);
  // Called when a DataConsumer RTC DataState changes.
  void (*on_data_consumer_state_changed)(const void *ctx,
                                         const char *data_consumer_id,
                                         const char *state);
};

void init(const char *argv0);

Broadcaster *broadcaster_new(const void *ctx, SignalHandler signal_handler);
void broadcaster_delete(Broadcaster *broadcaster);
char *broadcaster_get_recv_transport_id(Broadcaster *b);

mediasoupclient::DataConsumer *
data_consumer_new(Broadcaster *b, const char *data_consumer_id,
                  const char *data_producer_id,
                  const char *sctp_stream_parameters);
void data_consumer_delete(mediasoupclient::DataConsumer *consumer);

mediasoupclient::Producer *producer_new_from_fake_audio(Broadcaster *b);
mediasoupclient::Producer *producer_new_from_fake_video(Broadcaster *b);
mediasoupclient::Producer *producer_new_from_vcm_capturer(Broadcaster *b);
void producer_delete(mediasoupclient::Producer *producer);

void debug_enumerate_capture_devices();

enum GlogLogLevel { INFO, WARNING, ERROR, FATAL };
enum RtcLogLevel { LS_VERBOSE, LS_INFO, LS_WARNING, LS_ERROR, LS_NONE };
enum MediasoupLogLevel { LOG_NONE, LOG_ERROR, LOG_WARN, LOG_DEBUG, LOG_TRACE };

void set_glog_log_level(GlogLogLevel level);
void set_mediasoup_log_level(MediasoupLogLevel level);
void set_rtc_log_level(RtcLogLevel level);

void delete_str(char *str);