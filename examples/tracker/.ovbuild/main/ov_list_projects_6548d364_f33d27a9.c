#include "ov_types.h"

ov_list* ov_db_rows_2c7fa5b5_89b40a99(int64_t, ov_str*, ov_list*);
ov_list* ov_map_9a617375_74a341c7(ov_list*, ov_closure*);
ovt_Project_73b03799* ov_row_project_ca4a0485_7b4b99ad(ov_list*);
ov_list* ov_list_projects_6548d364_f33d27a9(int64_t p0);

static ovt_Project_73b03799* ovl_ov_list_projects_6548d364_f33d27a9_1833(void* ovenv, ov_list* l1833_0) {
  ovt_Project_73b03799* t4 = ov_row_project_ca4a0485_7b4b99ad(l1833_0);
  return t4;
}

ov_list* ov_list_projects_6548d364_f33d27a9(int64_t p0) {
  ov_str* t1 = ov_str_new("select id, key, name from projects order by key", 47);
  ov_list* t2 = ov_list_empty();
  ov_list* t3 = ov_db_rows_2c7fa5b5_89b40a99(p0, t1, t2);
  ov_closure* t5 = ov_closure_mk((void*)ovl_ov_list_projects_6548d364_f33d27a9_1833, NULL);
  ov_list* t6 = ov_map_9a617375_74a341c7(t3, t5);
  return t6;
}
