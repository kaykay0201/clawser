#include "clawser/screen_spoof.h"

#include "clawser/clawser_config.h"

namespace clawser {

bool IsScreenSpoofEnabled() {
  return ClawserConfigManager::GetInstance().IsLoaded();
}

int GetSpoofedScreenWidth() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return 0;
  return ClawserConfigManager::GetInstance().GetConfig().screen.width;
}

int GetSpoofedScreenHeight() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return 0;
  return ClawserConfigManager::GetInstance().GetConfig().screen.height;
}

int GetSpoofedAvailWidth() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return 0;
  return ClawserConfigManager::GetInstance().GetConfig().screen.avail_width;
}

int GetSpoofedAvailHeight() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return 0;
  return ClawserConfigManager::GetInstance().GetConfig().screen.avail_height;
}

int GetSpoofedColorDepth() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return 0;
  return ClawserConfigManager::GetInstance().GetConfig().screen.color_depth;
}

int GetSpoofedPixelDepth() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return 0;
  return ClawserConfigManager::GetInstance().GetConfig().screen.pixel_depth;
}

double GetSpoofedDevicePixelRatio() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return 0.0;
  return ClawserConfigManager::GetInstance()
      .GetConfig()
      .screen.device_pixel_ratio;
}

}  // namespace clawser
