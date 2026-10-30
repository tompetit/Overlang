#include "ov_types.h"

int64_t ov_unwrap_or_cf818e0c_bde9d67c(ovt_Option_d5f2c30a*, int64_t);
int64_t ov_port_from_e599bbcb_aa28173d(ov_list* p0);

int64_t ov_port_from_e599bbcb_aa28173d(ov_list* p0) {
  int64_t t1 = 0; (void)t1;
  bool t2 = false; (void)t2;
  if (!t2) {
    if ((p0->len >= 1)) {
      ov_str* v660 = ((ov_str*)(p0->items[0]).p);
      ov_list* v661 = ov_list_slice(p0, 1, p0->len - 0);
      t2 = true;
      int64_t t3 = 0;
      ovt_Option_d5f2c30a* t4 = ((ov_parse_int(v660, &t3)) ? ovt_Option_d5f2c30a_mk0(t3) : ovt_Option_d5f2c30a_mk1());
      int64_t t5 = ov_unwrap_or_cf818e0c_bde9d67c(t4, INT64_C(8080));
      t1 = t5;
    }
  }
  if (!t2) {
    if (true) {
      t2 = true;
      t1 = INT64_C(8080);
    }
  }
  return t1;
}
