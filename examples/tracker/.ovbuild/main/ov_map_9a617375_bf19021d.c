#include "ov_types.h"

ov_list* ov_map_9a617375_bf19021d(ov_list* p0, ov_closure* p1);

ov_list* ov_map_9a617375_bf19021d(ov_list* p0, ov_closure* p1) {
  ov_list* t1 = ov_list_empty();
  ov_list* v21 = t1;
  (void)v21;
  ov_list* t2 = p0;
  for (int64_t t3 = 0; t3 < t2->len; t3++) {
    ovt_Label_da07b2c4* t4 = ((ovt_Label_da07b2c4*)(t2->items[t3]).p);
    ovt_Label_da07b2c4* v23 = t4;
    ov_list* t6 = v21;
    ov_str* t7 = ((ov_str* (*)(void*, ovt_Label_da07b2c4*))p1->fn)(p1->env, v23);
    ov_list* t8 = ov_list_push(t6, (ov_value){.p = t7});
    v21 = t8;
  }
  ov_list* t9 = v21;
  return t9;
}
