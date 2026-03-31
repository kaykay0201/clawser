#include "clawser/canvas_noise.h"

namespace clawser {

void ApplyCanvasNoise(uint8_t* data, size_t length, uint64_t seed) {
  if (seed == 0)
    return;
  if (!data || length < 4)
    return;

  Xorshift128Plus rng(seed);
  for (size_t i = 0; i + 3 < length; i += 4) {
    uint64_t r = rng.Next();
    if (data[i + 3] == 0)
      continue;
    data[i] ^= (r & 1);
    data[i + 1] ^= ((r >> 1) & 1);
    data[i + 2] ^= ((r >> 2) & 1);
  }
}

}  // namespace clawser
