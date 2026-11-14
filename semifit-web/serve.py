from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer

class SinCache(SimpleHTTPRequestHandler):
    """Servidor de desarrollo: obliga al navegador a pedir siempre la versión actual."""
    def end_headers(self):
        self.send_header("Cache-Control", "no-store")
        super().end_headers()

print("semifit en http://127.0.0.1:8080")
ThreadingHTTPServer(("127.0.0.1", 8080), SinCache).serve_forever()
