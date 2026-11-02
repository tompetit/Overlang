#include "ov_types.h"
#include "ov_http.h"

ov_str* ov_http_method_e4d563f3_4312fe9c(int64_t p0);

ov_str* ov_http_method_e4d563f3_4312fe9c(int64_t p0) {
  const char* r = ov_http_method(p0);
  if (!r) ov_trap("foreign_null", "http_method", "e4d563f3a7c362b939fc74ca513a81462784e5948cd29c614c1e4b0dbaa82fd5", "\"`ov_http_method` returned NULL\"");
  return ov_str_lit(r);
}
