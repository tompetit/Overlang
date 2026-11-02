#include "ov_types.h"

bool ov_is_lower_390d9774_f60e8723(ov_str*);
bool ov_is_upper_474465f2_876a8502(ov_str*);
bool ov_is_alpha_d80fcb1a_218a1f42(ov_str* p0);

bool ov_is_alpha_d80fcb1a_218a1f42(ov_str* p0) {
  bool t1 = ov_is_upper_474465f2_876a8502(p0);
  bool t2 = t1;
  if (!t2) {
    bool t3 = ov_is_lower_390d9774_f60e8723(p0);
    t2 = t3;
  }
  return t2;
}
