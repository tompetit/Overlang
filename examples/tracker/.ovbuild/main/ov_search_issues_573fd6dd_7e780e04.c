#include "ov_types.h"

int64_t ov_count_issues_3f95e492_f3049723(int64_t, int64_t, ovt_Filter_50e09095*);
int64_t ov_max_c0a98029_5b0fcca4(int64_t, int64_t);
int64_t ov_page_count_678cf533_30ac5d54(int64_t);
int64_t ov_page_offset_650fdf0d_bcd456be(int64_t);
ov_list* ov_db_rows_2c7fa5b5_89b40a99(int64_t, ov_str*, ov_list*);
ov_list* ov_map_9a617375_2aeb273a(ov_list*, ov_closure*);
ov_str* ov_issue_select_sql_91164f3f_1b0791dc(void);
ov_str* ov_p_int_4640db5b_f21961d6(int64_t);
ovt_Issue_b825796a* ov_row_issue_9a6bac26_e115135c(ov_list*);
ovt_tup2_24c1b644* ov_issue_where_8bfaaf6f_1a38c29a(ovt_Filter_50e09095*);
ovt_IssuePage_114bf070* ov_search_issues_573fd6dd_7e780e04(int64_t p0, int64_t p1, ovt_Filter_50e09095* p2);

static ovt_Issue_b825796a* ovl_ov_search_issues_573fd6dd_7e780e04_2080(void* ovenv, ov_list* l2080_0) {
  ovt_Issue_b825796a* t26 = ov_row_issue_9a6bac26_e115135c(l2080_0);
  return t26;
}

ovt_IssuePage_114bf070* ov_search_issues_573fd6dd_7e780e04(int64_t p0, int64_t p1, ovt_Filter_50e09095* p2) {
  int64_t t1 = p2->f_page;
  bool t2 = (t1 >= INT64_C(1));
  bool t3 = t2;
  if (t3) {
    int64_t t4 = p2->f_page;
    bool t5 = (t4 <= INT64_C(1000000));
    t3 = t5;
  }
  if (!t3) {
    ov_sb* ovsb = ov_sb_new();
    ov_sb_push_cstr(ovsb, "{");
    ov_sb_push_cstr(ovsb, "\"db\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_int(p0)));
    ov_sb_push_cstr(ovsb, ", \"project_id\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_int(p1)));
    ov_sb_push_cstr(ovsb, ", \"f\": ");
    ov_sb_push(ovsb, ovsup_json_str(ovshow_Filter_50e09095(p2)));
    ov_sb_push_cstr(ovsb, "}");
    ov_trap_contract("requires", "search_issues", "573fd6dd20cef705dd845dcfc486b28bec5d50ecb8ba881e363d2bec04e6a0a6", "f.page >= 1 && f.page <= 1000000", ov_sb_finish(ovsb)->data);
  }
  ovt_tup2_24c1b644* t6 = ov_issue_where_8bfaaf6f_1a38c29a(p2);
  ovt_tup2_24c1b644* v2032 = t6;
  ov_str* t7 = ov_issue_select_sql_91164f3f_1b0791dc();
  ov_str* t8 = ov_str_new(" where i.project_id = \?", 23);
  ov_str* t9 = ov_str_concat(t7, t8);
  ov_str* t10 = v2032->f0;
  ov_str* t11 = ov_str_concat(t9, t10);
  ov_str* t12 = ov_str_new(" order by i.number limit 20 offset \?", 36);
  ov_str* t13 = ov_str_concat(t11, t12);
  ov_str* v2036 = t13;
  ov_str* t14 = ov_p_int_4640db5b_f21961d6(p1);
  ov_list* t15 = ov_list_new(1);
  t15->items[0] = (ov_value){.p = t14};
  ov_list* t16 = v2032->f1;
  ov_list* t17 = ov_list_concat(t15, t16);
  int64_t t18 = p2->f_page;
  int64_t t19 = ov_page_offset_650fdf0d_bcd456be(t18);
  ov_str* t20 = ov_p_int_4640db5b_f21961d6(t19);
  ov_list* t21 = ov_list_new(1);
  t21->items[0] = (ov_value){.p = t20};
  ov_list* t22 = ov_list_concat(t17, t21);
  ov_list* t23 = ov_db_rows_2c7fa5b5_89b40a99(p0, v2036, t22);
  ov_list* v2046 = t23;
  int64_t t24 = ov_count_issues_3f95e492_f3049723(p0, p1, p2);
  int64_t t25 = ov_max_c0a98029_5b0fcca4(INT64_C(0), t24);
  int64_t v2066 = t25;
  ov_closure* t27 = ov_closure_mk((void*)ovl_ov_search_issues_573fd6dd_7e780e04_2080, NULL);
  ov_list* t28 = ov_map_9a617375_2aeb273a(v2046, t27);
  int64_t t29 = p2->f_page;
  int64_t t30 = ov_page_count_678cf533_30ac5d54(v2066);
  ovt_IssuePage_114bf070* t31 = ovt_IssuePage_114bf070_mk(t28, v2066, t29, t30);
  return t31;
}
