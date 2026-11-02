#include "ov_types.h"
#include <stdio.h>
#include <string.h>

ov_str* ovsup_json_str(ov_str* s) {
  ov_sb* sb = ov_sb_new();
  ov_sb_push_cstr(sb, "\"");
  char buf[8];
  for (int64_t i = 0; i < s->len; i++) {
    unsigned char c = (unsigned char)s->data[i];
    if (c == '"') ov_sb_push_cstr(sb, "\\\"");
    else if (c == '\\') ov_sb_push_cstr(sb, "\\\\");
    else if (c == '\n') ov_sb_push_cstr(sb, "\\n");
    else if (c == '\t') ov_sb_push_cstr(sb, "\\t");
    else if (c == '\r') ov_sb_push_cstr(sb, "\\r");
    else if (c < 0x20) { snprintf(buf, sizeof buf, "\\u%04x", c); ov_sb_push_cstr(sb, buf); }
    else { buf[0] = (char)c; buf[1] = 0; ov_sb_push_cstr(sb, buf); }
  }
  ov_sb_push_cstr(sb, "\"");
  return ov_sb_finish(sb);
}

/* shortcut: ASCII and Latin-1 Supplement only, upgrade when full Unicode case mapping is needed */
static ov_str* map_case(ov_str* s, bool upper) {
  char* d = GC_MALLOC_ATOMIC((size_t)s->len + 1);
  for (int64_t i = 0; i < s->len; i++) {
    unsigned char c = (unsigned char)s->data[i];
    if (upper && c >= 'a' && c <= 'z') c = (unsigned char)(c - 32);
    else if (!upper && c >= 'A' && c <= 'Z') c = (unsigned char)(c + 32);
    else if (c == 0xC3 && i + 1 < s->len) {
      unsigned char n = (unsigned char)s->data[i + 1];
      if (upper && n >= 0xA0 && n <= 0xBE && n != 0xB7) n = (unsigned char)(n - 0x20);
      else if (!upper && n >= 0x80 && n <= 0x9E && n != 0x97) n = (unsigned char)(n + 0x20);
      d[i] = (char)c;
      d[++i] = (char)n;
      continue;
    }
    d[i] = (char)c;
  }
  d[s->len] = 0;
  return ov_str_new(d, s->len);
}
ov_str* ovsup_to_upper(ov_str* s) { return map_case(s, true); }
ov_str* ovsup_to_lower(ov_str* s) { return map_case(s, false); }

ov_list* ovsup_unique(ov_list* xs, ov_key_kind k) {
  ov_set* seen = ov_set_new(k);
  ov_list* out = ov_list_new(xs->len);
  int64_t m = 0;
  for (int64_t i = 0; i < xs->len; i++) {
    if (ov_set_contains(seen, xs->items[i])) continue;
    seen = ov_set_add(seen, xs->items[i]);
    out->items[m++] = xs->items[i];
  }
  out->len = m;
  return out;
}

ov_value ovsup_extreme(ov_list* xs, int (*cmp)(ov_value, ov_value), int sign) {
  ov_value best = xs->items[0];
  for (int64_t i = 1; i < xs->len; i++) {
    int c = cmp(xs->items[i], best);
    if (sign < 0 ? c < 0 : c > 0) best = xs->items[i];
  }
  return best;
}

ov_list* ovsup_sort_by_keys(ov_list* xs, int64_t* keys) {
  int64_t n = xs->len;
  int64_t* idx = GC_MALLOC_ATOMIC(sizeof(int64_t) * (size_t)(n + 1));
  int64_t* tmp = GC_MALLOC_ATOMIC(sizeof(int64_t) * (size_t)(n + 1));
  for (int64_t i = 0; i < n; i++) idx[i] = i;
  for (int64_t w = 1; w < n; w *= 2) {
    for (int64_t lo = 0; lo < n; lo += 2 * w) {
      int64_t mid = lo + w < n ? lo + w : n, hi = lo + 2 * w < n ? lo + 2 * w : n;
      int64_t i = lo, j = mid, k = lo;
      while (i < mid && j < hi) tmp[k++] = keys[idx[j]] < keys[idx[i]] ? idx[j++] : idx[i++];
      while (i < mid) tmp[k++] = idx[i++];
      while (j < hi) tmp[k++] = idx[j++];
    }
    int64_t* t = idx;
    idx = tmp;
    tmp = t;
  }
  ov_list* r = ov_list_new(n);
  for (int64_t i = 0; i < n; i++) r->items[i] = xs->items[idx[i]];
  return r;
}

ov_value ovsup_map_index(ov_map* m, ov_value k, const char* fn, const char* hash) {
  ov_value v = {0};
  if (!ov_map_get(m, k, &v)) ov_trap("bounds", fn, hash, "\"key not found\"");
  return v;
}

int64_t ovsup_to_int(double x, const char* fn, const char* hash) {
  if (!(x >= -9223372036854775808.0 && x < 9223372036854775808.0)) ov_trap("overflow", fn, hash, "\"Float out of Int range\"");
  return (int64_t)x;
}

void ovsup_assert_eq_fail(const char* fn, const char* hash, ov_str* a, ov_str* b) {
  ov_str* msg = ov_str_concat(ov_str_concat(ov_str_lit("assert_eq failed: "), a), ov_str_concat(ov_str_lit(" != "), b));
  ov_trap("assert", fn, hash, ovsup_json_str(msg)->data);
}

// List[List[Str]]
bool oveq_List_b24373fb(ov_list* a, ov_list* b) {
  if (a == b) return true;
  if (a->len != b->len) return false;
  for (int64_t i = 0; i < a->len; i++) if (!oveq_List_f1666589(((ov_list*)(a->items[i]).p), ((ov_list*)(b->items[i]).p))) return false;
  return true;
}

// List[Comment]
bool oveq_List_ba0027a5(ov_list* a, ov_list* b) {
  if (a == b) return true;
  if (a->len != b->len) return false;
  for (int64_t i = 0; i < a->len; i++) if (!oveq_Comment_21df0379(((ovt_Comment_21df0379*)(a->items[i]).p), ((ovt_Comment_21df0379*)(b->items[i]).p))) return false;
  return true;
}

// List[Int]
bool oveq_List_d31dced1(ov_list* a, ov_list* b) {
  if (a == b) return true;
  if (a->len != b->len) return false;
  for (int64_t i = 0; i < a->len; i++) if (!((a->items[i]).i == (b->items[i]).i)) return false;
  return true;
}

// List[Str]
bool oveq_List_f1666589(ov_list* a, ov_list* b) {
  if (a == b) return true;
  if (a->len != b->len) return false;
  for (int64_t i = 0; i < a->len; i++) if (!ov_str_eq(((ov_str*)(a->items[i]).p), ((ov_str*)(b->items[i]).p))) return false;
  return true;
}

// Option[Str]
bool oveq_Option_6133f15b(ovt_Option_6133f15b* a, ovt_Option_6133f15b* b) {
  if (a == b) return true;
  if (a->tag != b->tag) return false;
  if (a->tag == 0) return ov_str_eq(a->u.c0.f0, b->u.c0.f0);
  return true;
}

// Option[User]
bool oveq_Option_7fe81862(ovt_Option_7fe81862* a, ovt_Option_7fe81862* b) {
  if (a == b) return true;
  if (a->tag != b->tag) return false;
  if (a->tag == 0) return oveq_User_9c127ab2(a->u.c0.f0, b->u.c0.f0);
  return true;
}

// Option[Issue]
bool oveq_Option_c3749d6a(ovt_Option_c3749d6a* a, ovt_Option_c3749d6a* b) {
  if (a == b) return true;
  if (a->tag != b->tag) return false;
  if (a->tag == 0) return oveq_Issue_b825796a(a->u.c0.f0, b->u.c0.f0);
  return true;
}

// Option[Project]
bool oveq_Option_da3cc0d5(ovt_Option_da3cc0d5* a, ovt_Option_da3cc0d5* b) {
  if (a == b) return true;
  if (a->tag != b->tag) return false;
  if (a->tag == 0) return oveq_Project_73b03799(a->u.c0.f0, b->u.c0.f0);
  return true;
}

// (Str, List[Str])
bool oveq_tup2_24c1b644(ovt_tup2_24c1b644* a, ovt_tup2_24c1b644* b) {
  if (a == b) return true;
  return ov_str_eq(a->f0, b->f0) && oveq_List_f1666589(a->f1, b->f1);
}

// Filter
ov_str* ovshow_Filter_50e09095(ovt_Filter_50e09095* x) {
  ov_sb* sb = ov_sb_new();
  ov_sb_push_cstr(sb, "Filter { ");
  ov_sb_push_cstr(sb, "status: ");
  ov_sb_push(sb, ov_show_str(x->f_status));
  ov_sb_push_cstr(sb, ", assignee: ");
  ov_sb_push(sb, ov_show_str(x->f_assignee));
  ov_sb_push_cstr(sb, ", label: ");
  ov_sb_push(sb, ov_show_str(x->f_label));
  ov_sb_push_cstr(sb, ", priority: ");
  ov_sb_push(sb, ov_show_int(x->f_priority));
  ov_sb_push_cstr(sb, ", q: ");
  ov_sb_push(sb, ov_show_str(x->f_q));
  ov_sb_push_cstr(sb, ", page: ");
  ov_sb_push(sb, ov_show_int(x->f_page));
  ov_sb_push_cstr(sb, " }");
  return ov_sb_finish(sb);
}

// Issue
ov_str* ovshow_Issue_b825796a(ovt_Issue_b825796a* x) {
  ov_sb* sb = ov_sb_new();
  ov_sb_push_cstr(sb, "Issue { ");
  ov_sb_push_cstr(sb, "id: ");
  ov_sb_push(sb, ov_show_int(x->f_id));
  ov_sb_push_cstr(sb, ", project_id: ");
  ov_sb_push(sb, ov_show_int(x->f_project_id));
  ov_sb_push_cstr(sb, ", project_key: ");
  ov_sb_push(sb, ov_show_str(x->f_project_key));
  ov_sb_push_cstr(sb, ", number: ");
  ov_sb_push(sb, ov_show_int(x->f_number));
  ov_sb_push_cstr(sb, ", title: ");
  ov_sb_push(sb, ov_show_str(x->f_title));
  ov_sb_push_cstr(sb, ", body: ");
  ov_sb_push(sb, ov_show_str(x->f_body));
  ov_sb_push_cstr(sb, ", status: ");
  ov_sb_push(sb, ov_show_str(x->f_status));
  ov_sb_push_cstr(sb, ", priority: ");
  ov_sb_push(sb, ov_show_int(x->f_priority));
  ov_sb_push_cstr(sb, ", assignee: ");
  ov_sb_push(sb, ov_show_str(x->f_assignee));
  ov_sb_push_cstr(sb, ", labels: ");
  ov_sb_push(sb, ovshow_List_f1666589(x->f_labels));
  ov_sb_push_cstr(sb, " }");
  return ov_sb_finish(sb);
}

// List[List[Str]]
ov_str* ovshow_List_b24373fb(ov_list* x) {
  ov_sb* sb = ov_sb_new();
  ov_sb_push_cstr(sb, "[");
  for (int64_t i = 0; i < x->len; i++) {
    if (i > 0) ov_sb_push_cstr(sb, ", ");
    ov_sb_push(sb, ovshow_List_f1666589(((ov_list*)(x->items[i]).p)));
  }
  ov_sb_push_cstr(sb, "]");
  return ov_sb_finish(sb);
}

// List[Comment]
ov_str* ovshow_List_ba0027a5(ov_list* x) {
  ov_sb* sb = ov_sb_new();
  ov_sb_push_cstr(sb, "[");
  for (int64_t i = 0; i < x->len; i++) {
    if (i > 0) ov_sb_push_cstr(sb, ", ");
    ov_sb_push(sb, ovshow_Comment_21df0379(((ovt_Comment_21df0379*)(x->items[i]).p)));
  }
  ov_sb_push_cstr(sb, "]");
  return ov_sb_finish(sb);
}

// List[Int]
ov_str* ovshow_List_d31dced1(ov_list* x) {
  ov_sb* sb = ov_sb_new();
  ov_sb_push_cstr(sb, "[");
  for (int64_t i = 0; i < x->len; i++) {
    if (i > 0) ov_sb_push_cstr(sb, ", ");
    ov_sb_push(sb, ov_show_int((x->items[i]).i));
  }
  ov_sb_push_cstr(sb, "]");
  return ov_sb_finish(sb);
}

// List[Str]
ov_str* ovshow_List_f1666589(ov_list* x) {
  ov_sb* sb = ov_sb_new();
  ov_sb_push_cstr(sb, "[");
  for (int64_t i = 0; i < x->len; i++) {
    if (i > 0) ov_sb_push_cstr(sb, ", ");
    ov_sb_push(sb, ov_show_str(((ov_str*)(x->items[i]).p)));
  }
  ov_sb_push_cstr(sb, "]");
  return ov_sb_finish(sb);
}

// Option[Str]
ov_str* ovshow_Option_6133f15b(ovt_Option_6133f15b* x) {
  ov_sb* sb = ov_sb_new();
  if (x->tag == 0) {
  ov_sb_push_cstr(sb, "Some");
  ov_sb_push_cstr(sb, "(");
  ov_sb_push(sb, ov_show_str(x->u.c0.f0));
  ov_sb_push_cstr(sb, ")");
  }
  if (x->tag == 1) {
  ov_sb_push_cstr(sb, "None");
  }
  return ov_sb_finish(sb);
}

// Option[User]
ov_str* ovshow_Option_7fe81862(ovt_Option_7fe81862* x) {
  ov_sb* sb = ov_sb_new();
  if (x->tag == 0) {
  ov_sb_push_cstr(sb, "Some");
  ov_sb_push_cstr(sb, "(");
  ov_sb_push(sb, ovshow_User_9c127ab2(x->u.c0.f0));
  ov_sb_push_cstr(sb, ")");
  }
  if (x->tag == 1) {
  ov_sb_push_cstr(sb, "None");
  }
  return ov_sb_finish(sb);
}

// Option[Issue]
ov_str* ovshow_Option_c3749d6a(ovt_Option_c3749d6a* x) {
  ov_sb* sb = ov_sb_new();
  if (x->tag == 0) {
  ov_sb_push_cstr(sb, "Some");
  ov_sb_push_cstr(sb, "(");
  ov_sb_push(sb, ovshow_Issue_b825796a(x->u.c0.f0));
  ov_sb_push_cstr(sb, ")");
  }
  if (x->tag == 1) {
  ov_sb_push_cstr(sb, "None");
  }
  return ov_sb_finish(sb);
}

// Option[Project]
ov_str* ovshow_Option_da3cc0d5(ovt_Option_da3cc0d5* x) {
  ov_sb* sb = ov_sb_new();
  if (x->tag == 0) {
  ov_sb_push_cstr(sb, "Some");
  ov_sb_push_cstr(sb, "(");
  ov_sb_push(sb, ovshow_Project_73b03799(x->u.c0.f0));
  ov_sb_push_cstr(sb, ")");
  }
  if (x->tag == 1) {
  ov_sb_push_cstr(sb, "None");
  }
  return ov_sb_finish(sb);
}

// (Str, List[Str])
ov_str* ovshow_tup2_24c1b644(ovt_tup2_24c1b644* x) {
  ov_sb* sb = ov_sb_new();
  ov_sb_push_cstr(sb, "(");
  ov_sb_push(sb, ov_show_str(x->f0));
  ov_sb_push_cstr(sb, ", ");
  ov_sb_push(sb, ovshow_List_f1666589(x->f1));
  ov_sb_push_cstr(sb, ")");
  return ov_sb_finish(sb);
}

// Comment
bool oveq_Comment_21df0379(ovt_Comment_21df0379* a, ovt_Comment_21df0379* b) {
  if (a == b) return true;
  return (a->f_id == b->f_id) && (a->f_issue_id == b->f_issue_id) && ov_str_eq(a->f_author, b->f_author) && ov_str_eq(a->f_body, b->f_body);
}

// Issue
bool oveq_Issue_b825796a(ovt_Issue_b825796a* a, ovt_Issue_b825796a* b) {
  if (a == b) return true;
  return (a->f_id == b->f_id) && (a->f_project_id == b->f_project_id) && ov_str_eq(a->f_project_key, b->f_project_key) && (a->f_number == b->f_number) && ov_str_eq(a->f_title, b->f_title) && ov_str_eq(a->f_body, b->f_body) && ov_str_eq(a->f_status, b->f_status) && (a->f_priority == b->f_priority) && ov_str_eq(a->f_assignee, b->f_assignee) && oveq_List_f1666589(a->f_labels, b->f_labels);
}

// Project
bool oveq_Project_73b03799(ovt_Project_73b03799* a, ovt_Project_73b03799* b) {
  if (a == b) return true;
  return (a->f_id == b->f_id) && ov_str_eq(a->f_key, b->f_key) && ov_str_eq(a->f_name, b->f_name);
}

// User
bool oveq_User_9c127ab2(ovt_User_9c127ab2* a, ovt_User_9c127ab2* b) {
  if (a == b) return true;
  return (a->f_id == b->f_id) && ov_str_eq(a->f_name, b->f_name);
}

// Comment
ov_str* ovshow_Comment_21df0379(ovt_Comment_21df0379* x) {
  ov_sb* sb = ov_sb_new();
  ov_sb_push_cstr(sb, "Comment { ");
  ov_sb_push_cstr(sb, "id: ");
  ov_sb_push(sb, ov_show_int(x->f_id));
  ov_sb_push_cstr(sb, ", issue_id: ");
  ov_sb_push(sb, ov_show_int(x->f_issue_id));
  ov_sb_push_cstr(sb, ", author: ");
  ov_sb_push(sb, ov_show_str(x->f_author));
  ov_sb_push_cstr(sb, ", body: ");
  ov_sb_push(sb, ov_show_str(x->f_body));
  ov_sb_push_cstr(sb, " }");
  return ov_sb_finish(sb);
}

// Project
ov_str* ovshow_Project_73b03799(ovt_Project_73b03799* x) {
  ov_sb* sb = ov_sb_new();
  ov_sb_push_cstr(sb, "Project { ");
  ov_sb_push_cstr(sb, "id: ");
  ov_sb_push(sb, ov_show_int(x->f_id));
  ov_sb_push_cstr(sb, ", key: ");
  ov_sb_push(sb, ov_show_str(x->f_key));
  ov_sb_push_cstr(sb, ", name: ");
  ov_sb_push(sb, ov_show_str(x->f_name));
  ov_sb_push_cstr(sb, " }");
  return ov_sb_finish(sb);
}

// User
ov_str* ovshow_User_9c127ab2(ovt_User_9c127ab2* x) {
  ov_sb* sb = ov_sb_new();
  ov_sb_push_cstr(sb, "User { ");
  ov_sb_push_cstr(sb, "id: ");
  ov_sb_push(sb, ov_show_int(x->f_id));
  ov_sb_push_cstr(sb, ", name: ");
  ov_sb_push(sb, ov_show_str(x->f_name));
  ov_sb_push_cstr(sb, " }");
  return ov_sb_finish(sb);
}
