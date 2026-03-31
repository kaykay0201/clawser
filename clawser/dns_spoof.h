#ifndef CLAWSER_DNS_SPOOF_H_
#define CLAWSER_DNS_SPOOF_H_

#include <string>

namespace clawser {

bool ShouldForceDnsOverProxy();

bool IsProxyConfigured();

bool ShouldPreventDirectDns();

}  // namespace clawser

#endif  // CLAWSER_DNS_SPOOF_H_
