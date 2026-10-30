#include "ov_types.h"
#include "ov_http.h"

ov_str* ov_http_path_53b97211_14991708(int64_t p0);

ov_str* ov_http_path_53b97211_14991708(int64_t p0) {
  const char* r = ov_http_path(p0);
  if (!r) ov_trap("foreign_null", "http_path", "", "\"`ov_http_path` returned NULL\"");
  return ov_str_lit(r);
}
