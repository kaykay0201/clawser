#ifndef CLAWSER_WEBGL_SPOOF_H_
#define CLAWSER_WEBGL_SPOOF_H_

#include <cstddef>
#include <cstdint>
#include <optional>
#include <string>
#include <vector>

#include "third_party/khronos/GLES2/gl2.h"

#include "clawser/clawser_export.h"

namespace clawser {

struct WebGlSpoofedInt {
  GLint value;
};

struct WebGlSpoofedIntPair {
  GLint values[2];
};

CLAWSER_EXPORT std::optional<WebGlSpoofedInt> SpoofWebGlIntParameter(GLenum pname);

CLAWSER_EXPORT std::optional<WebGlSpoofedIntPair> SpoofWebGlIntPairParameter(GLenum pname);

CLAWSER_EXPORT void ApplyWebGlReadPixelsNoise(uint8_t* data,
                                size_t length,
                                uint64_t seed);

CLAWSER_EXPORT std::vector<std::string> GetSpoofedExtensions();

CLAWSER_EXPORT std::string GetSpoofedGlVendorString();

CLAWSER_EXPORT std::string GetSpoofedGlRendererString();

}  // namespace clawser

#endif  // CLAWSER_WEBGL_SPOOF_H_
