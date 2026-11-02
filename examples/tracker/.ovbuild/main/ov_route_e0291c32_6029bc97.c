#include "ov_types.h"

int64_t ov_issue_ref_3f7d9f9a_ccc011e4(ov_str*, ov_str*);
ov_list* ov_path_segments_dbe7e6ce_5890a3e7(ov_str*);
ovt_Response_dae56844* ov_h_activity_text_d94ba9ba_f3661b48(int64_t, ov_str*, int64_t);
ovt_Response_dae56844* ov_h_add_comment_d6f21b12_aad5dc72(int64_t, ov_str*, int64_t, ov_str*);
ovt_Response_dae56844* ov_h_add_label_f4cdf2a1_5a132cdd(int64_t, ov_str*, int64_t, ov_str*);
ovt_Response_dae56844* ov_h_assign_ec816036_c8b031b0(int64_t, ov_str*, int64_t, ov_str*);
ovt_Response_dae56844* ov_h_comments_text_ade12ed6_5b2777fb(int64_t, ov_str*, int64_t);
ovt_Response_dae56844* ov_h_create_issue_f1044a28_8cd69ed1(int64_t, ov_str*, ov_str*);
ovt_Response_dae56844* ov_h_create_label_2c50ed71_19fb2a59(int64_t, ov_str*, ov_str*);
ovt_Response_dae56844* ov_h_create_project_c89b4e75_6e4af73a(int64_t, ov_str*);
ovt_Response_dae56844* ov_h_create_user_cee59f87_20244805(int64_t, ov_str*);
ovt_Response_dae56844* ov_h_edit_issue_58bc3aca_46ced106(int64_t, ov_str*, int64_t, ov_str*);
ovt_Response_dae56844* ov_h_health_49c56747_0a13e3ef(void);
ovt_Response_dae56844* ov_h_home_3c806b73_b6da5e50(int64_t);
ovt_Response_dae56844* ov_h_issue_json_1ca34239_9a537815(int64_t, ov_str*, int64_t);
ovt_Response_dae56844* ov_h_issue_page_d85f4494_98419fd6(int64_t, ov_str*, int64_t);
ovt_Response_dae56844* ov_h_issue_text_d7b2d354_587a2631(int64_t, ov_str*, int64_t);
ovt_Response_dae56844* ov_h_issues_csv_f73ab561_a85af68f(int64_t, ov_str*);
ovt_Response_dae56844* ov_h_issues_text_835bb9e7_b4f940c2(int64_t, ov_str*, ov_str*);
ovt_Response_dae56844* ov_h_labels_text_22273db7_34801495(int64_t, ov_str*);
ovt_Response_dae56844* ov_h_project_page_67a36ce3_7d08a0ab(int64_t, ov_str*, ov_str*);
ovt_Response_dae56844* ov_h_projects_text_5f52daa8_83354531(int64_t);
ovt_Response_dae56844* ov_h_remove_label_cf75d164_267cd1f5(int64_t, ov_str*, int64_t, ov_str*);
ovt_Response_dae56844* ov_h_set_status_1e1a3ecd_347ed678(int64_t, ov_str*, int64_t, ov_str*);
ovt_Response_dae56844* ov_h_users_page_4d7202e0_54da708a(int64_t);
ovt_Response_dae56844* ov_h_users_text_e96da07a_0a65f636(int64_t);
ovt_Response_dae56844* ov_resp_error_b47b9fc9_7be2787d(int64_t, ov_str*);
ovt_Response_dae56844* ov_route_e0291c32_6029bc97(int64_t p0, ovt_Request_da024db6* p1);

ovt_Response_dae56844* ov_route_e0291c32_6029bc97(int64_t p0, ovt_Request_da024db6* p1) {
  ov_str* t1 = p1->f_body;
  ov_str* v5936 = t1;
  ov_str* t2 = p1->f_query;
  ov_str* v5939 = t2;
  ov_str* t3 = p1->f_method;
  ov_str* t4 = p1->f_path;
  ov_list* t5 = ov_path_segments_dbe7e6ce_5890a3e7(t4);
  ovt_tup2_24c1b644* t6 = ovt_tup2_24c1b644_mk(t3, t5);
  ovt_Response_dae56844* t7 = 0; (void)t7;
  bool t8 = false; (void)t8;
  if (!t8) {
    if ((ov_str_eq(t6->f0, ov_str_new("GET", 3)) && (t6->f1->len == 0))) {
      t8 = true;
      ovt_Response_dae56844* t9 = ov_h_home_3c806b73_b6da5e50(p0);
      t7 = t9;
    }
  }
  if (!t8) {
    if ((ov_str_eq(t6->f0, ov_str_new("GET", 3)) && (t6->f1->len == 1 && ov_str_eq(((ov_str*)(t6->f1->items[0]).p), ov_str_new("health", 6))))) {
      t8 = true;
      ovt_Response_dae56844* t10 = ov_h_health_49c56747_0a13e3ef();
      t7 = t10;
    }
  }
  if (!t8) {
    if ((ov_str_eq(t6->f0, ov_str_new("GET", 3)) && (t6->f1->len == 1 && ov_str_eq(((ov_str*)(t6->f1->items[0]).p), ov_str_new("users", 5))))) {
      t8 = true;
      ovt_Response_dae56844* t11 = ov_h_users_page_4d7202e0_54da708a(p0);
      t7 = t11;
    }
  }
  if (!t8) {
    if ((ov_str_eq(t6->f0, ov_str_new("GET", 3)) && (t6->f1->len == 1 && ov_str_eq(((ov_str*)(t6->f1->items[0]).p), ov_str_new("users.txt", 9))))) {
      t8 = true;
      ovt_Response_dae56844* t12 = ov_h_users_text_e96da07a_0a65f636(p0);
      t7 = t12;
    }
  }
  if (!t8) {
    if ((ov_str_eq(t6->f0, ov_str_new("POST", 4)) && (t6->f1->len == 1 && ov_str_eq(((ov_str*)(t6->f1->items[0]).p), ov_str_new("users", 5))))) {
      t8 = true;
      ovt_Response_dae56844* t13 = ov_h_create_user_cee59f87_20244805(p0, v5936);
      t7 = t13;
    }
  }
  if (!t8) {
    if ((ov_str_eq(t6->f0, ov_str_new("GET", 3)) && (t6->f1->len == 1 && ov_str_eq(((ov_str*)(t6->f1->items[0]).p), ov_str_new("projects.txt", 12))))) {
      t8 = true;
      ovt_Response_dae56844* t14 = ov_h_projects_text_5f52daa8_83354531(p0);
      t7 = t14;
    }
  }
  if (!t8) {
    if ((ov_str_eq(t6->f0, ov_str_new("POST", 4)) && (t6->f1->len == 1 && ov_str_eq(((ov_str*)(t6->f1->items[0]).p), ov_str_new("projects", 8))))) {
      t8 = true;
      ovt_Response_dae56844* t15 = ov_h_create_project_c89b4e75_6e4af73a(p0, v5936);
      t7 = t15;
    }
  }
  if (!t8) {
    if ((ov_str_eq(t6->f0, ov_str_new("GET", 3)) && (t6->f1->len == 2 && ov_str_eq(((ov_str*)(t6->f1->items[0]).p), ov_str_new("p", 1))))) {
      ov_str* v6000 = ((ov_str*)(t6->f1->items[1]).p);
      t8 = true;
      ovt_Response_dae56844* t16 = ov_h_project_page_67a36ce3_7d08a0ab(p0, v6000, v5939);
      t7 = t16;
    }
  }
  if (!t8) {
    if ((ov_str_eq(t6->f0, ov_str_new("GET", 3)) && (t6->f1->len == 3 && ov_str_eq(((ov_str*)(t6->f1->items[0]).p), ov_str_new("p", 1)) && ov_str_eq(((ov_str*)(t6->f1->items[2]).p), ov_str_new("issues.txt", 10))))) {
      ov_str* v6010 = ((ov_str*)(t6->f1->items[1]).p);
      t8 = true;
      ovt_Response_dae56844* t17 = ov_h_issues_text_835bb9e7_b4f940c2(p0, v6010, v5939);
      t7 = t17;
    }
  }
  if (!t8) {
    if ((ov_str_eq(t6->f0, ov_str_new("GET", 3)) && (t6->f1->len == 3 && ov_str_eq(((ov_str*)(t6->f1->items[0]).p), ov_str_new("p", 1)) && ov_str_eq(((ov_str*)(t6->f1->items[2]).p), ov_str_new("issues.csv", 10))))) {
      ov_str* v6021 = ((ov_str*)(t6->f1->items[1]).p);
      t8 = true;
      ovt_Response_dae56844* t18 = ov_h_issues_csv_f73ab561_a85af68f(p0, v6021);
      t7 = t18;
    }
  }
  if (!t8) {
    if ((ov_str_eq(t6->f0, ov_str_new("POST", 4)) && (t6->f1->len == 3 && ov_str_eq(((ov_str*)(t6->f1->items[0]).p), ov_str_new("p", 1)) && ov_str_eq(((ov_str*)(t6->f1->items[2]).p), ov_str_new("issues", 6))))) {
      ov_str* v6031 = ((ov_str*)(t6->f1->items[1]).p);
      t8 = true;
      ovt_Response_dae56844* t19 = ov_h_create_issue_f1044a28_8cd69ed1(p0, v6031, v5936);
      t7 = t19;
    }
  }
  if (!t8) {
    if ((ov_str_eq(t6->f0, ov_str_new("GET", 3)) && (t6->f1->len == 3 && ov_str_eq(((ov_str*)(t6->f1->items[0]).p), ov_str_new("p", 1)) && ov_str_eq(((ov_str*)(t6->f1->items[2]).p), ov_str_new("labels.txt", 10))))) {
      ov_str* v6042 = ((ov_str*)(t6->f1->items[1]).p);
      t8 = true;
      ovt_Response_dae56844* t20 = ov_h_labels_text_22273db7_34801495(p0, v6042);
      t7 = t20;
    }
  }
  if (!t8) {
    if ((ov_str_eq(t6->f0, ov_str_new("POST", 4)) && (t6->f1->len == 3 && ov_str_eq(((ov_str*)(t6->f1->items[0]).p), ov_str_new("p", 1)) && ov_str_eq(((ov_str*)(t6->f1->items[2]).p), ov_str_new("labels", 6))))) {
      ov_str* v6052 = ((ov_str*)(t6->f1->items[1]).p);
      t8 = true;
      ovt_Response_dae56844* t21 = ov_h_create_label_2c50ed71_19fb2a59(p0, v6052, v5936);
      t7 = t21;
    }
  }
  if (!t8) {
    if ((ov_str_eq(t6->f0, ov_str_new("GET", 3)) && (t6->f1->len == 3 && ov_str_eq(((ov_str*)(t6->f1->items[0]).p), ov_str_new("p", 1))))) {
      ov_str* v6063 = ((ov_str*)(t6->f1->items[1]).p);
      ov_str* v6064 = ((ov_str*)(t6->f1->items[2]).p);
      ov_str* t22 = ov_str_new(".json", 5);
      int64_t t23 = ov_issue_ref_3f7d9f9a_ccc011e4(v6064, t22);
      bool t24 = (t23 > INT64_C(0));
      if (t24) {
        t8 = true;
        ov_str* t25 = ov_str_new(".json", 5);
        int64_t t26 = ov_issue_ref_3f7d9f9a_ccc011e4(v6064, t25);
        ovt_Response_dae56844* t27 = ov_h_issue_json_1ca34239_9a537815(p0, v6063, t26);
        t7 = t27;
      }
    }
  }
  if (!t8) {
    if ((ov_str_eq(t6->f0, ov_str_new("GET", 3)) && (t6->f1->len == 3 && ov_str_eq(((ov_str*)(t6->f1->items[0]).p), ov_str_new("p", 1))))) {
      ov_str* v6083 = ((ov_str*)(t6->f1->items[1]).p);
      ov_str* v6084 = ((ov_str*)(t6->f1->items[2]).p);
      ov_str* t28 = ov_str_new(".txt", 4);
      int64_t t29 = ov_issue_ref_3f7d9f9a_ccc011e4(v6084, t28);
      bool t30 = (t29 > INT64_C(0));
      if (t30) {
        t8 = true;
        ov_str* t31 = ov_str_new(".txt", 4);
        int64_t t32 = ov_issue_ref_3f7d9f9a_ccc011e4(v6084, t31);
        ovt_Response_dae56844* t33 = ov_h_issue_text_d7b2d354_587a2631(p0, v6083, t32);
        t7 = t33;
      }
    }
  }
  if (!t8) {
    if ((ov_str_eq(t6->f0, ov_str_new("GET", 3)) && (t6->f1->len == 3 && ov_str_eq(((ov_str*)(t6->f1->items[0]).p), ov_str_new("p", 1))))) {
      ov_str* v6103 = ((ov_str*)(t6->f1->items[1]).p);
      ov_str* v6104 = ((ov_str*)(t6->f1->items[2]).p);
      ov_str* t34 = ov_str_new("", 0);
      int64_t t35 = ov_issue_ref_3f7d9f9a_ccc011e4(v6104, t34);
      bool t36 = (t35 > INT64_C(0));
      if (t36) {
        t8 = true;
        ov_str* t37 = ov_str_new("", 0);
        int64_t t38 = ov_issue_ref_3f7d9f9a_ccc011e4(v6104, t37);
        ovt_Response_dae56844* t39 = ov_h_issue_page_d85f4494_98419fd6(p0, v6103, t38);
        t7 = t39;
      }
    }
  }
  if (!t8) {
    if ((ov_str_eq(t6->f0, ov_str_new("GET", 3)) && (t6->f1->len == 4 && ov_str_eq(((ov_str*)(t6->f1->items[0]).p), ov_str_new("p", 1)) && ov_str_eq(((ov_str*)(t6->f1->items[3]).p), ov_str_new("comments.txt", 12))))) {
      ov_str* v6123 = ((ov_str*)(t6->f1->items[1]).p);
      ov_str* v6124 = ((ov_str*)(t6->f1->items[2]).p);
      ov_str* t40 = ov_str_new("", 0);
      int64_t t41 = ov_issue_ref_3f7d9f9a_ccc011e4(v6124, t40);
      bool t42 = (t41 > INT64_C(0));
      if (t42) {
        t8 = true;
        ov_str* t43 = ov_str_new("", 0);
        int64_t t44 = ov_issue_ref_3f7d9f9a_ccc011e4(v6124, t43);
        ovt_Response_dae56844* t45 = ov_h_comments_text_ade12ed6_5b2777fb(p0, v6123, t44);
        t7 = t45;
      }
    }
  }
  if (!t8) {
    if ((ov_str_eq(t6->f0, ov_str_new("GET", 3)) && (t6->f1->len == 4 && ov_str_eq(((ov_str*)(t6->f1->items[0]).p), ov_str_new("p", 1)) && ov_str_eq(((ov_str*)(t6->f1->items[3]).p), ov_str_new("activity.txt", 12))))) {
      ov_str* v6144 = ((ov_str*)(t6->f1->items[1]).p);
      ov_str* v6145 = ((ov_str*)(t6->f1->items[2]).p);
      ov_str* t46 = ov_str_new("", 0);
      int64_t t47 = ov_issue_ref_3f7d9f9a_ccc011e4(v6145, t46);
      bool t48 = (t47 > INT64_C(0));
      if (t48) {
        t8 = true;
        ov_str* t49 = ov_str_new("", 0);
        int64_t t50 = ov_issue_ref_3f7d9f9a_ccc011e4(v6145, t49);
        ovt_Response_dae56844* t51 = ov_h_activity_text_d94ba9ba_f3661b48(p0, v6144, t50);
        t7 = t51;
      }
    }
  }
  if (!t8) {
    if ((ov_str_eq(t6->f0, ov_str_new("POST", 4)) && (t6->f1->len == 4 && ov_str_eq(((ov_str*)(t6->f1->items[0]).p), ov_str_new("p", 1)) && ov_str_eq(((ov_str*)(t6->f1->items[3]).p), ov_str_new("edit", 4))))) {
      ov_str* v6165 = ((ov_str*)(t6->f1->items[1]).p);
      ov_str* v6166 = ((ov_str*)(t6->f1->items[2]).p);
      ov_str* t52 = ov_str_new("", 0);
      int64_t t53 = ov_issue_ref_3f7d9f9a_ccc011e4(v6166, t52);
      bool t54 = (t53 > INT64_C(0));
      if (t54) {
        t8 = true;
        ov_str* t55 = ov_str_new("", 0);
        int64_t t56 = ov_issue_ref_3f7d9f9a_ccc011e4(v6166, t55);
        ovt_Response_dae56844* t57 = ov_h_edit_issue_58bc3aca_46ced106(p0, v6165, t56, v5936);
        t7 = t57;
      }
    }
  }
  if (!t8) {
    if ((ov_str_eq(t6->f0, ov_str_new("POST", 4)) && (t6->f1->len == 4 && ov_str_eq(((ov_str*)(t6->f1->items[0]).p), ov_str_new("p", 1)) && ov_str_eq(((ov_str*)(t6->f1->items[3]).p), ov_str_new("status", 6))))) {
      ov_str* v6187 = ((ov_str*)(t6->f1->items[1]).p);
      ov_str* v6188 = ((ov_str*)(t6->f1->items[2]).p);
      ov_str* t58 = ov_str_new("", 0);
      int64_t t59 = ov_issue_ref_3f7d9f9a_ccc011e4(v6188, t58);
      bool t60 = (t59 > INT64_C(0));
      if (t60) {
        t8 = true;
        ov_str* t61 = ov_str_new("", 0);
        int64_t t62 = ov_issue_ref_3f7d9f9a_ccc011e4(v6188, t61);
        ovt_Response_dae56844* t63 = ov_h_set_status_1e1a3ecd_347ed678(p0, v6187, t62, v5936);
        t7 = t63;
      }
    }
  }
  if (!t8) {
    if ((ov_str_eq(t6->f0, ov_str_new("POST", 4)) && (t6->f1->len == 4 && ov_str_eq(((ov_str*)(t6->f1->items[0]).p), ov_str_new("p", 1)) && ov_str_eq(((ov_str*)(t6->f1->items[3]).p), ov_str_new("assign", 6))))) {
      ov_str* v6209 = ((ov_str*)(t6->f1->items[1]).p);
      ov_str* v6210 = ((ov_str*)(t6->f1->items[2]).p);
      ov_str* t64 = ov_str_new("", 0);
      int64_t t65 = ov_issue_ref_3f7d9f9a_ccc011e4(v6210, t64);
      bool t66 = (t65 > INT64_C(0));
      if (t66) {
        t8 = true;
        ov_str* t67 = ov_str_new("", 0);
        int64_t t68 = ov_issue_ref_3f7d9f9a_ccc011e4(v6210, t67);
        ovt_Response_dae56844* t69 = ov_h_assign_ec816036_c8b031b0(p0, v6209, t68, v5936);
        t7 = t69;
      }
    }
  }
  if (!t8) {
    if ((ov_str_eq(t6->f0, ov_str_new("POST", 4)) && (t6->f1->len == 4 && ov_str_eq(((ov_str*)(t6->f1->items[0]).p), ov_str_new("p", 1)) && ov_str_eq(((ov_str*)(t6->f1->items[3]).p), ov_str_new("comments", 8))))) {
      ov_str* v6231 = ((ov_str*)(t6->f1->items[1]).p);
      ov_str* v6232 = ((ov_str*)(t6->f1->items[2]).p);
      ov_str* t70 = ov_str_new("", 0);
      int64_t t71 = ov_issue_ref_3f7d9f9a_ccc011e4(v6232, t70);
      bool t72 = (t71 > INT64_C(0));
      if (t72) {
        t8 = true;
        ov_str* t73 = ov_str_new("", 0);
        int64_t t74 = ov_issue_ref_3f7d9f9a_ccc011e4(v6232, t73);
        ovt_Response_dae56844* t75 = ov_h_add_comment_d6f21b12_aad5dc72(p0, v6231, t74, v5936);
        t7 = t75;
      }
    }
  }
  if (!t8) {
    if ((ov_str_eq(t6->f0, ov_str_new("POST", 4)) && (t6->f1->len == 4 && ov_str_eq(((ov_str*)(t6->f1->items[0]).p), ov_str_new("p", 1)) && ov_str_eq(((ov_str*)(t6->f1->items[3]).p), ov_str_new("labels", 6))))) {
      ov_str* v6253 = ((ov_str*)(t6->f1->items[1]).p);
      ov_str* v6254 = ((ov_str*)(t6->f1->items[2]).p);
      ov_str* t76 = ov_str_new("", 0);
      int64_t t77 = ov_issue_ref_3f7d9f9a_ccc011e4(v6254, t76);
      bool t78 = (t77 > INT64_C(0));
      if (t78) {
        t8 = true;
        ov_str* t79 = ov_str_new("", 0);
        int64_t t80 = ov_issue_ref_3f7d9f9a_ccc011e4(v6254, t79);
        ovt_Response_dae56844* t81 = ov_h_add_label_f4cdf2a1_5a132cdd(p0, v6253, t80, v5936);
        t7 = t81;
      }
    }
  }
  if (!t8) {
    if ((ov_str_eq(t6->f0, ov_str_new("POST", 4)) && (t6->f1->len == 5 && ov_str_eq(((ov_str*)(t6->f1->items[0]).p), ov_str_new("p", 1)) && ov_str_eq(((ov_str*)(t6->f1->items[3]).p), ov_str_new("labels", 6)) && ov_str_eq(((ov_str*)(t6->f1->items[4]).p), ov_str_new("remove", 6))))) {
      ov_str* v6275 = ((ov_str*)(t6->f1->items[1]).p);
      ov_str* v6276 = ((ov_str*)(t6->f1->items[2]).p);
      ov_str* t82 = ov_str_new("", 0);
      int64_t t83 = ov_issue_ref_3f7d9f9a_ccc011e4(v6276, t82);
      bool t84 = (t83 > INT64_C(0));
      if (t84) {
        t8 = true;
        ov_str* t85 = ov_str_new("", 0);
        int64_t t86 = ov_issue_ref_3f7d9f9a_ccc011e4(v6276, t85);
        ovt_Response_dae56844* t87 = ov_h_remove_label_cf75d164_267cd1f5(p0, v6275, t86, v5936);
        t7 = t87;
      }
    }
  }
  if (!t8) {
    if (true) {
      t8 = true;
      ov_str* t88 = ov_str_new("not found", 9);
      ovt_Response_dae56844* t89 = ov_resp_error_b47b9fc9_7be2787d(INT64_C(404), t88);
      t7 = t89;
    }
  }
  return t7;
}
