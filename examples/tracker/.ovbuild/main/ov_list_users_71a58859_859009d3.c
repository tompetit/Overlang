#include "ov_types.h"

ov_list* ov_db_rows_2c7fa5b5_89b40a99(int64_t, ov_str*, ov_list*);
ov_list* ov_map_9a617375_d27dbd02(ov_list*, ov_closure*);
ovt_User_9c127ab2* ov_row_user_e48587ac_cecb4b39(ov_list*);
ov_list* ov_list_users_71a58859_859009d3(int64_t p0);

static ovt_User_9c127ab2* ovl_ov_list_users_71a58859_859009d3_1786(void* ovenv, ov_list* l1786_0) {
  ovt_User_9c127ab2* t4 = ov_row_user_e48587ac_cecb4b39(l1786_0);
  return t4;
}

ov_list* ov_list_users_71a58859_859009d3(int64_t p0) {
  ov_str* t1 = ov_str_new("select id, name from users order by id", 38);
  ov_list* t2 = ov_list_empty();
  ov_list* t3 = ov_db_rows_2c7fa5b5_89b40a99(p0, t1, t2);
  ov_closure* t5 = ov_closure_mk((void*)ovl_ov_list_users_71a58859_859009d3_1786, NULL);
  ov_list* t6 = ov_map_9a617375_d27dbd02(t3, t5);
  return t6;
}
