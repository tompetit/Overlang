#include "ov_types.h"

ovt_Response_dae56844* ov_resp_text_8a5de9a9_e3271fe2(ov_str*);
ovt_Response_dae56844* ov_h_health_49c56747_0a13e3ef(void);

ovt_Response_dae56844* ov_h_health_49c56747_0a13e3ef(void) {
  ov_str* t1 = ov_str_new("ok\n", 3);
  ovt_Response_dae56844* t2 = ov_resp_text_8a5de9a9_e3271fe2(t1);
  return t2;
}
