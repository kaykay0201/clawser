#include "clawser/hardware_profiles.h"

#include "base/rand_util.h"

namespace clawser {

HardwareProfile::HardwareProfile() = default;
HardwareProfile::HardwareProfile(const std::string& vendor,
                                 const std::string& renderer,
                                 int concurrency,
                                 int memory,
                                 int width,
                                 int height)
    : gl_vendor(vendor),
      gl_renderer(renderer),
      hardware_concurrency(concurrency),
      device_memory(memory),
      screen_width(width),
      screen_height(height) {}
HardwareProfile::HardwareProfile(const HardwareProfile&) = default;
HardwareProfile& HardwareProfile::operator=(const HardwareProfile&) = default;
HardwareProfile::HardwareProfile(HardwareProfile&&) = default;
HardwareProfile& HardwareProfile::operator=(HardwareProfile&&) = default;
HardwareProfile::~HardwareProfile() = default;

const std::vector<HardwareProfile>& GetHardwareProfiles() {
  static const std::vector<HardwareProfile> profiles = {

      // ================================================================
      // NVIDIA DESKTOP — GeForce RTX 30 series (2020-2022, still common)
      // ================================================================
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 3060 Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       12, 16, 1920, 1080},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 3060 Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       8, 16, 1920, 1080},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 3060 Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       16, 32, 2560, 1440},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 3060 Ti Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       12, 16, 1920, 1080},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 3060 Ti Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       16, 16, 2560, 1440},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 3060 Ti Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       16, 32, 2560, 1440},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 3070 Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       12, 16, 2560, 1440},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 3070 Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       16, 32, 2560, 1440},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 3070 Ti Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       16, 32, 2560, 1440},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 3080 Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       16, 32, 2560, 1440},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 3080 Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       16, 32, 3840, 2160},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 3080 Ti Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       16, 32, 3840, 2160},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 3090 Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       24, 64, 3840, 2160},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 3090 Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       16, 32, 3840, 2160},

      // ================================================================
      // NVIDIA DESKTOP — GeForce RTX 40 series (2022-2024)
      // ================================================================
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 4060 Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       8, 16, 1920, 1080},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 4060 Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       12, 16, 1920, 1080},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 4060 Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       16, 16, 2560, 1440},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 4060 Ti Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       12, 16, 1920, 1080},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 4060 Ti Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       16, 16, 2560, 1440},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 4060 Ti Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       16, 32, 2560, 1440},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 4070 Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       12, 16, 2560, 1440},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 4070 Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       16, 32, 2560, 1440},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 4070 Super Direct3D11 vs_5_0 "
       "ps_5_0, D3D11)",
       16, 32, 2560, 1440},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 4070 Ti Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       16, 32, 2560, 1440},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 4070 Ti Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       16, 32, 3840, 2160},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 4070 Ti Super Direct3D11 vs_5_0 "
       "ps_5_0, D3D11)",
       16, 32, 3840, 2160},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 4080 Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       16, 32, 3840, 2160},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 4080 Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       24, 32, 3840, 2160},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 4080 Super Direct3D11 vs_5_0 "
       "ps_5_0, D3D11)",
       24, 32, 3840, 2160},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 4090 Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       24, 64, 3840, 2160},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 4090 Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       32, 64, 3840, 2160},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 4090 Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       24, 32, 3840, 2160},

      // ================================================================
      // NVIDIA DESKTOP — GeForce RTX 50 series (2025)
      // ================================================================
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 5070 Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       12, 32, 2560, 1440},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 5070 Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       16, 32, 2560, 1440},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 5070 Ti Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       16, 32, 2560, 1440},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 5070 Ti Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       16, 32, 3840, 2160},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 5080 Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       16, 32, 3840, 2160},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 5080 Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       24, 32, 3840, 2160},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 5090 Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       24, 64, 3840, 2160},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 5090 Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       32, 64, 3840, 2160},

      // ================================================================
      // NVIDIA LAPTOP — RTX 30 Mobile (2021-2023)
      // ================================================================
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 3050 Laptop GPU Direct3D11 vs_5_0 "
       "ps_5_0, D3D11)",
       8, 8, 1920, 1080},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 3050 Laptop GPU Direct3D11 vs_5_0 "
       "ps_5_0, D3D11)",
       8, 16, 1920, 1080},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 3050 Ti Laptop GPU Direct3D11 "
       "vs_5_0 ps_5_0, D3D11)",
       8, 16, 1920, 1080},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 3060 Laptop GPU Direct3D11 vs_5_0 "
       "ps_5_0, D3D11)",
       8, 16, 1920, 1080},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 3060 Laptop GPU Direct3D11 vs_5_0 "
       "ps_5_0, D3D11)",
       12, 16, 1920, 1080},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 3070 Laptop GPU Direct3D11 vs_5_0 "
       "ps_5_0, D3D11)",
       8, 16, 2560, 1440},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 3070 Laptop GPU Direct3D11 vs_5_0 "
       "ps_5_0, D3D11)",
       12, 32, 2560, 1440},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 3070 Ti Laptop GPU Direct3D11 "
       "vs_5_0 ps_5_0, D3D11)",
       12, 32, 2560, 1440},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 3080 Laptop GPU Direct3D11 vs_5_0 "
       "ps_5_0, D3D11)",
       16, 32, 2560, 1440},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 3080 Ti Laptop GPU Direct3D11 "
       "vs_5_0 ps_5_0, D3D11)",
       16, 32, 2560, 1440},

      // ================================================================
      // NVIDIA LAPTOP — RTX 40 Mobile (2023-2025)
      // ================================================================
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 4050 Laptop GPU Direct3D11 vs_5_0 "
       "ps_5_0, D3D11)",
       8, 16, 1920, 1080},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 4050 Laptop GPU Direct3D11 vs_5_0 "
       "ps_5_0, D3D11)",
       12, 16, 1920, 1080},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 4060 Laptop GPU Direct3D11 vs_5_0 "
       "ps_5_0, D3D11)",
       12, 16, 1920, 1080},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 4060 Laptop GPU Direct3D11 vs_5_0 "
       "ps_5_0, D3D11)",
       12, 16, 2560, 1440},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 4060 Laptop GPU Direct3D11 vs_5_0 "
       "ps_5_0, D3D11)",
       16, 32, 2560, 1440},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 4070 Laptop GPU Direct3D11 vs_5_0 "
       "ps_5_0, D3D11)",
       12, 16, 2560, 1440},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 4070 Laptop GPU Direct3D11 vs_5_0 "
       "ps_5_0, D3D11)",
       16, 32, 2560, 1440},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 4080 Laptop GPU Direct3D11 vs_5_0 "
       "ps_5_0, D3D11)",
       16, 32, 2560, 1440},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 4090 Laptop GPU Direct3D11 vs_5_0 "
       "ps_5_0, D3D11)",
       16, 32, 2560, 1440},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 4090 Laptop GPU Direct3D11 vs_5_0 "
       "ps_5_0, D3D11)",
       24, 64, 3840, 2160},

      // ================================================================
      // NVIDIA LAPTOP — RTX 50 Mobile (2025)
      // ================================================================
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 5070 Laptop GPU Direct3D11 vs_5_0 "
       "ps_5_0, D3D11)",
       12, 16, 2560, 1440},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 5070 Laptop GPU Direct3D11 vs_5_0 "
       "ps_5_0, D3D11)",
       16, 32, 2560, 1440},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 5070 Ti Laptop GPU Direct3D11 "
       "vs_5_0 ps_5_0, D3D11)",
       16, 32, 2560, 1440},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 5080 Laptop GPU Direct3D11 vs_5_0 "
       "ps_5_0, D3D11)",
       16, 32, 2560, 1440},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce RTX 5090 Laptop GPU Direct3D11 vs_5_0 "
       "ps_5_0, D3D11)",
       24, 64, 3840, 2160},

      // ================================================================
      // NVIDIA DESKTOP — GTX 16 series (budget, still very common)
      // ================================================================
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce GTX 1650 Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       8, 8, 1920, 1080},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce GTX 1650 Super Direct3D11 vs_5_0 "
       "ps_5_0, D3D11)",
       8, 16, 1920, 1080},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce GTX 1660 Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       8, 16, 1920, 1080},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce GTX 1660 Super Direct3D11 vs_5_0 "
       "ps_5_0, D3D11)",
       12, 16, 1920, 1080},
      {"Google Inc. (NVIDIA)",
       "ANGLE (NVIDIA, NVIDIA GeForce GTX 1660 Ti Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       12, 16, 1920, 1080},

      // ================================================================
      // AMD DESKTOP — Radeon RX 6000 series (2020-2023)
      // ================================================================
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon RX 6500 XT Direct3D11 vs_5_0 ps_5_0, D3D11)",
       8, 16, 1920, 1080},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon RX 6600 Direct3D11 vs_5_0 ps_5_0, D3D11)",
       8, 16, 1920, 1080},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon RX 6600 XT Direct3D11 vs_5_0 ps_5_0, D3D11)",
       12, 16, 1920, 1080},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon RX 6600 XT Direct3D11 vs_5_0 ps_5_0, D3D11)",
       8, 16, 1920, 1080},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon RX 6650 XT Direct3D11 vs_5_0 ps_5_0, D3D11)",
       12, 16, 1920, 1080},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon RX 6700 XT Direct3D11 vs_5_0 ps_5_0, D3D11)",
       12, 16, 2560, 1440},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon RX 6700 XT Direct3D11 vs_5_0 ps_5_0, D3D11)",
       16, 32, 2560, 1440},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon RX 6750 XT Direct3D11 vs_5_0 ps_5_0, D3D11)",
       12, 16, 2560, 1440},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon RX 6800 Direct3D11 vs_5_0 ps_5_0, D3D11)",
       16, 32, 2560, 1440},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon RX 6800 XT Direct3D11 vs_5_0 ps_5_0, D3D11)",
       16, 32, 2560, 1440},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon RX 6800 XT Direct3D11 vs_5_0 ps_5_0, D3D11)",
       16, 32, 3840, 2160},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon RX 6900 XT Direct3D11 vs_5_0 ps_5_0, D3D11)",
       16, 32, 3840, 2160},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon RX 6950 XT Direct3D11 vs_5_0 ps_5_0, D3D11)",
       16, 32, 3840, 2160},

      // ================================================================
      // AMD DESKTOP — Radeon RX 7000 series (2023-2025)
      // ================================================================
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon RX 7600 Direct3D11 vs_5_0 ps_5_0, D3D11)",
       8, 16, 1920, 1080},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon RX 7600 Direct3D11 vs_5_0 ps_5_0, D3D11)",
       12, 16, 1920, 1080},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon RX 7600 XT Direct3D11 vs_5_0 ps_5_0, D3D11)",
       12, 16, 1920, 1080},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon RX 7700 XT Direct3D11 vs_5_0 ps_5_0, D3D11)",
       12, 16, 2560, 1440},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon RX 7700 XT Direct3D11 vs_5_0 ps_5_0, D3D11)",
       16, 32, 2560, 1440},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon RX 7800 XT Direct3D11 vs_5_0 ps_5_0, D3D11)",
       12, 16, 2560, 1440},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon RX 7800 XT Direct3D11 vs_5_0 ps_5_0, D3D11)",
       16, 32, 2560, 1440},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon RX 7900 GRE Direct3D11 vs_5_0 ps_5_0, D3D11)",
       16, 32, 2560, 1440},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon RX 7900 XT Direct3D11 vs_5_0 ps_5_0, D3D11)",
       16, 32, 3840, 2160},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon RX 7900 XTX Direct3D11 vs_5_0 ps_5_0, D3D11)",
       16, 32, 3840, 2160},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon RX 7900 XTX Direct3D11 vs_5_0 ps_5_0, D3D11)",
       24, 64, 3840, 2160},

      // ================================================================
      // AMD DESKTOP — Radeon RX 9000 series (2025)
      // ================================================================
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon RX 9070 Direct3D11 vs_5_0 ps_5_0, D3D11)",
       12, 16, 2560, 1440},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon RX 9070 XT Direct3D11 vs_5_0 ps_5_0, D3D11)",
       16, 32, 2560, 1440},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon RX 9070 XT Direct3D11 vs_5_0 ps_5_0, D3D11)",
       16, 32, 3840, 2160},

      // ================================================================
      // AMD LAPTOP — Radeon RX Mobile (2022-2025)
      // ================================================================
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon RX 6500M Direct3D11 vs_5_0 ps_5_0, D3D11)",
       8, 8, 1920, 1080},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon RX 6600M Direct3D11 vs_5_0 ps_5_0, D3D11)",
       8, 16, 1920, 1080},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon RX 6700M Direct3D11 vs_5_0 ps_5_0, D3D11)",
       8, 16, 1920, 1080},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon RX 7600M XT Direct3D11 vs_5_0 ps_5_0, D3D11)",
       12, 16, 1920, 1080},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon RX 7700S Direct3D11 vs_5_0 ps_5_0, D3D11)",
       12, 16, 2560, 1440},

      // ================================================================
      // AMD INTEGRATED — Radeon Graphics (Ryzen APU, very common laptops)
      // ================================================================
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon(TM) Graphics Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       8, 8, 1920, 1080},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon(TM) Graphics Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       12, 16, 1920, 1080},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon(TM) Graphics Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       16, 16, 1920, 1080},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon 780M Graphics Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       16, 16, 1920, 1080},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon 780M Graphics Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       16, 32, 2560, 1440},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon 760M Graphics Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       12, 16, 1920, 1080},
      {"Google Inc. (AMD)",
       "ANGLE (AMD, AMD Radeon 890M Graphics Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       16, 32, 2560, 1440},

      // ================================================================
      // INTEL INTEGRATED — UHD/Iris Xe (very common, 2020-2025)
      // ================================================================
      {"Google Inc. (Intel)",
       "ANGLE (Intel, Intel(R) UHD Graphics 630 Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       8, 8, 1920, 1080},
      {"Google Inc. (Intel)",
       "ANGLE (Intel, Intel(R) UHD Graphics 630 Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       12, 16, 1920, 1080},
      {"Google Inc. (Intel)",
       "ANGLE (Intel, Intel(R) UHD Graphics 730 Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       8, 16, 1920, 1080},
      {"Google Inc. (Intel)",
       "ANGLE (Intel, Intel(R) UHD Graphics 730 Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       12, 16, 1920, 1080},
      {"Google Inc. (Intel)",
       "ANGLE (Intel, Intel(R) UHD Graphics 770 Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       16, 16, 1920, 1080},
      {"Google Inc. (Intel)",
       "ANGLE (Intel, Intel(R) UHD Graphics 770 Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       16, 32, 2560, 1440},
      {"Google Inc. (Intel)",
       "ANGLE (Intel, Intel(R) Iris(R) Xe Graphics Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       4, 8, 1920, 1080},
      {"Google Inc. (Intel)",
       "ANGLE (Intel, Intel(R) Iris(R) Xe Graphics Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       8, 8, 1920, 1080},
      {"Google Inc. (Intel)",
       "ANGLE (Intel, Intel(R) Iris(R) Xe Graphics Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       8, 16, 1920, 1080},
      {"Google Inc. (Intel)",
       "ANGLE (Intel, Intel(R) Iris(R) Xe Graphics Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       12, 16, 1920, 1080},
      {"Google Inc. (Intel)",
       "ANGLE (Intel, Intel(R) Iris(R) Xe Graphics Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       12, 16, 2560, 1440},
      {"Google Inc. (Intel)",
       "ANGLE (Intel, Intel(R) Iris(R) Xe Graphics Direct3D11 vs_5_0 ps_5_0, "
       "D3D11)",
       16, 16, 1920, 1080},

      // ================================================================
      // INTEL DISCRETE — Arc (2022-2025)
      // ================================================================
      {"Google Inc. (Intel)",
       "ANGLE (Intel, Intel(R) Arc(TM) A380 Graphics Direct3D11 vs_5_0 "
       "ps_5_0, D3D11)",
       8, 16, 1920, 1080},
      {"Google Inc. (Intel)",
       "ANGLE (Intel, Intel(R) Arc(TM) A580 Graphics Direct3D11 vs_5_0 "
       "ps_5_0, D3D11)",
       12, 16, 1920, 1080},
      {"Google Inc. (Intel)",
       "ANGLE (Intel, Intel(R) Arc(TM) A750 Graphics Direct3D11 vs_5_0 "
       "ps_5_0, D3D11)",
       12, 16, 2560, 1440},
      {"Google Inc. (Intel)",
       "ANGLE (Intel, Intel(R) Arc(TM) A770 Graphics Direct3D11 vs_5_0 "
       "ps_5_0, D3D11)",
       16, 16, 2560, 1440},
      {"Google Inc. (Intel)",
       "ANGLE (Intel, Intel(R) Arc(TM) A770 Graphics Direct3D11 vs_5_0 "
       "ps_5_0, D3D11)",
       16, 32, 2560, 1440},
      {"Google Inc. (Intel)",
       "ANGLE (Intel, Intel(R) Arc(TM) B580 Graphics Direct3D11 vs_5_0 "
       "ps_5_0, D3D11)",
       12, 16, 2560, 1440},
      {"Google Inc. (Intel)",
       "ANGLE (Intel, Intel(R) Arc(TM) B580 Graphics Direct3D11 vs_5_0 "
       "ps_5_0, D3D11)",
       16, 32, 2560, 1440},
  };
  return profiles;
}

const HardwareProfile& SelectRandomProfile() {
  const auto& profiles = GetHardwareProfiles();
  size_t index = base::RandGenerator(profiles.size());
  return profiles[index];
}

}  // namespace clawser
