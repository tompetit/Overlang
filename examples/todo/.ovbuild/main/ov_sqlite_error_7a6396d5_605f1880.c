#include "ov_types.h"
#include "ov_sqlite.h"

ov_str* ov_sqlite_error_7a6396d5_605f1880(int64_t p0);

ov_str* ov_sqlite_error_7a6396d5_605f1880(int64_t p0) {
  const char* r = ov_sqlite_error(p0);
  if (!r) ov_trap("foreign_null", "sqlite_error", "", "\"`ov_sqlite_error` returned NULL\"");
  return ov_str_lit(r);
}
