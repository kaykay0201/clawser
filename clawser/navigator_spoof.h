#ifndef CLAWSER_NAVIGATOR_SPOOF_H_
#define CLAWSER_NAVIGATOR_SPOOF_H_

#include <string>
#include <vector>

#include "clawser/clawser_export.h"

namespace clawser {

CLAWSER_EXPORT bool IsNavigatorSpoofEnabled();

CLAWSER_EXPORT std::string GetSpoofedUserAgent();
CLAWSER_EXPORT std::string GetSpoofedPlatform();
CLAWSER_EXPORT std::string GetSpoofedVendor();
CLAWSER_EXPORT std::string GetSpoofedAppVersion();
CLAWSER_EXPORT std::string GetSpoofedLanguage();
CLAWSER_EXPORT std::vector<std::string> GetSpoofedLanguages();
CLAWSER_EXPORT int GetSpoofedHardwareConcurrency();
CLAWSER_EXPORT int GetSpoofedDeviceMemory();
CLAWSER_EXPORT int GetSpoofedMaxTouchPoints();
CLAWSER_EXPORT std::string GetSpoofedDoNotTrack();

struct CLAWSER_EXPORT SpoofedBrandVersion {
  std::string brand;
  std::string version;
};

CLAWSER_EXPORT std::vector<SpoofedBrandVersion> GetSpoofedBrands();

}  // namespace clawser

#endif  // CLAWSER_NAVIGATOR_SPOOF_H_
