#include "ov_types.h"

ov_str* ov_unwrap_or_e7341bb9_412aa135(ovt_Option_6133f15b*, ov_str*);
ov_str* ov_cell_str_c5e3a532_bb12c045(ov_list* p0, int64_t p1);

ov_str* ov_cell_str_c5e3a532_bb12c045(ov_list* p0, int64_t p1) {
  ov_value t1 = {0};
  ovt_Option_6133f15b* t2 = ((ov_list_try_get(p0, p1, &t1)) ? ovt_Option_6133f15b_mk0(((ov_str*)(t1).p)) : ovt_Option_6133f15b_mk1());
  ov_str* t3 = ov_str_new("", 0);
  ov_str* t4 = ov_unwrap_or_e7341bb9_412aa135(t2, t3);
  return t4;
}
