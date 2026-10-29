#include "ov_types.h"

int64_t ov_sqlite_exec_1s_7dfe8d0b_ebf25ed6(int64_t, ov_str*, ov_str*);
int64_t ov_add_todo_cbaf52b2_7dd0667a(int64_t p0, ov_str* p1);

int64_t ov_add_todo_cbaf52b2_7dd0667a(int64_t p0, ov_str* p1) {
  ov_str* t1 = ov_str_trim(p1);
  int64_t t2 = ov_str_len_cps(t1);
  bool t3 = (t2 > INT64_C(0));
  if (!t3) {
    ov_sb* ovsb = ov_sb_new();
    ov_sb_push_cstr(ovsb, "{");
    ov_sb_push_cstr(ovsb, "\"db\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_int(p0)));
    ov_sb_push_cstr(ovsb, ", \"title\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_str(p1)));
    ov_sb_push_cstr(ovsb, "}");
    ov_trap_contract("requires", "add_todo", "", "str_len(trim(title)) > 0", ov_sb_finish(ovsb)->data);
  }
  ov_str* t4 = ov_str_new("insert into todos (title) values (\?)", 36);
  ov_str* t5 = ov_str_trim(p1);
  int64_t t6 = ov_sqlite_exec_1s_7dfe8d0b_ebf25ed6(p0, t4, t5);
  return t6;
}
