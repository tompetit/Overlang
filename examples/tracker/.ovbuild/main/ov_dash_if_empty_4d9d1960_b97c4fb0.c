#include "ov_types.h"

ov_str* ov_dash_if_empty_4d9d1960_b97c4fb0(ov_str* p0);

ov_str* ov_dash_if_empty_4d9d1960_b97c4fb0(ov_str* p0) {
  ov_str* t1 = ov_str_new("", 0);
  bool t2 = ov_str_eq(p0, t1);
  ov_str* t3 = 0; (void)t3;
  if (t2) {
    ov_str* t4 = ov_str_new("-", 1);
    t3 = t4;
  } else {
    t3 = p0;
  }
  return t3;
}
