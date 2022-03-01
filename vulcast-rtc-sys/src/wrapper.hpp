#pragma once

#include <cstddef>
#include <cstdint>

class Broadcaster;
namespace mediasoupclient {
class DataConsumer;
class DataProducer;
class Producer;
} // namespace mediasoupclient

// foreign callback requesting frame in RGB (little-endian) format
typedef void (*frame_callback_t)(const void *ctx, uint32_t width,
                                 uint32_t height, int64_t timestamp, uint8_t *);

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
  // Called when client wants to produce data. Expects ProducerId.
  char *(*on_produce_data)(const void *ctx, const char *transport_id,
                           const char *sctp_stream_parameters);
  // Called when client wants to connect WebRTC transport.
  void (*on_connect_webrtc_transport)(const void *ctx, const char *transport_id,
                                      const char *dtls_parameters);

  // Called when new message is available from a DataConsumer.
  void (*on_data_consumer_message)(const void *ctx,
                                   const char *data_consumer_id,
                                   const char *data, size_t len);
  // Called when a DataConsumer RTC DataState changes.
  void (*on_data_consumer_state_changed)(const void *ctx,
                                         const char *data_consumer_id,
                                         const char *state);
  // Called when a DataProducer RTC DataState changes.
  void (*on_data_producer_state_changed)(const void *ctx,
                                         const char *data_producer_id,
                                         const char *state);
  // Called when a transport connection state changes.
  void (*on_connection_state_changed)(const void *ctx, const char *transport_id,
                                      const char *state);
};

void init(const char *argv0);

Broadcaster *broadcaster_new(const void *ctx, SignalHandler signal_handler);
void broadcaster_delete(Broadcaster *broadcaster);
char *broadcaster_marshal_recv_transport_id(Broadcaster *b);

mediasoupclient::DataConsumer *
data_consumer_new(Broadcaster *b, const char *data_consumer_id,
                  const char *data_producer_id,
                  const char *sctp_stream_parameters);
void data_consumer_delete(mediasoupclient::DataConsumer *consumer);

mediasoupclient::Producer *producer_new_from_default_audio(Broadcaster *b);
mediasoupclient::Producer *producer_new_from_fake_video(Broadcaster *b);
mediasoupclient::Producer *
producer_new_from_vcm_capturer(Broadcaster *b, int device_idx, uint32_t width,
                               uint32_t height, uint32_t fps, int video_type);
mediasoupclient::Producer *
producer_new_from_foreign(Broadcaster *b, uint32_t width, uint32_t height,
                          uint32_t fps, void *ctx, frame_callback_t callback);
void producer_delete(mediasoupclient::Producer *producer);

mediasoupclient::DataProducer *data_producer_new(Broadcaster *b);
char *data_producer_marshal_id(mediasoupclient::DataProducer *data_producer);
void data_producer_send(mediasoupclient::DataProducer *data_producer,
                        const uint8_t *data, size_t len);
void data_producer_delete(mediasoupclient::DataProducer *data_producer);

void debug_enumerate_capture_devices();

enum GlogLogLevel { INFO, WARNING, ERROR, FATAL };
enum RtcLogLevel { LS_VERBOSE, LS_INFO, LS_WARNING, LS_ERROR, LS_NONE };
enum MediasoupLogLevel { LOG_NONE, LOG_ERROR, LOG_WARN, LOG_DEBUG, LOG_TRACE };

void set_glog_log_level(GlogLogLevel level);
void set_mediasoup_log_level(MediasoupLogLevel level);
void set_rtc_log_level(RtcLogLevel level);

void cpp_unmarshal_str(char *str);
