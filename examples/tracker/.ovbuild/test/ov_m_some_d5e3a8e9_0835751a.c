#include "ov_types.h"

ovt_Issue_b825796a* ov_m_some_d5e3a8e9_0835751a(ovt_Option_c3749d6a* p0);

ovt_Issue_b825796a* ov_m_some_d5e3a8e9_0835751a(ovt_Option_c3749d6a* p0) {
  ovt_Issue_b825796a* t1 = 0; (void)t1;
  bool t2 = false; (void)t2;
  if (!t2) {
    if ((p0->tag == 0)) {
      ovt_Issue_b825796a* v2350 = p0->u.c0.f0;
      t2 = true;
      t1 = v2350;
    }
  }
  if (!t2) {
    if ((p0->tag == 1)) {
      t2 = true;
      ov_trap("todo", "m_some", "d5e3a8e9eac00b3d5c179a81c995e28b544c1b75942e3351294690050ffba006", "\"not yet implemented\"");
      ovt_Issue_b825796a* t3 = 0;
      t1 = t3;
    }
  }
  return t1;
}
