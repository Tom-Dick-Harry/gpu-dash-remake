use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tiny_http::{Server, Response};
use serde::Serialize;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MemoryInfo {
    pub total_phys: u64,
    pub free_phys: u64,
    pub memory_load: u32,
    pub total_virtual: u64,
    pub free_virtual: u64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DiskInfo {
    pub mount: [u8; 4],
    pub total_space: u64,
    pub free_space: u64,
    pub used_space: u64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TelemetryInfoFull {
    pub uptime: f64,
    pub user_time: f64,
    pub system_time: f64,
    pub idle_time: f64,
    pub mem: MemoryInfo,
    pub disks: [DiskInfo; 26],
    pub num_disks: i32,
}

#[link(name = "win_sysinfo", kind = "dylib")]
extern "C" {
    fn sysinfo_get_telemetry_full(out: *mut TelemetryInfoFull) -> i32;
}

#[derive(Serialize, Debug, Clone)]
struct MemoryJson {
    total_phys: u64,
    free_phys: u64,
    memory_load: u32,
    total_virtual: u64,
    free_virtual: u64,
}

#[derive(Serialize, Debug, Clone)]
struct DiskJson {
    mount: String,
    total_space: u64,
    free_space: u64,
    used_space: u64,
}

#[derive(Serialize, Debug, Clone)]
struct TelemetryJsonFull {
    uptime: f64,
    user_time: f64,
    system_time: f64,
    idle_time: f64,
    mem: MemoryJson,
    disks: Vec<DiskJson>,
}

fn fetch_telemetry_full() -> Option<TelemetryJsonFull> {
    let mut info = TelemetryInfoFull {
        uptime: 0.0,
        user_time: 0.0,
        system_time: 0.0,
        idle_time: 0.0,
        mem: MemoryInfo {
            total_phys: 0,
            free_phys: 0,
            memory_load: 0,
            total_virtual: 0,
            free_virtual: 0,
        },
        disks: [DiskInfo {
            mount: [0; 4],
            total_space: 0,
            free_space: 0,
            used_space: 0,
        }; 26],
        num_disks: 0,
    };
    let ok = unsafe { sysinfo_get_telemetry_full(&mut info as *mut TelemetryInfoFull) };
    if ok == 1 {
        let mem = MemoryJson {
            total_phys: info.mem.total_phys,
            free_phys: info.mem.free_phys,
            memory_load: info.mem.memory_load,
            total_virtual: info.mem.total_virtual,
            free_virtual: info.mem.free_virtual,
        };
        let disks = info.disks[..info.num_disks as usize].iter().map(|d| DiskJson {
            mount: String::from_utf8_lossy(&d.mount).trim_end_matches('\0').to_string(),
            total_space: d.total_space,
            free_space: d.free_space,
            used_space: d.used_space,
        }).collect();
        Some(TelemetryJsonFull {
            uptime: info.uptime,
            user_time: info.user_time,
            system_time: info.system_time,
            idle_time: info.idle_time,
            mem,
            disks,
        })
    } else {
        None
    }
}

fn main() {
    let telemetry = Arc::new(Mutex::new(TelemetryJsonFull {
        uptime: 0.0,
        user_time: 0.0,
        system_time: 0.0,
        idle_time: 0.0,
        mem: MemoryJson {
            total_phys: 0,
            free_phys: 0,
            memory_load: 0,
            total_virtual: 0,
            free_virtual: 0,
        },
        disks: vec![],
    }));
    let telemetry_bg = telemetry.clone();
    thread::spawn(move || {
        loop {
            if let Some(new_data) = fetch_telemetry_full() {
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