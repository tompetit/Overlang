#include "ov_types.h"

ov_str* ov_r_cut_1531a99c_866cf162(ov_list* p0, int64_t p1, int64_t p2);

ov_str* ov_r_cut_1531a99c_866cf162(ov_list* p0, int64_t p1, int64_t p2) {
  ov_list* t1 = ov_list_slice(p0, p1, p2);
  ov_str* t2 = ov_str_new("", 0);
  ov_str* t3 = ov_str_join(t1, t2);
  return t3;
}
