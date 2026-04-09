#include "clawser/canvas_noise.h"

// TODO(clawser): migrate to base::span.
// NOLINTBEGIN(unsafe-buffer-usage)
#pragma allow_unsafe_buffers

namespace clawser {

void ApplyCanvasNoise(uint8_t* data, size_t length, uint64_t seed) {
  if (seed == 0)
    return;
  if (!data || length < 4)
    return;

  Xorshift128Plus rng(seed);
  for (size_t i = 0; i + 3 < length; i += 4) {
    uint64_t r = rng.Next();
    // Skip transparent pixels.
    if (data[i + 3] == 0)
      continue;
    // Sparse: only ~4% of pixels get noise (avoids statistical detection).
    if ((r & 0x1F) != 0)
      continue;
    // Vary which channel gets noise (not all 3 every time).
    uint8_t channel = (r >> 5) % 3;
    data[i + channel] ^= 1;
  }
}

}  // namespace clawser
