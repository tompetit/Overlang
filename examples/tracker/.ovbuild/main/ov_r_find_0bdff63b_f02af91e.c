#include "ov_types.h"

bool ov_r_at_f1a24fc5_55a16962(ov_list*, int64_t, ov_str*);
int64_t ov_max_c0a98029_5b0fcca4(int64_t, int64_t);
int64_t ov_r_find_0bdff63b_f02af91e(ov_list* p0, ov_str* p1, int64_t p2);

int64_t ov_r_find_0bdff63b_f02af91e(ov_list* p0, ov_str* p1, int64_t p2) {
  int64_t t1 = ov_max_c0a98029_5b0fcca4(p2, INT64_C(0));
  int64_t v3303 = t1;
  (void)v3303;
  int64_t t2 = ov_neg(INT64_C(1), "r_find", "0bdff63b21a720dc6db651d7931b5281d359d93f697a9306f4ea22ce47796bcf");
  int64_t v3308 = t2;
  (void)v3308;
  while (1) {
    int64_t t3 = v3308;
    bool t4 = (t3 < INT64_C(0));
    bool t5 = t4;
    if (t5) {
      int64_t t6 = v3303;
      int64_t t7 = p0->len;
      bool t8 = (t6 < t7);
      t5 = t8;
    }
    if (!t5) break;
    int64_t t10 = v3303;
    bool t11 = ov_r_at_f1a24fc5_55a16962(p0, t10, p1);
    if (t11) {
      int64_t t12 = v3303;
      v3308 = t12;
    }
    int64_t t13 = v3303;
    int64_t t14 = ov_add(t13, INT64_C(1), "r_find", "0bdff63b21a720dc6db651d7931b5281d359d93f697a9306f4ea22ce47796bcf");
    v3303 = t14;
  }
  int64_t t15 = v3308;
  return t15;
}
