#include "clawser/client_rects_noise.h"

#include "clawser/canvas_noise.h"
#include "clawser/clawser_config.h"

namespace clawser {

namespace {

double PrngToNoise(uint64_t value) {
  // Sub-pixel noise: ±0.000005 — within natural GPU rendering variance.
  // Large noise (±0.05) is statistically detectable by CreepJS.
  double normalized =
      static_cast<double>(value % 10000) / 10000.0;
  return (normalized - 0.5) * 0.00001;
}

uint64_t MixHash(uint64_t a, uint64_t b) {
  a ^= b;
  a ^= a >> 33;
  a *= 0xff51afd7ed558ccdULL;
  a ^= a >> 33;
  a *= 0xc4ceb9fe1a85ec53ULL;
  a ^= a >> 33;
  return a;
}

}  // namespace

uint64_t HashElementIdentity(const std::string& tag_name,
                              const std::string& class_name,
                              const std::string& id) {
  uint64_t hash = 0xcbf29ce484222325ULL;
  for (char c : tag_name) {
    hash ^= static_cast<uint64_t>(c);
    hash *= 0x100000001b3ULL;
  }
  for (char c : class_name) {
    hash ^= static_cast<uint64_t>(c);
    hash *= 0x100000001b3ULL;
  }
  for (char c : id) {
    hash ^= static_cast<uint64_t>(c);
    hash *= 0x100000001b3ULL;
  }
  return hash;
}

void ApplyRectNoise(double& x,
                    double& y,
                    double& width,
                    double& height,
                    uint64_t seed,
                    uint64_t element_hash) {
  if (seed == 0)
    return;

  // Only noise position (x, y). Keep width/height untouched so that
  // right == x + width and bottom == y + height remain consistent.
  // CreepJS validates these invariants to detect spoofing.
  Xorshift128Plus rng(MixHash(seed, element_hash));
  double dx = PrngToNoise(rng.Next());
  double dy = PrngToNoise(rng.Next());
  x += dx;
  y += dy;
  // width and height stay unchanged — dimensions are consistent.
}

}  // namespace clawser
