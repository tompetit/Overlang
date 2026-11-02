#include "ov_types.h"

ov_str* ov_issues_text_2e2d8929_b997bacc(ov_list*);
ovt_IssuePage_114bf070* ov_search_issues_573fd6dd_7e780e04(int64_t, int64_t, ovt_Filter_50e09095*);
ovt_Response_dae56844* ov_resp_error_b47b9fc9_7be2787d(int64_t, ov_str*);
ovt_Response_dae56844* ov_resp_text_8a5de9a9_e3271fe2(ov_str*);
ovt_Response_dae56844* ov_with_project_4b2a8feb_720ea08b(int64_t, ov_str*, ov_closure*);
ovt_Result_3d1782a4* ov_parse_filter_1eb2981a_72fbb606(ov_str*);
ovt_Response_dae56844* ov_h_issues_text_835bb9e7_b4f940c2(int64_t p0, ov_str* p1, ov_str* p2);

struct ovenv_5371 { ov_str* p2; int64_t p0; };

static ovt_Response_dae56844* ovl_ov_h_issues_text_835bb9e7_b4f940c2_5371(void* ovenv, ovt_Project_73b03799* l5371_0) {
  struct ovenv_5371* env = (struct ovenv_5371*)ovenv;
  ov_str* p2 = env->p2;
  int64_t p0 = env->p0;
  ovt_Result_3d1782a4* t1 = ov_parse_filter_1eb2981a_72fbb606(p2);
  ovt_Response_dae56844* t2 = 0; (void)t2;
  bool t3 = false; (void)t3;
  if (!t3) {
    if ((t1->tag == 0)) {
      ovt_Filter_50e09095* v5351 = t1->u.c0.f0;
      t3 = true;
      int64_t t4 = l5371_0->f_id;
      ovt_IssuePage_114bf070* t5 = ov_search_issues_573fd6dd_7e780e04(p0, t4, v5351);
      ov_list* t6 = t5->f_issues;
      ov_str* t7 = ov_issues_text_2e2d8929_b997bacc(t6);
      ovt_Response_dae56844* t8 = ov_resp_text_8a5de9a9_e3271fe2(t7);
      t2 = t8;
    }
  }
  if (!t3) {
    if ((t1->tag == 1)) {
      ov_str* v5364 = t1->u.c1.f0;
      t3 = true;
      ovt_Response_dae56844* t9 = ov_resp_error_b47b9fc9_7be2787d(INT64_C(400), v5364);
      t2 = t9;
    }
  }
  return t2;
}

ovt_Response_dae56844* ov_h_issues_text_835bb9e7_b4f940c2(int64_t p0, ov_str* p1, ov_str* p2) {
  struct ovenv_5371* t10 = GC_MALLOC(sizeof(struct ovenv_5371));
  t10->p2 = p2;
  t10->p0 = p0;
  ov_closure* t11 = ov_closure_mk((void*)ovl_ov_h_issues_text_835bb9e7_b4f940c2_5371, t10);
  ovt_Response_dae56844* t12 = ov_with_project_4b2a8feb_720ea08b(p0, p1, t11);
  return t12;
}
