#include "ov_types.h"

ov_list* ov_map_9a617375_4c51e675(ov_list*, ov_closure*);
ov_list* ov_map_9a617375_bf19021d(ov_list*, ov_closure*);
ov_str* ov_html_escape_114cb911_d5a1eb27(ov_str*);
ov_str* ov_html_page_82184164_f910056c(ov_str*, ov_str*);
ov_str* ov_labels_display_122cd589_821b1221(ov_list*);
ov_str* ov_r_input_7aab51f2_751f51d6(ov_str*, ov_str*, ov_str*);
ov_str* ov_render_issue_row_2d3f3036_72855283(ovt_Issue_b825796a*);
ov_str* ov_render_pager_7afb21ff_1ae49da9(ovt_Project_73b03799*, ovt_Filter_50e09095*, ovt_IssuePage_114bf070*);
ov_str* ov_render_project_d1f1ea37_8a57e1d8(ovt_Project_73b03799* p0, ov_list* p1, ovt_Filter_50e09095* p2, ovt_IssuePage_114bf070* p3);

static ov_str* ovl_ov_render_project_d1f1ea37_8a57e1d8_3983(void* ovenv, ovt_Issue_b825796a* l3983_0) {
  ov_str* t47 = ov_render_issue_row_2d3f3036_72855283(l3983_0);
  return t47;
}

static ov_str* ovl_ov_render_project_d1f1ea37_8a57e1d8_4000(void* ovenv, ovt_Label_da07b2c4* l4000_0) {
  ov_str* t55 = l4000_0->f_name;
  return t55;
}

ov_str* ov_render_project_d1f1ea37_8a57e1d8(ovt_Project_73b03799* p0, ov_list* p1, ovt_Filter_50e09095* p2, ovt_IssuePage_114bf070* p3) {
  ov_str* t1 = p0->f_key;
  ov_str* t2 = ov_html_escape_114cb911_d5a1eb27(t1);
  ov_str* v3904 = t2;
  int64_t t3 = p2->f_priority;
  bool t4 = (t3 > INT64_C(0));
  ov_str* t5 = 0; (void)t5;
  if (t4) {
    int64_t t6 = p2->f_priority;
    ov_str* t7 = ov_show_int(t6);
    t5 = t7;
  } else {
    ov_str* t8 = ov_str_new("", 0);
    t5 = t8;
  }
  ov_str* v3909 = t5;
  ov_str* t9 = p0->f_key;
  ov_str* t10 = ov_str_new(": ", 2);
  ov_str* t11 = ov_str_concat(t9, t10);
  ov_str* t12 = p0->f_name;
  ov_str* t13 = ov_str_concat(t11, t12);
  ov_str* t14 = ov_str_new("<h1>", 4);
  ov_str* t15 = ov_str_new(": ", 2);
  ov_str* t16 = p0->f_name;
  ov_str* t17 = ov_html_escape_114cb911_d5a1eb27(t16);
  ov_str* t18 = ov_str_new("</h1>", 5);
  ov_str* t19 = ov_str_new("<form method=\"get\" action=\"/p/", 30);
  ov_str* t20 = ov_str_new("\">", 2);
  ov_str* t21 = ov_str_new("status", 6);
  ov_str* t22 = p2->f_status;
  ov_str* t23 = ov_str_new("status", 6);
  ov_str* t24 = ov_r_input_7aab51f2_751f51d6(t21, t22, t23);
  ov_str* t25 = ov_str_new(" ", 1);
  ov_str* t26 = ov_str_new("assignee", 8);
  ov_str* t27 = p2->f_assignee;
  ov_str* t28 = ov_str_new("assignee", 8);
  ov_str* t29 = ov_r_input_7aab51f2_751f51d6(t26, t27, t28);
  ov_str* t30 = ov_str_new(" ", 1);
  ov_str* t31 = ov_str_new("label", 5);
  ov_str* t32 = p2->f_label;
  ov_str* t33 = ov_str_new("label", 5);
  ov_str* t34 = ov_r_input_7aab51f2_751f51d6(t31, t32, t33);
  ov_str* t35 = ov_str_new(" ", 1);
  ov_str* t36 = ov_str_new("priority", 8);
  ov_str* t37 = ov_str_new("priority", 8);
  ov_str* t38 = ov_r_input_7aab51f2_751f51d6(t36, v3909, t37);
  ov_str* t39 = ov_str_new(" ", 1);
  ov_str* t40 = ov_str_new("q", 1);
  ov_str* t41 = p2->f_q;
  ov_str* t42 = ov_str_new("search", 6);
  ov_str* t43 = ov_r_input_7aab51f2_751f51d6(t40, t41, t42);
  ov_str* t44 = ov_str_new(" <button>filter</button></form>", 31);
  ov_str* t45 = ov_str_new("<table class=\"issues\"><tr><th>Issue</th><th>Title</th><th>Status</th><th>Priority</th><th>Assignee</th><th>Labels</th></tr>", 123);
  ov_list* t46 = p3->f_issues;
  ov_closure* t48 = ov_closure_mk((void*)ovl_ov_render_project_d1f1ea37_8a57e1d8_3983, NULL);
  ov_list* t49 = ov_map_9a617375_4c51e675(t46, t48);
  ov_str* t50 = ov_str_new("", 0);
  ov_str* t51 = ov_str_join(t49, t50);
  ov_str* t52 = ov_str_new("</table>", 8);
  ov_str* t53 = ov_render_pager_7afb21ff_1ae49da9(p0, p2, p3);
  ov_str* t54 = ov_str_new("<h2>Labels</h2><p class=\"labels\">", 33);
  ov_closure* t56 = ov_closure_mk((void*)ovl_ov_render_project_d1f1ea37_8a57e1d8_4000, NULL);
  ov_list* t57 = ov_map_9a617375_bf19021d(p1, t56);
  ov_str* t58 = ov_labels_display_122cd589_821b1221(t57);
  ov_str* t59 = ov_html_escape_114cb911_d5a1eb27(t58);
  ov_str* t60 = ov_str_new("</p>", 4);
  ov_str* t61 = ov_str_new("<form method=\"post\" action=\"/p/", 31);
  ov_str* t62 = ov_str_new("/labels\"><input name=\"name\" placeholder=\"label\"> <button>add label</button></form>", 82);
  ov_str* t63 = ov_str_new("<h2>New issue</h2><form method=\"post\" action=\"/p/", 49);
  ov_str* t64 = ov_str_new("/issues\"><input name=\"title\" placeholder=\"title\"> <input name=\"priority\" value=\"3\"> <input name=\"assignee\" placeholder=\"assignee\"> <input name=\"labels\" placeholder=\"labels\"> <textarea name=\"body\"></textarea> <button>create issue</button></form>", 244);
  ov_list* t65 = ov_list_new(31);
  t65->items[0] = (ov_value){.p = t14};
  t65->items[1] = (ov_value){.p = v3904};
  t65->items[2] = (ov_value){.p = t15};
  t65->items[3] = (ov_value){.p = t17};
  t65->items[4] = (ov_value){.p = t18};
  t65->items[5] = (ov_value){.p = t19};
  t65->items[6] = (ov_value){.p = v3904};
  t65->items[7] = (ov_value){.p = t20};
  t65->items[8] = (ov_value){.p = t24};
  t65->items[9] = (ov_value){.p = t25};
  t65->items[10] = (ov_value){.p = t29};
  t65->items[11] = (ov_value){.p = t30};
  t65->items[12] = (ov_value){.p = t34};
  t65->items[13] = (ov_value){.p = t35};
  t65->items[14] = (ov_value){.p = t38};
  t65->items[15] = (ov_value){.p = t39};
  t65->items[16] = (ov_value){.p = t43};
  t65->items[17] = (ov_value){.p = t44};
  t65->items[18] = (ov_value){.p = t45};
  t65->items[19] = (ov_value){.p = t51};
  t65->items[20] = (ov_value){.p = t52};
  t65->items[21] = (ov_value){.p = t53};
  t65->items[22] = (ov_value){.p = t54};
  t65->items[23] = (ov_value){.p = t59};
  t65->items[24] = (ov_value){.p = t60};
  t65->items[25] = (ov_value){.p = t61};
  t65->items[26] = (ov_value){.p = v3904};
  t65->items[27] = (ov_value){.p = t62};
  t65->items[28] = (ov_value){.p = t63};
  t65->items[29] = (ov_value){.p = v3904};
  t65->items[30] = (ov_value){.p = t64};
  ov_str* t66 = ov_str_new("", 0);
  ov_str* t67 = ov_str_join(t65, t66);
  ov_str* t68 = ov_html_page_82184164_f910056c(t13, t67);
  return t68;
}
