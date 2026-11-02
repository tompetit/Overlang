#include "ov_types.h"

bool ov_is_some_5d16aee5_0ba6f2de(ovt_Option_c278faef* p0);

bool ov_is_some_5d16aee5_0ba6f2de(ovt_Option_c278faef* p0) {
  bool t1 = 0; (void)t1;
  bool t2 = false; (void)t2;
  if (!t2) {
    if ((p0->tag == 0)) {
      t2 = true;
      t1 = true;
    }
  }
  if (!t2) {
    if ((p0->tag == 1)) {
      t2 = true;
      t1 = false;
    }
  }
  return t1;
}
