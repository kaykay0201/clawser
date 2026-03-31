#include "clawser/client_rects_noise.h"

#include "clawser/canvas_noise.h"
#include "clawser/clawser_config.h"

namespace clawser {

namespace {

double PrngToNoise(uint64_t value) {
  double normalized =
      static_cast<double>(value % 10000) / 10000.0;
  return (normalized - 0.5) * 0.1;
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

  Xorshift128Plus rng(MixHash(seed, element_hash));
  x += PrngToNoise(rng.Next());
  y += PrngToNoise(rng.Next());
  width += PrngToNoise(rng.Next());
  height += PrngToNoise(rng.Next());
}

}  // namespace clawser
