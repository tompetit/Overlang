#include "ov_types.h"

ovt_Response_dae56844* ov_resp_error_b47b9fc9_7be2787d(int64_t p0, ov_str* p1);

ovt_Response_dae56844* ov_resp_error_b47b9fc9_7be2787d(int64_t p0, ov_str* p1) {
  ov_str* t1 = ov_str_new("text/plain; charset=utf-8", 25);
  ov_str* t2 = ov_str_new("\n", 1);
  ov_str* t3 = ov_str_concat(p1, t2);
  ovt_Response_dae56844* t4 = ovt_Response_dae56844_mk(p0, t1, t3);
  return t4;
}
