#include "ov_types.h"

ov_list* ov_db_rows_2c7fa5b5_89b40a99(int64_t, ov_str*, ov_list*);
ov_list* ov_map_9a617375_994fd81f(ov_list*, ov_closure*);
ov_str* ov_p_int_4640db5b_f21961d6(int64_t);
ovt_Label_da07b2c4* ov_row_label_0b6fd1dc_fa016a2c(ov_list*);
ov_list* ov_list_labels_c35c266b_8bb2dc64(int64_t p0, int64_t p1);

static ovt_Label_da07b2c4* ovl_ov_list_labels_c35c266b_8bb2dc64_1875(void* ovenv, ov_list* l1875_0) {
  ovt_Label_da07b2c4* t5 = ov_row_label_0b6fd1dc_fa016a2c(l1875_0);
  return t5;
}

ov_list* ov_list_labels_c35c266b_8bb2dc64(int64_t p0, int64_t p1) {
  ov_str* t1 = ov_str_new("select id, project_id, name from labels where project_id = \? order by name", 74);
  ov_str* t2 = ov_p_int_4640db5b_f21961d6(p1);
  ov_list* t3 = ov_list_new(1);
  t3->items[0] = (ov_value){.p = t2};
  ov_list* t4 = ov_db_rows_2c7fa5b5_89b40a99(p0, t1, t3);
  ov_closure* t6 = ov_closure_mk((void*)ovl_ov_list_labels_c35c266b_8bb2dc64_1875, NULL);
  ov_list* t7 = ov_map_9a617375_994fd81f(t4, t6);
  return t7;
}
