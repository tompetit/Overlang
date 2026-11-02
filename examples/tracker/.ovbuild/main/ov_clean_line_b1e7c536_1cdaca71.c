#include "ov_types.h"

ov_str* ov_clean_line_b1e7c536_1cdaca71(ov_str* p0);

ov_str* ov_clean_line_b1e7c536_1cdaca71(ov_str* p0) {
  ov_str* t1 = ov_str_new("\015", 1);
  ov_str* t2 = ov_str_new(" ", 1);
  ov_str* t3 = ov_str_replace(p0, t1, t2);
  ov_str* t4 = ov_str_new("\n", 1);
  ov_str* t5 = ov_str_new(" ", 1);
  ov_str* t6 = ov_str_replace(t3, t4, t5);
  ov_str* t7 = ov_str_new("\036", 1);
  ov_str* t8 = ov_str_new(" ", 1);
  ov_str* t9 = ov_str_replace(t6, t7, t8);
  ov_str* t10 = ov_str_new("\037", 1);
  ov_str* t11 = ov_str_new(" ", 1);
  ov_str* t12 = ov_str_replace(t9, t10, t11);
  ov_str* t13 = ov_str_trim(t12);
  return t13;
}
