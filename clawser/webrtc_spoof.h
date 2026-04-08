#ifndef CLAWSER_WEBRTC_SPOOF_H_
#define CLAWSER_WEBRTC_SPOOF_H_

#include <string>

#include "clawser/clawser_config.h"
#include "clawser/clawser_export.h"

namespace clawser {

CLAWSER_EXPORT WebRtcPolicy GetWebRtcPolicy();

CLAWSER_EXPORT std::string GetFakeLocalIp();

CLAWSER_EXPORT bool ShouldSpoofWebRtc();

CLAWSER_EXPORT bool ShouldBlockAllWebRtc();

CLAWSER_EXPORT bool ShouldForceRelayOnly();

CLAWSER_EXPORT std::string RewriteCandidateIp(const std::string& candidate);

}  // namespace clawser

#endif  // CLAWSER_WEBRTC_SPOOF_H_
