#include "ov_types.h"

ov_list* ov_list_activity_66c9a057_4d5ab5f1(int64_t, int64_t);
ov_list* ov_list_comments_625aa8cf_c111fe21(int64_t, int64_t);
ov_str* ov_render_issue_57e4bb34_d2db72ef(ovt_Issue_b825796a*, ov_list*, ov_list*);
ovt_Option_c3749d6a* ov_find_issue_68dd4779_d24e6f3d(int64_t, int64_t, int64_t);
ovt_Response_dae56844* ov_resp_error_b47b9fc9_7be2787d(int64_t, ov_str*);
ovt_Response_dae56844* ov_resp_html_e2b10d08_fbb70ecb(ov_str*);
ovt_Response_dae56844* ov_respond_issue_61e7c0dc_9db72996(int64_t p0, ovt_Project_73b03799* p1, int64_t p2);

ovt_Response_dae56844* ov_respond_issue_61e7c0dc_9db72996(int64_t p0, ovt_Project_73b03799* p1, int64_t p2) {
  int64_t t1 = p1->f_id;
  ovt_Option_c3749d6a* t2 = ov_find_issue_68dd4779_d24e6f3d(p0, t1, p2);
  ovt_Response_dae56844* t3 = 0; (void)t3;
  bool t4 = false; (void)t4;
  if (!t4) {
    if ((t2->tag == 0)) {
      ovt_Issue_b825796a* v5139 = t2->u.c0.f0;
      t4 = true;
      int64_t t5 = v5139->f_id;
      ov_list* t6 = ov_list_comments_625aa8cf_c111fe21(p0, t5);
      int64_t t7 = v5139->f_id;
      ov_list* t8 = ov_list_activity_66c9a057_4d5ab5f1(p0, t7);
      ov_str* t9 = ov_render_issue_57e4bb34_d2db72ef(v5139, t6, t8);
      ovt_Response_dae56844* t10 = ov_resp_html_e2b10d08_fbb70ecb(t9);
      t3 = t10;
    }
  }
  if (!t4) {
    if ((t2->tag == 1)) {
      t4 = true;
      ov_str* t11 = ov_str_new("no such issue: ", 15);
      ov_str* t12 = p1->f_key;
      ov_str* t13 = ov_str_concat(t11, t12);
      ov_str* t14 = ov_str_new("-", 1);
      ov_str* t15 = ov_str_concat(t13, t14);
      ov_str* t16 = ov_show_int(p2);
      ov_str* t17 = ov_str_concat(t15, t16);
      ovt_Response_dae56844* t18 = ov_resp_error_b47b9fc9_7be2787d(INT64_C(404), t17);
      t3 = t18;
    }
  }
  return t3;
}
