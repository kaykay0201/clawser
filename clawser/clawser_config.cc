#include "clawser/clawser_config.h"

#include <string>

#include "clawser/hardware_profiles.h"

#include "base/files/file_util.h"
#include "base/rand_util.h"
#include "base/json/json_reader.h"
#include "base/logging.h"
#include "base/strings/string_number_conversions.h"
#include "base/values.h"

namespace clawser {

namespace {

std::string GetString(const base::Value::Dict& dict,
                      const char* key,
                      const std::string& default_value = std::string()) {
  const std::string* value = dict.FindString(key);
  if (value)
    return *value;
  return default_value;
}

int GetInt(const base::Value::Dict& dict, const char* key, int default_value) {
  std::optional<int> value = dict.FindInt(key);
  if (value.has_value())
    return value.value();
  return default_value;
}

double GetDouble(const base::Value::Dict& dict,
                 const char* key,
                 double default_value) {
  std::optional<double> value = dict.FindDouble(key);
  if (value.has_value())
    return value.value();
  std::optional<int> int_value = dict.FindInt(key);
  if (int_value.has_value())
    return static_cast<double>(int_value.value());
  return default_value;
}

bool GetBool(const base::Value::Dict& dict,
             const char* key,
             bool default_value) {
  std::optional<bool> value = dict.FindBool(key);
  if (value.has_value())
    return value.value();
  return default_value;
}

std::vector<std::string> GetStringList(const base::Value::Dict& dict,
                                       const char* key) {
  std::vector<std::string> result;
  const base::Value::List* list = dict.FindList(key);
  if (!list)
    return result;
  for (const auto& item : *list) {
    if (item.is_string())
      result.push_back(item.GetString());
  }
  return result;
}

std::vector<int> GetIntList(const base::Value::Dict& dict, const char* key) {
  std::vector<int> result;
  const base::Value::List* list = dict.FindList(key);
  if (!list)
    return result;
  for (const auto& item : *list) {
    if (item.is_int())
      result.push_back(item.GetInt());
  }
  return result;
}

uint64_t GetUint64(const base::Value::Dict& dict,
                   const char* key,
                   uint64_t default_value) {
  std::optional<double> dbl = dict.FindDouble(key);
  if (dbl.has_value())
    return static_cast<uint64_t>(dbl.value());
  std::optional<int> int_val = dict.FindInt(key);
  if (int_val.has_value())
    return static_cast<uint64_t>(int_val.value());
  const std::string* str = dict.FindString(key);
  if (str) {
    uint64_t parsed = 0;
    if (base::StringToUint64(*str, &parsed))
      return parsed;
  }
  return default_value;
}

}  // namespace

BrandVersion::BrandVersion() = default;
BrandVersion::BrandVersion(const BrandVersion&) = default;
BrandVersion& BrandVersion::operator=(const BrandVersion&) = default;
BrandVersion::BrandVersion(BrandVersion&&) = default;
BrandVersion& BrandVersion::operator=(BrandVersion&&) = default;
BrandVersion::~BrandVersion() = default;

UserAgentData::UserAgentData() = default;
UserAgentData::UserAgentData(const UserAgentData&) = default;
UserAgentData& UserAgentData::operator=(const UserAgentData&) = default;
UserAgentData::UserAgentData(UserAgentData&&) = default;
UserAgentData& UserAgentData::operator=(UserAgentData&&) = default;
UserAgentData::~UserAgentData() = default;

NavigatorConfig::NavigatorConfig() = default;
NavigatorConfig::NavigatorConfig(const NavigatorConfig&) = default;
NavigatorConfig& NavigatorConfig::operator=(const NavigatorConfig&) = default;
NavigatorConfig::NavigatorConfig(NavigatorConfig&&) = default;
NavigatorConfig& NavigatorConfig::operator=(NavigatorConfig&&) = default;
NavigatorConfig::~NavigatorConfig() = default;

WebGlParams::WebGlParams() = default;
WebGlParams::WebGlParams(const WebGlParams&) = default;
WebGlParams& WebGlParams::operator=(const WebGlParams&) = default;
WebGlParams::WebGlParams(WebGlParams&&) = default;
WebGlParams& WebGlParams::operator=(WebGlParams&&) = default;
WebGlParams::~WebGlParams() = default;

GpuConfig::GpuConfig() = default;
GpuConfig::GpuConfig(const GpuConfig&) = default;
GpuConfig& GpuConfig::operator=(const GpuConfig&) = default;
GpuConfig::GpuConfig(GpuConfig&&) = default;
GpuConfig& GpuConfig::operator=(GpuConfig&&) = default;
GpuConfig::~GpuConfig() = default;

SpeechVoice::SpeechVoice() = default;
SpeechVoice::SpeechVoice(const SpeechVoice&) = default;
SpeechVoice& SpeechVoice::operator=(const SpeechVoice&) = default;
SpeechVoice::SpeechVoice(SpeechVoice&&) = default;
SpeechVoice& SpeechVoice::operator=(SpeechVoice&&) = default;
SpeechVoice::~SpeechVoice() = default;

WebRtcConfig::WebRtcConfig() = default;
WebRtcConfig::WebRtcConfig(const WebRtcConfig&) = default;
WebRtcConfig& WebRtcConfig::operator=(const WebRtcConfig&) = default;
WebRtcConfig::WebRtcConfig(WebRtcConfig&&) = default;
WebRtcConfig& WebRtcConfig::operator=(WebRtcConfig&&) = default;
WebRtcConfig::~WebRtcConfig() = default;

ClawserConfig::ClawserConfig() = default;
ClawserConfig::ClawserConfig(const ClawserConfig&) = default;
ClawserConfig& ClawserConfig::operator=(const ClawserConfig&) = default;
ClawserConfig::ClawserConfig(ClawserConfig&&) = default;
ClawserConfig& ClawserConfig::operator=(ClawserConfig&&) = default;
ClawserConfig::~ClawserConfig() = default;

ClawserConfigManager& ClawserConfigManager::GetInstance() {
  static ClawserConfigManager instance;
  return instance;
}

ClawserConfigManager::ClawserConfigManager() = default;
ClawserConfigManager::~ClawserConfigManager() = default;

bool ClawserConfigManager::IsLoaded() const {
  return loaded_;
}

const ClawserConfig& ClawserConfigManager::GetConfig() const {
  return config_;
}

bool ClawserConfigManager::LoadFromFile(const std::string& path) {
  base::FilePath file_path = base::FilePath::FromUTF8Unsafe(path);

  std::string json_content;
  if (!base::ReadFileToString(file_path, &json_content)) {
    LOG(ERROR) << "Failed to read clawser config file: " << path;
    return false;
  }

  if (json_content.empty()) {
    LOG(ERROR) << "Clawser config file is empty: " << path;
    return false;
  }

  return ParseJson(json_content);
}

bool ClawserConfigManager::ParseJson(const std::string& json_content) {
  auto result = base::JSONReader::ReadAndReturnValueWithError(json_content);

  if (!result.has_value()) {
    LOG(ERROR) << "Failed to parse clawser config JSON: "
               << result.error().message;
    return false;
  }

  if (!result->is_dict()) {
    LOG(ERROR) << "Clawser config root is not a JSON object";
    return false;
  }

  const base::Value::Dict& root = result->GetDict();

  config_.version = GetInt(root, "version", 1);
  config_.profile_id = GetString(root, "profile_id");
  config_.timezone = GetString(root, "timezone", "UTC");
  config_.locale = GetString(root, "locale", "en-US");
  config_.fonts = GetStringList(root, "fonts");

  const base::Value::Dict* navigator_dict = root.FindDict("navigator");
  if (navigator_dict) {
    if (!ParseNavigator(*navigator_dict)) {
      LOG(ERROR) << "Failed to parse navigator config";
      return false;
    }
  }

  const base::Value::Dict* screen_dict = root.FindDict("screen");
  if (screen_dict) {
    if (!ParseScreen(*screen_dict)) {
      LOG(ERROR) << "Failed to parse screen config";
      return false;
    }
  }

  const base::Value::Dict* gpu_dict = root.FindDict("gpu");
  if (gpu_dict) {
    if (!ParseGpu(*gpu_dict)) {
      LOG(ERROR) << "Failed to parse gpu config";
      return false;
    }
  }

  const base::Value::Dict* noise_dict = root.FindDict("noise_seeds");
  if (noise_dict) {
    if (!ParseNoiseSeeds(*noise_dict)) {
      LOG(ERROR) << "Failed to parse noise_seeds config";
      return false;
    }
  }

  const base::Value::Dict* media_dict = root.FindDict("media_devices");
  if (media_dict) {
    if (!ParseMediaDevices(*media_dict)) {
      LOG(ERROR) << "Failed to parse media_devices config";
      return false;
    }
  }

  const base::Value::List* voices_list = root.FindList("speech_voices");
  if (voices_list) {
    config_.speech_voices.clear();
    for (const auto& voice_val : *voices_list) {
      if (!voice_val.is_dict())
        continue;
      const base::Value::Dict& voice_dict = voice_val.GetDict();
      SpeechVoice voice;
      voice.name = GetString(voice_dict, "name");
      voice.lang = GetString(voice_dict, "lang");
      config_.speech_voices.push_back(std::move(voice));
    }
  }

  const base::Value::Dict* webrtc_dict = root.FindDict("webrtc");
  if (webrtc_dict) {
    if (!ParseWebRtc(*webrtc_dict)) {
      LOG(ERROR) << "Failed to parse webrtc config";
      return false;
    }
  }

  const base::Value::Dict* battery_dict = root.FindDict("battery");
  if (battery_dict) {
    config_.battery.enabled = GetBool(*battery_dict, "enabled", false);
  }

  const base::Value::Dict* bluetooth_dict = root.FindDict("bluetooth");
  if (bluetooth_dict) {
    config_.bluetooth_enabled = GetBool(*bluetooth_dict, "enabled", false);
  }

  const base::Value::Dict* usb_dict = root.FindDict("usb");
  if (usb_dict) {
    config_.usb_enabled = GetBool(*usb_dict, "enabled", false);
  }

  if (config_.gpu.vendor.empty() || config_.gpu.renderer.empty()) {
    const HardwareProfile& hw = SelectRandomProfile();
    config_.gpu.vendor = hw.gl_vendor;
    config_.gpu.renderer = hw.gl_renderer;
    if (config_.navigator.hardware_concurrency == 0)
      config_.navigator.hardware_concurrency = hw.hardware_concurrency;
    if (config_.navigator.device_memory == 0)
      config_.navigator.device_memory = hw.device_memory;
    if (config_.screen.width == 0) {
      config_.screen.width = hw.screen_width;
      config_.screen.height = hw.screen_height;
      config_.screen.avail_width = hw.screen_width;
      config_.screen.avail_height = hw.screen_height - 40;
    }
    LOG(INFO) << "Clawser randomized hardware: " << hw.gl_renderer;
  }

  loaded_ = true;
  LOG(INFO) << "Clawser config loaded for profile: " << config_.profile_id;
  return true;
}

bool ClawserConfigManager::ParseNavigator(const base::Value::Dict& dict) {
  config_.navigator.user_agent = GetString(dict, "user_agent");
  config_.navigator.platform = GetString(dict, "platform", "Win32");
  config_.navigator.vendor = GetString(dict, "vendor", "Google Inc.");
  config_.navigator.app_version = GetString(dict, "app_version");
  config_.navigator.language = GetString(dict, "language", "en-US");
  config_.navigator.languages = GetStringList(dict, "languages");
  config_.navigator.hardware_concurrency =
      GetInt(dict, "hardware_concurrency", 4);
  config_.navigator.device_memory = GetInt(dict, "device_memory", 8);
  config_.navigator.max_touch_points = GetInt(dict, "max_touch_points", 0);

  const base::Value* dnt_value = dict.Find("do_not_track");
  if (dnt_value && !dnt_value->is_none()) {
    if (dnt_value->is_bool()) {
      config_.navigator.do_not_track = dnt_value->GetBool();
    } else if (dnt_value->is_string()) {
      config_.navigator.do_not_track = (dnt_value->GetString() == "1");
    }
  } else {
    config_.navigator.do_not_track = std::nullopt;
  }

  const base::Value::Dict* ua_data = dict.FindDict("user_agent_data");
  if (ua_data) {
    config_.navigator.user_agent_data.mobile =
        GetBool(*ua_data, "mobile", false);
    config_.navigator.user_agent_data.platform =
        GetString(*ua_data, "platform", "Windows");
    config_.navigator.user_agent_data.platform_version =
        GetString(*ua_data, "platform_version");
    config_.navigator.user_agent_data.architecture =
        GetString(*ua_data, "architecture", "x86");
    config_.navigator.user_agent_data.model =
        GetString(*ua_data, "model");
    config_.navigator.user_agent_data.bitness =
        GetString(*ua_data, "bitness", "64");

    const base::Value::List* brands_list = ua_data->FindList("brands");
    if (brands_list) {
      config_.navigator.user_agent_data.brands.clear();
      for (const auto& brand_val : *brands_list) {
        if (!brand_val.is_dict())
          continue;
        const base::Value::Dict& brand_dict = brand_val.GetDict();
        BrandVersion bv;
        bv.brand = GetString(brand_dict, "brand");
        bv.version = GetString(brand_dict, "version");
        config_.navigator.user_agent_data.brands.push_back(std::move(bv));
      }
    }
  }

  return true;
}

bool ClawserConfigManager::ParseScreen(const base::Value::Dict& dict) {
  config_.screen.width = GetInt(dict, "width", 1920);
  config_.screen.height = GetInt(dict, "height", 1080);
  config_.screen.avail_width = GetInt(dict, "avail_width", 1920);
  config_.screen.avail_height = GetInt(dict, "avail_height", 1040);
  config_.screen.color_depth = GetInt(dict, "color_depth", 24);
  config_.screen.pixel_depth = GetInt(dict, "pixel_depth", 24);
  config_.screen.device_pixel_ratio =
      GetDouble(dict, "device_pixel_ratio", 1.0);
  return true;
}

bool ClawserConfigManager::ParseGpu(const base::Value::Dict& dict) {
  config_.gpu.vendor = GetString(dict, "vendor");
  config_.gpu.renderer = GetString(dict, "renderer");

  const base::Value::Dict* params = dict.FindDict("webgl_params");
  if (params) {
    config_.gpu.webgl_params.max_texture_size =
        GetInt(*params, "max_texture_size", 16384);
    config_.gpu.webgl_params.max_renderbuffer_size =
        GetInt(*params, "max_renderbuffer_size", 16384);
    config_.gpu.webgl_params.max_viewport_dims =
        GetIntList(*params, "max_viewport_dims");
    if (config_.gpu.webgl_params.max_viewport_dims.empty()) {
      config_.gpu.webgl_params.max_viewport_dims = {32767, 32767};
    }
    config_.gpu.webgl_params.max_vertex_attribs =
        GetInt(*params, "max_vertex_attribs", 16);
    config_.gpu.webgl_params.max_varying_vectors =
        GetInt(*params, "max_varying_vectors", 30);
    config_.gpu.webgl_params.aliased_line_width_range =
        GetIntList(*params, "aliased_line_width_range");
    if (config_.gpu.webgl_params.aliased_line_width_range.empty()) {
      config_.gpu.webgl_params.aliased_line_width_range = {1, 1};
    }
    config_.gpu.webgl_params.aliased_point_size_range =
        GetIntList(*params, "aliased_point_size_range");
    if (config_.gpu.webgl_params.aliased_point_size_range.empty()) {
      config_.gpu.webgl_params.aliased_point_size_range = {1, 1024};
    }
    config_.gpu.webgl_params.max_fragment_uniform_vectors =
        GetInt(*params, "max_fragment_uniform_vectors", 1024);
    config_.gpu.webgl_params.max_vertex_uniform_vectors =
        GetInt(*params, "max_vertex_uniform_vectors", 4096);
    config_.gpu.webgl_params.extensions =
        GetStringList(*params, "extensions");
  }

  return true;
}

bool ClawserConfigManager::ParseNoiseSeeds(const base::Value::Dict& dict) {
  config_.noise_seeds.canvas = GetUint64(dict, "canvas", 0);
  config_.noise_seeds.webgl = GetUint64(dict, "webgl", 0);
  config_.noise_seeds.audio = GetUint64(dict, "audio", 0);
  config_.noise_seeds.client_rects = GetUint64(dict, "client_rects", 0);

  if (config_.noise_seeds.canvas == 0)
    config_.noise_seeds.canvas = base::RandUint64();
  if (config_.noise_seeds.webgl == 0)
    config_.noise_seeds.webgl = base::RandUint64();
  if (config_.noise_seeds.audio == 0)
    config_.noise_seeds.audio = base::RandUint64();
  if (config_.noise_seeds.client_rects == 0)
    config_.noise_seeds.client_rects = base::RandUint64();

  return true;
}

bool ClawserConfigManager::ParseMediaDevices(const base::Value::Dict& dict) {
  config_.media_devices.audio_inputs = GetInt(dict, "audio_inputs", 1);
  config_.media_devices.audio_outputs = GetInt(dict, "audio_outputs", 2);
  config_.media_devices.video_inputs = GetInt(dict, "video_inputs", 1);
  return true;
}

bool ClawserConfigManager::ParseWebRtc(const base::Value::Dict& dict) {
  std::string policy_str = GetString(dict, "policy", "disabled");

  if (policy_str == "disabled") {
    config_.webrtc.policy = WebRtcPolicy::kDisabled;
  } else if (policy_str == "proxy_only") {
    config_.webrtc.policy = WebRtcPolicy::kProxyOnly;
  } else if (policy_str == "spoofed") {
    config_.webrtc.policy = WebRtcPolicy::kSpoofed;
  } else {
    LOG(WARNING) << "Unknown webrtc policy: " << policy_str
                 << ", defaulting to disabled";
    config_.webrtc.policy = WebRtcPolicy::kDisabled;
  }

  config_.webrtc.fake_local_ip = GetString(dict, "fake_local_ip");
  return true;
}

}  // namespace clawser
