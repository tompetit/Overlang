#include "ov_types.h"

bool ov_all_ba8a189d_8d783bfe(ov_list* p0, ov_closure* p1);

bool ov_all_ba8a189d_8d783bfe(ov_list* p0, ov_closure* p1) {
  ov_list* t1 = p0;
  for (int64_t t2 = 0; t2 < t1->len; t2++) {
    ov_str* t3 = ((ov_str*)(t1->items[t2]).p);
    ov_str* v67 = t3;
    bool t5 = ((bool (*)(void*, ov_str*))p1->fn)(p1->env, v67);
    bool t6 = (!t5);
    if (t6) {
      return false;
    }
  }
  return true;
}
