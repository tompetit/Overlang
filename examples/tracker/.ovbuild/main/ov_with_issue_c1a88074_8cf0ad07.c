#include "ov_types.h"

ovt_Option_c3749d6a* ov_find_issue_68dd4779_d24e6f3d(int64_t, int64_t, int64_t);
ovt_Response_dae56844* ov_resp_error_b47b9fc9_7be2787d(int64_t, ov_str*);
ovt_Response_dae56844* ov_with_project_4b2a8feb_720ea08b(int64_t, ov_str*, ov_closure*);
ovt_Response_dae56844* ov_with_issue_c1a88074_8cf0ad07(int64_t p0, ov_str* p1, int64_t p2, ov_closure* p3);

struct ovenv_5131 { int64_t p0; int64_t p2; ov_closure* p3; ov_str* p1; };

static ovt_Response_dae56844* ovl_ov_with_issue_c1a88074_8cf0ad07_5131(void* ovenv, ovt_Project_73b03799* l5131_0) {
  struct ovenv_5131* env = (struct ovenv_5131*)ovenv;
  int64_t p0 = env->p0;
  int64_t p2 = env->p2;
  ov_closure* p3 = env->p3;
  ov_str* p1 = env->p1;
  int64_t t1 = l5131_0->f_id;
  ovt_Option_c3749d6a* t2 = ov_find_issue_68dd4779_d24e6f3d(p0, t1, p2);
  ovt_Response_dae56844* t3 = 0; (void)t3;
  bool t4 = false; (void)t4;
  if (!t4) {
    if ((t2->tag == 0)) {
      ovt_Issue_b825796a* v5111 = t2->u.c0.f0;
      t4 = true;
      ovt_Response_dae56844* t5 = ((ovt_Response_dae56844* (*)(void*, ovt_Project_73b03799*, ovt_Issue_b825796a*))p3->fn)(p3->env, l5131_0, v5111);
      t3 = t5;
    }
  }
  if (!t4) {
    if ((t2->tag == 1)) {
      t4 = true;
      ov_str* t6 = ov_str_new("no such issue: ", 15);
      ov_str* t7 = ov_str_concat(t6, p1);
      ov_str* t8 = ov_str_new("-", 1);
      ov_str* t9 = ov_str_concat(t7, t8);
      ov_str* t10 = ov_show_int(p2);
      ov_str* t11 = ov_str_concat(t9, t10);
      ovt_Response_dae56844* t12 = ov_resp_error_b47b9fc9_7be2787d(INT64_C(404), t11);
      t3 = t12;
    }
  }
  return t3;
}

ovt_Response_dae56844* ov_with_issue_c1a88074_8cf0ad07(int64_t p0, ov_str* p1, int64_t p2, ov_closure* p3) {
  struct ovenv_5131* t13 = GC_MALLOC(sizeof(struct ovenv_5131));
  t13->p0 = p0;
  t13->p2 = p2;
  t13->p3 = p3;
  t13->p1 = p1;
  ov_closure* t14 = ov_closure_mk((void*)ovl_ov_with_issue_c1a88074_8cf0ad07_5131, t13);
  ovt_Response_dae56844* t15 = ov_with_project_4b2a8feb_720ea08b(p0, p1, t14);
  return t15;
}
