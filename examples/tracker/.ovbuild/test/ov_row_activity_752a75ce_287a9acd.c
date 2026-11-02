#include "ov_types.h"

int64_t ov_cell_int_241695c3_c8b376c8(ov_list*, int64_t);
ov_str* ov_cell_str_c5e3a532_bb12c045(ov_list*, int64_t);
ovt_Activity_dd4dbed0* ov_row_activity_752a75ce_287a9acd(ov_list* p0);

ovt_Activity_dd4dbed0* ov_row_activity_752a75ce_287a9acd(ov_list* p0) {
  int64_t t1 = ov_cell_int_241695c3_c8b376c8(p0, INT64_C(0));
  int64_t t2 = ov_cell_int_241695c3_c8b376c8(p0, INT64_C(1));
  ov_str* t3 = ov_cell_str_c5e3a532_bb12c045(p0, INT64_C(2));
  ov_str* t4 = ov_cell_str_c5e3a532_bb12c045(p0, INT64_C(3));
  ovt_Activity_dd4dbed0* t5 = ovt_Activity_dd4dbed0_mk(t1, t2, t3, t4);
  return t5;
}
