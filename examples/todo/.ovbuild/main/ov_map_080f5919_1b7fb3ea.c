#include "ov_types.h"

ov_list* ov_map_080f5919_1b7fb3ea(ov_list* p0, ov_closure* p1);

ov_list* ov_map_080f5919_1b7fb3ea(ov_list* p0, ov_closure* p1) {
  ov_list* t1 = ov_list_empty();
  ov_list* v21 = t1;
  (void)v21;
  ov_list* t2 = p0;
  for (int64_t t3 = 0; t3 < t2->len; t3++) {
    ovt_Todo_6462003c* t4 = ((ovt_Todo_6462003c*)(t2->items[t3]).p);
    ovt_Todo_6462003c* v23 = t4;
    ov_list* t5 = v21;
    ov_str* t6 = ((ov_str* (*)(void*, ovt_Todo_6462003c*))p1->fn)(p1->env, v23);
    ov_list* t7 = ov_list_push(t5, (ov_value){.p = t6});
    v21 = t7;
  }
  ov_list* t8 = v21;
  return t8;
}
