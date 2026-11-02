#include "ov_types.h"

bool ov_is_digit_d7695ac0_63fa906b(ov_str* p0);

bool ov_is_digit_d7695ac0_63fa906b(ov_str* p0) {
  bool t1 = false;
  int64_t t2 = ov_str_code_point(p0, &t1);
  ovt_Option_d5f2c30a* t3 = ((t1) ? ovt_Option_d5f2c30a_mk0(t2) : ovt_Option_d5f2c30a_mk1());
  bool t4 = 0; (void)t4;
  bool t5 = false; (void)t5;
  if (!t5) {
    if ((t3->tag == 0)) {
      int64_t v454 = t3->u.c0.f0;
      t5 = true;
      bool t6 = (v454 >= INT64_C(48));
      bool t7 = t6;
      if (t7) {
        bool t8 = (v454 <= INT64_C(57));
        t7 = t8;
      }
      t4 = t7;
    }
  }
  if (!t5) {
    if ((t3->tag == 1)) {
      t5 = true;
      t4 = false;
    }
  }
  return t4;
}
