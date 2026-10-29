#include "ov_types.h"

bool ov_all_e6f09b19_e9cb4669(ov_list* p0, ov_closure* p1);

bool ov_all_e6f09b19_e9cb4669(ov_list* p0, ov_closure* p1) {
  ov_list* t1 = p0;
  for (int64_t t2 = 0; t2 < t1->len; t2++) {
    ovt_Todo_6462003c* t3 = ((ovt_Todo_6462003c*)(t1->items[t2]).p);
    ovt_Todo_6462003c* v67 = t3;
    bool t4 = ((bool (*)(void*, ovt_Todo_6462003c*))p1->fn)(p1->env, v67);
    bool t5 = (!t4);
    if (t5) {
      return false;
    }
  }
  return true;
}
