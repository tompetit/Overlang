#include "ov_types.h"

ov_str* ov_activity_line_a767f335_9e5c839b(ovt_Activity_dd4dbed0* p0);

ov_str* ov_activity_line_a767f335_9e5c839b(ovt_Activity_dd4dbed0* p0) {
  ov_str* t1 = p0->f_detail;
  ov_str* t2 = ov_str_new("", 0);
  bool t3 = ov_str_eq(t1, t2);
  ov_str* t4 = 0; (void)t4;
  if (t3) {
    ov_str* t5 = p0->f_kind;
    t4 = t5;
  } else {
    ov_str* t6 = p0->f_kind;
    ov_str* t7 = ov_str_new(" ", 1);
    ov_str* t8 = ov_str_concat(t6, t7);
    ov_str* t9 = p0->f_detail;
    ov_str* t10 = ov_str_concat(t8, t9);
    t4 = t10;
  }
  return t4;
}
