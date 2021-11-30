#include "broadcaster.hpp"

#include <PeerConnection.hpp>
#include <chrono>
#include <cstdlib>
#include <ctime>
#include <functional>
#include <string>
#include <thread>

#include <glog/logging.h>
#include <json.hpp>
#include <mediasoupclient.hpp>

#include "media_stream_track_factory.hpp"

using json = nlohmann::json;

Broadcaster::Broadcaster(Signaller signaller) : signaller_(signaller) {}

Broadcaster::~Broadcaster() { this->Stop(); }

void Broadcaster::OnTransportClose(mediasoupclient::Producer *producer) {
  LOG(INFO) << "Broadcaster::OnTransportClose(" << producer->GetId() << ")";
}

/* Transport::Listener::OnConnect
 *
 * Fired for the first Transport::Consume() or Transport::Produce().
 * Update the already created remote transport with the local DTLS parameters.
 */
std::future<void> Broadcaster::OnConnect(mediasoupclient::Transport *transport,
                                         const json &dtlsParameters) {
  LOG(INFO) << "Broadcaster::OnConnect(" << transport->GetId() << ","
            << dtlsParameters << ")";
  std::promise<void> promise;
  signaller_.OnConnectWebrtcTransport(transport->GetId(), dtlsParameters);
  promise.set_value();
  return promise.get_future();
}

/*
 * Transport::Listener::OnConnectionStateChange.
 */
void Broadcaster::OnConnectionStateChange(mediasoupclient::Transport *transport,
                                          const std::string &connectionState) {
  LOG(INFO) << "Broadcaster::OnConnectionStateChange(" << transport->GetId()
            << "," << connectionState << ")";
  signaller_.OnConnectionStateChanged(transport->GetId(), connectionState);
}

/* Producer::Listener::OnProduce
 *
 * Fired when a producer needs to be created in mediasoup.
 * Retrieve the remote producer ID and feed the caller with it.
 */
std::future<std::string>
Broadcaster::OnProduce(mediasoupclient::SendTransport *transport,
                       const std::string &kind, json rtpParameters,
                       const json &appData) {
  LOG(INFO) << "Broadcaster::OnProduce(" << transport->GetId() << "," << kind
            << "," << rtpParameters << "," << appData << ")";
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
  LOG(FATAL) << "Broadcaster::OnProduceData()";
}

void Broadcaster::Start() {
  LOG(INFO) << "Broadcaster::Start()";
  auto routerRtpCapabilities = signaller_.GetServerRtpCapabilities();

  auto factory = GetPeerConnectionFactory();
  mediasoupclient::PeerConnection::Options options;
  options.factory = factory.get();
  this->device_.Load(routerRtpCapabilities, &options);
  // this->device_.Load(routerRtpCapabilities);

  auto rtp_capabilities = device_.GetRtpCapabilities();
  signaller_.OnRtpCapabilities(rtp_capabilities);

  this->CreateSendTransport();
  this->CreateRecvTransport();
}

mediasoupclient::DataConsumer *
Broadcaster::ConsumeData(const std::string &data_consumer_id,
                         const std::string &data_producer_id,
                         const nlohmann::json &sctp_stream_parameters) {
  LOG(INFO) << "Broadcaster::CreateDataConsumer(" << data_producer_id << ")";
  return recv_transport_->ConsumeData(
      this, data_consumer_id, data_producer_id,
      sctp_stream_parameters["streamId"].get<uint16_t>(), "");
}

mediasoupclient::Producer *Broadcaster::Produce(
    webrtc::MediaStreamTrackInterface *track,
    const std::vector<webrtc::RtpEncodingParameters> *encodings,
    const nlohmann::json &codec_options, const nlohmann::json &appdata) {
  LOG(INFO) << "Broadcaster::Produce(" << std::hex << track << "," << std::hex
            << encodings << "," << codec_options << "," << appdata << ")";
  return send_transport_->Produce(this, track, encodings, &codec_options,
                                  appdata);
}

void Broadcaster::CreateSendTransport() {
  LOG(INFO) << "Broadcaster::CreateSendTransport()";
  auto response = signaller_.CreateWebrtcTransport();

  auto factory = GetPeerConnectionFactory();
  mediasoupclient::PeerConnection::Options options;
  options.factory = factory.get();
  this->send_transport_ = device_.CreateSendTransport(
      this, response["id"], response["iceParameters"],
      response["iceCandidates"], response["dtlsParameters"],
      response["sctpParameters"], &options);
  // this->send_transport_ = device_.CreateSendTransport(
  //     this, response["id"], response["iceParameters"],
  //     response["iceCandidates"], response["dtlsParameters"],
  //     response["sctpParameters"]);
}

void Broadcaster::CreateRecvTransport() {
  LOG(INFO) << "Broadcaster::CreateRecvTransport()";
  auto response = signaller_.CreateWebrtcTransport();

  auto factory = GetPeerConnectionFactory();
  mediasoupclient::PeerConnection::Options options;
  options.factory = factory.get();
  this->recv_transport_ = device_.CreateRecvTransport(
      this, response["id"], response["iceParameters"],
      response["iceCandidates"], response["dtlsParameters"],
      response["sctpParameters"], &options);
  // this->recv_transport_ = device_.CreateRecvTransport(
  //     this, response["id"], response["iceParameters"],
  //     response["iceCandidates"], response["dtlsParameters"],
  //     response["sctpParameters"]);
}

void Broadcaster::OnMessage(mediasoupclient::DataConsumer *data_consumer,
                            const webrtc::DataBuffer &buffer) {
  LOG(INFO) << "Broadcaster::OnMessage(" << data_consumer->GetId()
            << ",len=" << buffer.data.size() << ")";
  signaller_.OnDataConsumerMessage(
      data_consumer->GetId(), buffer.data.data<char>(), buffer.data.size());
}
void Broadcaster::OnConnecting(mediasoupclient::DataConsumer *data_consumer) {
  LOG(INFO) << "Broadcaster::OnConnecting(" << data_consumer->GetId() << ")";
  signaller_.OnDataConsumerStateChanged(
      data_consumer->GetId(), webrtc::DataChannelInterface::DataStateString(
                                  data_consumer->GetReadyState()));
}
void Broadcaster::OnClosing(mediasoupclient::DataConsumer *data_consumer) {
  LOG(INFO) << "Broadcaster::OnClosing(" << data_consumer->GetId() << ")";
  signaller_.OnDataConsumerStateChanged(
      data_consumer->GetId(), webrtc::DataChannelInterface::DataStateString(
                                  data_consumer->GetReadyState()));
}
void Broadcaster::OnClose(mediasoupclient::DataConsumer *data_consumer) {
  LOG(INFO) << "Broadcaster::OnClose(" << data_consumer->GetId() << ")";
  signaller_.OnDataConsumerStateChanged(
      data_consumer->GetId(), webrtc::DataChannelInterface::DataStateString(
                                  data_consumer->GetReadyState()));
}
void Broadcaster::OnOpen(mediasoupclient::DataConsumer *data_consumer) {
  LOG(INFO) << "Broadcaster::OnOpen(" << data_consumer->GetId() << ")";
  signaller_.OnDataConsumerStateChanged(
      data_consumer->GetId(), webrtc::DataChannelInterface::DataStateString(
                                  data_consumer->GetReadyState()));
}
void Broadcaster::OnTransportClose(
    mediasoupclient::DataConsumer *data_consumer) {
  LOG(INFO) << "Broadcaster::OnTransportClose(" << data_consumer->GetId()
            << ")";
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
