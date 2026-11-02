#include "ov_types.h"

ovt_Option_c278faef* ov_m_first_fb63178c_4d2fb34d(ov_list* p0, ov_closure* p1);

ovt_Option_c278faef* ov_m_first_fb63178c_4d2fb34d(ov_list* p0, ov_closure* p1) {
  ovt_Option_c278faef* t1 = 0; (void)t1;
  bool t2 = false; (void)t2;
  if (!t2) {
    if ((p0->len >= 1)) {
      ov_list* v1658 = ((ov_list*)(p0->items[0]).p);
      t2 = true;
      ovt_Label_da07b2c4* t3 = ((ovt_Label_da07b2c4* (*)(void*, ov_list*))p1->fn)(p1->env, v1658);
      ovt_Option_c278faef* t4 = ovt_Option_c278faef_mk0(t3);
      t1 = t4;
    }
  }
  if (!t2) {
    if (true) {
      t2 = true;
      ovt_Option_c278faef* t5 = ovt_Option_c278faef_mk1();
      t1 = t5;
    }
  }
  return t1;
}
