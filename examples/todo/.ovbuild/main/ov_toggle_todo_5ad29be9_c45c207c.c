#include "ov_types.h"

int64_t ov_sqlite_exec_1i_7dfea70b_cafb8cd6(int64_t, ov_str*, int64_t);
int64_t ov_toggle_todo_5ad29be9_c45c207c(int64_t p0, int64_t p1);

int64_t ov_toggle_todo_5ad29be9_c45c207c(int64_t p0, int64_t p1) {
  bool t1 = (p1 > INT64_C(0));
  if (!t1) {
    ov_sb* ovsb = ov_sb_new();
    ov_sb_push_cstr(ovsb, "{");
    ov_sb_push_cstr(ovsb, "\"db\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_int(p0)));
    ov_sb_push_cstr(ovsb, ", \"id\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_int(p1)));
    ov_sb_push_cstr(ovsb, "}");
    ov_trap_contract("requires", "toggle_todo", "", "id > 0", ov_sb_finish(ovsb)->data);
  }
  ov_str* t2 = ov_str_new("update todos set done = 1 - done where id = \?", 45);
  int64_t t3 = ov_sqlite_exec_1i_7dfea70b_cafb8cd6(p0, t2, p1);
  return t3;
}
