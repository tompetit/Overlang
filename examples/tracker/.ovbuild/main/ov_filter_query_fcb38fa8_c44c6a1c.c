#include "ov_types.h"

ov_list* ov_filter_021377ec_dc32a908(ov_list*, ov_closure*);
ov_list* ov_map_9a617375_92d8df96(ov_list*, ov_closure*);
ov_str* ov_url_encode_9b07eb27_7c143021(ov_str*);
ov_str* ov_filter_query_fcb38fa8_c44c6a1c(ovt_Filter_50e09095* p0, int64_t p1);

static bool ovl_ov_filter_query_fcb38fa8_c44c6a1c_3664(void* ovenv, ovt_tup2_b1cce6e4* l3664_0) {
  ov_str* t22 = l3664_0->f1;
  ov_str* t23 = ov_str_new("", 0);
  bool t24 = (!ov_str_eq(t22, t23));
  return t24;
}

static ov_str* ovl_ov_filter_query_fcb38fa8_c44c6a1c_3679(void* ovenv, ovt_tup2_b1cce6e4* l3679_0) {
  ov_str* t27 = l3679_0->f0;
  ov_str* t28 = ov_str_new("=", 1);
  ov_str* t29 = ov_str_concat(t27, t28);
  ov_str* t30 = l3679_0->f1;
  ov_str* t31 = ov_url_encode_9b07eb27_7c143021(t30);
  ov_str* t32 = ov_str_concat(t29, t31);
  return t32;
}

ov_str* ov_filter_query_fcb38fa8_c44c6a1c(ovt_Filter_50e09095* p0, int64_t p1) {
  ov_str* t1 = ov_str_new("status", 6);
  ov_str* t2 = p0->f_status;
  ovt_tup2_b1cce6e4* t3 = ovt_tup2_b1cce6e4_mk(t1, t2);
  ov_str* t4 = ov_str_new("assignee", 8);
  ov_str* t5 = p0->f_assignee;
  ovt_tup2_b1cce6e4* t6 = ovt_tup2_b1cce6e4_mk(t4, t5);
  ov_str* t7 = ov_str_new("label", 5);
  ov_str* t8 = p0->f_label;
  ovt_tup2_b1cce6e4* t9 = ovt_tup2_b1cce6e4_mk(t7, t8);
  ov_str* t10 = ov_str_new("priority", 8);
  int64_t t11 = p0->f_priority;
  bool t12 = (t11 > INT64_C(0));
  ov_str* t13 = 0; (void)t13;
  if (t12) {
    int64_t t14 = p0->f_priority;
    ov_str* t15 = ov_show_int(t14);
    t13 = t15;
  } else {
    ov_str* t16 = ov_str_new("", 0);
    t13 = t16;
  }
  ovt_tup2_b1cce6e4* t17 = ovt_tup2_b1cce6e4_mk(t10, t13);
  ov_str* t18 = ov_str_new("q", 1);
  ov_str* t19 = p0->f_q;
  ovt_tup2_b1cce6e4* t20 = ovt_tup2_b1cce6e4_mk(t18, t19);
  ov_list* t21 = ov_list_new(5);
  t21->items[0] = (ov_value){.p = t3};
  t21->items[1] = (ov_value){.p = t6};
  t21->items[2] = (ov_value){.p = t9};
  t21->items[3] = (ov_value){.p = t17};
  t21->items[4] = (ov_value){.p = t20};
  ov_list* v3626 = t21;
  ov_closure* t25 = ov_closure_mk((void*)ovl_ov_filter_query_fcb38fa8_c44c6a1c_3664, NULL);
  ov_list* t26 = ov_filter_021377ec_dc32a908(v3626, t25);
  ov_list* v3657 = t26;
  ov_closure* t33 = ov_closure_mk((void*)ovl_ov_filter_query_fcb38fa8_c44c6a1c_3679, NULL);
  ov_list* t34 = ov_map_9a617375_92d8df96(v3657, t33);
  ov_str* t35 = ov_str_new("page=", 5);
  ov_str* t36 = ov_show_int(p1);
  ov_str* t37 = ov_str_concat(t35, t36);
  ov_list* t38 = ov_list_push(t34, (ov_value){.p = t37});
  ov_str* t39 = ov_str_new("&", 1);
  ov_str* t40 = ov_str_join(t38, t39);
  return t40;
}
