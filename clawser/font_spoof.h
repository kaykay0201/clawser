#ifndef CLAWSER_FONT_SPOOF_H_
#define CLAWSER_FONT_SPOOF_H_

#include <string>
#include <vector>

namespace clawser {

bool ShouldControlFonts();

std::vector<std::string> GetAllowedFonts();

bool IsFontAllowed(const std::string& font_name);

}  // namespace clawser

#endif  // CLAWSER_FONT_SPOOF_H_
