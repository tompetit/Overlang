#include "ov_types.h"
#include "ov_sqlite.h"

ov_str* ov_sqlite_error_b3c0e2e3_605f1880(int64_t p0);

ov_str* ov_sqlite_error_b3c0e2e3_605f1880(int64_t p0) {
  const char* r = ov_sqlite_error(p0);
  if (!r) ov_trap("foreign_null", "sqlite_error", "b3c0e2e367a6c411a1d02f48f83e543332c2aed601f21a7651e661eaf939bbc3", "\"`ov_sqlite_error` returned NULL\"");
  return ov_str_lit(r);
}
