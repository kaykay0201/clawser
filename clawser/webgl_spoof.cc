#include "clawser/webgl_spoof.h"

#include <cstdint>

// TODO(clawser): migrate to base::span.
// NOLINTBEGIN(unsafe-buffer-usage)
#pragma allow_unsafe_buffers

#include "clawser/clawser_config.h"
#include "third_party/khronos/GLES2/gl2.h"

namespace clawser {

namespace {

struct Xorshift128PlusState {
  uint64_t s0;
  uint64_t s1;
};

uint64_t Xorshift128Plus(Xorshift128PlusState& state) {
  uint64_t s1 = state.s0;
  uint64_t s0 = state.s1;
  state.s0 = s0;
  s1 ^= s1 << 23;
  s1 ^= s1 >> 17;
  s1 ^= s0;
  s1 ^= s0 >> 26;
  state.s1 = s1;
  return state.s0 + state.s1;
}

}  // namespace

std::optional<WebGlSpoofedInt> SpoofWebGlIntParameter(GLenum pname) {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return std::nullopt;

  const auto& params =
      ClawserConfigManager::GetInstance().GetConfig().gpu.webgl_params;

  switch (pname) {
    case GL_MAX_TEXTURE_SIZE:
      return WebGlSpoofedInt{static_cast<GLint>(params.max_texture_size)};
    case GL_MAX_RENDERBUFFER_SIZE:
      return WebGlSpoofedInt{
          static_cast<GLint>(params.max_renderbuffer_size)};
    case GL_MAX_VERTEX_ATTRIBS:
      return WebGlSpoofedInt{static_cast<GLint>(params.max_vertex_attribs)};
    case GL_MAX_VARYING_VECTORS:
      return WebGlSpoofedInt{
          static_cast<GLint>(params.max_varying_vectors)};
    case GL_MAX_FRAGMENT_UNIFORM_VECTORS:
      return WebGlSpoofedInt{
          static_cast<GLint>(params.max_fragment_uniform_vectors)};
    case GL_MAX_VERTEX_UNIFORM_VECTORS:
      return WebGlSpoofedInt{
          static_cast<GLint>(params.max_vertex_uniform_vectors)};
    default:
      return std::nullopt;
  }
}

std::optional<WebGlSpoofedIntPair> SpoofWebGlIntPairParameter(GLenum pname) {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return std::nullopt;

  const auto& params =
      ClawserConfigManager::GetInstance().GetConfig().gpu.webgl_params;

  switch (pname) {
    case GL_MAX_VIEWPORT_DIMS:
      if (params.max_viewport_dims.size() >= 2) {
        return WebGlSpoofedIntPair{
            {static_cast<GLint>(params.max_viewport_dims[0]),
             static_cast<GLint>(params.max_viewport_dims[1])}};
      }
      return std::nullopt;
    case GL_ALIASED_LINE_WIDTH_RANGE:
      if (params.aliased_line_width_range.size() >= 2) {
        return WebGlSpoofedIntPair{
            {static_cast<GLint>(params.aliased_line_width_range[0]),
             static_cast<GLint>(params.aliased_line_width_range[1])}};
      }
      return std::nullopt;
    case GL_ALIASED_POINT_SIZE_RANGE:
      if (params.aliased_point_size_range.size() >= 2) {
        return WebGlSpoofedIntPair{
            {static_cast<GLint>(params.aliased_point_size_range[0]),
             static_cast<GLint>(params.aliased_point_size_range[1])}};
      }
      return std::nullopt;
    default:
      return std::nullopt;
  }
}

void ApplyWebGlReadPixelsNoise(uint8_t* data,
                                size_t length,
                                uint64_t seed) {
  if (seed == 0 || length == 0 || !data)
    return;

  Xorshift128PlusState state;
  state.s0 = seed;
  state.s1 = seed ^ 0x6A09E667F3BCC908ULL;

  for (size_t i = 0; i < 8; ++i)
    Xorshift128Plus(state);

  for (size_t i = 0; i < length; ++i) {
    if (i % 8 == 0) {
      uint64_t r = Xorshift128Plus(state);
      if ((r & 0x07) != 0)
        continue;
    }
    uint64_t noise_val = Xorshift128Plus(state);
    int delta = static_cast<int>(noise_val % 3) - 1;
    int pixel = static_cast<int>(data[i]) + delta;
    data[i] = static_cast<uint8_t>(std::max(0, std::min(255, pixel)));
  }
}

std::vector<std::string> GetSpoofedExtensions() {
  return ClawserConfigManager::GetInstance().GetConfig().gpu.webgl_params.extensions;
}

std::string GetSpoofedGlVendorString() {
  return ClawserConfigManager::GetInstance().GetConfig().gpu.vendor;
}

std::string GetSpoofedGlRendererString() {
  return ClawserConfigManager::GetInstance().GetConfig().gpu.renderer;
}

}  // namespace clawser
