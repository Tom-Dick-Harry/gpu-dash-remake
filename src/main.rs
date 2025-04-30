#[link(name = "win_sysinfo", kind = "dylib")]
extern "C" {
    fn sysinfo_get_uptime() -> f64;
}

use tiny_http::{Server, Response};

fn main() {
    let server = Server::http("0.0.0.0:8000").unwrap();
    println!("Listening on http://localhost:8000/uptime");
    for request in server.incoming_requests() {
        if request.url() == "/uptime" {
            let uptime = unsafe { sysinfo_get_uptime() };
            let body = format!("{{\"uptime_seconds\":{:.2}}}", uptime);
            let response = Response::from_string(body)
                .with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
            let _ = request.respond(response);
        } else {
            let response = Response::from_string("Not Found").with_status_code(404);
            let _ = request.respond(response);
        }
    }
}