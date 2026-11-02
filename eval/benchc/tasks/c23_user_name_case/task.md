# Case-insensitive user names

User names are matched exactly, so `Alice` can be created next to `alice`, and `assignee=Alice`
is `no such user: Alice`. People also like to write `@alice`. Normalise user names wherever a
request carries one:

1. clean the field as today (it is a line field, so it is trimmed);
2. if it then starts with `@`, drop that one `@` (only one: `@@bob` keeps `@bob`);
3. lower-case ASCII letters (`A` to `Z` become `a` to `z`; every other character stays as it is).

A value that is empty after this counts as empty, exactly like an empty field does today.

The fields: `name` on `POST /users`; `assignee` on `POST /p/{KEY}/issues`; `assignee` on
`POST /p/{KEY}/{N}/assign`; `author` on `POST /p/{KEY}/{N}/comments`; and the `assignee` filter of
`GET /p/{KEY}` and `GET /p/{KEY}/issues.txt`. The checks of each route and their order do not
change; they now see the normalised value, and every message that quotes a name quotes the
normalised one. So on `POST /users`, `name=Alice` when `alice` exists is 422 `user exists`,
`name=@` is 422 `name required` and `name=@@bob` is 422 `bad name`; `assignee=@Zed` is 422
`no such user: zed`; on the assign route `assignee=@` unassigns; and `?assignee=@BOB` filters on
`bob`. The project page's filter form shows the normalised filter value (and the pager links carry
it).

Names already stored are assumed to be lower case (no migration; a fresh database is fine).
