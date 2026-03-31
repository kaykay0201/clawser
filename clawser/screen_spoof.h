#ifndef CLAWSER_SCREEN_SPOOF_H_
#define CLAWSER_SCREEN_SPOOF_H_

namespace clawser {

bool IsScreenSpoofEnabled();

int GetSpoofedScreenWidth();
int GetSpoofedScreenHeight();
int GetSpoofedAvailWidth();
int GetSpoofedAvailHeight();
int GetSpoofedColorDepth();
int GetSpoofedPixelDepth();
double GetSpoofedDevicePixelRatio();

}  // namespace clawser

#endif  // CLAWSER_SCREEN_SPOOF_H_
