#include "ov_types.h"

ov_str* ov_r_page_link_e92049aa_69159c94(ovt_Project_73b03799*, ovt_Filter_50e09095*, int64_t, ov_str*);
ov_str* ov_render_pager_7afb21ff_1ae49da9(ovt_Project_73b03799* p0, ovt_Filter_50e09095* p1, ovt_IssuePage_114bf070* p2);

ov_str* ov_render_pager_7afb21ff_1ae49da9(ovt_Project_73b03799* p0, ovt_Filter_50e09095* p1, ovt_IssuePage_114bf070* p2) {
  ov_str* t1 = ov_str_new("<p class=\"pager\">Page ", 22);
  int64_t t2 = p2->f_page;
  ov_str* t3 = ov_show_int(t2);
  ov_str* t4 = ov_str_concat(t1, t3);
  ov_str* t5 = ov_str_new(" of ", 4);
  ov_str* t6 = ov_str_concat(t4, t5);
  int64_t t7 = p2->f_pages;
  ov_str* t8 = ov_show_int(t7);
  ov_str* t9 = ov_str_concat(t6, t8);
  ov_str* t10 = ov_str_new(" (", 2);
  ov_str* t11 = ov_str_concat(t9, t10);
  int64_t t12 = p2->f_total;
  ov_str* t13 = ov_show_int(t12);
  ov_str* t14 = ov_str_concat(t11, t13);
  ov_str* t15 = ov_str_new(" issues)", 8);
  ov_str* t16 = ov_str_concat(t14, t15);
  int64_t t17 = p2->f_page;
  bool t18 = (t17 > INT64_C(1));
  ov_str* t19 = 0; (void)t19;
  if (t18) {
    int64_t t20 = p2->f_page;
    int64_t t21 = ov_sub(t20, INT64_C(1), "render_pager", "7afb21ff6aee9583f0ef17e0e5288b85eef34d91ede07fe8a4861bb79b61e784");
    ov_str* t22 = ov_str_new("prev", 4);
    ov_str* t23 = ov_r_page_link_e92049aa_69159c94(p0, p1, t21, t22);
    t19 = t23;
  } else {
    ov_str* t24 = ov_str_new("", 0);
    t19 = t24;
  }
  ov_str* t25 = ov_str_concat(t16, t19);
  int64_t t26 = p2->f_page;
  int64_t t27 = p2->f_pages;
  bool t28 = (t26 < t27);
  ov_str* t29 = 0; (void)t29;
  if (t28) {
    int64_t t30 = p2->f_page;
    int64_t t31 = ov_add(t30, INT64_C(1), "render_pager", "7afb21ff6aee9583f0ef17e0e5288b85eef34d91ede07fe8a4861bb79b61e784");
    ov_str* t32 = ov_str_new("next", 4);
    ov_str* t33 = ov_r_page_link_e92049aa_69159c94(p0, p1, t31, t32);
    t29 = t33;
  } else {
    ov_str* t34 = ov_str_new("", 0);
    t29 = t34;
  }
  ov_str* t35 = ov_str_concat(t25, t29);
  ov_str* t36 = ov_str_new("</p>", 4);
  ov_str* t37 = ov_str_concat(t35, t36);
  return t37;
}
