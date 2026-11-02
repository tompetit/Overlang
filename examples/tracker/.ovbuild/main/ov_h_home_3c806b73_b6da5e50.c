#include "ov_types.h"

ov_list* ov_list_projects_6548d364_f33d27a9(int64_t);
ov_str* ov_render_home_15b65eb0_de9bb9d3(ov_list*);
ovt_Response_dae56844* ov_resp_html_e2b10d08_fbb70ecb(ov_str*);
ovt_Response_dae56844* ov_h_home_3c806b73_b6da5e50(int64_t p0);

ovt_Response_dae56844* ov_h_home_3c806b73_b6da5e50(int64_t p0) {
  ov_list* t1 = ov_list_projects_6548d364_f33d27a9(p0);
  ov_str* t2 = ov_render_home_15b65eb0_de9bb9d3(t1);
  ovt_Response_dae56844* t3 = ov_resp_html_e2b10d08_fbb70ecb(t2);
  return t3;
}
