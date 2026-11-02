#include "ov_types.h"
#include "ov_http.h"

ov_str* ov_http_body_81a71fd4_07410916(int64_t p0);

ov_str* ov_http_body_81a71fd4_07410916(int64_t p0) {
  const char* r = ov_http_body(p0);
  if (!r) ov_trap("foreign_null", "http_body", "81a71fd4f0d3847504490ec54203e4c57b49b22e889e7465d54457108c470cc7", "\"`ov_http_body` returned NULL\"");
  return ov_str_lit(r);
}
