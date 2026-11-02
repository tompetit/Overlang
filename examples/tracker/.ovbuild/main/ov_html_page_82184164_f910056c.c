#include "ov_types.h"

ov_str* ov_html_escape_114cb911_d5a1eb27(ov_str*);
ov_str* ov_html_page_82184164_f910056c(ov_str* p0, ov_str* p1);

ov_str* ov_html_page_82184164_f910056c(ov_str* p0, ov_str* p1) {
  ov_str* t1 = ov_str_new("<!doctype html><html><head><meta charset=\"utf-8\"><title>", 56);
  ov_str* t2 = ov_html_escape_114cb911_d5a1eb27(p0);
  ov_str* t3 = ov_str_concat(t1, t2);
  ov_str* t4 = ov_str_new("</title></head><body><nav><a href=\"/\">Tracker</a> <a href=\"/users\">Users</a></nav>", 82);
  ov_str* t5 = ov_str_concat(t3, t4);
  ov_str* t6 = ov_str_concat(t5, p1);
  ov_str* t7 = ov_str_new("</body></html>", 14);
  ov_str* t8 = ov_str_concat(t6, t7);
  return t8;
}
