#include "ov_types.h"

ov_list* ov_list_labels_c35c266b_8bb2dc64(int64_t, int64_t);
ov_str* ov_render_project_d1f1ea37_8a57e1d8(ovt_Project_73b03799*, ov_list*, ovt_Filter_50e09095*, ovt_IssuePage_114bf070*);
ovt_IssuePage_114bf070* ov_search_issues_573fd6dd_7e780e04(int64_t, int64_t, ovt_Filter_50e09095*);
ovt_Response_dae56844* ov_resp_html_e2b10d08_fbb70ecb(ov_str*);
ovt_Response_dae56844* ov_respond_project_dbff6379_4e076edf(int64_t p0, ovt_Project_73b03799* p1, ovt_Filter_50e09095* p2);

ovt_Response_dae56844* ov_respond_project_dbff6379_4e076edf(int64_t p0, ovt_Project_73b03799* p1, ovt_Filter_50e09095* p2) {
  int64_t t1 = p1->f_id;
  ov_list* t2 = ov_list_labels_c35c266b_8bb2dc64(p0, t1);
  int64_t t3 = p1->f_id;
  ovt_IssuePage_114bf070* t4 = ov_search_issues_573fd6dd_7e780e04(p0, t3, p2);
  ov_str* t5 = ov_render_project_d1f1ea37_8a57e1d8(p1, t2, p2, t4);
  ovt_Response_dae56844* t6 = ov_resp_html_e2b10d08_fbb70ecb(t5);
  return t6;
}
