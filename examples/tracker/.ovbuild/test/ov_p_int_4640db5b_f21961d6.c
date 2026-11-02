#include "ov_types.h"

ov_str* ov_p_int_4640db5b_f21961d6(int64_t p0);

ov_str* ov_p_int_4640db5b_f21961d6(int64_t p0) {
  ov_str* t1 = ov_str_new("i", 1);
  ov_str* t2 = ov_show_int(p0);
  ov_str* t3 = ov_str_concat(t1, t2);
  return t3;
}
