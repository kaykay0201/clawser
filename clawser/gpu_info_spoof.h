#ifndef CLAWSER_GPU_INFO_SPOOF_H_
#define CLAWSER_GPU_INFO_SPOOF_H_

#include <cstdint>
#include <string>

#include "clawser/clawser_export.h"

namespace clawser {

struct CLAWSER_EXPORT GpuHardwareIds {
  uint32_t vendor_id;
  uint32_t device_id;
};

CLAWSER_EXPORT bool ShouldSpoofGpuInfo();

CLAWSER_EXPORT std::string GetSpoofedGlVendor();
CLAWSER_EXPORT std::string GetSpoofedGlRenderer();

CLAWSER_EXPORT GpuHardwareIds GetSpoofedGpuHardwareIds();

CLAWSER_EXPORT std::string GetSpoofedDriverVendor();
CLAWSER_EXPORT std::string GetSpoofedDriverVersion();

}  // namespace clawser

#endif  // CLAWSER_GPU_INFO_SPOOF_H_
