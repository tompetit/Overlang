#include "ov_types.h"

ov_list* ov_all_issues_103a8b74_2317e431(int64_t, int64_t);
ov_str* ov_issues_csv_941007d4_2944cca3(ov_list*);
ovt_Response_dae56844* ov_resp_csv_4c191e5d_d74b9a49(ov_str*);
ovt_Response_dae56844* ov_with_project_4b2a8feb_720ea08b(int64_t, ov_str*, ov_closure*);
ovt_Response_dae56844* ov_h_issues_csv_f73ab561_a85af68f(int64_t p0, ov_str* p1);

struct ovenv_5385 { int64_t p0; };

static ovt_Response_dae56844* ovl_ov_h_issues_csv_f73ab561_a85af68f_5385(void* ovenv, ovt_Project_73b03799* l5385_0) {
  struct ovenv_5385* env = (struct ovenv_5385*)ovenv;
  int64_t p0 = env->p0;
  int64_t t1 = l5385_0->f_id;
  ov_list* t2 = ov_all_issues_103a8b74_2317e431(p0, t1);
  ov_str* t3 = ov_issues_csv_941007d4_2944cca3(t2);
  ovt_Response_dae56844* t4 = ov_resp_csv_4c191e5d_d74b9a49(t3);
  return t4;
}

ovt_Response_dae56844* ov_h_issues_csv_f73ab561_a85af68f(int64_t p0, ov_str* p1) {
  struct ovenv_5385* t5 = GC_MALLOC(sizeof(struct ovenv_5385));
  t5->p0 = p0;
  ov_closure* t6 = ov_closure_mk((void*)ovl_ov_h_issues_csv_f73ab561_a85af68f_5385, t5);
  ovt_Response_dae56844* t7 = ov_with_project_4b2a8feb_720ea08b(p0, p1, t6);
  return t7;
}
