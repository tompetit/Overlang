#include "ov_types.h"

ovt_Response_dae56844* ov_resp_text_8a5de9a9_e3271fe2(ov_str* p0);

ovt_Response_dae56844* ov_resp_text_8a5de9a9_e3271fe2(ov_str* p0) {
  ov_str* t1 = ov_str_new("text/plain; charset=utf-8", 25);
  ovt_Response_dae56844* t2 = ovt_Response_dae56844_mk(INT64_C(200), t1, p0);
  return t2;
}
