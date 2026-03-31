#ifndef CLAWSER_CLIENT_RECTS_NOISE_H_
#define CLAWSER_CLIENT_RECTS_NOISE_H_

#include <cstdint>
#include <string>

namespace clawser {

uint64_t HashElementIdentity(const std::string& tag_name,
                              const std::string& class_name,
                              const std::string& id);

void ApplyRectNoise(double& x,
                    double& y,
                    double& width,
                    double& height,
                    uint64_t seed,
                    uint64_t element_hash);

}  // namespace clawser

#endif  // CLAWSER_CLIENT_RECTS_NOISE_H_
