#include "ov_types.h"

int64_t ov_unwrap_or_e7341bb9_bde9d67c(ovt_Option_d5f2c30a*, int64_t);
ov_str* ov_cell_str_c5e3a532_bb12c045(ov_list*, int64_t);
int64_t ov_cell_int_241695c3_c8b376c8(ov_list* p0, int64_t p1);

int64_t ov_cell_int_241695c3_c8b376c8(ov_list* p0, int64_t p1) {
  ov_str* t1 = ov_cell_str_c5e3a532_bb12c045(p0, p1);
  int64_t t2 = 0;
  ovt_Option_d5f2c30a* t3 = ((ov_parse_int(t1, &t2)) ? ovt_Option_d5f2c30a_mk0(t2) : ovt_Option_d5f2c30a_mk1());
  int64_t t4 = ov_unwrap_or_e7341bb9_bde9d67c(t3, INT64_C(0));
  return t4;
}
