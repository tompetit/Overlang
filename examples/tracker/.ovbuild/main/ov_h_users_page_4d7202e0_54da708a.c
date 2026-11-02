#include "ov_types.h"

ov_list* ov_list_users_71a58859_859009d3(int64_t);
ov_str* ov_render_users_8d6c5327_5af2c1ac(ov_list*);
ovt_Response_dae56844* ov_resp_html_e2b10d08_fbb70ecb(ov_str*);
ovt_Response_dae56844* ov_h_users_page_4d7202e0_54da708a(int64_t p0);

ovt_Response_dae56844* ov_h_users_page_4d7202e0_54da708a(int64_t p0) {
  ov_list* t1 = ov_list_users_71a58859_859009d3(p0);
  ov_str* t2 = ov_render_users_8d6c5327_5af2c1ac(t1);
  ovt_Response_dae56844* t3 = ov_resp_html_e2b10d08_fbb70ecb(t2);
  return t3;
}
