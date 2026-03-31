#ifndef CLAWSER_NAVIGATOR_SPOOF_H_
#define CLAWSER_NAVIGATOR_SPOOF_H_

#include <string>
#include <vector>

namespace clawser {

bool IsNavigatorSpoofEnabled();

std::string GetSpoofedUserAgent();
std::string GetSpoofedPlatform();
std::string GetSpoofedVendor();
std::string GetSpoofedAppVersion();
std::string GetSpoofedLanguage();
std::vector<std::string> GetSpoofedLanguages();
int GetSpoofedHardwareConcurrency();
int GetSpoofedDeviceMemory();
int GetSpoofedMaxTouchPoints();
std::string GetSpoofedDoNotTrack();

struct SpoofedBrandVersion {
  std::string brand;
  std::string version;
};

std::vector<SpoofedBrandVersion> GetSpoofedBrands();

}  // namespace clawser

#endif  // CLAWSER_NAVIGATOR_SPOOF_H_
