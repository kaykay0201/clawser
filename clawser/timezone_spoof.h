#ifndef CLAWSER_TIMEZONE_SPOOF_H_
#define CLAWSER_TIMEZONE_SPOOF_H_

#include <string>

namespace clawser {

bool ShouldSpoofTimezone();

std::string GetSpoofedTimezone();

std::string GetSpoofedLocale();

}  // namespace clawser

#endif  // CLAWSER_TIMEZONE_SPOOF_H_
