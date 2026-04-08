#ifndef CLAWSER_TIMEZONE_SPOOF_H_
#define CLAWSER_TIMEZONE_SPOOF_H_

#include <string>

#include "clawser/clawser_export.h"

namespace clawser {

CLAWSER_EXPORT bool ShouldSpoofTimezone();

CLAWSER_EXPORT std::string GetSpoofedTimezone();

CLAWSER_EXPORT std::string GetSpoofedLocale();

}  // namespace clawser

#endif  // CLAWSER_TIMEZONE_SPOOF_H_
