use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tiny_http::{Server, Response};
use serde::Serialize;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TelemetryInfo {
    pub uptime: f64,
    pub user_time: f64,
    pub system_time: f64,
    pub idle_time: f64,
}

#[link(name = "win_sysinfo", kind = "dylib")]
extern "C" {
    fn sysinfo_get_telemetry(out: *mut TelemetryInfo) -> i32;
}

#[derive(Serialize, Debug, Clone)]
struct TelemetryJson {
    uptime: f64,
    user_time: f64,
    system_time: f64,
    idle_time: f64,
}

fn fetch_telemetry() -> Option<TelemetryJson> {
    let mut info = TelemetryInfo {
        uptime: 0.0,
        user_time: 0.0,
        system_time: 0.0,
        idle_time: 0.0,
    };
    let ok = unsafe { sysinfo_get_telemetry(&mut info as *mut TelemetryInfo) };
    if ok == 1 {
        Some(TelemetryJson {
            uptime: info.uptime,
            user_time: info.user_time,
            system_time: info.system_time,
            idle_time: info.idle_time,
        })
    } else {
        None
    }
}

fn main() {
    let telemetry = Arc::new(Mutex::new(TelemetryJson {
        uptime: 0.0,
        user_time: 0.0,
        system_time: 0.0,
        idle_time: 0.0,
    }));
    let telemetry_bg = telemetry.clone();
    thread::spawn(move || {
        loop {
            if let Some(new_data) = fetch_telemetry() {
                let mut t = telemetry_bg.lock().unwrap();
                *t = new_data;
            }
            thread::sleep(Duration::from_secs(1));
        }
    });

    let server = Server::http("0.0.0.0:8000").unwrap();
    println!("Listening on http://localhost:8000/telemetry");
    for request in server.incoming_requests() {
        match request.url() {
            "/telemetry" => {
                let t = telemetry.lock().unwrap().clone();
                let body = serde_json::to_string(&t).unwrap();
                let response = Response::from_string(body)
                    .with_header(tiny_http::Header::from_bytes(
                        &b"Content-Type"[..], &b"application/json"[..]
                    ).unwrap());
                let _ = request.respond(response);
            },
            _ => {
                let response = Response::from_string("Not Found").with_status_code(404);
                let _ = request.respond(response);
            }
        }
    }
}