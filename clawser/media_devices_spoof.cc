#include "clawser/media_devices_spoof.h"

#include <cstdint>
#include <sstream>

#include "clawser/clawser_config.h"

namespace clawser {

SpoofedMediaDevice::SpoofedMediaDevice() = default;
SpoofedMediaDevice::SpoofedMediaDevice(const SpoofedMediaDevice&) = default;
SpoofedMediaDevice& SpoofedMediaDevice::operator=(const SpoofedMediaDevice&) = default;
SpoofedMediaDevice::SpoofedMediaDevice(SpoofedMediaDevice&&) = default;
SpoofedMediaDevice& SpoofedMediaDevice::operator=(SpoofedMediaDevice&&) = default;
SpoofedMediaDevice::~SpoofedMediaDevice() = default;

namespace {

uint64_t FnvHash64(const std::string& data) {
  uint64_t hash = 14695981039346656037ULL;
  for (char c : data) {
    hash ^= static_cast<uint64_t>(static_cast<unsigned char>(c));
    hash *= 1099511628211ULL;
  }
  return hash;
}

std::string Uint64ToHex(uint64_t value) {
  static const char kHexChars[] = "0123456789abcdef";
  std::string result(16, '0');
  for (int i = 15; i >= 0; --i) {
    result[i] = kHexChars[value & 0xf];
    value >>= 4;
  }
  return result;
}

}  // namespace

std::string GenerateDeterministicDeviceId(const std::string& profile_id,
                                          const std::string& kind,
                                          int index) {
  std::string input = profile_id + ":" + kind + ":" + std::to_string(index);
  uint64_t hash1 = FnvHash64(input);
  uint64_t hash2 = FnvHash64(input + ":salt");
  return Uint64ToHex(hash1) + Uint64ToHex(hash2) + Uint64ToHex(hash1 ^ hash2) +
         Uint64ToHex(hash2 ^ 0xDEADBEEFCAFEBABEULL);
}

std::vector<SpoofedMediaDevice> GetSpoofedMediaDevices() {
  std::vector<SpoofedMediaDevice> devices;

  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return devices;

  const auto& config = ClawserConfigManager::GetInstance().GetConfig();
  const auto& media = config.media_devices;
  const std::string& profile_id = config.profile_id;

  for (int i = 0; i < media.audio_inputs; ++i) {
    SpoofedMediaDevice dev;
    dev.device_id = GenerateDeterministicDeviceId(profile_id, "audioinput", i);
    dev.kind = "audioinput";
    dev.label = "";
    dev.group_id = Uint64ToHex(
        FnvHash64(profile_id + ":group:audioinput:" + std::to_string(i)));
    devices.push_back(std::move(dev));
  }

  for (int i = 0; i < media.audio_outputs; ++i) {
    SpoofedMediaDevice dev;
    dev.device_id = GenerateDeterministicDeviceId(profile_id, "audiooutput", i);
    dev.kind = "audiooutput";
    dev.label = "";
    dev.group_id = Uint64ToHex(
        FnvHash64(profile_id + ":group:audiooutput:" + std::to_string(i)));
    devices.push_back(std::move(dev));
  }

  for (int i = 0; i < media.video_inputs; ++i) {
    SpoofedMediaDevice dev;
    dev.device_id = GenerateDeterministicDeviceId(profile_id, "videoinput", i);
    dev.kind = "videoinput";
    dev.label = "";
    dev.group_id = Uint64ToHex(
        FnvHash64(profile_id + ":group:videoinput:" + std::to_string(i)));
    devices.push_back(std::move(dev));
  }

  return devices;
}

}  // namespace clawser
