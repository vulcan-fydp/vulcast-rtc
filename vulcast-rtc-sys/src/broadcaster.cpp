#include "broadcaster.hpp"

#include <chrono>
#include <cstdlib>
#include <ctime>
#include <functional>
#include <string>
#include <thread>

#include <glog/logging.h>
#include <json.hpp>
#include <media_stream_track_factory.hpp>
#include <mediasoupclient.hpp>

using json = nlohmann::json;

Broadcaster::Broadcaster(Signaller signaller) : signaller_(signaller) {}

Broadcaster::~Broadcaster() { this->Stop(); }

void Broadcaster::OnTransportClose(mediasoupclient::Producer * /*producer*/) {
  LOG(INFO) << "Broadcaster::OnTransportClose()";
}

/* Transport::Listener::OnConnect
 *
 * Fired for the first Transport::Consume() or Transport::Produce().
 * Update the already created remote transport with the local DTLS parameters.
 */
std::future<void> Broadcaster::OnConnect(mediasoupclient::Transport *transport,
                                         const json &dtlsParameters) {
  LOG(INFO) << "Broadcaster::OnConnect()";
  std::promise<void> promise;
  signaller_.OnConnectWebrtcTransport(transport->GetId(), dtlsParameters);
  promise.set_value();
  return promise.get_future();
}

/*
 * Transport::Listener::OnConnectionStateChange.
 */
void Broadcaster::OnConnectionStateChange(
    mediasoupclient::Transport * /*transport*/,
    const std::string &connectionState) {
  LOG(INFO) << "Broadcaster::OnConnectionStateChange() [connectionState="
            << connectionState << "]";
  CHECK(connectionState != "failed");
}

/* Producer::Listener::OnProduce
 *
 * Fired when a producer needs to be created in mediasoup.
 * Retrieve the remote producer ID and feed the caller with it.
 */
std::future<std::string>
Broadcaster::OnProduce(mediasoupclient::SendTransport *transport,
                       const std::string &kind, json rtpParameters,
                       const json & /*appData*/) {
  LOG(INFO) << "Broadcaster::OnProduce()";
  std::promise<std::string> promise;
  promise.set_value(
      signaller_.OnProduce(transport->GetId(), kind, rtpParameters));
  return promise.get_future();
}

/* Producer::Listener::OnProduceData
 *
 * Fired when a data producer needs to be created in mediasoup.
 * Retrieve the remote producer ID and feed the caller with it.
 */
std::future<std::string> Broadcaster::OnProduceData(
    mediasoupclient::SendTransport * /*transport*/,
    const json & /*sctpStreamParameters*/, const std::string & /*label*/,
    const std::string & /*protocol*/, const json & /*appData*/) {
  // unreachable
  LOG(FATAL) << "Broadcaster::OnProduceData()" << std::endl;
}

void Broadcaster::Start() {
  LOG(INFO) << "Broadcaster::Start()";
  auto routerRtpCapabilities = signaller_.GetServerRtpCapabilities();
  this->device_.Load(routerRtpCapabilities);

  auto rtp_capabilities = device_.GetRtpCapabilities();
  signaller_.OnRtpCapabilities(rtp_capabilities);

  this->CreateSendTransport();
  this->CreateRecvTransport();
}

mediasoupclient::DataConsumer* Broadcaster::CreateDataConsumer(const std::string &data_consumer_id,
                                     const std::string &data_producer_id) {
  return recv_transport_->ConsumeData(
      this, data_consumer_id, data_producer_id, "", "", nlohmann::json());
}

void Broadcaster::CreateSendTransport() {
  LOG(INFO) << "creating mediasoup send WebRtcTransport...";
  auto response = signaller_.CreateWebrtcTransport();
  this->send_transport_ = device_.CreateSendTransport(
      this, response["id"], response["iceParameters"],
      response["iceCandidates"], response["dtlsParameters"],
      response["sctpParameters"]);
}

void Broadcaster::CreateRecvTransport() {
  LOG(INFO) << "creating mediasoup recv WebRtcTransport...";
  auto response = signaller_.CreateWebrtcTransport();
  this->recv_transport_ = device_.CreateRecvTransport(
      this, response["id"], response["iceParameters"],
      response["iceCandidates"], response["dtlsParameters"],
      response["sctpParameters"]);
}

void Broadcaster::OnMessage(mediasoupclient::DataConsumer *data_consumer,
                            const webrtc::DataBuffer &buffer) {
  LOG(INFO) << "Broadcaster::OnMessage() [len=" << buffer.data.size() << "]";
  signaller_.OnMessage(data_consumer->GetId(), buffer.data.data<char>(),
                       buffer.data.size());
}

void Broadcaster::Stop() {
  LOG(INFO) << "Broadcaster::Stop()";

  if (this->recv_transport_) {
    recv_transport_->Close();
  }

  if (this->send_transport_) {
    send_transport_->Close();
  }
}
