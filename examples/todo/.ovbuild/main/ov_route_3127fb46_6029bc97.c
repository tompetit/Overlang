#include "ov_types.h"

int64_t ov_add_todo_cbaf52b2_7dd0667a(int64_t, ov_str*);
int64_t ov_delete_todo_02f14ca2_91f6bf4f(int64_t, int64_t);
int64_t ov_toggle_todo_5ad29be9_c45c207c(int64_t, int64_t);
int64_t ov_unwrap_or_cf818e0c_bde9d67c(ovt_Option_d5f2c30a*, int64_t);
ov_list* ov_list_todos_24d1401c_1ec84d65(int64_t);
ov_str* ov_form_value_45246b5c_af36f5f9(ov_str*, ov_str*);
ov_str* ov_render_text_b3499619_840f04f8(ov_list*);
ovt_Response_dae56844* ov_bad_request_569742af_b0b3f10f(ov_str*);
ovt_Response_dae56844* ov_respond_page_4815483f_a5c54b45(int64_t);
ovt_Response_dae56844* ov_route_3127fb46_6029bc97(int64_t p0, ov_str* p1, ov_str* p2, ov_str* p3);

ovt_Response_dae56844* ov_route_3127fb46_6029bc97(int64_t p0, ov_str* p1, ov_str* p2, ov_str* p3) {
  ov_str* t1 = ov_str_new("id", 2);
  ov_str* t2 = ov_form_value_45246b5c_af36f5f9(p3, t1);
  int64_t t3 = 0;
  ovt_Option_d5f2c30a* t4 = ((ov_parse_int(t2, &t3)) ? ovt_Option_d5f2c30a_mk0(t3) : ovt_Option_d5f2c30a_mk1());
  int64_t t5 = ov_unwrap_or_cf818e0c_bde9d67c(t4, INT64_C(0));
  int64_t v558 = t5;
  ovt_tup2_b1cce6e4* t6 = ovt_tup2_b1cce6e4_mk(p1, p2);
  ovt_Response_dae56844* t7 = 0; (void)t7;
  bool t8 = false; (void)t8;
  if (!t8) {
    if ((ov_str_eq(t6->f0, ov_str_new("GET", 3)) && ov_str_eq(t6->f1, ov_str_new("/", 1)))) {
      t8 = true;
      ovt_Response_dae56844* t9 = ov_respond_page_4815483f_a5c54b45(p0);
      t7 = t9;
    }
  }
  if (!t8) {
    if ((ov_str_eq(t6->f0, ov_str_new("GET", 3)) && ov_str_eq(t6->f1, ov_str_new("/todos.txt", 10)))) {
      t8 = true;
      ov_str* t10 = ov_str_new("text/plain; charset=utf-8", 25);
      ov_list* t11 = ov_list_todos_24d1401c_1ec84d65(p0);
      ov_str* t12 = ov_render_text_b3499619_840f04f8(t11);
      ovt_Response_dae56844* t13 = ovt_Response_dae56844_mk(INT64_C(200), t10, t12);
      t7 = t13;
    }
  }
  if (!t8) {
    if ((ov_str_eq(t6->f0, ov_str_new("POST", 4)) && ov_str_eq(t6->f1, ov_str_new("/add", 4)))) {
      t8 = true;
      ov_str* t14 = ov_str_new("title", 5);
      ov_str* t15 = ov_form_value_45246b5c_af36f5f9(p3, t14);
      ov_str* t16 = ov_str_trim(t15);
      int64_t t17 = ov_str_len_cps(t16);
      bool t18 = (t17 == INT64_C(0));
      ovt_Response_dae56844* t19 = 0; (void)t19;
      if (t18) {
        ov_str* t20 = ov_str_new("title required", 14);
        ovt_Response_dae56844* t21 = ov_bad_request_569742af_b0b3f10f(t20);
        t19 = t21;
      } else {
        ov_str* t22 = ov_str_new("title", 5);
        ov_str* t23 = ov_form_value_45246b5c_af36f5f9(p3, t22);
        int64_t t24 = ov_add_todo_cbaf52b2_7dd0667a(p0, t23);
        ovt_Response_dae56844* t25 = ov_respond_page_4815483f_a5c54b45(p0);
        t19 = t25;
      }
      t7 = t19;
    }
  }
  if (!t8) {
    if ((ov_str_eq(t6->f0, ov_str_new("POST", 4)) && ov_str_eq(t6->f1, ov_str_new("/toggle", 7)))) {
      t8 = true;
      bool t26 = (v558 > INT64_C(0));
      ovt_Response_dae56844* t27 = 0; (void)t27;
      if (t26) {
        int64_t t28 = ov_toggle_todo_5ad29be9_c45c207c(p0, v558);
        ovt_Response_dae56844* t29 = ov_respond_page_4815483f_a5c54b45(p0);
        t27 = t29;
      } else {
        ov_str* t30 = ov_str_new("bad id", 6);
        ovt_Response_dae56844* t31 = ov_bad_request_569742af_b0b3f10f(t30);
        t27 = t31;
      }
      t7 = t27;
    }
  }
  if (!t8) {
    if ((ov_str_eq(t6->f0, ov_str_new("POST", 4)) && ov_str_eq(t6->f1, ov_str_new("/delete", 7)))) {
      t8 = true;
      bool t32 = (v558 > INT64_C(0));
      ovt_Response_dae56844* t33 = 0; (void)t33;
      if (t32) {
        int64_t t34 = ov_delete_todo_02f14ca2_91f6bf4f(p0, v558);
        ovt_Response_dae56844* t35 = ov_respond_page_4815483f_a5c54b45(p0);
        t33 = t35;
      } else {
        ov_str* t36 = ov_str_new("bad id", 6);
        ovt_Response_dae56844* t37 = ov_bad_request_569742af_b0b3f10f(t36);
        t33 = t37;
      }
      t7 = t33;
    }
  }
  if (!t8) {
    if (true) {
      t8 = true;
      ov_str* t38 = ov_str_new("text/plain", 10);
      ov_str* t39 = ov_str_new("not found\n", 10);
      ovt_Response_dae56844* t40 = ovt_Response_dae56844_mk(INT64_C(404), t38, t39);
      t7 = t40;
    }
  }
  return t7;
}
