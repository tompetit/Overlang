#include "ov_types.h"

ov_list* ov_filter_021377ec_06b1c4d5(ov_list* p0, ov_closure* p1);

ov_list* ov_filter_021377ec_06b1c4d5(ov_list* p0, ov_closure* p1) {
  ov_list* t1 = ov_list_empty();
  ov_list* v32 = t1;
  (void)v32;
  ov_list* t2 = p0;
  for (int64_t t3 = 0; t3 < t2->len; t3++) {
    ov_str* t4 = ((ov_str*)(t2->items[t3]).p);
    ov_str* v34 = t4;
    bool t6 = ((bool (*)(void*, ov_str*))p1->fn)(p1->env, v34);
    if (t6) {
      ov_list* t7 = v32;
      ov_list* t8 = ov_list_push(t7, (ov_value){.p = v34});
      v32 = t8;
    }
  }
  ov_list* t9 = v32;
  return t9;
}
