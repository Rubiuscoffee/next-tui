use ndev::analyzer::RouteTree;
use ndev::runner::parser::{LogKind, LogParser};
use ndev::system::cache::format_bytes;
use ndev::system::env_checker::check_env_status;
use std::fs;

#[test]
fn test_log_parser_local_url_and_port() {
    let parser = LogParser::new();
    let line = "   - Local:        http://localhost:3000";
    let (_log, meta) = parser.parse_line(line);

    assert_eq!(meta.local_url, Some("http://localhost:3000".to_string()));
    assert_eq!(meta.port, Some(3000));
}

#[test]
fn test_log_parser_network_url() {
    let parser = LogParser::new();
    let line = "   - Network:      http://192.168.1.15:3000";
    let (_log, meta) = parser.parse_line(line);

    assert_eq!(meta.network_url, Some("http://192.168.1.15:3000".to_string()));
}

#[test]
fn test_log_parser_version_and_turbopack() {
    let parser = LogParser::new();
    let line = " ▲ Next.js 16.0.0 (Turbopack)";
    let (_log, meta) = parser.parse_line(line);

    assert_eq!(meta.next_version, Some("v16.0.0".to_string()));
    assert_eq!(meta.turbopack, Some(true));
}

#[test]
fn test_log_parser_turbopack_compile_time() {
    let parser = LogParser::new();
    let line = "17:14:02 [Turbopack] Compiled /page in 42ms";
    let (log, _meta) = parser.parse_line(line);

    assert_eq!(log.kind, LogKind::Turbopack);
    assert!(log.message.contains("42ms"));
}

#[test]
fn test_log_parser_http_requests() {
    let parser = LogParser::new();
    let line = "GET /dashboard/billing 200 in 18ms";
    let (log, _meta) = parser.parse_line(line);

    assert_eq!(
        log.kind,
        LogKind::Http {
            method: "GET".to_string(),
            status: 200
        }
    );
}

#[test]
fn test_log_parser_server_action() {
    let parser = LogParser::new();
    let line = "[Action] updateUserProfile called";
    let (log, _meta) = parser.parse_line(line);

    assert_eq!(log.kind, LogKind::Action("updateUserProfile".to_string()));
}

#[test]
fn test_format_bytes() {
    assert_eq!(format_bytes(500), "500 B");
    assert_eq!(format_bytes(2048), "2.0 KB");
    assert_eq!(format_bytes(48 * 1024 * 1024), "48 MB");
    assert_eq!(format_bytes(2 * 1024 * 1024 * 1024), "2.0 GB");
}

#[test]
fn test_env_checker_no_files() {
    let temp_dir = std::env::temp_dir().join("ndev_test_empty_env");
    let _ = fs::create_dir_all(&temp_dir);

    let status = check_env_status(&temp_dir);
    assert_eq!(status.loaded_count, 0);
    assert!(status.missing_keys.is_empty());
    assert!(status.active_files.is_empty());

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_env_checker_real_diff() {
    let temp_dir = std::env::temp_dir().join("ndev_test_real_diff");
    let _ = fs::create_dir_all(&temp_dir);

    fs::write(
        temp_dir.join(".env.example"),
        "DATABASE_URL=\nSTRIPE_SECRET_KEY=\nNEXT_PUBLIC_API=\n",
    )
    .unwrap();

    fs::write(
        temp_dir.join(".env.local"),
        "DATABASE_URL=postgresql://localhost:5432/db\nNEXT_PUBLIC_API=http://localhost:3000\n",
    )
    .unwrap();

    let status = check_env_status(&temp_dir);
    assert_eq!(status.loaded_count, 2);
    assert_eq!(status.missing_keys, vec!["STRIPE_SECRET_KEY".to_string()]);
    assert_eq!(status.example_file, Some(".env.example".to_string()));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_routes_scan_empty_returns_empty() {
    let temp_dir = std::env::temp_dir().join("ndev_test_empty_app");
    let _ = fs::create_dir_all(&temp_dir);

    let routes = RouteTree::scan(&temp_dir);
    assert!(routes.items.is_empty());
    assert_eq!(routes.app_dir, None);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_routes_scan_real_app_router() {
    let temp_dir = std::env::temp_dir().join("ndev_test_real_app");
    let _ = fs::create_dir_all(&temp_dir);

    let app_dir = temp_dir.join("app");
    let dashboard_dir = app_dir.join("dashboard").join("(overview)");
    let billing_dir = app_dir.join("dashboard").join("billing");

    fs::create_dir_all(&dashboard_dir).unwrap();
    fs::create_dir_all(&billing_dir).unwrap();

    fs::write(app_dir.join("page.tsx"), "export default function Page() { return <h1>Home</h1>; }").unwrap();
    fs::write(dashboard_dir.join("page.tsx"), "export default function Overview() { return <div>Overview</div>; }").unwrap();
    fs::write(
        billing_dir.join("actions.ts"),
        "\"use server\";\nexport async function updateCard(data: any) {}\n",
    )
    .unwrap();

    let routes = RouteTree::scan(&temp_dir);
    assert!(!routes.items.is_empty());

    let displays: Vec<&str> = routes.items.iter().map(|i| i.display.as_str()).collect();
    assert!(displays.iter().any(|d| d.contains("page.tsx [λ]")));
    assert!(displays.iter().any(|d| d.contains("dashboard")));
    assert!(displays.iter().any(|d| d.contains("(overview)")));
    assert!(displays.iter().any(|d| d.contains("* updateCard()")));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_log_parser_port_reassignment() {
    let parser = LogParser::new();
    let line = "Port 3000 is in use, using available port 3001 instead";
    let (log, meta) = parser.parse_line(line);

    assert_eq!(meta.port, Some(3001));
    assert_eq!(meta.local_url, Some("http://localhost:3001".to_string()));
    assert_eq!(log.kind, LogKind::Warn);
}

#[test]
fn test_routes_collapse_directory() {
    let temp_dir = std::env::temp_dir().join("ndev_test_collapse_app");
    let _ = fs::create_dir_all(&temp_dir);

    let app_dir = temp_dir.join("app");
    let dashboard_dir = app_dir.join("dashboard");
    fs::create_dir_all(&dashboard_dir).unwrap();
    fs::write(app_dir.join("page.tsx"), "export default function Page() {}").unwrap();
    fs::write(dashboard_dir.join("page.tsx"), "export default function Dashboard() {}").unwrap();

    let routes = RouteTree::scan(&temp_dir);
    let mut collapsed = std::collections::HashSet::new();

    let visible_initial = routes.visible_items(&collapsed);
    assert!(visible_initial.iter().any(|v| v.display.contains("▾ dashboard")));

    collapsed.insert(dashboard_dir);
    let visible_collapsed = routes.visible_items(&collapsed);
    assert!(visible_collapsed.iter().any(|v| v.display.contains("▸ dashboard")));
    assert!(!visible_collapsed.iter().any(|v| v.item.path.ends_with("dashboard/page.tsx")));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_logs_horizontal_scroll() {
    let mut app = ndev::app::AppState::new(std::env::temp_dir());
    assert_eq!(app.logs_horizontal_scroll, 0);

    app.scroll_logs_right(6);
    assert_eq!(app.logs_horizontal_scroll, 6);

    app.scroll_logs_right(10);
    assert_eq!(app.logs_horizontal_scroll, 16);

    app.scroll_logs_left(8);
    assert_eq!(app.logs_horizontal_scroll, 8);

    app.scroll_logs_left(20); // should saturate to 0
    assert_eq!(app.logs_horizontal_scroll, 0);

    app.scroll_logs_right(600); // capped at 500
    assert_eq!(app.logs_horizontal_scroll, 500);

    app.reset_logs_horizontal_scroll();
    assert_eq!(app.logs_horizontal_scroll, 0);
}
