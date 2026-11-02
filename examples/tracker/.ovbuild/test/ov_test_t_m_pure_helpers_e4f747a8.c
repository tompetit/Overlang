#include "ov_types.h"

int64_t ov_page_count_678cf533_30ac5d54(int64_t);
int64_t ov_page_offset_650fdf0d_bcd456be(int64_t);
ov_str* ov_like_pattern_ec16774e_b5007031(ov_str*);
ovt_Filter_50e09095* ov_no_filter_53ccd73e_859849c3(void);
ovt_tup2_24c1b644* ov_issue_where_8bfaaf6f_1a38c29a(ovt_Filter_50e09095*);
uint8_t ov_test_t_m_pure_helpers_e4f747a8(void);

uint8_t ov_test_t_m_pure_helpers_e4f747a8(void) {
  ov_str* t1 = ov_str_new("50%_a\\b", 7);
  ov_str* t2 = ov_like_pattern_ec16774e_b5007031(t1);
  ov_str* t3 = ov_str_new("%50\\%\\_a\\\\b%", 12);
  if (!ov_str_eq(t2, t3)) ovsup_assert_eq_fail("t_m_pure_helpers", "e4f747a82838b7a30f474a86a11b157e4d4d17f952b404183517859a5c096189", ov_show_str(t2), ov_show_str(t3));
  int64_t t4 = ov_page_count_678cf533_30ac5d54(INT64_C(0));
  if (!(t4 == INT64_C(1))) ovsup_assert_eq_fail("t_m_pure_helpers", "e4f747a82838b7a30f474a86a11b157e4d4d17f952b404183517859a5c096189", ov_show_int(t4), ov_show_int(INT64_C(1)));
  int64_t t5 = ov_page_count_678cf533_30ac5d54(INT64_C(20));
  if (!(t5 == INT64_C(1))) ovsup_assert_eq_fail("t_m_pure_helpers", "e4f747a82838b7a30f474a86a11b157e4d4d17f952b404183517859a5c096189", ov_show_int(t5), ov_show_int(INT64_C(1)));
  int64_t t6 = ov_page_count_678cf533_30ac5d54(INT64_C(21));
  if (!(t6 == INT64_C(2))) ovsup_assert_eq_fail("t_m_pure_helpers", "e4f747a82838b7a30f474a86a11b157e4d4d17f952b404183517859a5c096189", ov_show_int(t6), ov_show_int(INT64_C(2)));
  int64_t t7 = ov_page_offset_650fdf0d_bcd456be(INT64_C(3));
  if (!(t7 == INT64_C(40))) ovsup_assert_eq_fail("t_m_pure_helpers", "e4f747a82838b7a30f474a86a11b157e4d4d17f952b404183517859a5c096189", ov_show_int(t7), ov_show_int(INT64_C(40)));
  ovt_Filter_50e09095* t8 = ov_no_filter_53ccd73e_859849c3();
  ovt_tup2_24c1b644* t9 = ov_issue_where_8bfaaf6f_1a38c29a(t8);
  ov_str* t10 = ov_str_new("", 0);
  ov_list* t11 = ov_list_empty();
  ovt_tup2_24c1b644* t12 = ovt_tup2_24c1b644_mk(t10, t11);
  if (!oveq_tup2_24c1b644(t9, t12)) ovsup_assert_eq_fail("t_m_pure_helpers", "e4f747a82838b7a30f474a86a11b157e4d4d17f952b404183517859a5c096189", ovshow_tup2_24c1b644(t9), ovshow_tup2_24c1b644(t12));
  ovt_Filter_50e09095* t13 = ov_no_filter_53ccd73e_859849c3();
  ovt_Filter_50e09095* t14 = GC_MALLOC(sizeof(ovt_Filter_50e09095));
  *t14 = *t13;
  ov_str* t15 = ov_str_new("open", 4);
  t14->f_status = t15;
  t14->f_priority = INT64_C(2);
  ovt_tup2_24c1b644* t16 = ov_issue_where_8bfaaf6f_1a38c29a(t14);
  ov_str* t17 = ov_str_new(" and i.status = \? and i.priority = \?", 36);
  ov_str* t18 = ov_str_new("sopen", 5);
  ov_str* t19 = ov_str_new("i2", 2);
  ov_list* t20 = ov_list_new(2);
  t20->items[0] = (ov_value){.p = t18};
  t20->items[1] = (ov_value){.p = t19};
  ovt_tup2_24c1b644* t21 = ovt_tup2_24c1b644_mk(t17, t20);
  if (!oveq_tup2_24c1b644(t16, t21)) ovsup_assert_eq_fail("t_m_pure_helpers", "e4f747a82838b7a30f474a86a11b157e4d4d17f952b404183517859a5c096189", ovshow_tup2_24c1b644(t16), ovshow_tup2_24c1b644(t21));
  return 0;
}
