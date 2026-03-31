#include "clawser/timezone_spoof.h"

#include "clawser/clawser_config.h"

namespace clawser {

bool ShouldSpoofTimezone() {
  return ClawserConfigManager::GetInstance().IsLoaded();
}

std::string GetSpoofedTimezone() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return std::string();
  const std::string& tz =
      ClawserConfigManager::GetInstance().GetConfig().timezone;
  if (tz.empty())
    return "UTC";
  return tz;
}

std::string GetSpoofedLocale() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return std::string();
  return ClawserConfigManager::GetInstance().GetConfig().locale;
}

}  // namespace clawser
