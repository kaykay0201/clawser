#include "clawser/gpu_info_spoof.h"

#include <string>

#include "base/strings/string_number_conversions.h"
#include "clawser/clawser_config.h"

namespace clawser {

namespace {

struct VendorMapping {
  const char* name;
  uint32_t vendor_id;
  uint32_t default_device_id;
  const char* driver_vendor;
};

const VendorMapping kVendorMappings[] = {
    {"NVIDIA", 0x10DE, 0x2504, "NVIDIA"},
    {"nvidia", 0x10DE, 0x2504, "NVIDIA"},
    {"Intel", 0x8086, 0x9A49, "Intel"},
    {"intel", 0x8086, 0x9A49, "Intel"},
    {"AMD", 0x1002, 0x73DF, "AMD"},
    {"amd", 0x1002, 0x73DF, "AMD"},
    {"ATI", 0x1002, 0x73DF, "AMD"},
    {"Google", 0x1AE0, 0x003D, "Google"},
};

const VendorMapping* FindVendorMapping(const std::string& vendor_str) {
  for (const auto& mapping : kVendorMappings) {
    if (vendor_str.find(mapping.name) != std::string::npos)
      return &mapping;
  }
  return nullptr;
}

uint32_t ExtractDeviceIdFromRenderer(const std::string& renderer) {
  size_t pos = renderer.find("0x");
  if (pos != std::string::npos && pos + 6 <= renderer.size()) {
    std::string hex_str = renderer.substr(pos + 2, 4);
    uint32_t val = 0;
    if (base::HexStringToUInt(hex_str, &val))
      return val;
  }
  return 0;
}

}  // namespace

bool ShouldSpoofGpuInfo() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return false;
  const auto& config = ClawserConfigManager::GetInstance().GetConfig();
  return !config.gpu.vendor.empty() || !config.gpu.renderer.empty();
}

std::string GetSpoofedGlVendor() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return std::string();
  return ClawserConfigManager::GetInstance().GetConfig().gpu.vendor;
}

std::string GetSpoofedGlRenderer() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return std::string();
  return ClawserConfigManager::GetInstance().GetConfig().gpu.renderer;
}

GpuHardwareIds GetSpoofedGpuHardwareIds() {
  GpuHardwareIds ids = {0, 0};

  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return ids;

  const auto& config = ClawserConfigManager::GetInstance().GetConfig();
  const VendorMapping* mapping = FindVendorMapping(config.gpu.vendor);

  if (mapping) {
    ids.vendor_id = mapping->vendor_id;
    uint32_t extracted = ExtractDeviceIdFromRenderer(config.gpu.renderer);
    ids.device_id = extracted != 0 ? extracted : mapping->default_device_id;
  }

  return ids;
}

std::string GetSpoofedDriverVendor() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return std::string();

  const auto& config = ClawserConfigManager::GetInstance().GetConfig();
  const VendorMapping* mapping = FindVendorMapping(config.gpu.vendor);
  if (mapping)
    return mapping->driver_vendor;
  return config.gpu.vendor;
}

std::string GetSpoofedDriverVersion() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return std::string();

  const auto& config = ClawserConfigManager::GetInstance().GetConfig();
  if (config.gpu.renderer.find("Direct3D11") != std::string::npos)
    return "31.0.15.4601";
  return "31.0.101.4502";
}

}  // namespace clawser
