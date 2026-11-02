#include "ov_types.h"

ov_list* ov_map_9a617375_a8867022(ov_list* p0, ov_closure* p1);

ov_list* ov_map_9a617375_a8867022(ov_list* p0, ov_closure* p1) {
  ov_list* t1 = ov_list_empty();
  ov_list* v21 = t1;
  (void)v21;
  ov_list* t2 = p0;
  for (int64_t t3 = 0; t3 < t2->len; t3++) {
    ovt_Issue_b825796a* t4 = ((ovt_Issue_b825796a*)(t2->items[t3]).p);
    ovt_Issue_b825796a* v23 = t4;
    ov_list* t6 = v21;
    int64_t t7 = ((int64_t (*)(void*, ovt_Issue_b825796a*))p1->fn)(p1->env, v23);
    ov_list* t8 = ov_list_push(t6, (ov_value){.i = t7});
    v21 = t8;
  }
  ov_list* t9 = v21;
  return t9;
}
