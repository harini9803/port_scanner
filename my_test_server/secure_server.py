from flask import Flask, make_response

app = Flask(__name__)

@app.route("/")
def index():
    html = "<h1>Hello, Secure World</h1>"
    response = make_response(html)
    
    response.headers["X-Frame-Options"] = "DENY"
    response.headers["X-Content-Type-Options"] = "nosniff"
    response.headers["X-XSS-Protection"] = "1; mode=block"
    response.headers["Strict-Transport-Security"] = "max-age=63072000; includeSubDomains"
    response.headers["Content-Security-Policy"] = "default-src 'self'"
    
    return response

app.run(host="0.0.0.0", port=8080)
