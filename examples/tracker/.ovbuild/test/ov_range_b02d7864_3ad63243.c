#include "ov_types.h"

int64_t ov_max_c0a98029_5b0fcca4(int64_t, int64_t);
ov_list* ov_range_b02d7864_3ad63243(int64_t p0, int64_t p1);

ov_list* ov_range_b02d7864_3ad63243(int64_t p0, int64_t p1) {
  ov_list* ovres = 0;
  ov_list* t1 = ov_list_empty();
  ov_list* v118 = t1;
  (void)v118;
  int64_t v120 = p0;
  (void)v120;
  while (1) {
    int64_t t2 = v120;
    bool t3 = (t2 < p1);
    if (!t3) break;
    ov_list* t5 = v118;
    int64_t t6 = v120;
    ov_list* t7 = ov_list_push(t5, (ov_value){.i = t6});
    v118 = t7;
    int64_t t8 = v120;
    int64_t t9 = ov_add(t8, INT64_C(1), "range", "b02d78640e3bebc9a588c845656409f628008a7607a145917eaf5f5bafd005bc");
    v120 = t9;
  }
  ov_list* t10 = v118;
  ovres = t10;
  int64_t t11 = ovres->len;
  int64_t t12 = ov_sub(p1, p0, "range", "b02d78640e3bebc9a588c845656409f628008a7607a145917eaf5f5bafd005bc");
  int64_t t13 = ov_max_c0a98029_5b0fcca4(INT64_C(0), t12);
  bool t14 = (t11 == t13);
  if (!t14) {
    ov_sb* ovsb = ov_sb_new();
    ov_sb_push_cstr(ovsb, "{");
    ov_sb_push_cstr(ovsb, "\"from\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_int(p0)));
    ov_sb_push_cstr(ovsb, ", \"to\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_int(p1)));
    ov_sb_push_cstr(ovsb, ", \"result\": ");
    ov_sb_push(ovsb, ovsup_json_str(ovshow_List_d31dced1(ovres)));
    ov_sb_push_cstr(ovsb, "}");
    ov_trap_contract("ensures", "range", "b02d78640e3bebc9a588c845656409f628008a7607a145917eaf5f5bafd005bc", "len(result) == max(0, to - from)", ov_sb_finish(ovsb)->data);
  }
  return ovres;
}
