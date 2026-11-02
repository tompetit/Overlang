#include "ov_types.h"

ovt_Label_da07b2c4* ov_m_some_d5e3a8e9_54d21d74(ovt_Option_c278faef* p0);

ovt_Label_da07b2c4* ov_m_some_d5e3a8e9_54d21d74(ovt_Option_c278faef* p0) {
  ovt_Label_da07b2c4* t1 = 0; (void)t1;
  bool t2 = false; (void)t2;
  if (!t2) {
    if ((p0->tag == 0)) {
      ovt_Label_da07b2c4* v2350 = p0->u.c0.f0;
      t2 = true;
      t1 = v2350;
    }
  }
  if (!t2) {
    if ((p0->tag == 1)) {
      t2 = true;
      ov_trap("todo", "m_some", "d5e3a8e9eac00b3d5c179a81c995e28b544c1b75942e3351294690050ffba006", "\"not yet implemented\"");
      ovt_Label_da07b2c4* t3 = 0;
      t1 = t3;
    }
  }
  return t1;
}
