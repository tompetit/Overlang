#include "ov_types.h"
#include "ov_sqlite.h"

ovt_Option_6133f15b* ov_sqlite_query_0f96a2fa_d75bed94(int64_t p0, ov_str* p1);

ovt_Option_6133f15b* ov_sqlite_query_0f96a2fa_d75bed94(int64_t p0, ov_str* p1) {
  const char* r = ov_sqlite_query(p0, p1->data);
  return ((r != NULL) ? ovt_Option_6133f15b_mk0(ov_str_lit(r)) : ovt_Option_6133f15b_mk1());
}
