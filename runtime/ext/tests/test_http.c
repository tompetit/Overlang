/* Tests for ov_http: the server runs in this process, clients are forked children (curl or raw sockets). */
#include "ov_http.h"

#include <arpa/inet.h>
#include <gc.h>
#include <netinet/in.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/wait.h>
#include <unistd.h>

static int checks, fails;
#define CHECK(c) do { checks++; if (!(c)) { fails++; printf("FAIL %s:%d: %s\n", __FILE__, __LINE__, #c); } } while (0)
#define CHECK_STR(a, b) do { const char* a_ = (a); checks++; \
    if (!a_ || strcmp(a_, (b))) { fails++; printf("FAIL %s:%d: got \"%s\" want \"%s\"\n", __FILE__, __LINE__, a_ ? a_ : "(null)", (b)); } } while (0)

typedef struct { pid_t pid; int fd; } client;

/* Fork a child whose stdout is a pipe; the child runs curl with argv (NULL-terminated). */
static client spawn_curl(char* const argv[]) {
    int p[2];
    if (pipe(p)) exit(2);
    pid_t pid = fork();
    if (pid == 0) {
        dup2(p[1], 1);
        close(p[0]);
        execvp("curl", argv);
        _exit(127);
    }
    close(p[1]);
    return (client){pid, p[0]};
}

/* Fork a child that connects to port, sends data, and relays everything it receives until EOF. */
static client spawn_raw(int port, const char* data) {
    int p[2];
    if (pipe(p)) exit(2);
    pid_t pid = fork();
    if (pid == 0) {
        close(p[0]);
        int s = socket(AF_INET, SOCK_STREAM, 0);
        struct sockaddr_in a = {0};
        a.sin_family = AF_INET;
        a.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
        a.sin_port = htons((uint16_t)port);
        if (connect(s, (struct sockaddr*)&a, sizeof a)) _exit(1);
        if (write(s, data, strlen(data)) < 0) _exit(1);
        char b[4096];
        ssize_t n;
        while ((n = read(s, b, sizeof b)) > 0) if (write(p[1], b, (size_t)n) < 0) break;
        _exit(0);
    }
    close(p[1]);
    return (client){pid, p[0]};
}

/* Wait for the child and return everything it wrote (GC memory). */
static char* finish(client c) {
    char* out = GC_MALLOC_ATOMIC(1 << 16);
    size_t len = 0;
    ssize_t n;
    while (len < (1 << 16) - 1 && (n = read(c.fd, out + len, (1 << 16) - 1 - len)) > 0) len += (size_t)n;
    out[len] = 0;
    close(c.fd);
    waitpid(c.pid, NULL, 0);
    return out;
}

int main(void) {
    GC_INIT();
    alarm(60); /* a hung test is a failed test */
    int64_t srv = ov_http_listen(0);
    CHECK(srv >= 1);
    int port = (int)ov_http_port(srv);
    CHECK(port > 0);
    char url[128];

    /* pure helpers */
    CHECK_STR(ov_http_url_decode("a%20b+c%2Fd%e2%82%ac"), "a b c/d\xe2\x82\xac");
    CHECK_STR(ov_http_url_decode("100%"), "100%");
    CHECK_STR(ov_http_url_decode("%zz%4"), "%zz%4");
    CHECK_STR(ov_http_url_decode(""), "");
    CHECK_STR(ov_http_form_value("title=buy+milk&done=1&x=%26y", "title"), "buy milk");
    CHECK_STR(ov_http_form_value("title=buy+milk&done=1&x=%26y", "done"), "1");
    CHECK_STR(ov_http_form_value("title=buy+milk&done=1&x=%26y", "x"), "&y");
    CHECK_STR(ov_http_form_value("title=buy+milk&done=1", "missing"), "");
    CHECK_STR(ov_http_form_value("a=1&flag&b=", "flag"), "");
    CHECK_STR(ov_http_form_value("t%69tle=ok", "title"), "ok");
    CHECK_STR(ov_http_form_value("", "a"), "");
    CHECK_STR(ov_http_html_escape("<a href=\"x\">Tom & Jerry's</a>"), "&lt;a href=&quot;x&quot;&gt;Tom &amp; Jerry&#39;s&lt;/a&gt;");
    CHECK_STR(ov_http_html_escape("plain"), "plain");

    /* GET with query and headers, via curl; also checks the status code curl sees */
    snprintf(url, sizeof url, "http://127.0.0.1:%d/todos/list?x=1&y=a%%20b", port);
    char* a1[] = {"curl", "-s", "-o", "/dev/null", "-w", "%{http_code}", "-H", "X-Test:   hello  ", url, NULL};
    client c = spawn_curl(a1);
    int64_t r = ov_http_accept(srv);
    CHECK(r >= 1);
    CHECK_STR(ov_http_method(r), "GET");
    CHECK_STR(ov_http_path(r), "/todos/list");
    CHECK_STR(ov_http_query(r), "x=1&y=a%20b");
    CHECK_STR(ov_http_form_value(ov_http_query(r), "y"), "a b");
    CHECK_STR(ov_http_header(r, "x-test"), "hello");
    CHECK_STR(ov_http_header(r, "X-TEST"), "hello");
    CHECK(strlen(ov_http_header(r, "Host")) > 0);
    CHECK_STR(ov_http_header(r, "X-Absent"), "");
    CHECK_STR(ov_http_header(r, "X-Tes"), ""); /* prefix of a header name is not a match */
    CHECK_STR(ov_http_body(r), "");
    CHECK(ov_http_respond(r, 404, "text/plain", "nope") == 0);
    CHECK(ov_http_respond(r, 200, "text/plain", "again") < 0); /* slot already freed */
    CHECK_STR(ov_http_method(r), "");
    CHECK_STR(finish(c), "404");

    /* POST form body, status 201, body round-trips through curl */
    snprintf(url, sizeof url, "http://127.0.0.1:%d/add", port);
    char* a2[] = {"curl", "-s", "-d", "title=it%27s+a+%3Ctest%3E&done=0", url, NULL};
    c = spawn_curl(a2);
    r = ov_http_accept(srv);
    CHECK(r >= 1);
    CHECK_STR(ov_http_method(r), "POST");
    CHECK_STR(ov_http_path(r), "/add");
    CHECK_STR(ov_http_query(r), "");
    CHECK_STR(ov_http_header(r, "Content-Type"), "application/x-www-form-urlencoded");
    CHECK_STR(ov_http_header(r, "Content-Length"), "32");
    CHECK_STR(ov_http_body(r), "title=it%27s+a+%3Ctest%3E&done=0");
    CHECK_STR(ov_http_form_value(ov_http_body(r), "title"), "it's a <test>");
    CHECK(ov_http_respond(r, 201, "text/html; charset=utf-8", ov_http_html_escape(ov_http_form_value(ov_http_body(r), "title"))) == 0);
    CHECK_STR(finish(c), "it&#39;s a &lt;test&gt;");

    /* raw: lower-case header names, no query, 200 headers on the wire */
    c = spawn_raw(port, "PUT /a/b HTTP/1.1\r\nhost: x\r\ncontent-length: 5\r\nX-Multi:  v1 \r\n\r\nhello");
    r = ov_http_accept(srv);
    CHECK(r >= 1);
    CHECK_STR(ov_http_method(r), "PUT");
    CHECK_STR(ov_http_path(r), "/a/b");
    CHECK_STR(ov_http_header(r, "X-Multi"), "v1");
    CHECK_STR(ov_http_header(r, "Content-Length"), "5");
    CHECK_STR(ov_http_body(r), "hello");
    CHECK(ov_http_respond(r, 200, "text/plain", "") == 0);
    char* resp = finish(c);
    CHECK(strncmp(resp, "HTTP/1.1 200 OK\r\n", 17) == 0);
    CHECK(strstr(resp, "Content-Length: 0\r\n") != NULL);
    CHECK(strstr(resp, "Connection: close\r\n") != NULL);

    /* raw: bare request, no headers */
    c = spawn_raw(port, "GET /x HTTP/1.1\r\n\r\n");
    r = ov_http_accept(srv);
    CHECK(r >= 1);
    CHECK_STR(ov_http_path(r), "/x");
    CHECK(ov_http_respond(r, 200, "text/plain", "x") == 0);
    CHECK(strstr(finish(c), "\r\n\r\nx") != NULL);

    /* raw: oversized body is refused with 413 and accept reports an error */
    c = spawn_raw(port, "POST /big HTTP/1.1\r\nContent-Length: 2000000\r\n\r\n");
    r = ov_http_accept(srv);
    CHECK(r < 0);
    CHECK(strncmp(finish(c), "HTTP/1.1 413", 12) == 0);

    /* raw: garbage request line gets 400 */
    c = spawn_raw(port, "GARBAGE\r\n\r\n");
    r = ov_http_accept(srv);
    CHECK(r < 0);
    CHECK(strncmp(finish(c), "HTTP/1.1 400", 12) == 0);

    /* the server is still healthy after errors */
    snprintf(url, sizeof url, "http://127.0.0.1:%d/", port);
    char* a3[] = {"curl", "-s", "-o", "/dev/null", "-w", "%{http_code}", url, NULL};
    c = spawn_curl(a3);
    r = ov_http_accept(srv);
    CHECK(r >= 1);
    CHECK_STR(ov_http_path(r), "/");
    CHECK(ov_http_respond(r, 500, "text/plain", "boom") == 0);
    CHECK_STR(finish(c), "500");

    /* bad handles */
    CHECK(ov_http_accept(0) < 0);
    CHECK(ov_http_accept(99) < 0);
    CHECK(ov_http_respond(0, 200, "text/plain", "") < 0);
    CHECK(ov_http_listen(70000) < 0);

    /* close, then the port can be rebound at once (SO_REUSEADDR) */
    ov_http_close(srv);
    ov_http_close(srv);
    CHECK(ov_http_accept(srv) < 0);
    int64_t srv2 = ov_http_listen(port);
    CHECK(srv2 >= 1);
    CHECK((int)ov_http_port(srv2) == port);
    ov_http_close(srv2);

    printf("test_http: %d checks, %d failed\n", checks, fails);
    return fails ? 1 : 0;
}
