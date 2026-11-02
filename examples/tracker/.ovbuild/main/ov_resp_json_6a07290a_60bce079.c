#include "ov_types.h"

ovt_Response_dae56844* ov_resp_json_6a07290a_60bce079(ov_str* p0);

ovt_Response_dae56844* ov_resp_json_6a07290a_60bce079(ov_str* p0) {
  ov_str* t1 = ov_str_new("application/json", 16);
  ovt_Response_dae56844* t2 = ovt_Response_dae56844_mk(INT64_C(200), t1, p0);
  return t2;
}
