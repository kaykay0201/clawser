#ifndef CLAWSER_WEBGL_SPOOF_H_
#define CLAWSER_WEBGL_SPOOF_H_

#include <cstddef>
#include <cstdint>
#include <optional>
#include <string>
#include <vector>

#include "third_party/khronos/GLES2/gl2.h"

namespace clawser {

struct WebGlSpoofedInt {
  GLint value;
};

struct WebGlSpoofedIntPair {
  GLint values[2];
};

std::optional<WebGlSpoofedInt> SpoofWebGlIntParameter(GLenum pname);

std::optional<WebGlSpoofedIntPair> SpoofWebGlIntPairParameter(GLenum pname);

void ApplyWebGlReadPixelsNoise(uint8_t* data,
                                size_t length,
                                uint64_t seed);

std::vector<std::string> GetSpoofedExtensions();

std::string GetSpoofedGlVendorString();

std::string GetSpoofedGlRendererString();

}  // namespace clawser

#endif  // CLAWSER_WEBGL_SPOOF_H_
