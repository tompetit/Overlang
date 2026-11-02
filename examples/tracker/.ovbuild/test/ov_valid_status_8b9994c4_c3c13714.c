#include "ov_types.h"

bool ov_valid_status_8b9994c4_c3c13714(ov_str* p0);

bool ov_valid_status_8b9994c4_c3c13714(ov_str* p0) {
  ov_str* t1 = ov_str_new("open", 4);
  bool t2 = ov_str_eq(p0, t1);
  bool t3 = t2;
  if (!t3) {
    ov_str* t4 = ov_str_new("closed", 6);
    bool t5 = ov_str_eq(p0, t4);
    t3 = t5;
  }
  return t3;
}
