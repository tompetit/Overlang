#include "ov_types.h"

ov_str* ov_like_pattern_ec16774e_b5007031(ov_str* p0);

ov_str* ov_like_pattern_ec16774e_b5007031(ov_str* p0) {
  ov_str* t1 = ov_str_new("%", 1);
  ov_str* t2 = ov_str_new("\\", 1);
  ov_str* t3 = ov_str_new("\\\\", 2);
  ov_str* t4 = ov_str_replace(p0, t2, t3);
  ov_str* t5 = ov_str_new("%", 1);
  ov_str* t6 = ov_str_new("\\%", 2);
  ov_str* t7 = ov_str_replace(t4, t5, t6);
  ov_str* t8 = ov_str_new("_", 1);
  ov_str* t9 = ov_str_new("\\_", 2);
  ov_str* t10 = ov_str_replace(t7, t8, t9);
  ov_str* t11 = ov_str_concat(t1, t10);
  ov_str* t12 = ov_str_new("%", 1);
  ov_str* t13 = ov_str_concat(t11, t12);
  return t13;
}
