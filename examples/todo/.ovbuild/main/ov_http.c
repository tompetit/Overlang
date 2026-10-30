/* HTTP shim: see ov_http.h. Plain C (POSIX), not dependent on rt.h. */
#define _POSIX_C_SOURCE 200809L
#include "ov_http.h"

#include <arpa/inet.h>
#include <errno.h>
#include <gc.h>
#include <netinet/in.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <strings.h>
#include <sys/socket.h>
#include <sys/time.h>
#include <unistd.h>

#define MAX_SERVERS 16
#define MAX_REQS 64
#define MAX_HEAD (64 * 1024)
#define MAX_BODY (1024 * 1024)

typedef struct {
    int fd;
    const char *method, *path, *query, *headers, *body; /* headers: raw lines after the request line, \r\n separated */
} req_t;

static int servers[MAX_SERVERS]; /* fd + 1, 0 = free */
static req_t reqs[MAX_REQS];
static int req_used[MAX_REQS]; /* 1 = slot holds a live request */

static char* gcdup(const char* s, size_t n) {
    char* p = GC_MALLOC_ATOMIC(n + 1);
    memcpy(p, s, n);
    p[n] = 0;
    return p;
}

int64_t ov_http_listen(int64_t port) {
    if (port < 0 || port > 65535) return -1;
    signal(SIGPIPE, SIG_IGN);
    int slot = -1;
    for (int i = 0; i < MAX_SERVERS; i++) if (!servers[i]) { slot = i; break; }
    if (slot < 0) return -2;
    int fd = socket(AF_INET, SOCK_STREAM, 0);
    if (fd < 0) return -3;
    int one = 1;
    setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &one, sizeof one);
    struct sockaddr_in a = {0};
    a.sin_family = AF_INET;
    a.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
    a.sin_port = htons((uint16_t)port);
    if (bind(fd, (struct sockaddr*)&a, sizeof a) < 0 || listen(fd, 64) < 0) { close(fd); return -4; }
    servers[slot] = fd + 1;
    return slot + 1;
}

static int server_fd(int64_t s) { return s >= 1 && s <= MAX_SERVERS && servers[s - 1] ? servers[s - 1] - 1 : -1; }

int64_t ov_http_port(int64_t server) {
    int fd = server_fd(server);
    struct sockaddr_in a;
    socklen_t n = sizeof a;
    if (fd < 0 || getsockname(fd, (struct sockaddr*)&a, &n) < 0) return -1;
    return ntohs(a.sin_port);
}

void ov_http_close(int64_t server) {
    int fd = server_fd(server);
    if (fd < 0) return;
    close(fd);
    servers[server - 1] = 0;
}

static req_t* get(int64_t r) { return r >= 1 && r <= MAX_REQS && req_used[r - 1] ? &reqs[r - 1] : NULL; }

static void reply_and_close(int fd, const char* status) {
    char m[160];
    int n = snprintf(m, sizeof m, "HTTP/1.1 %s\r\nContent-Length: 0\r\nConnection: close\r\n\r\n", status);
    ssize_t w = write(fd, m, (size_t)n);
    (void)w;
    close(fd);
}

/* Read one request from fd into r; returns 0, or a negative code after answering and closing fd. */
static int read_request(int fd, req_t* r) {
    char* b = GC_MALLOC_ATOMIC(MAX_HEAD + 1);
    size_t len = 0, hend = 0;
    char* sep = NULL;
    while (!sep) {
        if (len >= MAX_HEAD) { reply_and_close(fd, "431 Request Header Fields Too Large"); return -5; }
        ssize_t n = read(fd, b + len, MAX_HEAD - len);
        if (n < 0 && errno == EINTR) continue;
        if (n <= 0) { close(fd); return -6; }
        size_t from = len > 3 ? len - 3 : 0;
        len += (size_t)n;
        b[len] = 0;
        sep = strstr(b + from, "\r\n\r\n");
    }
    hend = (size_t)(sep - b) + 4;
    *sep = 0;
    /* request line: METHOD SP target SP HTTP/x */
    char* sp1 = strchr(b, ' ');
    char* sp2 = sp1 ? strchr(sp1 + 1, ' ') : NULL;
    char* eol = strstr(b, "\r\n");
    if (!sp1 || !sp2 || (eol && eol < sp2)) { reply_and_close(fd, "400 Bad Request"); return -7; }
    *sp1 = 0;
    *sp2 = 0;
    char* q = strchr(sp1 + 1, '?');
    r->method = b;
    r->query = q ? q + 1 : "";
    if (q) *q = 0;
    r->path = sp1 + 1;
    r->headers = eol ? eol + 2 : "";
    /* body per Content-Length */
    size_t want = 0;
    for (const char* h = r->headers; *h;) {
        const char* e = strstr(h, "\r\n");
        size_t ll = e ? (size_t)(e - h) : strlen(h);
        if (ll > 15 && !strncasecmp(h, "content-length:", 15)) {
            char* end;
            long long v = strtoll(h + 15, &end, 10);
            if (v < 0 || end == h + 15) { reply_and_close(fd, "400 Bad Request"); return -7; }
            if (v > MAX_BODY) { reply_and_close(fd, "413 Payload Too Large"); return -8; }
            want = (size_t)v;
        }
        h = e ? e + 2 : h + ll;
    }
    char* body = GC_MALLOC_ATOMIC(want + 1);
    size_t have = len > hend ? len - hend : 0;
    if (have > want) have = want;
    memcpy(body, b + hend, have);
    while (have < want) {
        ssize_t n = read(fd, body + have, want - have);
        if (n < 0 && errno == EINTR) continue;
        if (n <= 0) { close(fd); return -6; }
        have += (size_t)n;
    }
    body[want] = 0;
    r->body = body;
    r->fd = fd;
    return 0;
}

int64_t ov_http_accept(int64_t server) {
    int sfd = server_fd(server);
    if (sfd < 0) return -1;
    int slot = -1;
    for (int i = 0; i < MAX_REQS; i++) if (!req_used[i]) { slot = i; break; }
    if (slot < 0) return -2;
    int fd;
    while ((fd = accept(sfd, NULL, NULL)) < 0) if (errno != EINTR) return -3;
    /* shortcut: 10 s read timeout stops one stalled client blocking the single thread, tune or go threaded later */
    struct timeval tv = {10, 0};
    setsockopt(fd, SOL_SOCKET, SO_RCVTIMEO, &tv, sizeof tv);
    int rc = read_request(fd, &reqs[slot]);
    if (rc) return rc;
    req_used[slot] = 1;
    return slot + 1;
}

const char* ov_http_method(int64_t req) { req_t* r = get(req); return r ? r->method : ""; }
const char* ov_http_path(int64_t req) { req_t* r = get(req); return r ? r->path : ""; }
const char* ov_http_query(int64_t req) { req_t* r = get(req); return r ? r->query : ""; }
const char* ov_http_body(int64_t req) { req_t* r = get(req); return r ? r->body : ""; }

const char* ov_http_header(int64_t req, const char* name) {
    req_t* r = get(req);
    size_t nl = strlen(name);
    if (!r || !nl) return "";
    for (const char* h = r->headers; *h;) {
        const char* e = strstr(h, "\r\n");
        size_t ll = e ? (size_t)(e - h) : strlen(h);
        if (ll > nl && h[nl] == ':' && !strncasecmp(h, name, nl)) {
            const char *v = h + nl + 1, *ve = h + ll;
            while (v < ve && (*v == ' ' || *v == '\t')) v++;
            while (ve > v && (ve[-1] == ' ' || ve[-1] == '\t')) ve--;
            return gcdup(v, (size_t)(ve - v));
        }
        h = e ? e + 2 : h + ll;
    }
    return "";
}

static int hexv(char c) {
    if (c >= '0' && c <= '9') return c - '0';
    if (c >= 'a' && c <= 'f') return c - 'a' + 10;
    if (c >= 'A' && c <= 'F') return c - 'A' + 10;
    return -1;
}

/* Decode the first n bytes of s. */
static const char* decode_n(const char* s, size_t n) {
    char* o = GC_MALLOC_ATOMIC(n + 1);
    size_t j = 0;
    for (size_t i = 0; i < n; i++) {
        if (s[i] == '+') o[j++] = ' ';
        else if (s[i] == '%' && i + 2 < n && hexv(s[i + 1]) >= 0 && hexv(s[i + 2]) >= 0) {
            o[j++] = (char)(hexv(s[i + 1]) * 16 + hexv(s[i + 2]));
            i += 2;
        } else o[j++] = s[i];
    }
    o[j] = 0;
    return o;
}

const char* ov_http_url_decode(const char* s) { return decode_n(s, strlen(s)); }

const char* ov_http_form_value(const char* body, const char* key) {
    size_t kl = strlen(key);
    for (const char* p = body; *p;) {
        const char* e = strchr(p, '&');
        size_t pl = e ? (size_t)(e - p) : strlen(p);
        const char* eq = memchr(p, '=', pl);
        size_t nl = eq ? (size_t)(eq - p) : pl;
        const char* k = decode_n(p, nl);
        if (strlen(k) == kl && !memcmp(k, key, kl))
            return eq ? decode_n(eq + 1, pl - nl - 1) : "";
        p = e ? e + 1 : p + pl;
    }
    return "";
}

const char* ov_http_html_escape(const char* s) {
    size_t n = 0;
    for (const char* p = s; *p; p++) n += *p == '&' ? 5 : *p == '<' || *p == '>' ? 4 : *p == '"' || *p == '\'' ? 6 : 1;
    char* o = GC_MALLOC_ATOMIC(n + 1);
    char* w = o;
    for (const char* p = s; *p; p++) {
        const char* rep = *p == '&' ? "&amp;" : *p == '<' ? "&lt;" : *p == '>' ? "&gt;" : *p == '"' ? "&quot;" : *p == '\'' ? "&#39;" : NULL;
        if (rep) { size_t l = strlen(rep); memcpy(w, rep, l); w += l; }
        else *w++ = *p;
    }
    *w = 0;
    return o;
}

static const char* reason(int64_t st) {
    switch (st) {
    case 200: return "OK";
    case 201: return "Created";
    case 204: return "No Content";
    case 301: return "Moved Permanently";
    case 302: return "Found";
    case 303: return "See Other";
    case 400: return "Bad Request";
    case 403: return "Forbidden";
    case 404: return "Not Found";
    case 405: return "Method Not Allowed";
    case 409: return "Conflict";
    case 500: return "Internal Server Error";
    default: return "Status";
    }
}

int64_t ov_http_respond(int64_t req, int64_t status, const char* content_type, const char* body) {
    req_t* r = get(req);
    if (!r) return -1;
    if (status < 100 || status > 599) status = 500;
    size_t bl = strlen(body);
    char head[256];
    int hn = snprintf(head, sizeof head, "HTTP/1.1 %d %s\r\nContent-Type: %s\r\nContent-Length: %zu\r\nConnection: close\r\n\r\n",
                      (int)status, reason(status), content_type, bl);
    int64_t rc = 0;
    if (hn < 0 || (size_t)hn >= sizeof head) rc = -2;
    else {
        /* head and body in separate writes: simplest, and the body can be large */
        const char* parts[2] = {head, body};
        size_t lens[2] = {(size_t)hn, bl};
        for (int i = 0; i < 2 && !rc; i++)
            for (size_t off = 0; off < lens[i];) {
                ssize_t w = send(r->fd, parts[i] + off, lens[i] - off, MSG_NOSIGNAL);
                if (w < 0 && errno == EINTR) continue;
                if (w <= 0) { rc = -3; break; }
                off += (size_t)w;
            }
    }
    close(r->fd);
    req_used[req - 1] = 0;
    memset(r, 0, sizeof *r);
    return rc;
}
