#include "ov_types.h"

ovt_Option_da3cc0d5* ov_m_first_fb63178c_5c96a8df(ov_list* p0, ov_closure* p1);

ovt_Option_da3cc0d5* ov_m_first_fb63178c_5c96a8df(ov_list* p0, ov_closure* p1) {
  ovt_Option_da3cc0d5* t1 = 0; (void)t1;
  bool t2 = false; (void)t2;
  if (!t2) {
    if ((p0->len >= 1)) {
      ov_list* v1658 = ((ov_list*)(p0->items[0]).p);
      t2 = true;
      ovt_Project_73b03799* t3 = ((ovt_Project_73b03799* (*)(void*, ov_list*))p1->fn)(p1->env, v1658);
      ovt_Option_da3cc0d5* t4 = ovt_Option_da3cc0d5_mk0(t3);
      t1 = t4;
    }
  }
  if (!t2) {
    if (true) {
      t2 = true;
      ovt_Option_da3cc0d5* t5 = ovt_Option_da3cc0d5_mk1();
      t1 = t5;
    }
  }
  return t1;
}
