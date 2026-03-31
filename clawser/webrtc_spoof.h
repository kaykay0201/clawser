#ifndef CLAWSER_WEBRTC_SPOOF_H_
#define CLAWSER_WEBRTC_SPOOF_H_

#include <string>

#include "clawser/clawser_config.h"

namespace clawser {

WebRtcPolicy GetWebRtcPolicy();

std::string GetFakeLocalIp();

bool ShouldSpoofWebRtc();

bool ShouldBlockAllWebRtc();

bool ShouldForceRelayOnly();

std::string RewriteCandidateIp(const std::string& candidate);

}  // namespace clawser

#endif  // CLAWSER_WEBRTC_SPOOF_H_
