#include "ov_types.h"

int64_t ov_http_accept_5f360067_6b72cd93(int64_t);
int64_t ov_http_respond_5639bb22_f73d4847(int64_t, int64_t, ov_str*, ov_str*);
ov_str* ov_http_body_81a71fd4_07410916(int64_t);
ov_str* ov_http_method_e4d563f3_4312fe9c(int64_t);
ov_str* ov_http_path_44821edd_14991708(int64_t);
ov_str* ov_http_query_d2f16e79_db4f805c(int64_t);
ovt_Response_dae56844* ov_route_e0291c32_6029bc97(int64_t, ovt_Request_da024db6*);
uint8_t ov_serve_5fc523c8_347af594(int64_t p0, int64_t p1);

uint8_t ov_serve_5fc523c8_347af594(int64_t p0, int64_t p1) {
  while (1) {
    if (!true) break;
    int64_t t2 = ov_http_accept_5f360067_6b72cd93(p1);
    int64_t v6325 = t2;
    bool t3 = (v6325 >= INT64_C(0));
    if (t3) {
      ov_str* t4 = ov_http_method_e4d563f3_4312fe9c(v6325);
      ov_str* t5 = ov_http_path_44821edd_14991708(v6325);
      ov_str* t6 = ov_http_query_d2f16e79_db4f805c(v6325);
      ov_str* t7 = ov_http_body_81a71fd4_07410916(v6325);
      ovt_Request_da024db6* t8 = ovt_Request_da024db6_mk(t4, t5, t6, t7);
      ovt_Response_dae56844* t9 = ov_route_e0291c32_6029bc97(p0, t8);
      ovt_Response_dae56844* v6332 = t9;
      int64_t t10 = v6332->f_status;
      ov_str* t11 = v6332->f_content_type;
      ov_str* t12 = v6332->f_body;
      int64_t t13 = ov_http_respond_5639bb22_f73d4847(v6325, t10, t11, t12);
    }
  }
  return 0;
}
