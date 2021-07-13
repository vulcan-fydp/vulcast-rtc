#include "signaller.hpp"

#include <glog/logging.h>

#include "ffi.hpp"

Signaller::Signaller(const void *ctx, SignalHandler handler)
    : ctx_(ctx), handler_(handler) {}

nlohmann::json Signaller::GetServerRtpCapabilities() const {
  char *rtp_capabilities_cstr = handler_.server_rtp_capabilities(ctx_);
  DCHECK(rtp_capabilities_cstr != nullptr);
  auto rtp_capabilities = nlohmann::json::parse(rtp_capabilities_cstr);
  retake_cstr(rtp_capabilities_cstr);
  return rtp_capabilities;
}

nlohmann::json Signaller::CreateWebrtcTransport() const {
  char *webrtc_transport_options_cstr = handler_.create_webrtc_transport(ctx_);
  DCHECK(webrtc_transport_options_cstr != nullptr);
  auto webrtc_transport_options =
      nlohmann::json::parse(webrtc_transport_options_cstr);
  retake_cstr(webrtc_transport_options_cstr);
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
                                 const nlohmann::json &rtp_parameters) {
  auto rtp_parameters_str = rtp_parameters.dump();
  char *producer_id_cstr = handler_.on_produce(
      ctx_, transport_id.c_str(), kind.c_str(), rtp_parameters_str.c_str());
  DCHECK(producer_id_cstr != nullptr);
  std::string producer_id(producer_id_cstr);
  return producer_id;
}

void Signaller::OnMessage(const std::string data_consumer_id, const char *data,
                          std::size_t len) {
  handler_.on_message(ctx_, data_consumer_id.c_str(), data, len);
}