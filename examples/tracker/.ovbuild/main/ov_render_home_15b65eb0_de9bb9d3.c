#include "ov_types.h"

ov_list* ov_map_9a617375_6bc6a23d(ov_list*, ov_closure*);
ov_str* ov_html_escape_114cb911_d5a1eb27(ov_str*);
ov_str* ov_html_page_82184164_f910056c(ov_str*, ov_str*);
ov_str* ov_render_home_15b65eb0_de9bb9d3(ov_list* p0);

static ov_str* ovl_ov_render_home_15b65eb0_de9bb9d3_3717(void* ovenv, ovt_Project_73b03799* l3717_0) {
  ov_str* t3 = ov_str_new("<li><a href=\"/p/", 16);
  ov_str* t4 = l3717_0->f_key;
  ov_str* t5 = ov_html_escape_114cb911_d5a1eb27(t4);
  ov_str* t6 = ov_str_concat(t3, t5);
  ov_str* t7 = ov_str_new("\">", 2);
  ov_str* t8 = ov_str_concat(t6, t7);
  ov_str* t9 = l3717_0->f_key;
  ov_str* t10 = ov_html_escape_114cb911_d5a1eb27(t9);
  ov_str* t11 = ov_str_concat(t8, t10);
  ov_str* t12 = ov_str_new("</a> ", 5);
  ov_str* t13 = ov_str_concat(t11, t12);
  ov_str* t14 = l3717_0->f_name;
  ov_str* t15 = ov_html_escape_114cb911_d5a1eb27(t14);
  ov_str* t16 = ov_str_concat(t13, t15);
  ov_str* t17 = ov_str_new("</li>", 5);
  ov_str* t18 = ov_str_concat(t16, t17);
  return t18;
}

ov_str* ov_render_home_15b65eb0_de9bb9d3(ov_list* p0) {
  ov_str* t1 = ov_str_new("Tracker", 7);
  ov_str* t2 = ov_str_new("<h1>Projects</h1><ul class=\"projects\">", 38);
  ov_closure* t19 = ov_closure_mk((void*)ovl_ov_render_home_15b65eb0_de9bb9d3_3717, NULL);
  ov_list* t20 = ov_map_9a617375_6bc6a23d(p0, t19);
  ov_str* t21 = ov_str_new("", 0);
  ov_str* t22 = ov_str_join(t20, t21);
  ov_str* t23 = ov_str_concat(t2, t22);
  ov_str* t24 = ov_str_new("</ul><form method=\"post\" action=\"/projects\"><input name=\"key\" placeholder=\"KEY\"> <input name=\"name\" placeholder=\"name\"> <button>create project</button></form>", 158);
  ov_str* t25 = ov_str_concat(t23, t24);
  ov_str* t26 = ov_html_page_82184164_f910056c(t1, t25);
  return t26;
}
