#include "ov_types.h"

ov_list* ov_list_users_71a58859_859009d3(int64_t);
ov_str* ov_users_text_c122fcd7_0d9a4446(ov_list*);
ovt_Response_dae56844* ov_resp_text_8a5de9a9_e3271fe2(ov_str*);
ovt_Response_dae56844* ov_h_users_text_e96da07a_0a65f636(int64_t p0);

ovt_Response_dae56844* ov_h_users_text_e96da07a_0a65f636(int64_t p0) {
  ov_list* t1 = ov_list_users_71a58859_859009d3(p0);
  ov_str* t2 = ov_users_text_c122fcd7_0d9a4446(t1);
  ovt_Response_dae56844* t3 = ov_resp_text_8a5de9a9_e3271fe2(t2);
  return t3;
}
