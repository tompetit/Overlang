#include "ov_types.h"

ov_str* ov_schema_sql_ec81cf05_68aabf13(void);

ov_str* ov_schema_sql_ec81cf05_68aabf13(void) {
  ov_str* t1 = ov_str_new("create table if not exists users (id integer primary key autoincrement, name text not null unique);\n", 100);
  ov_str* t2 = ov_str_new("create table if not exists projects (id integer primary key autoincrement, key text not null unique, name text not null);\n", 122);
  ov_str* t3 = ov_str_concat(t1, t2);
  ov_str* t4 = ov_str_new("create table if not exists issues (id integer primary key autoincrement, project_id integer not null references projects(id), ", 126);
  ov_str* t5 = ov_str_concat(t3, t4);
  ov_str* t6 = ov_str_new("number integer not null, title text not null, body text not null default '', ", 77);
  ov_str* t7 = ov_str_concat(t5, t6);
  ov_str* t8 = ov_str_new("status text not null default 'open' check (status in ('open', 'closed')), ", 74);
  ov_str* t9 = ov_str_concat(t7, t8);
  ov_str* t10 = ov_str_new("priority integer not null default 3 check (priority between 1 and 4), ", 70);
  ov_str* t11 = ov_str_concat(t9, t10);
  ov_str* t12 = ov_str_new("assignee_id integer references users(id), unique (project_id, number));\n", 72);
  ov_str* t13 = ov_str_concat(t11, t12);
  ov_str* t14 = ov_str_new("create table if not exists labels (id integer primary key autoincrement, project_id integer not null references projects(id), ", 126);
  ov_str* t15 = ov_str_concat(t13, t14);
  ov_str* t16 = ov_str_new("name text not null, unique (project_id, name));\n", 48);
  ov_str* t17 = ov_str_concat(t15, t16);
  ov_str* t18 = ov_str_new("create table if not exists issue_labels (issue_id integer not null references issues(id), ", 90);
  ov_str* t19 = ov_str_concat(t17, t18);
  ov_str* t20 = ov_str_new("label_id integer not null references labels(id), primary key (issue_id, label_id));\n", 84);
  ov_str* t21 = ov_str_concat(t19, t20);
  ov_str* t22 = ov_str_new("create table if not exists comments (id integer primary key autoincrement, issue_id integer not null references issues(id), ", 124);
  ov_str* t23 = ov_str_concat(t21, t22);
  ov_str* t24 = ov_str_new("author_id integer not null references users(id), body text not null);\n", 70);
  ov_str* t25 = ov_str_concat(t23, t24);
  ov_str* t26 = ov_str_new("create table if not exists activity (id integer primary key autoincrement, issue_id integer not null references issues(id), ", 124);
  ov_str* t27 = ov_str_concat(t25, t26);
  ov_str* t28 = ov_str_new("kind text not null check (kind in ('created', 'status', 'assigned', 'commented')), detail text not null default '');\n", 117);
  ov_str* t29 = ov_str_concat(t27, t28);
  return t29;
}
