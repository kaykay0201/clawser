#ifndef CLAWSER_AUDIO_NOISE_H_
#define CLAWSER_AUDIO_NOISE_H_

#include <cstddef>
#include <cstdint>

namespace clawser {

void ApplyAudioNoise(float* data, size_t length, uint64_t seed);
void ApplyAudioNoiseUint8(unsigned char* data, size_t length, uint64_t seed);

}  // namespace clawser

#endif  // CLAWSER_AUDIO_NOISE_H_
