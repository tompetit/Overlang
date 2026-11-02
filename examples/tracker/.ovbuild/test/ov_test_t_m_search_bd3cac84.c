#include "ov_types.h"

bool ov_set_status_7fbb65d5_46ec26ca(int64_t, ovt_Issue_b825796a*, ov_str*);
int64_t ov_count_issues_3f95e492_f3049723(int64_t, int64_t, ovt_Filter_50e09095*);
int64_t ov_create_issue_c9275d15_8de03dd4(int64_t, int64_t, ov_str*, ov_str*, int64_t, int64_t, ov_list*);
int64_t ov_db_open_1ac4b90f_bff7d5eb(ov_str*);
ov_list* ov_all_issues_103a8b74_2317e431(int64_t, int64_t);
ov_list* ov_map_9a617375_a8867022(ov_list*, ov_closure*);
ov_list* ov_range_b02d7864_3ad63243(int64_t, int64_t);
ovt_Filter_50e09095* ov_no_filter_53ccd73e_859849c3(void);
ovt_IssuePage_114bf070* ov_search_issues_573fd6dd_7e780e04(int64_t, int64_t, ovt_Filter_50e09095*);
ovt_Issue_b825796a* ov_m_some_d5e3a8e9_0835751a(ovt_Option_c3749d6a*);
ovt_Option_c3749d6a* ov_find_issue_68dd4779_d24e6f3d(int64_t, int64_t, int64_t);
uint8_t ov_m_fixture_6ba66877_59f8675e(int64_t);
uint8_t ov_test_t_m_search_bd3cac84(void);

static int64_t ovl_ov_test_t_m_search_bd3cac84_2798(void* ovenv, ovt_Issue_b825796a* l2798_0) {
  int64_t t67 = l2798_0->f_number;
  return t67;
}

uint8_t ov_test_t_m_search_bd3cac84(void) {
  ov_str* t1 = ov_str_new(":memory:", 8);
  int64_t t2 = ov_db_open_1ac4b90f_bff7d5eb(t1);
  int64_t v2629 = t2;
  (void)(ov_m_fixture_6ba66877_59f8675e(v2629));
  ov_list* t3 = ov_range_b02d7864_3ad63243(INT64_C(1), INT64_C(26));
  ov_list* t4 = t3;
  for (int64_t t5 = 0; t5 < t4->len; t5++) {
    int64_t t6 = (t4->items[t5]).i;
    int64_t v2636 = t6;
    bool t8 = (v2636 <= INT64_C(5));
    ov_list* t9 = 0; (void)t9;
    if (t8) {
      ov_list* t10 = ov_list_new(1);
      t10->items[0] = (ov_value){.i = INT64_C(1)};
      t9 = t10;
    } else {
      ov_list* t11 = ov_list_empty();
      t9 = t11;
    }
    ov_list* v2641 = t9;
    bool t12 = (v2636 == INT64_C(7));
    ov_str* t13 = 0; (void)t13;
    if (t12) {
      ov_str* t14 = ov_str_new("50% done", 8);
      t13 = t14;
    } else {
      ov_str* t15 = ov_str_new("", 0);
      t13 = t15;
    }
    ov_str* v2650 = t13;
    ov_str* t16 = ov_str_new("issue ", 6);
    ov_str* t17 = ov_show_int(v2636);
    ov_str* t18 = ov_str_concat(t16, t17);
    int64_t t19 = ov_mod(v2636, INT64_C(4), "t_m_search", "bd3cac849e8a9a9f0d7166ebd364c87a7361d292530a589c7e27574c018b2d7e");
    int64_t t20 = ov_add(INT64_C(1), t19, "t_m_search", "bd3cac849e8a9a9f0d7166ebd364c87a7361d292530a589c7e27574c018b2d7e");
    int64_t t21 = ov_create_issue_c9275d15_8de03dd4(v2629, INT64_C(1), t18, v2650, t20, INT64_C(0), v2641);
  }
  ov_list* t22 = ov_list_new(3);
  t22->items[0] = (ov_value){.i = INT64_C(1)};
  t22->items[1] = (ov_value){.i = INT64_C(2)};
  t22->items[2] = (ov_value){.i = INT64_C(3)};
  ov_list* t23 = t22;
  for (int64_t t24 = 0; t24 < t23->len; t24++) {
    int64_t t25 = (t23->items[t24]).i;
    int64_t v2675 = t25;
    ovt_Option_c3749d6a* t27 = ov_find_issue_68dd4779_d24e6f3d(v2629, INT64_C(1), v2675);
    ovt_Issue_b825796a* t28 = ov_m_some_d5e3a8e9_0835751a(t27);
    ov_str* t29 = ov_str_new("closed", 6);
    bool t30 = ov_set_status_7fbb65d5_46ec26ca(v2629, t28, t29);
    if (!t30) ov_trap("assert", "t_m_search", "bd3cac849e8a9a9f0d7166ebd364c87a7361d292530a589c7e27574c018b2d7e", "\"assertion failed: assert(set_status(db, m_some(find_issue(db, 1, n)), \\\"closed\\\"))\"");
  }
  ovt_Filter_50e09095* t31 = ov_no_filter_53ccd73e_859849c3();
  ovt_IssuePage_114bf070* t32 = ov_search_issues_573fd6dd_7e780e04(v2629, INT64_C(1), t31);
  ovt_IssuePage_114bf070* v2693 = t32;
  ov_list* t33 = v2693->f_issues;
  int64_t t34 = t33->len;
  if (!(t34 == INT64_C(20))) ovsup_assert_eq_fail("t_m_search", "bd3cac849e8a9a9f0d7166ebd364c87a7361d292530a589c7e27574c018b2d7e", ov_show_int(t34), ov_show_int(INT64_C(20)));
  int64_t t35 = v2693->f_total;
  if (!(t35 == INT64_C(25))) ovsup_assert_eq_fail("t_m_search", "bd3cac849e8a9a9f0d7166ebd364c87a7361d292530a589c7e27574c018b2d7e", ov_show_int(t35), ov_show_int(INT64_C(25)));
  int64_t t36 = v2693->f_pages;
  if (!(t36 == INT64_C(2))) ovsup_assert_eq_fail("t_m_search", "bd3cac849e8a9a9f0d7166ebd364c87a7361d292530a589c7e27574c018b2d7e", ov_show_int(t36), ov_show_int(INT64_C(2)));
  ov_list* t37 = v2693->f_issues;
  ovt_Issue_b825796a* t38 = ((ovt_Issue_b825796a*)(ov_list_get(t37, INT64_C(0), "t_m_search", "bd3cac849e8a9a9f0d7166ebd364c87a7361d292530a589c7e27574c018b2d7e")).p);
  int64_t t39 = t38->f_number;
  if (!(t39 == INT64_C(1))) ovsup_assert_eq_fail("t_m_search", "bd3cac849e8a9a9f0d7166ebd364c87a7361d292530a589c7e27574c018b2d7e", ov_show_int(t39), ov_show_int(INT64_C(1)));
  ovt_Filter_50e09095* t40 = ov_no_filter_53ccd73e_859849c3();
  ovt_Filter_50e09095* t41 = GC_MALLOC(sizeof(ovt_Filter_50e09095));
  *t41 = *t40;
  t41->f_page = INT64_C(2);
  ovt_IssuePage_114bf070* t42 = ov_search_issues_573fd6dd_7e780e04(v2629, INT64_C(1), t41);
  ovt_IssuePage_114bf070* v2725 = t42;
  ov_list* t43 = v2725->f_issues;
  int64_t t44 = t43->len;
  if (!(t44 == INT64_C(5))) ovsup_assert_eq_fail("t_m_search", "bd3cac849e8a9a9f0d7166ebd364c87a7361d292530a589c7e27574c018b2d7e", ov_show_int(t44), ov_show_int(INT64_C(5)));
  ov_list* t45 = v2725->f_issues;
  ovt_Issue_b825796a* t46 = ((ovt_Issue_b825796a*)(ov_list_get(t45, INT64_C(0), "t_m_search", "bd3cac849e8a9a9f0d7166ebd364c87a7361d292530a589c7e27574c018b2d7e")).p);
  int64_t t47 = t46->f_number;
  if (!(t47 == INT64_C(21))) ovsup_assert_eq_fail("t_m_search", "bd3cac849e8a9a9f0d7166ebd364c87a7361d292530a589c7e27574c018b2d7e", ov_show_int(t47), ov_show_int(INT64_C(21)));
  ovt_Filter_50e09095* t48 = ov_no_filter_53ccd73e_859849c3();
  ovt_Filter_50e09095* t49 = GC_MALLOC(sizeof(ovt_Filter_50e09095));
  *t49 = *t48;
  ov_str* t50 = ov_str_new("closed", 6);
  t49->f_status = t50;
  ovt_IssuePage_114bf070* t51 = ov_search_issues_573fd6dd_7e780e04(v2629, INT64_C(1), t49);
  int64_t t52 = t51->f_total;
  if (!(t52 == INT64_C(3))) ovsup_assert_eq_fail("t_m_search", "bd3cac849e8a9a9f0d7166ebd364c87a7361d292530a589c7e27574c018b2d7e", ov_show_int(t52), ov_show_int(INT64_C(3)));
  ovt_Filter_50e09095* t53 = ov_no_filter_53ccd73e_859849c3();
  ovt_Filter_50e09095* t54 = GC_MALLOC(sizeof(ovt_Filter_50e09095));
  *t54 = *t53;
  ov_str* t55 = ov_str_new("bug", 3);
  t54->f_label = t55;
  ovt_IssuePage_114bf070* t56 = ov_search_issues_573fd6dd_7e780e04(v2629, INT64_C(1), t54);
  int64_t t57 = t56->f_total;
  if (!(t57 == INT64_C(5))) ovsup_assert_eq_fail("t_m_search", "bd3cac849e8a9a9f0d7166ebd364c87a7361d292530a589c7e27574c018b2d7e", ov_show_int(t57), ov_show_int(INT64_C(5)));
  ovt_Filter_50e09095* t58 = ov_no_filter_53ccd73e_859849c3();
  ovt_Filter_50e09095* t59 = GC_MALLOC(sizeof(ovt_Filter_50e09095));
  *t59 = *t58;
  t59->f_priority = INT64_C(1);
  ovt_IssuePage_114bf070* t60 = ov_search_issues_573fd6dd_7e780e04(v2629, INT64_C(1), t59);
  int64_t t61 = t60->f_total;
  if (!(t61 == INT64_C(6))) ovsup_assert_eq_fail("t_m_search", "bd3cac849e8a9a9f0d7166ebd364c87a7361d292530a589c7e27574c018b2d7e", ov_show_int(t61), ov_show_int(INT64_C(6)));
  ovt_Filter_50e09095* t62 = ov_no_filter_53ccd73e_859849c3();
  ovt_Filter_50e09095* t63 = GC_MALLOC(sizeof(ovt_Filter_50e09095));
  *t63 = *t62;
  ov_str* t64 = ov_str_new("%", 1);
  t63->f_q = t64;
  ovt_IssuePage_114bf070* t65 = ov_search_issues_573fd6dd_7e780e04(v2629, INT64_C(1), t63);
  ov_list* t66 = t65->f_issues;
  ov_closure* t68 = ov_closure_mk((void*)ovl_ov_test_t_m_search_bd3cac84_2798, NULL);
  ov_list* t69 = ov_map_9a617375_a8867022(t66, t68);
  ov_list* t70 = ov_list_new(1);
  t70->items[0] = (ov_value){.i = INT64_C(7)};
  if (!oveq_List_d31dced1(t69, t70)) ovsup_assert_eq_fail("t_m_search", "bd3cac849e8a9a9f0d7166ebd364c87a7361d292530a589c7e27574c018b2d7e", ovshow_List_d31dced1(t69), ovshow_List_d31dced1(t70));
  ovt_Filter_50e09095* t71 = ov_no_filter_53ccd73e_859849c3();
  ovt_Filter_50e09095* t72 = GC_MALLOC(sizeof(ovt_Filter_50e09095));
  *t72 = *t71;
  ov_str* t73 = ov_str_new("ISSUE 2", 7);
  t72->f_q = t73;
  ovt_IssuePage_114bf070* t74 = ov_search_issues_573fd6dd_7e780e04(v2629, INT64_C(1), t72);
  int64_t t75 = t74->f_total;
  if (!(t75 == INT64_C(7))) ovsup_assert_eq_fail("t_m_search", "bd3cac849e8a9a9f0d7166ebd364c87a7361d292530a589c7e27574c018b2d7e", ov_show_int(t75), ov_show_int(INT64_C(7)));
  ovt_Filter_50e09095* t76 = ov_no_filter_53ccd73e_859849c3();
  ovt_Filter_50e09095* t77 = GC_MALLOC(sizeof(ovt_Filter_50e09095));
  *t77 = *t76;
  ov_str* t78 = ov_str_new("alice", 5);
  t77->f_assignee = t78;
  ovt_IssuePage_114bf070* t79 = ov_search_issues_573fd6dd_7e780e04(v2629, INT64_C(1), t77);
  int64_t t80 = t79->f_total;
  if (!(t80 == INT64_C(0))) ovsup_assert_eq_fail("t_m_search", "bd3cac849e8a9a9f0d7166ebd364c87a7361d292530a589c7e27574c018b2d7e", ov_show_int(t80), ov_show_int(INT64_C(0)));
  ovt_Filter_50e09095* t81 = ov_no_filter_53ccd73e_859849c3();
  ovt_Filter_50e09095* t82 = GC_MALLOC(sizeof(ovt_Filter_50e09095));
  *t82 = *t81;
  ov_str* t83 = ov_str_new("bug", 3);
  t82->f_label = t83;
  ov_str* t84 = ov_str_new("open", 4);
  t82->f_status = t84;
  ovt_Filter_50e09095* v2827 = t82;
  int64_t t85 = ov_count_issues_3f95e492_f3049723(v2629, INT64_C(1), v2827);
  ovt_IssuePage_114bf070* t86 = ov_search_issues_573fd6dd_7e780e04(v2629, INT64_C(1), v2827);
  int64_t t87 = t86->f_total;
  if (!(t85 == t87)) ovsup_assert_eq_fail("t_m_search", "bd3cac849e8a9a9f0d7166ebd364c87a7361d292530a589c7e27574c018b2d7e", ov_show_int(t85), ov_show_int(t87));
  ovt_Filter_50e09095* t88 = ov_no_filter_53ccd73e_859849c3();
  int64_t t89 = ov_count_issues_3f95e492_f3049723(v2629, INT64_C(1), t88);
  if (!(t89 == INT64_C(25))) ovsup_assert_eq_fail("t_m_search", "bd3cac849e8a9a9f0d7166ebd364c87a7361d292530a589c7e27574c018b2d7e", ov_show_int(t89), ov_show_int(INT64_C(25)));
  ov_list* t90 = ov_all_issues_103a8b74_2317e431(v2629, INT64_C(1));
  int64_t t91 = t90->len;
  if (!(t91 == INT64_C(25))) ovsup_assert_eq_fail("t_m_search", "bd3cac849e8a9a9f0d7166ebd364c87a7361d292530a589c7e27574c018b2d7e", ov_show_int(t91), ov_show_int(INT64_C(25)));
  return 0;
}
