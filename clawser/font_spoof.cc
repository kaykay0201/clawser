#include "clawser/font_spoof.h"

#include <algorithm>
#include <cctype>

#include "clawser/clawser_config.h"

namespace clawser {

namespace {

std::string ToLower(const std::string& input) {
  std::string result = input;
  std::transform(result.begin(), result.end(), result.begin(),
                 [](unsigned char c) { return std::tolower(c); });
  return result;
}

}  // namespace

bool ShouldControlFonts() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return false;
  return !ClawserConfigManager::GetInstance().GetConfig().fonts.empty();
}

std::vector<std::string> GetAllowedFonts() {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return {};
  return ClawserConfigManager::GetInstance().GetConfig().fonts;
}

bool IsFontAllowed(const std::string& font_name) {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return true;
  const auto& fonts = ClawserConfigManager::GetInstance().GetConfig().fonts;
  if (fonts.empty())
    return true;
  std::string lower_name = ToLower(font_name);
  for (const auto& f : fonts) {
    if (ToLower(f) == lower_name)
      return true;
  }
  return false;
}

}  // namespace clawser
