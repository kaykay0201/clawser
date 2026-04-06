#ifndef CLAWSER_FETCH_CLAWSER_FETCH_EXPORT_H_
#define CLAWSER_FETCH_CLAWSER_FETCH_EXPORT_H_

#if defined(COMPONENT_BUILD) || defined(IS_CLAWSER_FETCH_IMPL)

#if defined(WIN32)
#if defined(IS_CLAWSER_FETCH_IMPL)
#define CLAWSER_FETCH_EXPORT __declspec(dllexport)
#else
#define CLAWSER_FETCH_EXPORT __declspec(dllimport)
#endif
#else
#if defined(IS_CLAWSER_FETCH_IMPL)
#define CLAWSER_FETCH_EXPORT __attribute__((visibility("default")))
#else
#define CLAWSER_FETCH_EXPORT
#endif
#endif

#else
#define CLAWSER_FETCH_EXPORT
#endif

#endif  // CLAWSER_FETCH_CLAWSER_FETCH_EXPORT_H_
