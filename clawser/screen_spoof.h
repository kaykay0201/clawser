#ifndef CLAWSER_SCREEN_SPOOF_H_
#define CLAWSER_SCREEN_SPOOF_H_

#include "clawser/clawser_export.h"

namespace clawser {

CLAWSER_EXPORT bool IsScreenSpoofEnabled();

CLAWSER_EXPORT int GetSpoofedScreenWidth();
CLAWSER_EXPORT int GetSpoofedScreenHeight();
CLAWSER_EXPORT int GetSpoofedAvailWidth();
CLAWSER_EXPORT int GetSpoofedAvailHeight();
CLAWSER_EXPORT int GetSpoofedColorDepth();
CLAWSER_EXPORT int GetSpoofedPixelDepth();
CLAWSER_EXPORT double GetSpoofedDevicePixelRatio();

}  // namespace clawser

#endif  // CLAWSER_SCREEN_SPOOF_H_
