#include "ov_types.h"

int64_t ov_http_accept_10182d5b_6b72cd93(int64_t);
int64_t ov_http_respond_4cc639fd_f73d4847(int64_t, int64_t, ov_str*, ov_str*);
ov_str* ov_http_body_498e6a7b_07410916(int64_t);
ov_str* ov_http_method_c930d1b8_4312fe9c(int64_t);
ov_str* ov_http_path_53b97211_14991708(int64_t);
ovt_Response_dae56844* ov_route_3127fb46_6029bc97(int64_t, ov_str*, ov_str*, ov_str*);
uint8_t ov_serve_21e06530_347af594(int64_t p0, int64_t p1);

uint8_t ov_serve_21e06530_347af594(int64_t p0, int64_t p1) {
  while (1) {
    if (!true) break;
    int64_t t1 = ov_http_accept_10182d5b_6b72cd93(p1);
    int64_t v672 = t1;
    bool t2 = (v672 >= INT64_C(0));
    if (t2) {
      ov_str* t3 = ov_http_method_c930d1b8_4312fe9c(v672);
      ov_str* t4 = ov_http_path_53b97211_14991708(v672);
      ov_str* t5 = ov_http_body_498e6a7b_07410916(v672);
      ovt_Response_dae56844* t6 = ov_route_3127fb46_6029bc97(p0, t3, t4, t5);
      ovt_Response_dae56844* v679 = t6;
      int64_t t7 = v679->f_status;
      ov_str* t8 = v679->f_content_type;
      ov_str* t9 = v679->f_body;
      int64_t t10 = ov_http_respond_4cc639fd_f73d4847(v672, t7, t8, t9);
    }
  }
  return 0;
}
