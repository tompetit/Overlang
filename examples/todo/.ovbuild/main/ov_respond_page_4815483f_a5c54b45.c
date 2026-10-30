#include "ov_types.h"

ov_list* ov_list_todos_24d1401c_1ec84d65(int64_t);
ov_str* ov_render_page_aafa113d_06878f9d(ov_list*);
ovt_Response_dae56844* ov_respond_page_4815483f_a5c54b45(int64_t p0);

ovt_Response_dae56844* ov_respond_page_4815483f_a5c54b45(int64_t p0) {
  ov_str* t1 = ov_str_new("text/html; charset=utf-8", 24);
  ov_list* t2 = ov_list_todos_24d1401c_1ec84d65(p0);
  ov_str* t3 = ov_render_page_aafa113d_06878f9d(t2);
  ovt_Response_dae56844* t4 = ovt_Response_dae56844_mk(INT64_C(200), t1, t3);
  return t4;
}
