#include "ov_types.h"
#include "ov_http.h"

ov_str* ov_http_body_498e6a7b_07410916(int64_t p0);

ov_str* ov_http_body_498e6a7b_07410916(int64_t p0) {
  const char* r = ov_http_body(p0);
  if (!r) ov_trap("foreign_null", "http_body", "", "\"`ov_http_body` returned NULL\"");
  return ov_str_lit(r);
}
