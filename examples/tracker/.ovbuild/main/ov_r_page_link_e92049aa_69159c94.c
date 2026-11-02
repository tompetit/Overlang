#include "ov_types.h"

ov_str* ov_filter_query_fcb38fa8_c44c6a1c(ovt_Filter_50e09095*, int64_t);
ov_str* ov_html_escape_114cb911_d5a1eb27(ov_str*);
ov_str* ov_r_page_link_e92049aa_69159c94(ovt_Project_73b03799* p0, ovt_Filter_50e09095* p1, int64_t p2, ov_str* p3);

ov_str* ov_r_page_link_e92049aa_69159c94(ovt_Project_73b03799* p0, ovt_Filter_50e09095* p1, int64_t p2, ov_str* p3) {
  ov_str* t1 = ov_str_new(" <a href=\"", 10);
  ov_str* t2 = ov_str_new("/p/", 3);
  ov_str* t3 = p0->f_key;
  ov_str* t4 = ov_str_concat(t2, t3);
  ov_str* t5 = ov_str_new("\?", 1);
  ov_str* t6 = ov_str_concat(t4, t5);
  ov_str* t7 = ov_filter_query_fcb38fa8_c44c6a1c(p1, p2);
  ov_str* t8 = ov_str_concat(t6, t7);
  ov_str* t9 = ov_html_escape_114cb911_d5a1eb27(t8);
  ov_str* t10 = ov_str_concat(t1, t9);
  ov_str* t11 = ov_str_new("\">", 2);
  ov_str* t12 = ov_str_concat(t10, t11);
  ov_str* t13 = ov_str_concat(t12, p3);
  ov_str* t14 = ov_str_new("</a>", 4);
  ov_str* t15 = ov_str_concat(t13, t14);
  return t15;
}
