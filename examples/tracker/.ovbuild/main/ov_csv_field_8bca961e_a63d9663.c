#include "ov_types.h"

ov_str* ov_csv_field_8bca961e_a63d9663(ov_str* p0);

ov_str* ov_csv_field_8bca961e_a63d9663(ov_str* p0) {
  ov_str* t1 = ov_str_new(",", 1);
  bool t2 = ov_str_contains(p0, t1);
  bool t3 = t2;
  if (!t3) {
    ov_str* t4 = ov_str_new("\"", 1);
    bool t5 = ov_str_contains(p0, t4);
    t3 = t5;
  }
  bool t6 = t3;
  if (!t6) {
    ov_str* t7 = ov_str_new("\n", 1);
    bool t8 = ov_str_contains(p0, t7);
    t6 = t8;
  }
  bool t9 = t6;
  if (!t9) {
    ov_str* t10 = ov_str_new("\015", 1);
    bool t11 = ov_str_contains(p0, t10);
    t9 = t11;
  }
  ov_str* t12 = 0; (void)t12;
  if (t9) {
    ov_str* t13 = ov_str_new("\"", 1);
    ov_str* t14 = ov_str_new("\"", 1);
    ov_str* t15 = ov_str_new("\"\"", 2);
    ov_str* t16 = ov_str_replace(p0, t14, t15);
    ov_str* t17 = ov_str_concat(t13, t16);
    ov_str* t18 = ov_str_new("\"", 1);
    ov_str* t19 = ov_str_concat(t17, t18);
    t12 = t19;
  } else {
    t12 = p0;
  }
  return t12;
}
