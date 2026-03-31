#ifndef CLAWSER_CLAWSER_CONFIG_H_
#define CLAWSER_CLAWSER_CONFIG_H_

#include <cstdint>
#include <optional>
#include <string>
#include <vector>

#include "base/values.h"

namespace clawser {

struct BrandVersion {
  BrandVersion();
  BrandVersion(const BrandVersion&);
  BrandVersion& operator=(const BrandVersion&);
  BrandVersion(BrandVersion&&);
  BrandVersion& operator=(BrandVersion&&);
  ~BrandVersion();

  std::string brand;
  std::string version;
};

struct UserAgentData {
  UserAgentData();
  UserAgentData(const UserAgentData&);
  UserAgentData& operator=(const UserAgentData&);
  UserAgentData(UserAgentData&&);
  UserAgentData& operator=(UserAgentData&&);
  ~UserAgentData();

  std::vector<BrandVersion> brands;
  bool mobile = false;
  std::string platform;
  std::string platform_version;
  std::string architecture;
  std::string model;
  std::string bitness;
};

struct NavigatorConfig {
  NavigatorConfig();
  NavigatorConfig(const NavigatorConfig&);
  NavigatorConfig& operator=(const NavigatorConfig&);
  NavigatorConfig(NavigatorConfig&&);
  NavigatorConfig& operator=(NavigatorConfig&&);
  ~NavigatorConfig();

  std::string user_agent;
  std::string platform;
  std::string vendor;
  std::string app_version;
  std::string language;
  std::vector<std::string> languages;
  int hardware_concurrency = 4;
  int device_memory = 8;
  int max_touch_points = 0;
  std::optional<bool> do_not_track;
  UserAgentData user_agent_data;
};

struct ScreenConfig {
  int width = 1920;
  int height = 1080;
  int avail_width = 1920;
  int avail_height = 1040;
  int color_depth = 24;
  int pixel_depth = 24;
  double device_pixel_ratio = 1.0;
};

struct WebGlParams {
  WebGlParams();
  WebGlParams(const WebGlParams&);
  WebGlParams& operator=(const WebGlParams&);
  WebGlParams(WebGlParams&&);
  WebGlParams& operator=(WebGlParams&&);
  ~WebGlParams();

  int max_texture_size = 16384;
  int max_renderbuffer_size = 16384;
  std::vector<int> max_viewport_dims = {32767, 32767};
  int max_vertex_attribs = 16;
  int max_varying_vectors = 30;
  std::vector<int> aliased_line_width_range = {1, 1};
  std::vector<int> aliased_point_size_range = {1, 1024};
  int max_fragment_uniform_vectors = 1024;
  int max_vertex_uniform_vectors = 4096;
  std::vector<std::string> extensions;
};

struct GpuConfig {
  GpuConfig();
  GpuConfig(const GpuConfig&);
  GpuConfig& operator=(const GpuConfig&);
  GpuConfig(GpuConfig&&);
  GpuConfig& operator=(GpuConfig&&);
  ~GpuConfig();

  std::string vendor;
  std::string renderer;
  WebGlParams webgl_params;
};

struct NoiseSeeds {
  uint64_t canvas = 0;
  uint64_t webgl = 0;
  uint64_t audio = 0;
  uint64_t client_rects = 0;
};

struct MediaDevicesConfig {
  int audio_inputs = 1;
  int audio_outputs = 2;
  int video_inputs = 1;
};

struct SpeechVoice {
  SpeechVoice();
  SpeechVoice(const SpeechVoice&);
  SpeechVoice& operator=(const SpeechVoice&);
  SpeechVoice(SpeechVoice&&);
  SpeechVoice& operator=(SpeechVoice&&);
  ~SpeechVoice();

  std::string name;
  std::string lang;
};

enum class WebRtcPolicy {
  kDisabled,
  kProxyOnly,
  kSpoofed,
};

struct WebRtcConfig {
  WebRtcConfig();
  WebRtcConfig(const WebRtcConfig&);
  WebRtcConfig& operator=(const WebRtcConfig&);
  WebRtcConfig(WebRtcConfig&&);
  WebRtcConfig& operator=(WebRtcConfig&&);
  ~WebRtcConfig();

  WebRtcPolicy policy = WebRtcPolicy::kDisabled;
  std::string fake_local_ip;
};

struct BatteryConfig {
  bool enabled = false;
};

struct ClawserConfig {
  ClawserConfig();
  ClawserConfig(const ClawserConfig&);
  ClawserConfig& operator=(const ClawserConfig&);
  ClawserConfig(ClawserConfig&&);
  ClawserConfig& operator=(ClawserConfig&&);
  ~ClawserConfig();

  int version = 1;
  std::string profile_id;
  NavigatorConfig navigator;
  ScreenConfig screen;
  GpuConfig gpu;
  NoiseSeeds noise_seeds;
  std::string timezone;
  std::string locale;
  std::vector<std::string> fonts;
  MediaDevicesConfig media_devices;
  std::vector<SpeechVoice> speech_voices;
  WebRtcConfig webrtc;
  BatteryConfig battery;
  bool bluetooth_enabled = false;
  bool usb_enabled = false;
};

class ClawserConfigManager {
 public:
  static ClawserConfigManager& GetInstance();

  bool LoadFromFile(const std::string& path);
  bool IsLoaded() const;
  const ClawserConfig& GetConfig() const;

 private:
  ClawserConfigManager();
  ~ClawserConfigManager();

  bool ParseJson(const std::string& json_content);
  bool ParseNavigator(const base::Value::Dict& dict);
  bool ParseScreen(const base::Value::Dict& dict);
  bool ParseGpu(const base::Value::Dict& dict);
  bool ParseNoiseSeeds(const base::Value::Dict& dict);
  bool ParseMediaDevices(const base::Value::Dict& dict);
  bool ParseWebRtc(const base::Value::Dict& dict);

  bool loaded_ = false;
  ClawserConfig config_;
};

}  // namespace clawser

#endif  // CLAWSER_CLAWSER_CONFIG_H_
