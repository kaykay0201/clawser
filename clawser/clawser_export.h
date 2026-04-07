#ifndef CLAWSER_CLAWSER_EXPORT_H_
#define CLAWSER_CLAWSER_EXPORT_H_

#if defined(COMPONENT_BUILD)
#if defined(WIN32)

#if defined(IS_CLAWSER_IMPL)
#define CLAWSER_EXPORT __declspec(dllexport)
#else
#define CLAWSER_EXPORT __declspec(dllimport)
#endif  // defined(IS_CLAWSER_IMPL)

#else  // defined(WIN32)

#if defined(IS_CLAWSER_IMPL)
#define CLAWSER_EXPORT __attribute__((visibility("default")))
#else
#define CLAWSER_EXPORT
#endif  // defined(IS_CLAWSER_IMPL)

#endif  // defined(WIN32)

#else  // defined(COMPONENT_BUILD)
#define CLAWSER_EXPORT
#endif  // defined(COMPONENT_BUILD)

#endif  // CLAWSER_CLAWSER_EXPORT_H_
