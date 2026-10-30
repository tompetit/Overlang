#include "ov_types.h"

ov_list* ov_map_080f5919_1b7fb3ea(ov_list*, ov_closure*);
ov_str* ov_render_todo_e954a119_536c220b(ovt_Todo_6462003c*);
ov_str* ov_render_page_aafa113d_06878f9d(ov_list* p0);

static ov_str* ovl_ov_render_page_aafa113d_06878f9d_512(void* ovenv, ovt_Todo_6462003c* l512_0) {
  ov_str* t6 = ov_render_todo_e954a119_536c220b(l512_0);
  return t6;
}

ov_str* ov_render_page_aafa113d_06878f9d(ov_list* p0) {
  ov_str* t1 = ov_str_new("<!doctype html><html><head><meta charset=\"utf-8\"><title>Todos</title></head><body><h1>Todos</h1>", 96);
  ov_str* t2 = ov_str_new("<form method=\"post\" action=\"/add\"><input name=\"title\" placeholder=\"What needs doing\?\">", 86);
  ov_str* t3 = ov_str_concat(t1, t2);
  ov_str* t4 = ov_str_new("<button>add</button></form><ul>", 31);
  ov_str* t5 = ov_str_concat(t3, t4);
  ov_closure* t7 = ov_closure_mk((void*)ovl_ov_render_page_aafa113d_06878f9d_512, NULL);
  ov_list* t8 = ov_map_080f5919_1b7fb3ea(p0, t7);
  ov_str* t9 = ov_str_new("", 0);
  ov_str* t10 = ov_str_join(t8, t9);
  ov_str* t11 = ov_str_concat(t5, t10);
  ov_str* t12 = ov_str_new("</ul></body></html>", 19);
  ov_str* t13 = ov_str_concat(t11, t12);
  return t13;
}
