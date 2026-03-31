#include "clawser/navigator_spoof.h"

#include "clawser/clawser_config.h"

namespace clawser {

bool IsNavigatorSpoofEnabled() {
  return ClawserConfigManager::GetInstance().IsLoaded();
}

std::string GetSpoofedUserAgent() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return std::string();
  return ClawserConfigManager::GetInstance().GetConfig().navigator.user_agent;
}

std::string GetSpoofedPlatform() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return std::string();
  return ClawserConfigManager::GetInstance().GetConfig().navigator.platform;
}

std::string GetSpoofedVendor() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return std::string();
  return ClawserConfigManager::GetInstance().GetConfig().navigator.vendor;
}

std::string GetSpoofedAppVersion() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return std::string();
  return ClawserConfigManager::GetInstance().GetConfig().navigator.app_version;
}

std::string GetSpoofedLanguage() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return std::string();
  return ClawserConfigManager::GetInstance().GetConfig().navigator.language;
}

std::vector<std::string> GetSpoofedLanguages() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return {};
  return ClawserConfigManager::GetInstance().GetConfig().navigator.languages;
}

int GetSpoofedHardwareConcurrency() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return 0;
  return ClawserConfigManager::GetInstance()
      .GetConfig()
      .navigator.hardware_concurrency;
}

int GetSpoofedDeviceMemory() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return 0;
  return ClawserConfigManager::GetInstance()
      .GetConfig()
      .navigator.device_memory;
}

int GetSpoofedMaxTouchPoints() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return -1;
  return ClawserConfigManager::GetInstance()
      .GetConfig()
      .navigator.max_touch_points;
}

std::string GetSpoofedDoNotTrack() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return std::string();
  const auto& dnt =
      ClawserConfigManager::GetInstance().GetConfig().navigator.do_not_track;
  if (!dnt.has_value())
    return "unspecified";
  return dnt.value() ? "1" : "0";
}

std::vector<SpoofedBrandVersion> GetSpoofedBrands() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return {};
  const auto& brands = ClawserConfigManager::GetInstance()
                            .GetConfig()
                            .navigator.user_agent_data.brands;
  std::vector<SpoofedBrandVersion> result;
  result.reserve(brands.size());
  for (const auto& bv : brands) {
    SpoofedBrandVersion sbv;
    sbv.brand = bv.brand;
    sbv.version = bv.version;
    result.push_back(std::move(sbv));
  }
  return result;
}

}  // namespace clawser
