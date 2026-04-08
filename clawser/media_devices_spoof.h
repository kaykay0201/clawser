#ifndef CLAWSER_MEDIA_DEVICES_SPOOF_H_
#define CLAWSER_MEDIA_DEVICES_SPOOF_H_

#include <string>
#include <vector>

#include "clawser/clawser_export.h"

namespace clawser {

struct CLAWSER_EXPORT SpoofedMediaDevice {
  SpoofedMediaDevice();
  SpoofedMediaDevice(const SpoofedMediaDevice&);
  SpoofedMediaDevice& operator=(const SpoofedMediaDevice&);
  SpoofedMediaDevice(SpoofedMediaDevice&&);
  SpoofedMediaDevice& operator=(SpoofedMediaDevice&&);
  ~SpoofedMediaDevice();

  std::string device_id;
  std::string kind;
  std::string label;
  std::string group_id;
};

CLAWSER_EXPORT std::vector<SpoofedMediaDevice> GetSpoofedMediaDevices();

CLAWSER_EXPORT std::string GenerateDeterministicDeviceId(const std::string& profile_id,
                                          const std::string& kind,
                                          int index);

}  // namespace clawser

#endif  // CLAWSER_MEDIA_DEVICES_SPOOF_H_
