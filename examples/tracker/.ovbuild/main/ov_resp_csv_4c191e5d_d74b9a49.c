#include "ov_types.h"

ovt_Response_dae56844* ov_resp_csv_4c191e5d_d74b9a49(ov_str* p0);

ovt_Response_dae56844* ov_resp_csv_4c191e5d_d74b9a49(ov_str* p0) {
  ov_str* t1 = ov_str_new("text/csv; charset=utf-8", 23);
  ovt_Response_dae56844* t2 = ovt_Response_dae56844_mk(INT64_C(200), t1, p0);
  return t2;
}
