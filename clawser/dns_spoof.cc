#include "clawser/dns_spoof.h"

#include "clawser/clawser_config.h"

namespace clawser {

bool IsProxyConfigured() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return false;
  const auto& config = ClawserConfigManager::GetInstance().GetConfig();
  if (config.webrtc.policy == WebRtcPolicy::kProxyOnly)
    return true;
  return false;
}

bool ShouldForceDnsOverProxy() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return false;
  if (!IsProxyConfigured())
    return false;
  return true;
}

bool ShouldPreventDirectDns() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return false;
  if (!IsProxyConfigured())
    return false;
  return true;
}

}  // namespace clawser
