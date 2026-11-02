#include "ov_types.h"

ovt_Option_da3cc0d5* ov_find_project_da7b7099_8f536708(int64_t, ov_str*);
ovt_Response_dae56844* ov_resp_error_b47b9fc9_7be2787d(int64_t, ov_str*);
ovt_Response_dae56844* ov_with_project_4b2a8feb_720ea08b(int64_t p0, ov_str* p1, ov_closure* p2);

ovt_Response_dae56844* ov_with_project_4b2a8feb_720ea08b(int64_t p0, ov_str* p1, ov_closure* p2) {
  ovt_Option_da3cc0d5* t1 = ov_find_project_da7b7099_8f536708(p0, p1);
  ovt_Response_dae56844* t2 = 0; (void)t2;
  bool t3 = false; (void)t3;
  if (!t3) {
    if ((t1->tag == 0)) {
      ovt_Project_73b03799* v5089 = t1->u.c0.f0;
      t3 = true;
      ovt_Response_dae56844* t4 = ((ovt_Response_dae56844* (*)(void*, ovt_Project_73b03799*))p2->fn)(p2->env, v5089);
      t2 = t4;
    }
  }
  if (!t3) {
    if ((t1->tag == 1)) {
      t3 = true;
      ov_str* t5 = ov_str_new("no such project: ", 17);
      ov_str* t6 = ov_str_concat(t5, p1);
      ovt_Response_dae56844* t7 = ov_resp_error_b47b9fc9_7be2787d(INT64_C(404), t6);
      t2 = t7;
    }
  }
  return t2;
}
