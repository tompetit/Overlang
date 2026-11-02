# Label name normalisation and duplicates

Label names must be lower case with dashes (`bug`, `needs-review`), which keeps tripping people
up: `name=Bug` is `bad label`, `labels=Needs Review` is `no such label: Needs Review`. Normalise
every label name a request carries:

1. clean the field as today (a line field: `\r` and `\n` become spaces, then trimmed);
2. lower-case ASCII letters (`A` to `Z` become `a` to `z`; every other character stays as it is);
3. replace each run of one or more spaces with a single `-` (`Needs  Review` becomes
   `needs-review`; `a - b` becomes `a---b`).

The normalised name is then used for everything: validation (`valid_label_name` as today, so the
20 code point limit applies to the normalised name), lookups, storage and messages.

The fields: `name` on `POST /p/{KEY}/labels` (checks unchanged: `name required`, `bad label`,
`label exists`, so `name=BUG` when `bug` exists is 422 `label exists`); each name in `labels` on
`POST /p/{KEY}/issues` (split on `,`, each piece trimmed, empty pieces dropped, as today, then
normalised); `label` on `POST /p/{KEY}/{N}/labels` and `POST /p/{KEY}/{N}/labels/remove`; and the
`label` filter of `GET /p/{KEY}` and `GET /p/{KEY}/issues.txt` (the filter form shows the
normalised value). Messages quote the normalised name: 422 `no such label: needs-review`.

**Duplicates on create.** On `POST /p/{KEY}/issues` the label names are checked one by one in the
order given, after the title, priority and assignee checks as today: a name that is not a label of
the project is 422 `no such label: {name}`, and a name equal to an earlier name in the same list
(after normalisation) is 422 `duplicate label: {name}`; the first failing name decides. So
`bug,nope,BUG` is `no such label: nope` and `bug,BUG,nope` is `duplicate label: bug`.

Existing rows are not migrated (a fresh database is fine).
