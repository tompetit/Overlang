#include "ov_types.h"

ovt_Response_dae56844* ov_bad_request_569742af_b0b3f10f(ov_str* p0);

ovt_Response_dae56844* ov_bad_request_569742af_b0b3f10f(ov_str* p0) {
  ov_str* t1 = ov_str_new("text/plain", 10);
  ov_str* t2 = ov_str_new("\n", 1);
  ov_str* t3 = ov_str_concat(p0, t2);
  ovt_Response_dae56844* t4 = ovt_Response_dae56844_mk(INT64_C(400), t1, t3);
  return t4;
}
