#include "ov_types.h"

ov_str* ov_issue_select_sql_91164f3f_1b0791dc(void);

ov_str* ov_issue_select_sql_91164f3f_1b0791dc(void) {
  ov_str* t1 = ov_str_new("select i.id, i.project_id, p.key, i.number, i.title, i.body, i.status, i.priority, ", 83);
  ov_str* t2 = ov_str_new("coalesce(u.name, ''), ", 22);
  ov_str* t3 = ov_str_concat(t1, t2);
  ov_str* t4 = ov_str_new("coalesce((select group_concat(n, ',') from (select l.name as n from issue_labels il ", 84);
  ov_str* t5 = ov_str_concat(t3, t4);
  ov_str* t6 = ov_str_new("join labels l on l.id = il.label_id where il.issue_id = i.id order by l.name)), '') ", 84);
  ov_str* t7 = ov_str_concat(t5, t6);
  ov_str* t8 = ov_str_new("from issues i join projects p on p.id = i.project_id left join users u on u.id = i.assignee_id", 94);
  ov_str* t9 = ov_str_concat(t7, t8);
  return t9;
}
