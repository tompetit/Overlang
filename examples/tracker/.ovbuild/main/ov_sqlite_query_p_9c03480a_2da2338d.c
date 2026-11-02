#include "ov_types.h"
#include "ov_sqlite.h"

ovt_Option_6133f15b* ov_sqlite_query_p_9c03480a_2da2338d(int64_t p0, ov_str* p1, ov_str* p2);

ovt_Option_6133f15b* ov_sqlite_query_p_9c03480a_2da2338d(int64_t p0, ov_str* p1, ov_str* p2) {
  const char* r = ov_sqlite_query_p(p0, p1->data, p2->data);
  return ((r != NULL) ? ovt_Option_6133f15b_mk0(ov_str_lit(r)) : ovt_Option_6133f15b_mk1());
}
