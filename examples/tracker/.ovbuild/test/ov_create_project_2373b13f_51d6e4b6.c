#include "ov_types.h"

bool ov_valid_project_key_0acaa387_69ec39bf(ov_str*);
int64_t ov_db_int_36007c3c_9a9bc110(int64_t, ov_str*, ov_list*);
ov_str* ov_p_str_73f85a82_17f92fe7(ov_str*);
int64_t ov_create_project_2373b13f_51d6e4b6(int64_t p0, ov_str* p1, ov_str* p2);

int64_t ov_create_project_2373b13f_51d6e4b6(int64_t p0, ov_str* p1, ov_str* p2) {
  bool t1 = ov_valid_project_key_0acaa387_69ec39bf(p1);
  bool t2 = t1;
  if (t2) {
    ov_str* t3 = ov_str_trim(p2);
    int64_t t4 = ov_str_len_cps(t3);
    bool t5 = (t4 > INT64_C(0));
    t2 = t5;
  }
  if (!t2) {
    ov_sb* ovsb = ov_sb_new();
    ov_sb_push_cstr(ovsb, "{");
    ov_sb_push_cstr(ovsb, "\"db\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_int(p0)));
    ov_sb_push_cstr(ovsb, ", \"key\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_str(p1)));
    ov_sb_push_cstr(ovsb, ", \"name\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_str(p2)));
    ov_sb_push_cstr(ovsb, "}");
    ov_trap_contract("requires", "create_project", "2373b13f818ff3bc2a35c24741f008aee346016163cc5e8141474ea7e489e701", "valid_project_key(key) && str_len(trim(name)) > 0", ov_sb_finish(ovsb)->data);
  }
  ov_str* t6 = ov_str_new("insert into projects (key, name) values (\?, \?) returning id", 59);
  ov_str* t7 = ov_p_str_73f85a82_17f92fe7(p1);
  ov_str* t8 = ov_p_str_73f85a82_17f92fe7(p2);
  ov_list* t9 = ov_list_new(2);
  t9->items[0] = (ov_value){.p = t7};
  t9->items[1] = (ov_value){.p = t8};
  int64_t t10 = ov_db_int_36007c3c_9a9bc110(p0, t6, t9);
  return t10;
}
