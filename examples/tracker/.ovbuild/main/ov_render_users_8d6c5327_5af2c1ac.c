#include "ov_types.h"

ov_list* ov_map_9a617375_2918ff5c(ov_list*, ov_closure*);
ov_str* ov_html_escape_114cb911_d5a1eb27(ov_str*);
ov_str* ov_html_page_82184164_f910056c(ov_str*, ov_str*);
ov_str* ov_render_users_8d6c5327_5af2c1ac(ov_list* p0);

static ov_str* ovl_ov_render_users_8d6c5327_5af2c1ac_3739(void* ovenv, ovt_User_9c127ab2* l3739_0) {
  ov_str* t3 = ov_str_new("<li>", 4);
  ov_str* t4 = l3739_0->f_name;
  ov_str* t5 = ov_html_escape_114cb911_d5a1eb27(t4);
  ov_str* t6 = ov_str_concat(t3, t5);
  ov_str* t7 = ov_str_new("</li>", 5);
  ov_str* t8 = ov_str_concat(t6, t7);
  return t8;
}

ov_str* ov_render_users_8d6c5327_5af2c1ac(ov_list* p0) {
  ov_str* t1 = ov_str_new("Users", 5);
  ov_str* t2 = ov_str_new("<h1>Users</h1><ul class=\"users\">", 32);
  ov_closure* t9 = ov_closure_mk((void*)ovl_ov_render_users_8d6c5327_5af2c1ac_3739, NULL);
  ov_list* t10 = ov_map_9a617375_2918ff5c(p0, t9);
  ov_str* t11 = ov_str_new("", 0);
  ov_str* t12 = ov_str_join(t10, t11);
  ov_str* t13 = ov_str_concat(t2, t12);
  ov_str* t14 = ov_str_new("</ul><form method=\"post\" action=\"/users\"><input name=\"name\" placeholder=\"name\"> <button>create user</button></form>", 115);
  ov_str* t15 = ov_str_concat(t13, t14);
  ov_str* t16 = ov_html_page_82184164_f910056c(t1, t15);
  return t16;
}
