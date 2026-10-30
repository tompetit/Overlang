"""Unit tests, port of the three `test` blocks in main.ov. Run: python3 -I -m unittest"""
import unittest

from main import Todo, parse_rows, render_page, route


class TestTodo(unittest.TestCase):
    def test_parse_rows_decodes_rows(self):
        text = "id\x1ftitle\x1fdone\x1fcreated_at\x1e1\x1fmilk\x1f0\x1f2026-01-31 10:00:00\x1e2\x1feggs, bread\x1f1\x1f2026-02-01 11:30:00"
        self.assertEqual(parse_rows(text), [Todo(id=1, title="milk", done=False, created_at="2026-01-31 10:00:00"), Todo(id=2, title="eggs, bread", done=True, created_at="2026-02-01 11:30:00")])
        self.assertEqual(parse_rows("id\x1ftitle\x1fdone\x1fcreated_at"), [])

    def test_render_page_escapes_titles(self):
        html = render_page([Todo(id=3, title='<b>x</b> & "y"', done=False, created_at="2026-01-31 10:00:00")])
        self.assertNotIn("<b>x</b>", html)
        self.assertIn("&lt;b&gt;x&lt;/b&gt; &amp; &quot;y&quot;", html)
        self.assertIn('name="id" value="3"', html)

    def test_route_unknown_is_404(self):
        r = route(None, "GET", "/nope", "")
        self.assertEqual(r.status, 404)
        self.assertEqual(route(None, "POST", "/", "").status, 404)


if __name__ == "__main__":
    unittest.main()
