#ifndef CLAWSER_GPU_INFO_SPOOF_H_
#define CLAWSER_GPU_INFO_SPOOF_H_

#include <cstdint>
#include <string>

namespace clawser {

struct GpuHardwareIds {
  uint32_t vendor_id;
  uint32_t device_id;
};

bool ShouldSpoofGpuInfo();

std::string GetSpoofedGlVendor();
std::string GetSpoofedGlRenderer();

GpuHardwareIds GetSpoofedGpuHardwareIds();

std::string GetSpoofedDriverVendor();
std::string GetSpoofedDriverVersion();

}  // namespace clawser

#endif  // CLAWSER_GPU_INFO_SPOOF_H_
