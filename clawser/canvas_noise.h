#ifndef CLAWSER_CANVAS_NOISE_H_
#define CLAWSER_CANVAS_NOISE_H_

#include <cstddef>
#include <cstdint>

namespace clawser {

struct Xorshift128Plus {
  uint64_t s[2];

  explicit Xorshift128Plus(uint64_t seed) {
    s[0] = seed;
    s[1] = seed ^ 0x6a09e667f3bcc908ULL;
  }

  uint64_t Next() {
    uint64_t s1 = s[0];
    uint64_t s0 = s[1];
    s[0] = s0;
    s1 ^= s1 << 23;
    s1 ^= s1 >> 17;
    s1 ^= s0;
    s1 ^= s0 >> 26;
    s[1] = s1;
    return s[0] + s[1];
  }
};

void ApplyCanvasNoise(uint8_t* data, size_t length, uint64_t seed);

}  // namespace clawser

#endif  // CLAWSER_CANVAS_NOISE_H_
