#include "ov_types.h"

int64_t ov_cell_int_241695c3_c8b376c8(ov_list*, int64_t);
ov_str* ov_cell_str_c5e3a532_bb12c045(ov_list*, int64_t);
ovt_Project_73b03799* ov_row_project_ca4a0485_7b4b99ad(ov_list* p0);

ovt_Project_73b03799* ov_row_project_ca4a0485_7b4b99ad(ov_list* p0) {
  int64_t t1 = ov_cell_int_241695c3_c8b376c8(p0, INT64_C(0));
  ov_str* t2 = ov_cell_str_c5e3a532_bb12c045(p0, INT64_C(1));
  ov_str* t3 = ov_cell_str_c5e3a532_bb12c045(p0, INT64_C(2));
  ovt_Project_73b03799* t4 = ovt_Project_73b03799_mk(t1, t2, t3);
  return t4;
}
