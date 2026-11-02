#include "ov_types.h"

int64_t ov_unwrap_or_e7341bb9_bde9d67c(ovt_Option_d5f2c30a*, int64_t);
ovt_tup2_7b8729bb* ov_parse_args_03e41b86_829441a9(ov_list* p0);

ovt_tup2_7b8729bb* ov_parse_args_03e41b86_829441a9(ov_list* p0) {
  ovt_tup2_7b8729bb* t1 = 0; (void)t1;
  bool t2 = false; (void)t2;
  if (!t2) {
    if ((p0->len == 0)) {
      t2 = true;
      ov_str* t3 = ov_str_new("tracker.db", 10);
      ovt_tup2_7b8729bb* t4 = ovt_tup2_7b8729bb_mk(t3, INT64_C(8080));
      t1 = t4;
    }
  }
  if (!t2) {
    if ((p0->len == 1)) {
      ov_str* v6307 = ((ov_str*)(p0->items[0]).p);
      t2 = true;
      ovt_tup2_7b8729bb* t5 = ovt_tup2_7b8729bb_mk(v6307, INT64_C(8080));
      t1 = t5;
    }
  }
  if (!t2) {
    if ((p0->len >= 2)) {
      ov_str* v6312 = ((ov_str*)(p0->items[0]).p);
      ov_str* v6313 = ((ov_str*)(p0->items[1]).p);
      t2 = true;
      int64_t t6 = 0;
      ovt_Option_d5f2c30a* t7 = ((ov_parse_int(v6313, &t6)) ? ovt_Option_d5f2c30a_mk0(t6) : ovt_Option_d5f2c30a_mk1());
      int64_t t8 = ov_unwrap_or_e7341bb9_bde9d67c(t7, INT64_C(8080));
      ovt_tup2_7b8729bb* t9 = ovt_tup2_7b8729bb_mk(v6312, t8);
      t1 = t9;
    }
  }
  return t1;
}
