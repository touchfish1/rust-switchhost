use std::time::{Duration, Instant};

use serde::Serialize;
use sysinfo::{Networks, System, MINIMUM_CPU_UPDATE_INTERVAL};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIcon, TrayIconBuilder},
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalRect, Position, Runtime, WebviewUrl,
    WebviewWindow, WebviewWindowBuilder,
};

const METRICS_REFRESH_INTERVAL: Duration = Duration::from_secs(2);
const METRICS_WINDOW_LABEL: &str = "tray-metrics";
const METRICS_WINDOW_WIDTH: f64 = 320.0;
const METRICS_WINDOW_HEIGHT: f64 = 188.0;
const METRICS_WINDOW_MARGIN: i32 = 14;

pub struct TrayController<R: Runtime> {
    tray: TrayIcon<R>,
    cpu_item: MenuItem<R>,
    memory_item: MenuItem<R>,
    network_item: MenuItem<R>,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct MetricsSnapshot {
    cpu_usage: f32,
    used_memory: u64,
    total_memory: u64,
    download_bytes_per_sec: u64,
    upload_bytes_per_sec: u64,
}

impl MetricsSnapshot {
    fn memory_percent(self) -> u64 {
        if self.total_memory == 0 {
            return 0;
        }

        ((self.used_memory as f64 / self.total_memory as f64) * 100.0).round() as u64
    }

    fn cpu_menu_text(self) -> String {
        format!("CPU: {}%", self.cpu_usage.round() as u64)
    }

    fn memory_menu_text(self) -> String {
        format!(
            "内存: {} / {} ({}%)",
            format_bytes(self.used_memory),
            format_bytes(self.total_memory),
            self.memory_percent()
        )
    }

    fn network_menu_text(self) -> String {
        format!(
            "网络: ↓{} ↑{}",
            format_rate(self.download_bytes_per_sec),
            format_rate(self.upload_bytes_per_sec)
        )
    }
}

impl<R: Runtime> TrayController<R> {
    fn update_metrics(&self, snapshot: MetricsSnapshot) {
        let _ = self.cpu_item.set_text(snapshot.cpu_menu_text());
        let _ = self.memory_item.set_text(snapshot.memory_menu_text());
        let _ = self.network_item.set_text(snapshot.network_menu_text());
        let _ = self.tray.app_handle().emit("metrics-updated", snapshot);
    }
}

pub fn setup_tray<R: Runtime>(
    app: &AppHandle<R>,
) -> Result<TrayController<R>, Box<dyn std::error::Error>> {
    let toggle_metrics = MenuItem::with_id(app, "toggle_metrics", "性能浮窗", true, None::<&str>)?;
    let cpu_item = MenuItem::with_id(app, "metrics_cpu", "CPU: --", false, None::<&str>)?;
    let memory_item = MenuItem::with_id(app, "metrics_memory", "内存: --", false, None::<&str>)?;
    let network_item = MenuItem::with_id(app, "metrics_network", "网络: --", false, None::<&str>)?;
    let show = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &toggle_metrics,
            &cpu_item,
            &memory_item,
            &network_item,
            &show,
            &quit,
        ],
    )?;

    ensure_metrics_window(app)?;

    let mut builder = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                button_state: tauri::tray::MouseButtonState::Up,
                position,
                ..
            } = event
            {
                toggle_metrics_window(tray.app_handle(), Some(position));
            }
        })
        .on_menu_event(|app, event| match event.id.as_ref() {
            "toggle_metrics" => {
                toggle_metrics_window(app, None);
            }
            "quit" => {
                app.exit(0);
            }
            "show" => {
                show_main_window(app);
            }
            _ => {}
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    let tray = builder.build(app)?;

    Ok(TrayController {
        tray,
        cpu_item,
        memory_item,
        network_item,
    })
}

pub fn start_metrics_monitor<R: Runtime + 'static>(controller: TrayController<R>) {
    tauri::async_runtime::spawn(async move {
        let mut system = System::new();
        let mut networks = Networks::new_with_refreshed_list();
        let interval_duration = metrics_refresh_interval();

        system.refresh_memory();
        system.refresh_cpu_usage();

        let initial_snapshot = MetricsSnapshot {
            cpu_usage: system.global_cpu_usage(),
            used_memory: system.used_memory(),
            total_memory: system.total_memory(),
            download_bytes_per_sec: 0,
            upload_bytes_per_sec: 0,
        };
        controller.update_metrics(initial_snapshot);

        let mut interval = tokio::time::interval(interval_duration);
        let mut last_refresh_at = Instant::now();

        loop {
            interval.tick().await;

            let elapsed_secs = last_refresh_at.elapsed().as_secs_f64().max(1.0);
            last_refresh_at = Instant::now();

            system.refresh_memory();
            system.refresh_cpu_usage();
            networks.refresh(true);

            let received_bytes = networks
                .iter()
                .map(|(_, network)| network.received())
                .sum::<u64>();
            let transmitted_bytes = networks
                .iter()
                .map(|(_, network)| network.transmitted())
                .sum::<u64>();

            let snapshot = MetricsSnapshot {
                cpu_usage: system.global_cpu_usage(),
                used_memory: system.used_memory(),
                total_memory: system.total_memory(),
                download_bytes_per_sec: (received_bytes as f64 / elapsed_secs).round() as u64,
                upload_bytes_per_sec: (transmitted_bytes as f64 / elapsed_secs).round() as u64,
            };

            controller.update_metrics(snapshot);
        }
    });
}

fn ensure_metrics_window<R: Runtime>(
    app: &AppHandle<R>,
) -> Result<WebviewWindow<R>, Box<dyn std::error::Error>> {
    if let Some(window) = app.get_webview_window(METRICS_WINDOW_LABEL) {
        return Ok(window);
    }

    let window = WebviewWindowBuilder::new(
        app,
        METRICS_WINDOW_LABEL,
        WebviewUrl::App("index.html?view=tray-metrics".into()),
    )
    .title("性能浮窗")
    .inner_size(METRICS_WINDOW_WIDTH, METRICS_WINDOW_HEIGHT)
    .resizable(false)
    .maximizable(false)
    .minimizable(false)
    .closable(true)
    .visible(false)
    .focused(false)
    .decorations(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .shadow(true)
    .build()?;

    Ok(window)
}

fn toggle_metrics_window<R: Runtime>(app: &AppHandle<R>, anchor: Option<PhysicalPosition<f64>>) {
    let Ok(window) = ensure_metrics_window(app) else {
        return;
    };

    let is_visible = window.is_visible().unwrap_or(false);
    if is_visible {
        let _ = window.hide();
        return;
    }

    position_metrics_window(app, &window, anchor);
    let _ = window.show();
    let _ = window.set_focus();
}

fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(metrics_window) = app.get_webview_window(METRICS_WINDOW_LABEL) {
        let _ = metrics_window.hide();
    }

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn position_metrics_window<R: Runtime>(
    app: &AppHandle<R>,
    window: &WebviewWindow<R>,
    anchor: Option<PhysicalPosition<f64>>,
) {
    let monitor = anchor
        .and_then(|point| app.monitor_from_point(point.x, point.y).ok().flatten())
        .or_else(|| {
            app.cursor_position()
                .ok()
                .and_then(|cursor| app.monitor_from_point(cursor.x, cursor.y).ok().flatten())
        })
        .or_else(|| app.primary_monitor().ok().flatten());

    let Some(monitor) = monitor else {
        return;
    };

    let work_area = monitor.work_area();
    let scale_factor = monitor.scale_factor();
    let width = (METRICS_WINDOW_WIDTH * scale_factor).round() as i32;
    let height = (METRICS_WINDOW_HEIGHT * scale_factor).round() as i32;
    let margin = (f64::from(METRICS_WINDOW_MARGIN) * scale_factor).round() as i32;

    let (target_x, target_y) = match anchor {
        Some(point) => anchored_position(point, work_area, width, height, margin),
        None => fallback_corner_position(work_area, width, height, margin),
    };

    let _ = window.set_position(Position::Physical(PhysicalPosition::new(
        target_x, target_y,
    )));
}

fn anchored_position(
    point: PhysicalPosition<f64>,
    work_area: &PhysicalRect<i32, u32>,
    width: i32,
    height: i32,
    margin: i32,
) -> (i32, i32) {
    let left = work_area.position.x + margin;
    let top = work_area.position.y + margin;
    let right = work_area.position.x + work_area.size.width as i32 - width - margin;
    let bottom = work_area.position.y + work_area.size.height as i32 - height - margin;
    let center_x = work_area.position.x + work_area.size.width as i32 / 2;
    let center_y = work_area.position.y + work_area.size.height as i32 / 2;

    let point_x = point.x.round() as i32;
    let point_y = point.y.round() as i32;

    let x = if point_x >= center_x {
        point_x - width
    } else {
        point_x
    }
    .clamp(left, right.max(left));

    let y = if point_y >= center_y {
        point_y - height - margin
    } else {
        point_y + margin
    }
    .clamp(top, bottom.max(top));

    (x, y)
}

fn fallback_corner_position(
    work_area: &PhysicalRect<i32, u32>,
    width: i32,
    height: i32,
    margin: i32,
) -> (i32, i32) {
    let x = work_area.position.x + work_area.size.width as i32 - width - margin;

    #[cfg(target_os = "macos")]
    let y = work_area.position.y + margin;

    #[cfg(not(target_os = "macos"))]
    let y = work_area.position.y + work_area.size.height as i32 - height - margin;

    (x, y)
}

fn metrics_refresh_interval() -> Duration {
    if METRICS_REFRESH_INTERVAL > MINIMUM_CPU_UPDATE_INTERVAL {
        METRICS_REFRESH_INTERVAL
    } else {
        MINIMUM_CPU_UPDATE_INTERVAL
    }
}

fn format_bytes(bytes: u64) -> String {
    format_with_units(bytes, false)
}

fn format_rate(bytes_per_sec: u64) -> String {
    format!("{}/s", format_with_units(bytes_per_sec, true))
}

fn format_with_units(value: u64, compact: bool) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

    let mut unit_index = 0;
    let mut scaled = value as f64;

    while scaled >= 1024.0 && unit_index < UNITS.len() - 1 {
        scaled /= 1024.0;
        unit_index += 1;
    }

    if compact {
        if scaled >= 100.0 || unit_index == 0 {
            format!("{scaled:.0} {}", UNITS[unit_index])
        } else if scaled >= 10.0 {
            format!("{scaled:.1} {}", UNITS[unit_index])
        } else {
            format!("{scaled:.2} {}", UNITS[unit_index])
        }
    } else if scaled >= 10.0 || unit_index == 0 {
        format!("{scaled:.1} {}", UNITS[unit_index])
    } else {
        format!("{scaled:.2} {}", UNITS[unit_index])
    }
}
