#include "ov_types.h"

bool ov_r_at_f1a24fc5_55a16962(ov_list*, int64_t, ov_str*);
int64_t ov_r_find_0bdff63b_f02af91e(ov_list*, ov_str*, int64_t);
ov_str* ov_r_cut_1531a99c_866cf162(ov_list*, int64_t, int64_t);
ov_str* ov_r_inline_rest_37e120d2_29f58b62(ov_str*);
ov_str* ov_r_inline_rest_37e120d2_29f58b62(ov_str* p0);

ov_str* ov_r_inline_rest_37e120d2_29f58b62(ov_str* p0) {
  ov_list* t1 = ov_str_chars(p0);
  ov_list* v3339 = t1;
  int64_t t2 = v3339->len;
  int64_t v3343 = t2;
  ov_str* t3 = ov_str_new("", 0);
  ov_str* v3347 = t3;
  (void)v3347;
  int64_t v3349 = INT64_C(0);
  (void)v3349;
  while (1) {
    int64_t t4 = v3349;
    bool t5 = (t4 < v3343);
    if (!t5) break;
    int64_t t7 = v3349;
    ov_str* t8 = ((ov_str*)(ov_list_get(v3339, t7, "r_inline_rest", "37e120d2901fc43fb43872867526af672121fb668564938ad9749a377e6ef993")).p);
    ov_str* v3354 = t8;
    ov_str* t9 = ov_str_new("`", 1);
    bool t10 = ov_str_eq(v3354, t9);
    int64_t t11 = 0; (void)t11;
    if (t10) {
      ov_str* t12 = ov_str_new("`", 1);
      int64_t t13 = v3349;
      int64_t t14 = ov_add(t13, INT64_C(1), "r_inline_rest", "37e120d2901fc43fb43872867526af672121fb668564938ad9749a377e6ef993");
      int64_t t15 = ov_r_find_0bdff63b_f02af91e(v3339, t12, t14);
      t11 = t15;
    } else {
      int64_t t16 = ov_neg(INT64_C(1), "r_inline_rest", "37e120d2901fc43fb43872867526af672121fb668564938ad9749a377e6ef993");
      t11 = t16;
    }
    int64_t v3358 = t11;
    bool t17 = (v3358 < INT64_C(0));
    bool t18 = t17;
    if (t18) {
      int64_t t19 = v3349;
      ov_str* t20 = ov_str_new("**", 2);
      bool t21 = ov_r_at_f1a24fc5_55a16962(v3339, t19, t20);
      t18 = t21;
    }
    int64_t t22 = 0; (void)t22;
    if (t18) {
      ov_str* t23 = ov_str_new("**", 2);
      int64_t t24 = v3349;
      int64_t t25 = ov_add(t24, INT64_C(3), "r_inline_rest", "37e120d2901fc43fb43872867526af672121fb668564938ad9749a377e6ef993");
      int64_t t26 = ov_r_find_0bdff63b_f02af91e(v3339, t23, t25);
      t22 = t26;
    } else {
      int64_t t27 = ov_neg(INT64_C(1), "r_inline_rest", "37e120d2901fc43fb43872867526af672121fb668564938ad9749a377e6ef993");
      t22 = t27;
    }
    int64_t v3373 = t22;
    bool t28 = (v3358 < INT64_C(0));
    bool t29 = t28;
    if (t29) {
      bool t30 = (v3373 < INT64_C(0));
      t29 = t30;
    }
    bool t31 = t29;
    if (t31) {
      ov_str* t32 = ov_str_new("[", 1);
      bool t33 = ov_str_eq(v3354, t32);
      t31 = t33;
    }
    int64_t t34 = 0; (void)t34;
    if (t31) {
      ov_str* t35 = ov_str_new("](", 2);
      int64_t t36 = v3349;
      int64_t t37 = ov_add(t36, INT64_C(1), "r_inline_rest", "37e120d2901fc43fb43872867526af672121fb668564938ad9749a377e6ef993");
      int64_t t38 = ov_r_find_0bdff63b_f02af91e(v3339, t35, t37);
      t34 = t38;
    } else {
      int64_t t39 = ov_neg(INT64_C(1), "r_inline_rest", "37e120d2901fc43fb43872867526af672121fb668564938ad9749a377e6ef993");
      t34 = t39;
    }
    int64_t v3394 = t34;
    bool t40 = (v3394 >= INT64_C(0));
    int64_t t41 = 0; (void)t41;
    if (t40) {
      ov_str* t42 = ov_str_new(")", 1);
      int64_t t43 = ov_add(v3394, INT64_C(2), "r_inline_rest", "37e120d2901fc43fb43872867526af672121fb668564938ad9749a377e6ef993");
      int64_t t44 = ov_r_find_0bdff63b_f02af91e(v3339, t42, t43);
      t41 = t44;
    } else {
      int64_t t45 = ov_neg(INT64_C(1), "r_inline_rest", "37e120d2901fc43fb43872867526af672121fb668564938ad9749a377e6ef993");
      t41 = t45;
    }
    int64_t v3417 = t41;
    bool t46 = (v3417 >= INT64_C(0));
    ov_str* t47 = 0; (void)t47;
    if (t46) {
      int64_t t48 = ov_add(v3394, INT64_C(2), "r_inline_rest", "37e120d2901fc43fb43872867526af672121fb668564938ad9749a377e6ef993");
      ov_str* t49 = ov_r_cut_1531a99c_866cf162(v3339, t48, v3417);
      t47 = t49;
    } else {
      ov_str* t50 = ov_str_new("", 0);
      t47 = t50;
    }
    ov_str* v3432 = t47;
    bool t51 = (v3417 >= INT64_C(0));
    bool t52 = t51;
    if (t52) {
      ov_str* t53 = ov_str_new("http://", 7);
      bool t54 = ov_str_starts_with(v3432, t53);
      bool t55 = t54;
      if (!t55) {
        ov_str* t56 = ov_str_new("https://", 8);
        bool t57 = ov_str_starts_with(v3432, t56);
        t55 = t57;
      }
      bool t58 = t55;
      if (!t58) {
        ov_str* t59 = ov_str_new("/", 1);
        bool t60 = ov_str_starts_with(v3432, t59);
        t58 = t60;
      }
      t52 = t58;
    }
    bool v3446 = t52;
    bool t61 = (v3358 >= INT64_C(0));
    if (t61) {
      ov_str* t62 = v3347;
      ov_str* t63 = ov_str_new("<code>", 6);
      ov_str* t64 = ov_str_concat(t62, t63);
      int64_t t65 = v3349;
      int64_t t66 = ov_add(t65, INT64_C(1), "r_inline_rest", "37e120d2901fc43fb43872867526af672121fb668564938ad9749a377e6ef993");
      ov_str* t67 = ov_r_cut_1531a99c_866cf162(v3339, t66, v3358);
      ov_str* t68 = ov_str_concat(t64, t67);
      ov_str* t69 = ov_str_new("</code>", 7);
      ov_str* t70 = ov_str_concat(t68, t69);
      v3347 = t70;
      int64_t t71 = ov_add(v3358, INT64_C(1), "r_inline_rest", "37e120d2901fc43fb43872867526af672121fb668564938ad9749a377e6ef993");
      v3349 = t71;
    } else {
      bool t72 = (v3373 >= INT64_C(0));
      if (t72) {
        ov_str* t73 = v3347;
        ov_str* t74 = ov_str_new("<strong>", 8);
        ov_str* t75 = ov_str_concat(t73, t74);
        int64_t t76 = v3349;
        int64_t t77 = ov_add(t76, INT64_C(2), "r_inline_rest", "37e120d2901fc43fb43872867526af672121fb668564938ad9749a377e6ef993");
        ov_str* t78 = ov_r_cut_1531a99c_866cf162(v3339, t77, v3373);
        ov_str* t79 = ov_r_inline_rest_37e120d2_29f58b62(t78);
        ov_str* t80 = ov_str_concat(t75, t79);
        ov_str* t81 = ov_str_new("</strong>", 9);
        ov_str* t82 = ov_str_concat(t80, t81);
        v3347 = t82;
        int64_t t83 = ov_add(v3373, INT64_C(2), "r_inline_rest", "37e120d2901fc43fb43872867526af672121fb668564938ad9749a377e6ef993");
        v3349 = t83;
      } else {
        if (v3446) {
          ov_str* t84 = v3347;
          ov_str* t85 = ov_str_new("<a href=\"", 9);
          ov_str* t86 = ov_str_concat(t84, t85);
          ov_str* t87 = ov_str_concat(t86, v3432);
          ov_str* t88 = ov_str_new("\">", 2);
          ov_str* t89 = ov_str_concat(t87, t88);
          int64_t t90 = v3349;
          int64_t t91 = ov_add(t90, INT64_C(1), "r_inline_rest", "37e120d2901fc43fb43872867526af672121fb668564938ad9749a377e6ef993");
          ov_str* t92 = ov_r_cut_1531a99c_866cf162(v3339, t91, v3394);
          ov_str* t93 = ov_str_concat(t89, t92);
          ov_str* t94 = ov_str_new("</a>", 4);
          ov_str* t95 = ov_str_concat(t93, t94);
          v3347 = t95;
          int64_t t96 = ov_add(v3417, INT64_C(1), "r_inline_rest", "37e120d2901fc43fb43872867526af672121fb668564938ad9749a377e6ef993");
          v3349 = t96;
        } else {
          ov_str* t97 = v3347;
          ov_str* t98 = ov_str_concat(t97, v3354);
          v3347 = t98;
          int64_t t99 = v3349;
          int64_t t100 = ov_add(t99, INT64_C(1), "r_inline_rest", "37e120d2901fc43fb43872867526af672121fb668564938ad9749a377e6ef993");
          v3349 = t100;
        }
      }
    }
  }
  ov_str* t101 = v3347;
  return t101;
}
