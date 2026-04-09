#include "clawser/audio_noise.h"

#include <algorithm>

// TODO(clawser): migrate to base::span.
// NOLINTBEGIN(unsafe-buffer-usage)
#pragma allow_unsafe_buffers

#include "clawser/canvas_noise.h"
#include "clawser/clawser_config.h"

namespace clawser {

void ApplyAudioNoise(float* data, size_t length, uint64_t seed) {
  if (!data || length == 0 || seed == 0)
    return;

  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return;

  Xorshift128Plus rng(seed);
  for (size_t i = 0; i < length; ++i) {
    uint64_t r = rng.Next();
    // Sparse: only ~3% of samples get noise (avoids trap signal corruption).
    if ((r & 0x1F) != 0)
      continue;
    float perturbation =
        static_cast<float>(static_cast<int64_t>(r % 1000) - 500) *
        0.0000001f;
    data[i] += perturbation;
  }
}

void ApplyAudioNoiseUint8(unsigned char* data,
                          size_t length,
                          uint64_t seed) {
  if (!data || length == 0 || seed == 0)
    return;

  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return;

  Xorshift128Plus rng(seed);
  for (size_t i = 0; i < length; ++i) {
    uint64_t r = rng.Next();
    // Sparse: only ~3% of samples.
    if ((r & 0x1F) != 0)
      continue;
    int noise = static_cast<int>(r % 3) - 1;
    int val = static_cast<int>(data[i]) + noise;
    data[i] = static_cast<unsigned char>(std::max(0, std::min(255, val)));
  }
}

}  // namespace clawser
