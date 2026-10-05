mod claude;
mod engine;
mod http;
mod procfs;
mod sessions;
mod socket;
mod store;

use std::fs;
use std::io;
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};
use std::time::{Duration, Instant, SystemTime};

use rog_hid::{Frame, HidSink, HidrawDevice, Profile, Rgb};

use engine::Engine;
use store::Store;

const FRAME_INTERVAL: Duration = Duration::from_millis(50);
const SWEEP_INTERVAL: Duration = Duration::from_secs(1);
const SAVE_INTERVAL: Duration = Duration::from_secs(1);
const DEVICE_RETRY: Duration = Duration::from_secs(1);
/// Fraction of the remaining distance to the target covered per frame (smooth fades between effects).
const SMOOTHING: f32 = 0.4;

fn restore(profile: &Profile, dev: &mut impl HidSink) -> io::Result<()> {
    profile.restore.iter().try_for_each(|p| dev.write(p))
}

fn open_device(profile: &Profile) -> io::Result<HidrawDevice> {
    let mut dev = HidrawDevice::open(profile)?;
    dev.write(&profile.packet.init)?;
    Ok(dev)
}

/// asusd writes the same hidraw; two writers fight over the LEDs.
fn warn_if_asusd_running() {
    let running = fs::read_dir("/proc").into_iter().flatten().flatten().any(|e| {
        fs::read_to_string(e.path().join("comm")).is_ok_and(|c| c.trim() == "asusd")
    });
    if running {
        eprintln!("asusd is running and also drives the Aura device; stop its aura control or LEDs will flicker");
    }
}

fn main() -> io::Result<()> {
    let profile = Arc::new(Profile::g614ju());
    let path = socket::path();
    if UnixStream::connect(&path).is_ok() {
        return Err(io::Error::new(io::ErrorKind::AddrInUse, "another rogd is already running"));
    }
    warn_if_asusd_running();
    let listener = socket::bind(&path)?;

    let store = Store::new(Store::default_dir());
    let engine = Arc::new(Mutex::new(store.load()));
    let running = Arc::new(AtomicBool::new(true));
    {
        let running = running.clone();
        ctrlc::set_handler(move || running.store(false, Ordering::SeqCst)).map_err(io::Error::other)?;
    }
    spawn_listener(listener, engine.clone());
    http::spawn(http::AppState { engine: engine.clone(), profile: profile.clone(), start: Instant::now() });

    let start = Instant::now();
    let mut shown = vec![[0.0_f32; 3]; profile.leds.len()];
    let (mut dev, mut retry_at, mut warned) = (None::<HidrawDevice>, start, false);
    let (mut last_sweep, mut last_save, mut unsaved) = (start, start, false);
    while running.load(Ordering::SeqCst) {
        let now = Instant::now();
        let targets = {
            let mut engine = engine.lock().unwrap_or_else(|e| e.into_inner());
            if now.duration_since(last_sweep) >= SWEEP_INTERVAL {
                last_sweep = now;
                engine.sweep(now, SystemTime::now(), procfs::claude_alive, procfs::transcript_interrupted);
            }
            unsaved |= engine.take_dirty();
            if unsaved && now.duration_since(last_save) >= SAVE_INTERVAL {
                (unsaved, last_save) = (false, now);
                if let Err(e) = store.save(&engine) {
                    eprintln!("could not save state ({e})");
                }
            }
            engine.targets(&profile, now - start, now)
        };

        let mut frame = Frame::new(&profile);
        for (i, (cur, target)) in shown.iter_mut().zip(&targets).enumerate() {
            for (c, t) in cur.iter_mut().zip([target.r, target.g, target.b]) {
                *c += (f32::from(t) - *c) * SMOOTHING;
            }
            frame.set(i, Rgb::new(cur[0].round() as u8, cur[1].round() as u8, cur[2].round() as u8));
        }

        if dev.is_none() && now >= retry_at {
            match open_device(&profile) {
                Ok(d) => {
                    eprintln!("aura device connected");
                    (dev, warned) = (Some(d), false);
                }
                Err(e) => {
                    if !warned {
                        eprintln!("aura device unavailable ({e}); retrying every {}s", DEVICE_RETRY.as_secs());
                        warned = true;
                    }
                    retry_at = now + DEVICE_RETRY;
                }
            }
        }
        if let Some(d) = dev.as_mut() {
            if let Err(e) = d.write(&frame.packet(&profile)) {
                eprintln!("aura write failed ({e}); reconnecting");
                (dev, retry_at) = (None, now + DEVICE_RETRY);
            }
        }
        sleep(FRAME_INTERVAL);
    }

    {
        let mut engine = engine.lock().unwrap_or_else(|e| e.into_inner());
        if unsaved || engine.take_dirty() {
            let _ = store.save(&engine);
        }
    }
    if let Some(mut d) = dev.or_else(|| HidrawDevice::open(&profile).ok()) {
        let _ = restore(&profile, &mut d);
    }
    let _ = fs::remove_file(&path);
    Ok(())
}

fn spawn_listener(listener: UnixListener, engine: Arc<Mutex<Engine>>) {
    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let engine = engine.clone();
            thread::spawn(move || socket::serve(stream, &engine));
        }
    });
}
