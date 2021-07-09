#pragma once

struct Signaller {
  const char *server_rtp_capabilities;
  void (*on_rtp_capabilities)(void *ctx, const char *rtp_capabilities);
  void (*on_produce)(void *ctx, const char *transport_id, const char *kind,
                     const char *rtp_parameters);
  void (*on_connect_webrtc_transport)(void *ctx, const char *transport_id,
                                      const char *dtls_parameters);
  void (*on_consume_data)(void *ctx, const char *transport_id,
                          const char *data_producer_id);
};

void hello();

void init();
void run(void *ctx, Signaller signaller);
void stop(void *ctx);

enum RtcLogLevel { LS_VERBOSE, LS_INFO, LS_WARNING, LS_ERROR, LS_NONE };
enum MediasoupLogLevel { LOG_NONE, LOG_ERROR, LOG_WARN, LOG_DEBUG, LOG_TRACE };

void set_mediasoup_log_level(MediasoupLogLevel level);
void set_rtc_log_level(RtcLogLevel level);