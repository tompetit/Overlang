CC = gcc
CFLAGS = -std=c11 -Wall -Wextra -Werror -O2
# Devbox quirks, same as ../Makefile: a linuxbrew ld can come first on PATH (-B/usr/bin picks the system one), and the
# system libasan.so.6 / libubsan.so.1 sonames are missing from the loader path, so $(B)/.sanlibs symlinks them next to
# the test binaries (rpath $$ORIGIN). Harmless on a normal install.
SAN = -g -fsanitize=address,undefined -fno-sanitize-recover=undefined \
  $(if $(wildcard /usr/bin/ld),-B/usr/bin) -Wl,-rpath,'$$ORIGIN'
B = tests/build

all: ov_sqlite.o ov_http.o

ov_sqlite.o: ov_sqlite.c ov_sqlite.h
	$(CC) $(CFLAGS) -c ov_sqlite.c -o $@

ov_http.o: ov_http.c ov_http.h
	$(CC) $(CFLAGS) -c ov_http.c -o $@

$(B):
	mkdir -p $(B)

$(B)/.sanlibs: | $(B)
	@for l in asan ubsan; do f=$$(realpath $$($(CC) -print-file-name=lib$$l.so)); \
	  so=$$(readelf -d $$f | sed -n 's/.*SONAME.*\[\(.*\)\]/\1/p'); \
	  [ -e /usr/lib64/$$so ] || [ -e /lib64/$$so ] || ln -sf $$f $(B)/$$so; done; touch $@

$(B)/test_sqlite: ov_sqlite.c ov_sqlite.h tests/test_sqlite.c | $(B)/.sanlibs
	$(CC) $(CFLAGS) $(SAN) -I. ov_sqlite.c tests/test_sqlite.c -lsqlite3 -lgc -o $@

$(B)/test_http: ov_http.c ov_http.h tests/test_http.c | $(B)/.sanlibs
	$(CC) $(CFLAGS) $(SAN) -I. ov_http.c tests/test_http.c -lgc -o $@

test: $(B)/test_sqlite $(B)/test_http
	ASAN_OPTIONS=detect_leaks=0 $(B)/test_sqlite
	ASAN_OPTIONS=detect_leaks=0 $(B)/test_http

clean:
	rm -rf $(B) *.o

.PHONY: all test clean
