#include "ov_types.h"

ovt_Option_6133f15b* ov_last_3b93ed9c_16222bc9(ov_list*);
ov_list* ov_lines_70f746c7_d92a2e51(ov_str* p0);

ov_list* ov_lines_70f746c7_d92a2e51(ov_str* p0) {
  ov_str* t1 = ov_str_new("\n", 1);
  ov_list* t2 = ov_str_split(p0, t1);
  ov_list* v429 = t2;
  ovt_Option_6133f15b* t3 = ov_last_3b93ed9c_16222bc9(v429);
  ov_str* t4 = ov_str_new("", 0);
  ovt_Option_6133f15b* t5 = ovt_Option_6133f15b_mk0(t4);
  bool t6 = oveq_Option_6133f15b(t3, t5);
  ov_list* t7 = 0; (void)t7;
  if (t6) {
    int64_t t8 = v429->len;
    int64_t t9 = ov_sub(t8, INT64_C(1), "lines", "70f746c720a9a518e0dc707ba6bf781c148f8cd371bfcc921fa02425f3a02458");
    ov_list* t10 = ov_list_take(v429, t9);
    t7 = t10;
  } else {
    t7 = v429;
  }
  return t7;
}
