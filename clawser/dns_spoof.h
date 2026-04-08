#ifndef CLAWSER_DNS_SPOOF_H_
#define CLAWSER_DNS_SPOOF_H_

#include <string>

#include "clawser/clawser_export.h"

namespace clawser {

CLAWSER_EXPORT bool ShouldForceDnsOverProxy();

CLAWSER_EXPORT bool IsProxyConfigured();

CLAWSER_EXPORT bool ShouldPreventDirectDns();

}  // namespace clawser

#endif  // CLAWSER_DNS_SPOOF_H_
