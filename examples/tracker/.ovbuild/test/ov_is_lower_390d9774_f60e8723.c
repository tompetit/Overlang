#include "ov_types.h"

bool ov_is_lower_390d9774_f60e8723(ov_str* p0);

bool ov_is_lower_390d9774_f60e8723(ov_str* p0) {
  bool t1 = false;
  int64_t t2 = ov_str_code_point(p0, &t1);
  ovt_Option_d5f2c30a* t3 = ((t1) ? ovt_Option_d5f2c30a_mk0(t2) : ovt_Option_d5f2c30a_mk1());
  bool t4 = 0; (void)t4;
  bool t5 = false; (void)t5;
  if (!t5) {
    if ((t3->tag == 0)) {
      int64_t v484 = t3->u.c0.f0;
      t5 = true;
      bool t6 = (v484 >= INT64_C(97));
      bool t7 = t6;
      if (t7) {
        bool t8 = (v484 <= INT64_C(122));
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
