#ifndef CLAWSER_FONT_SPOOF_H_
#define CLAWSER_FONT_SPOOF_H_

#include <string>
#include <vector>

#include "clawser/clawser_export.h"

namespace clawser {

CLAWSER_EXPORT bool ShouldControlFonts();

CLAWSER_EXPORT std::vector<std::string> GetAllowedFonts();

CLAWSER_EXPORT bool IsFontAllowed(const std::string& font_name);

}  // namespace clawser

#endif  // CLAWSER_FONT_SPOOF_H_
