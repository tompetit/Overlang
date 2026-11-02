#include "ov_types.h"

ovt_Response_dae56844* ov_resp_html_e2b10d08_fbb70ecb(ov_str* p0);

ovt_Response_dae56844* ov_resp_html_e2b10d08_fbb70ecb(ov_str* p0) {
  ov_str* t1 = ov_str_new("text/html; charset=utf-8", 24);
  ovt_Response_dae56844* t2 = ovt_Response_dae56844_mk(INT64_C(200), t1, p0);
  return t2;
}
