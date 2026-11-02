#include "ov_types.h"

bool ov_is_alpha_d80fcb1a_218a1f42(ov_str*);
bool ov_is_digit_d7695ac0_63fa906b(ov_str*);
bool ov_is_alnum_6cf22f95_00c25202(ov_str* p0);

bool ov_is_alnum_6cf22f95_00c25202(ov_str* p0) {
  bool t1 = ov_is_alpha_d80fcb1a_218a1f42(p0);
  bool t2 = t1;
  if (!t2) {
    bool t3 = ov_is_digit_d7695ac0_63fa906b(p0);
    t2 = t3;
  }
  return t2;
}
