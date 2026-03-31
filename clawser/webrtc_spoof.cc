#include "clawser/webrtc_spoof.h"

#include <regex>
#include <string>

#include "clawser/clawser_config.h"

namespace clawser {

WebRtcPolicy GetWebRtcPolicy() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return WebRtcPolicy::kDisabled;
  return ClawserConfigManager::GetInstance().GetConfig().webrtc.policy;
}

std::string GetFakeLocalIp() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return std::string();
  return ClawserConfigManager::GetInstance().GetConfig().webrtc.fake_local_ip;
}

bool ShouldSpoofWebRtc() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return false;
  WebRtcPolicy policy =
      ClawserConfigManager::GetInstance().GetConfig().webrtc.policy;
  return policy == WebRtcPolicy::kSpoofed ||
         policy == WebRtcPolicy::kDisabled ||
         policy == WebRtcPolicy::kProxyOnly;
}

bool ShouldBlockAllWebRtc() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return false;
  return ClawserConfigManager::GetInstance().GetConfig().webrtc.policy ==
         WebRtcPolicy::kDisabled;
}

bool ShouldForceRelayOnly() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return false;
  return ClawserConfigManager::GetInstance().GetConfig().webrtc.policy ==
         WebRtcPolicy::kProxyOnly;
}

namespace {

bool IsPrivateIp(const std::string& ip) {
  if (ip.substr(0, 4) == "10.")
    return true;
  if (ip.substr(0, 8) == "192.168.")
    return true;
  if (ip.substr(0, 4) == "172.") {
    size_t dot = ip.find('.', 4);
    if (dot != std::string::npos) {
      int second_octet = std::stoi(ip.substr(4, dot - 4));
      if (second_octet >= 16 && second_octet <= 31)
        return true;
    }
  }
  if (ip == "127.0.0.1" || ip.substr(0, 2) == "0.")
    return true;
  return false;
}

}  // namespace

std::string RewriteCandidateIp(const std::string& candidate) {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return candidate;

  const auto& config = ClawserConfigManager::GetInstance().GetConfig();
  if (config.webrtc.policy != WebRtcPolicy::kSpoofed)
    return candidate;

  std::string fake_ip = config.webrtc.fake_local_ip;
  if (fake_ip.empty())
    return candidate;

  std::regex ip_regex(
      R"((\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}))");
  std::smatch match;
  std::string result = candidate;

  if (std::regex_search(result, match, ip_regex)) {
    std::string found_ip = match[1].str();
    if (IsPrivateIp(found_ip)) {
      size_t pos = result.find(found_ip);
      if (pos != std::string::npos) {
        result.replace(pos, found_ip.length(), fake_ip);
      }
    }
  }

  return result;
}

}  // namespace clawser
