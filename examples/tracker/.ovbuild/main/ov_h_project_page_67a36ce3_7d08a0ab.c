#include "ov_types.h"

ovt_Response_dae56844* ov_resp_error_b47b9fc9_7be2787d(int64_t, ov_str*);
ovt_Response_dae56844* ov_respond_project_dbff6379_4e076edf(int64_t, ovt_Project_73b03799*, ovt_Filter_50e09095*);
ovt_Response_dae56844* ov_with_project_4b2a8feb_720ea08b(int64_t, ov_str*, ov_closure*);
ovt_Result_3d1782a4* ov_parse_filter_1eb2981a_72fbb606(ov_str*);
ovt_Response_dae56844* ov_h_project_page_67a36ce3_7d08a0ab(int64_t p0, ov_str* p1, ov_str* p2);

struct ovenv_5343 { ov_str* p2; int64_t p0; };

static ovt_Response_dae56844* ovl_ov_h_project_page_67a36ce3_7d08a0ab_5343(void* ovenv, ovt_Project_73b03799* l5343_0) {
  struct ovenv_5343* env = (struct ovenv_5343*)ovenv;
  ov_str* p2 = env->p2;
  int64_t p0 = env->p0;
  ovt_Result_3d1782a4* t1 = ov_parse_filter_1eb2981a_72fbb606(p2);
  ovt_Response_dae56844* t2 = 0; (void)t2;
  bool t3 = false; (void)t3;
  if (!t3) {
    if ((t1->tag == 0)) {
      ovt_Filter_50e09095* v5329 = t1->u.c0.f0;
      t3 = true;
      ovt_Response_dae56844* t4 = ov_respond_project_dbff6379_4e076edf(p0, l5343_0, v5329);
      t2 = t4;
    }
  }
  if (!t3) {
    if ((t1->tag == 1)) {
      ov_str* v5336 = t1->u.c1.f0;
      t3 = true;
      ovt_Response_dae56844* t5 = ov_resp_error_b47b9fc9_7be2787d(INT64_C(400), v5336);
      t2 = t5;
    }
  }
  return t2;
}

ovt_Response_dae56844* ov_h_project_page_67a36ce3_7d08a0ab(int64_t p0, ov_str* p1, ov_str* p2) {
  struct ovenv_5343* t6 = GC_MALLOC(sizeof(struct ovenv_5343));
  t6->p2 = p2;
  t6->p0 = p0;
  ov_closure* t7 = ov_closure_mk((void*)ovl_ov_h_project_page_67a36ce3_7d08a0ab_5343, t6);
  ovt_Response_dae56844* t8 = ov_with_project_4b2a8feb_720ea08b(p0, p1, t7);
  return t8;
}
