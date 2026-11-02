#include "ov_types.h"

int64_t ov_unwrap_or_e7341bb9_bde9d67c(ovt_Option_d5f2c30a*, int64_t);
ov_list* ov_db_rows_2c7fa5b5_89b40a99(int64_t, ov_str*, ov_list*);
int64_t ov_db_int_36007c3c_9a9bc110(int64_t p0, ov_str* p1, ov_list* p2);

int64_t ov_db_int_36007c3c_9a9bc110(int64_t p0, ov_str* p1, ov_list* p2) {
  ov_list* t1 = ov_db_rows_2c7fa5b5_89b40a99(p0, p1, p2);
  int64_t t2 = 0; (void)t2;
  bool t3 = false; (void)t3;
  if (!t3) {
    if ((t1->len >= 1 && (((ov_list*)(t1->items[0]).p)->len >= 1))) {
      ov_str* v1114 = ((ov_str*)(((ov_list*)(t1->items[0]).p)->items[0]).p);
      t3 = true;
      int64_t t4 = 0;
      ovt_Option_d5f2c30a* t5 = ((ov_parse_int(v1114, &t4)) ? ovt_Option_d5f2c30a_mk0(t4) : ovt_Option_d5f2c30a_mk1());
      int64_t t6 = ov_neg(INT64_C(1), "db_int", "36007c3c8ef73773f6219e8f41b6710a1bab966a2c3b0555dce92350f3f5eefb");
      int64_t t7 = ov_unwrap_or_e7341bb9_bde9d67c(t5, t6);
      t2 = t7;
    }
  }
  if (!t3) {
    if (true) {
      t3 = true;
      int64_t t8 = ov_neg(INT64_C(1), "db_int", "36007c3c8ef73773f6219e8f41b6710a1bab966a2c3b0555dce92350f3f5eefb");
      t2 = t8;
    }
  }
  return t2;
}
