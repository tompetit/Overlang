#include "ov_types.h"

bool ov_valid_user_name_eb496c11_934af82b(ov_str*);
int64_t ov_db_int_36007c3c_9a9bc110(int64_t, ov_str*, ov_list*);
ov_str* ov_p_str_73f85a82_17f92fe7(ov_str*);
int64_t ov_create_user_f316dd39_339cb8a1(int64_t p0, ov_str* p1);

int64_t ov_create_user_f316dd39_339cb8a1(int64_t p0, ov_str* p1) {
  bool t1 = ov_valid_user_name_eb496c11_934af82b(p1);
  if (!t1) {
    ov_sb* ovsb = ov_sb_new();
    ov_sb_push_cstr(ovsb, "{");
    ov_sb_push_cstr(ovsb, "\"db\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_int(p0)));
    ov_sb_push_cstr(ovsb, ", \"name\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_str(p1)));
    ov_sb_push_cstr(ovsb, "}");
    ov_trap_contract("requires", "create_user", "f316dd39bb9f37330164984a12442626bc2faad1a51cc5e2304873436293bae7", "valid_user_name(name)", ov_sb_finish(ovsb)->data);
  }
  ov_str* t2 = ov_str_new("insert into users (name) values (\?) returning id", 48);
  ov_str* t3 = ov_p_str_73f85a82_17f92fe7(p1);
  ov_list* t4 = ov_list_new(1);
  t4->items[0] = (ov_value){.p = t3};
  int64_t t5 = ov_db_int_36007c3c_9a9bc110(p0, t2, t4);
  return t5;
}
