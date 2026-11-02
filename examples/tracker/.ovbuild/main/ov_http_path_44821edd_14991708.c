#include "ov_types.h"
#include "ov_http.h"

ov_str* ov_http_path_44821edd_14991708(int64_t p0);

ov_str* ov_http_path_44821edd_14991708(int64_t p0) {
  const char* r = ov_http_path(p0);
  if (!r) ov_trap("foreign_null", "http_path", "44821edd966af21de168c6d14edf091a4965d5bef6b6108c9d256cf115dafc1f", "\"`ov_http_path` returned NULL\"");
  return ov_str_lit(r);
}
