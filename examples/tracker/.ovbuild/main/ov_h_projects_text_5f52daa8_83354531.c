#include "ov_types.h"

ov_list* ov_list_projects_6548d364_f33d27a9(int64_t);
ov_str* ov_projects_text_bab0fb0c_a03c894e(ov_list*);
ovt_Response_dae56844* ov_resp_text_8a5de9a9_e3271fe2(ov_str*);
ovt_Response_dae56844* ov_h_projects_text_5f52daa8_83354531(int64_t p0);

ovt_Response_dae56844* ov_h_projects_text_5f52daa8_83354531(int64_t p0) {
  ov_list* t1 = ov_list_projects_6548d364_f33d27a9(p0);
  ov_str* t2 = ov_projects_text_bab0fb0c_a03c894e(t1);
  ovt_Response_dae56844* t3 = ov_resp_text_8a5de9a9_e3271fe2(t2);
  return t3;
}
