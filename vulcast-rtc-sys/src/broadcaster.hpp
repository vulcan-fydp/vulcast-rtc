#pragma once

#include <chrono>
#include <condition_variable>
#include <future>
#include <mutex>
#include <string>

#include <json.hpp>
#include <mediasoupclient.hpp>

#include "signaller.hpp"

class Broadcaster : public mediasoupclient::SendTransport::Listener,
                    public mediasoupclient::RecvTransport::Listener,
                    mediasoupclient::Producer::Listener,
                    mediasoupclient::DataConsumer::Listener {
public:
  /* Virtual methods inherited from SendTransport::Listener. */
public:
  std::future<void> OnConnect(mediasoupclient::Transport *transport,
                              const nlohmann::json &dtlsParameters) override;
  void OnConnectionStateChange(mediasoupclient::Transport *transport,
                               const std::string &connectionState) override;
  std::future<std::string> OnProduce(mediasoupclient::SendTransport *transport,
                                     const std::string &kind,
                                     nlohmann::json rtpParameters,
                                     const nlohmann::json &appData) override;

  std::future<std::string>
  OnProduceData(mediasoupclient::SendTransport *transport,
                const nlohmann::json &sctpStreamParameters,
                const std::string &label, const std::string &protocol,
                const nlohmann::json &appData) override;

  /* Virtual methods inherited from Producer::Listener. */
public:
  void OnTransportClose(mediasoupclient::Producer *producer) override;

  /* Virtual methods inherited from DataConsumer::Listener */
public:
  void OnMessage(mediasoupclient::DataConsumer *data_consumer,
                 const webrtc::DataBuffer &buffer) override;
  void OnConnecting(mediasoupclient::DataConsumer *) override;
  void OnClosing(mediasoupclient::DataConsumer *) override;
  void OnClose(mediasoupclient::DataConsumer *) override;
  void OnOpen(mediasoupclient::DataConsumer *) override;
  void OnTransportClose(mediasoupclient::DataConsumer *) override;

public:
  void Start();
  void Stop();

  Broadcaster(Signaller signaller);
  virtual ~Broadcaster();

  mediasoupclient::DataConsumer *
  ConsumeData(const std::string &data_consumer_id,
              const std::string &data_producer_id,
              const nlohmann::json &sctp_stream_parameters);

  mediasoupclient::Producer *
  Produce(webrtc::MediaStreamTrackInterface *track,
          const std::vector<webrtc::RtpEncodingParameters> *encodings = nullptr,
          const nlohmann::json &codec_options = nlohmann::json::object(),
          const nlohmann::json &appdata = nlohmann::json::object());

  bool CanProduceAudio() { return device_.CanProduce("audio"); }
  bool CanProduceVideo() { return device_.CanProduce("video"); }

  std::string GetSendTransportId() const { return send_transport_->GetId(); }
  std::string GetRecvTransportId() const { return recv_transport_->GetId(); }

private:
  Signaller signaller_;

  mediasoupclient::Device device_;
  mediasoupclient::SendTransport *send_transport_{nullptr};
  mediasoupclient::RecvTransport *recv_transport_{nullptr};

  std::string id = std::to_string(rtc::CreateRandomId());

  void CreateSendTransport();
  void CreateRecvTransport();
};
