#include "ov_types.h"

ovt_Response_dae56844* ov_rt_create_issue_a5cfd3d8_60cdb210(int64_t, ovt_Project_73b03799*, ov_str*);
ovt_Response_dae56844* ov_with_project_4b2a8feb_720ea08b(int64_t, ov_str*, ov_closure*);
ovt_Response_dae56844* ov_h_create_issue_f1044a28_8cd69ed1(int64_t p0, ov_str* p1, ov_str* p2);

struct ovenv_5509 { int64_t p0; ov_str* p2; };

static ovt_Response_dae56844* ovl_ov_h_create_issue_f1044a28_8cd69ed1_5509(void* ovenv, ovt_Project_73b03799* l5509_0) {
  struct ovenv_5509* env = (struct ovenv_5509*)ovenv;
  int64_t p0 = env->p0;
  ov_str* p2 = env->p2;
  ovt_Response_dae56844* t1 = ov_rt_create_issue_a5cfd3d8_60cdb210(p0, l5509_0, p2);
  return t1;
}

ovt_Response_dae56844* ov_h_create_issue_f1044a28_8cd69ed1(int64_t p0, ov_str* p1, ov_str* p2) {
  struct ovenv_5509* t2 = GC_MALLOC(sizeof(struct ovenv_5509));
  t2->p0 = p0;
  t2->p2 = p2;
  ov_closure* t3 = ov_closure_mk((void*)ovl_ov_h_create_issue_f1044a28_8cd69ed1_5509, t2);
  ovt_Response_dae56844* t4 = ov_with_project_4b2a8feb_720ea08b(p0, p1, t3);
  return t4;
}
