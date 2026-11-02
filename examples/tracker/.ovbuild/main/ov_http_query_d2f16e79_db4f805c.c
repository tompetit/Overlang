#include "ov_types.h"
#include "ov_http.h"

ov_str* ov_http_query_d2f16e79_db4f805c(int64_t p0);

ov_str* ov_http_query_d2f16e79_db4f805c(int64_t p0) {
  const char* r = ov_http_query(p0);
  if (!r) ov_trap("foreign_null", "http_query", "d2f16e79113263c959602093f8c4f9f87b0b49aba8ac9b46e17e924926fd7711", "\"`ov_http_query` returned NULL\"");
  return ov_str_lit(r);
}
