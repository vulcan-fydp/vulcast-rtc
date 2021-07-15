#pragma once

#include <json.hpp>

#include "wrapper.hpp"

struct Signaller {
  Signaller(const void *ctx, SignalHandler handler);

  nlohmann::json GetServerRtpCapabilities() const;
  nlohmann::json CreateWebrtcTransport() const;

  void OnRtpCapabilities(const nlohmann::json &rtp_caps) const;

  void OnConnectWebrtcTransport(const std::string &transport_id,
                                const nlohmann::json &dtls_parameters) const;

  std::string OnProduce(const std::string &transport_id,
                        const std::string &kind,
                        const nlohmann::json &rtp_parameters) const;

  void OnDataConsumerMessage(const std::string &data_consumer_id, const char *data,
                 std::size_t len) const;

  void OnDataConsumerStateChanged(const std::string &data_consumer_id,
                                  const std::string &state) const;

  nlohmann::json ConsumeData(const std::string &transport_id,
                             const std::string &data_producer_id) const;

private:
  const void *ctx_;
  SignalHandler handler_;
};