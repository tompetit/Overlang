#include "ov_types.h"

bool ov_r_at_f1a24fc5_55a16962(ov_list* p0, int64_t p1, ov_str* p2);

bool ov_r_at_f1a24fc5_55a16962(ov_list* p0, int64_t p1, ov_str* p2) {
  ov_list* t1 = ov_str_chars(p2);
  ov_list* v3274 = t1;
  bool t2 = (p1 >= INT64_C(0));
  bool t3 = t2;
  if (t3) {
    int64_t t4 = v3274->len;
    int64_t t5 = ov_add(p1, t4, "r_at", "f1a24fc564ca68cf343ff8a37ef7273fe2566172f31bb0e8dc4e0028206e7e3d");
    int64_t t6 = p0->len;
    bool t7 = (t5 <= t6);
    t3 = t7;
  }
  bool t8 = t3;
  if (t8) {
    int64_t t9 = v3274->len;
    int64_t t10 = ov_add(p1, t9, "r_at", "f1a24fc564ca68cf343ff8a37ef7273fe2566172f31bb0e8dc4e0028206e7e3d");
    ov_list* t11 = ov_list_slice(p0, p1, t10);
    bool t12 = oveq_List_f1666589(t11, v3274);
    t8 = t12;
  }
  return t8;
}
