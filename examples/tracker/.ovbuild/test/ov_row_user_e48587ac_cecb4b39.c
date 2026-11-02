#include "ov_types.h"

int64_t ov_cell_int_241695c3_c8b376c8(ov_list*, int64_t);
ov_str* ov_cell_str_c5e3a532_bb12c045(ov_list*, int64_t);
ovt_User_9c127ab2* ov_row_user_e48587ac_cecb4b39(ov_list* p0);

ovt_User_9c127ab2* ov_row_user_e48587ac_cecb4b39(ov_list* p0) {
  int64_t t1 = ov_cell_int_241695c3_c8b376c8(p0, INT64_C(0));
  ov_str* t2 = ov_cell_str_c5e3a532_bb12c045(p0, INT64_C(1));
  ovt_User_9c127ab2* t3 = ovt_User_9c127ab2_mk(t1, t2);
  return t3;
}
