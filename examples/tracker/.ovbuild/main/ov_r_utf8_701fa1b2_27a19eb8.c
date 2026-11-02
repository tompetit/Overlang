#include "ov_types.h"

ov_list* ov_r_utf8_701fa1b2_27a19eb8(int64_t p0);

ov_list* ov_r_utf8_701fa1b2_27a19eb8(int64_t p0) {
  bool t1 = (p0 < INT64_C(128));
  ov_list* t2 = 0; (void)t2;
  if (t1) {
    ov_list* t3 = ov_list_new(1);
    t3->items[0] = (ov_value){.i = p0};
    t2 = t3;
  } else {
    bool t4 = (p0 < INT64_C(2048));
    ov_list* t5 = 0; (void)t5;
    if (t4) {
      int64_t t6 = ov_div(p0, INT64_C(64), "r_utf8", "701fa1b2647ae9fa9e22d5df99f649ee1538f71ab72f99eb94125e6507a29b1e");
      int64_t t7 = ov_add(INT64_C(192), t6, "r_utf8", "701fa1b2647ae9fa9e22d5df99f649ee1538f71ab72f99eb94125e6507a29b1e");
      int64_t t8 = ov_mod(p0, INT64_C(64), "r_utf8", "701fa1b2647ae9fa9e22d5df99f649ee1538f71ab72f99eb94125e6507a29b1e");
      int64_t t9 = ov_add(INT64_C(128), t8, "r_utf8", "701fa1b2647ae9fa9e22d5df99f649ee1538f71ab72f99eb94125e6507a29b1e");
      ov_list* t10 = ov_list_new(2);
      t10->items[0] = (ov_value){.i = t7};
      t10->items[1] = (ov_value){.i = t9};
      t5 = t10;
    } else {
      bool t11 = (p0 < INT64_C(65536));
      ov_list* t12 = 0; (void)t12;
      if (t11) {
        int64_t t13 = ov_div(p0, INT64_C(4096), "r_utf8", "701fa1b2647ae9fa9e22d5df99f649ee1538f71ab72f99eb94125e6507a29b1e");
        int64_t t14 = ov_add(INT64_C(224), t13, "r_utf8", "701fa1b2647ae9fa9e22d5df99f649ee1538f71ab72f99eb94125e6507a29b1e");
        int64_t t15 = ov_div(p0, INT64_C(64), "r_utf8", "701fa1b2647ae9fa9e22d5df99f649ee1538f71ab72f99eb94125e6507a29b1e");
        int64_t t16 = ov_mod(t15, INT64_C(64), "r_utf8", "701fa1b2647ae9fa9e22d5df99f649ee1538f71ab72f99eb94125e6507a29b1e");
        int64_t t17 = ov_add(INT64_C(128), t16, "r_utf8", "701fa1b2647ae9fa9e22d5df99f649ee1538f71ab72f99eb94125e6507a29b1e");
        int64_t t18 = ov_mod(p0, INT64_C(64), "r_utf8", "701fa1b2647ae9fa9e22d5df99f649ee1538f71ab72f99eb94125e6507a29b1e");
        int64_t t19 = ov_add(INT64_C(128), t18, "r_utf8", "701fa1b2647ae9fa9e22d5df99f649ee1538f71ab72f99eb94125e6507a29b1e");
        ov_list* t20 = ov_list_new(3);
        t20->items[0] = (ov_value){.i = t14};
        t20->items[1] = (ov_value){.i = t17};
        t20->items[2] = (ov_value){.i = t19};
        t12 = t20;
      } else {
        int64_t t21 = ov_div(p0, INT64_C(262144), "r_utf8", "701fa1b2647ae9fa9e22d5df99f649ee1538f71ab72f99eb94125e6507a29b1e");
        int64_t t22 = ov_add(INT64_C(240), t21, "r_utf8", "701fa1b2647ae9fa9e22d5df99f649ee1538f71ab72f99eb94125e6507a29b1e");
        int64_t t23 = ov_div(p0, INT64_C(4096), "r_utf8", "701fa1b2647ae9fa9e22d5df99f649ee1538f71ab72f99eb94125e6507a29b1e");
        int64_t t24 = ov_mod(t23, INT64_C(64), "r_utf8", "701fa1b2647ae9fa9e22d5df99f649ee1538f71ab72f99eb94125e6507a29b1e");
        int64_t t25 = ov_add(INT64_C(128), t24, "r_utf8", "701fa1b2647ae9fa9e22d5df99f649ee1538f71ab72f99eb94125e6507a29b1e");
        int64_t t26 = ov_div(p0, INT64_C(64), "r_utf8", "701fa1b2647ae9fa9e22d5df99f649ee1538f71ab72f99eb94125e6507a29b1e");
        int64_t t27 = ov_mod(t26, INT64_C(64), "r_utf8", "701fa1b2647ae9fa9e22d5df99f649ee1538f71ab72f99eb94125e6507a29b1e");
        int64_t t28 = ov_add(INT64_C(128), t27, "r_utf8", "701fa1b2647ae9fa9e22d5df99f649ee1538f71ab72f99eb94125e6507a29b1e");
        int64_t t29 = ov_mod(p0, INT64_C(64), "r_utf8", "701fa1b2647ae9fa9e22d5df99f649ee1538f71ab72f99eb94125e6507a29b1e");
        int64_t t30 = ov_add(INT64_C(128), t29, "r_utf8", "701fa1b2647ae9fa9e22d5df99f649ee1538f71ab72f99eb94125e6507a29b1e");
        ov_list* t31 = ov_list_new(4);
        t31->items[0] = (ov_value){.i = t22};
        t31->items[1] = (ov_value){.i = t25};
        t31->items[2] = (ov_value){.i = t28};
        t31->items[3] = (ov_value){.i = t30};
        t12 = t31;
      }
      t5 = t12;
    }
    t2 = t5;
  }
  return t2;
}
