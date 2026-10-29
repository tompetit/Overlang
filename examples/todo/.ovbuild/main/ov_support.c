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

ov_value ovsup_map_index(ov_map* m, ov_value k, const char* fn) {
  ov_value v = {0};
  if (!ov_map_get(m, k, &v)) ov_trap("bounds", fn, "", "\"key not found\"");
  return v;
}

int64_t ovsup_to_int(double x, const char* fn) {
  if (!(x >= -9223372036854775808.0 && x < 9223372036854775808.0)) ov_trap("overflow", fn, "", "\"Float out of Int range\"");
  return (int64_t)x;
}

void ovsup_assert_eq_fail(const char* fn, ov_str* a, ov_str* b) {
  ov_str* msg = ov_str_concat(ov_str_concat(ov_str_lit("assert_eq failed: "), a), ov_str_concat(ov_str_lit(" != "), b));
  ov_trap("assert", fn, "", ovsup_json_str(msg)->data);
}

// List[Todo]
ov_str* ovshow_List_fa550908(ov_list* x) {
  ov_sb* sb = ov_sb_new();
  ov_sb_push_cstr(sb, "[");
  for (int64_t i = 0; i < x->len; i++) {
    if (i > 0) ov_sb_push_cstr(sb, ", ");
    ov_sb_push(sb, ovshow_Todo_6462003c(((ovt_Todo_6462003c*)(x->items[i]).p)));
  }
  ov_sb_push_cstr(sb, "]");
  return ov_sb_finish(sb);
}

// Todo
ov_str* ovshow_Todo_6462003c(ovt_Todo_6462003c* x) {
  ov_sb* sb = ov_sb_new();
  ov_sb_push_cstr(sb, "Todo { ");
  ov_sb_push_cstr(sb, "id: ");
  ov_sb_push(sb, ov_show_int(x->f_id));
  ov_sb_push_cstr(sb, ", title: ");
  ov_sb_push(sb, ov_show_str(x->f_title));
  ov_sb_push_cstr(sb, ", done: ");
  ov_sb_push(sb, ov_show_bool(x->f_done));
  ov_sb_push_cstr(sb, " }");
  return ov_sb_finish(sb);
}
