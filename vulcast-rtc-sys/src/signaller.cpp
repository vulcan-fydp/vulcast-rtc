#include "signaller.hpp"

#include <glog/logging.h>

#include "ffi.hpp"

Signaller::Signaller(const void *ctx, SignalHandler handler)
    : ctx_(ctx), handler_(handler) {}

nlohmann::json Signaller::GetServerRtpCapabilities() const {
  char *rtp_capabilities_cstr = handler_.server_rtp_capabilities(ctx_);
  DCHECK(rtp_capabilities_cstr != nullptr);
  auto rtp_capabilities = nlohmann::json::parse(rtp_capabilities_cstr);
  cpp_unmarshal_str(rtp_capabilities_cstr);
  return rtp_capabilities;
}

nlohmann::json Signaller::CreateWebrtcTransport() const {
  char *webrtc_transport_options_cstr = handler_.create_webrtc_transport(ctx_);
  DCHECK(webrtc_transport_options_cstr != nullptr);
  auto webrtc_transport_options =
      nlohmann::json::parse(webrtc_transport_options_cstr);
  cpp_unmarshal_str(webrtc_transport_options_cstr);
  return webrtc_transport_options;
}

void Signaller::OnRtpCapabilities(const nlohmann::json &rtp_caps) const {
  auto rtp_caps_str = rtp_caps.dump();
  handler_.on_rtp_capabilities(ctx_, rtp_caps_str.c_str());
}

void Signaller::OnConnectWebrtcTransport(
    const std::string &transport_id,
    const nlohmann::json &dtls_parameters) const {
  auto dtls_params_str = dtls_parameters.dump();
  handler_.on_connect_webrtc_transport(ctx_, transport_id.c_str(),
                                       dtls_params_str.c_str());
}

std::string Signaller::OnProduce(const std::string &transport_id,
                                 const std::string &kind,
                                 const nlohmann::json &rtp_parameters) const {
  auto rtp_parameters_str = rtp_parameters.dump();
  char *producer_id_cstr = handler_.on_produce(
      ctx_, transport_id.c_str(), kind.c_str(), rtp_parameters_str.c_str());
  DCHECK(producer_id_cstr != nullptr);
  std::string producer_id(producer_id_cstr);
  return producer_id;
}

std::string
Signaller::OnProduceData(const std::string &transport_id,
                         const nlohmann::json &sctp_stream_parameters) const {
  auto sctp_stream_parameters_str = sctp_stream_parameters.dump();
  char *data_producer_id_cstr = handler_.on_produce_data(
      ctx_, transport_id.c_str(), sctp_stream_parameters_str.c_str());
  DCHECK(data_producer_id_cstr != nullptr);
  std::string data_producer_id(data_producer_id_cstr);
  return data_producer_id;
}

void Signaller::OnDataConsumerMessage(const std::string &data_consumer_id,
                                      const char *data, std::size_t len) const {
  handler_.on_data_consumer_message(ctx_, data_consumer_id.c_str(), data, len);
}

void Signaller::OnDataConsumerStateChanged(const std::string &data_consumer_id,
                                           const std::string &state) const {
  handler_.on_data_consumer_state_changed(ctx_, data_consumer_id.c_str(),
                                          state.c_str());
}

void Signaller::OnDataProducerStateChanged(const std::string &data_producer_id,
                                           const std::string &state) const {
  handler_.on_data_producer_state_changed(ctx_, data_producer_id.c_str(),
                                          state.c_str());
}

void Signaller::OnConnectionStateChanged(const std::string &transport_id,
                                         const std::string &state) const {
  handler_.on_connection_state_changed(ctx_, transport_id.c_str(),
                                       state.c_str());
}