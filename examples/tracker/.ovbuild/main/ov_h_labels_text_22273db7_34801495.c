#include "ov_types.h"

ov_list* ov_list_labels_c35c266b_8bb2dc64(int64_t, int64_t);
ov_str* ov_labels_text_50fce26e_e183be1c(ov_list*);
ovt_Response_dae56844* ov_resp_text_8a5de9a9_e3271fe2(ov_str*);
ovt_Response_dae56844* ov_with_project_4b2a8feb_720ea08b(int64_t, ov_str*, ov_closure*);
ovt_Response_dae56844* ov_h_labels_text_22273db7_34801495(int64_t p0, ov_str* p1);

struct ovenv_5523 { int64_t p0; };

static ovt_Response_dae56844* ovl_ov_h_labels_text_22273db7_34801495_5523(void* ovenv, ovt_Project_73b03799* l5523_0) {
  struct ovenv_5523* env = (struct ovenv_5523*)ovenv;
  int64_t p0 = env->p0;
  int64_t t1 = l5523_0->f_id;
  ov_list* t2 = ov_list_labels_c35c266b_8bb2dc64(p0, t1);
  ov_str* t3 = ov_labels_text_50fce26e_e183be1c(t2);
  ovt_Response_dae56844* t4 = ov_resp_text_8a5de9a9_e3271fe2(t3);
  return t4;
}

ovt_Response_dae56844* ov_h_labels_text_22273db7_34801495(int64_t p0, ov_str* p1) {
  struct ovenv_5523* t5 = GC_MALLOC(sizeof(struct ovenv_5523));
  t5->p0 = p0;
  ov_closure* t6 = ov_closure_mk((void*)ovl_ov_h_labels_text_22273db7_34801495_5523, t5);
  ovt_Response_dae56844* t7 = ov_with_project_4b2a8feb_720ea08b(p0, p1, t6);
  return t7;
}
