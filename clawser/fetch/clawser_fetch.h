#ifndef CLAWSER_FETCH_CLAWSER_FETCH_H_
#define CLAWSER_FETCH_CLAWSER_FETCH_H_

#include <stddef.h>
#include <stdint.h>

#include "clawser/fetch/clawser_fetch_export.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct ClawserSession ClawserSession;
typedef struct ClawserRequest ClawserRequest;
typedef struct ClawserResponse ClawserResponse;

#define CLAWSER_OK 0
#define CLAWSER_ERR_INIT_FAILED -1
#define CLAWSER_ERR_INVALID_ARG -2
#define CLAWSER_ERR_NETWORK -3
#define CLAWSER_ERR_TIMEOUT -4

// Seed struct: deterministic identity from 5 uint64 values.
// hw_seed: selects hardware profile + chrome version + H2 profile.
// canvas_seed, webgl_seed, audio_seed, client_rects_seed: noise seeds.
typedef struct ClawserSeed {
  uint64_t hw_seed;
  uint64_t canvas_seed;
  uint64_t webgl_seed;
  uint64_t audio_seed;
  uint64_t client_rects_seed;
} ClawserSeed;

// Session lifecycle.

// Create session with a randomly generated identity.
// Writes the generated seed to *out_seed (caller can save and reuse).
CLAWSER_FETCH_EXPORT ClawserSession* clawser_session_create_random(
    ClawserSeed* out_seed);

// Create session from a previously generated seed (deterministic replay).
CLAWSER_FETCH_EXPORT ClawserSession* clawser_session_from_seed(
    const ClawserSeed* seed);

// Create session from a config JSON file path (advanced, full control).
CLAWSER_FETCH_EXPORT ClawserSession* clawser_session_create(
    const char* config_json_path);

// Get all cookies as a JSON array string.
// Returns: [{"name":"x","value":"y","domain":".example.com","path":"/","secure":true,"httponly":false}]
// The returned pointer is valid until the next call to this function from the same thread.
CLAWSER_FETCH_EXPORT const char* clawser_session_get_cookies(
    ClawserSession* session);

CLAWSER_FETCH_EXPORT void clawser_session_destroy(ClawserSession* session);

// Thread-local error string, valid until next call from same thread.
CLAWSER_FETCH_EXPORT const char* clawser_last_error(void);

// Request building.
CLAWSER_FETCH_EXPORT ClawserRequest* clawser_request_new(
    ClawserSession* session,
    const char* method,
    const char* url);
CLAWSER_FETCH_EXPORT void clawser_request_set_header(
    ClawserRequest* req,
    const char* name,
    const char* value);
CLAWSER_FETCH_EXPORT void clawser_request_set_body(
    ClawserRequest* req,
    const uint8_t* data,
    size_t len);
CLAWSER_FETCH_EXPORT void clawser_request_set_max_redirects(
    ClawserRequest* req,
    int max_redirects);
CLAWSER_FETCH_EXPORT void clawser_request_set_timeout_ms(
    ClawserRequest* req,
    uint32_t timeout_ms);

// Blocking send. Consumes req (do not use after). Returns NULL on error.
CLAWSER_FETCH_EXPORT ClawserResponse* clawser_request_send(
    ClawserRequest* req);
// Free a request that was never sent.
CLAWSER_FETCH_EXPORT void clawser_request_destroy(ClawserRequest* req);

// Response reading.
CLAWSER_FETCH_EXPORT int clawser_response_status_code(
    const ClawserResponse* resp);
CLAWSER_FETCH_EXPORT const char* clawser_response_header(
    const ClawserResponse* resp,
    const char* name);
CLAWSER_FETCH_EXPORT size_t clawser_response_header_count(
    const ClawserResponse* resp);
CLAWSER_FETCH_EXPORT const char* clawser_response_header_name_at(
    const ClawserResponse* resp,
    size_t index);
CLAWSER_FETCH_EXPORT const char* clawser_response_header_value_at(
    const ClawserResponse* resp,
    size_t index);
CLAWSER_FETCH_EXPORT const uint8_t* clawser_response_body(
    const ClawserResponse* resp);
CLAWSER_FETCH_EXPORT size_t clawser_response_body_len(
    const ClawserResponse* resp);
CLAWSER_FETCH_EXPORT const char* clawser_response_url(
    const ClawserResponse* resp);
CLAWSER_FETCH_EXPORT void clawser_response_destroy(
    ClawserResponse* resp);

CLAWSER_FETCH_EXPORT const char* clawser_version(void);

#ifdef __cplusplus
}
#endif

#endif  // CLAWSER_FETCH_CLAWSER_FETCH_H_
