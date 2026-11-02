#include "ov_types.h"

ovt_Response_dae56844* ov_resp_error_b47b9fc9_7be2787d(int64_t, ov_str*);
ovt_Response_dae56844* ov_rt_bad_priority_626b8671_0dd3d100(int64_t p0);

ovt_Response_dae56844* ov_rt_bad_priority_626b8671_0dd3d100(int64_t p0) {
  bool t1 = (p0 < INT64_C(0));
  ovt_Response_dae56844* t2 = 0; (void)t2;
  if (t1) {
    ov_str* t3 = ov_str_new("bad priority", 12);
    ovt_Response_dae56844* t4 = ov_resp_error_b47b9fc9_7be2787d(INT64_C(400), t3);
    t2 = t4;
  } else {
    ov_str* t5 = ov_str_new("bad priority", 12);
    ovt_Response_dae56844* t6 = ov_resp_error_b47b9fc9_7be2787d(INT64_C(422), t5);
    t2 = t6;
  }
  return t2;
}
