#ifndef CLAWSER_HARDWARE_PROFILES_H_
#define CLAWSER_HARDWARE_PROFILES_H_

#include <string>
#include <vector>

namespace clawser {

struct HardwareProfile {
  HardwareProfile();
  HardwareProfile(const std::string& vendor,
                  const std::string& renderer,
                  int concurrency,
                  int memory,
                  int width,
                  int height);
  HardwareProfile(const HardwareProfile&);
  HardwareProfile& operator=(const HardwareProfile&);
  HardwareProfile(HardwareProfile&&);
  HardwareProfile& operator=(HardwareProfile&&);
  ~HardwareProfile();

  std::string gl_vendor;
  std::string gl_renderer;
  int hardware_concurrency;
  int device_memory;
  int screen_width;
  int screen_height;
};

const std::vector<HardwareProfile>& GetHardwareProfiles();

const HardwareProfile& SelectRandomProfile();

}  // namespace clawser

#endif  // CLAWSER_HARDWARE_PROFILES_H_
