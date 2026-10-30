# Split route by method

`route` is getting long. Split it: `route_get(db, path)` handles the GET routes and `route_post(db, path, body)`
handles the POST routes (each answers `404` with `not found` for a path it does not know, including a path that
exists only under the other method). `route(db, method, path, body)` keeps its signature and only dispatches on
the method: `GET` to `route_get`, `POST` to `route_post`, any other method is `404`. No change in behavior over
HTTP. The unit tests call `route_get` and `route_post` with a dummy database handle for the paths that never
reach the database.
