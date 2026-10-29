#include "ov_types.h"
#include "ov_http.h"

ov_str* ov_http_method_c930d1b8_4312fe9c(int64_t p0);

ov_str* ov_http_method_c930d1b8_4312fe9c(int64_t p0) {
  const char* r = ov_http_method(p0);
  if (!r) ov_trap("foreign_null", "http_method", "", "\"`ov_http_method` returned NULL\"");
  return ov_str_lit(r);
}
