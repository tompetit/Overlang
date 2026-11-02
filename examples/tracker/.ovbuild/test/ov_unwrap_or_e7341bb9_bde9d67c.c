#include "ov_types.h"

int64_t ov_unwrap_or_e7341bb9_bde9d67c(ovt_Option_d5f2c30a* p0, int64_t p1);

int64_t ov_unwrap_or_e7341bb9_bde9d67c(ovt_Option_d5f2c30a* p0, int64_t p1) {
  int64_t t1 = 0; (void)t1;
  bool t2 = false; (void)t2;
  if (!t2) {
    if ((p0->tag == 0)) {
      int64_t v603 = p0->u.c0.f0;
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
