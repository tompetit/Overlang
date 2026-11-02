#include "ov_types.h"

ov_str* ov_unwrap_or_e7341bb9_412aa135(ovt_Option_6133f15b* p0, ov_str* p1);

ov_str* ov_unwrap_or_e7341bb9_412aa135(ovt_Option_6133f15b* p0, ov_str* p1) {
  ov_str* t1 = 0; (void)t1;
  bool t2 = false; (void)t2;
  if (!t2) {
    if ((p0->tag == 0)) {
      ov_str* v603 = p0->u.c0.f0;
      t2 = true;
      t1 = v603;
    }
  }
  if (!t2) {
    if ((p0->tag == 1)) {
      t2 = true;
      t1 = p1;
    }
  }
  return t1;
}
