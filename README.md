# IpawsAlert — Rust Rewrite

A complete rewrite of the C#/.NET WPF IPAWS Alert Console in **Rust**,
using [egui](https://github.com/emilk/egui) / [eframe](https://github.com/emilk/egui/tree/master/crates/eframe)
for the GUI. Ships as a single native binary with no runtime dependencies.

---

## Project Layout

```
IpawsAlert-Rust/
├── Cargo.toml                  Workspace root
├── ipaws_core/                 Core library crate (no GUI)
│   └── src/
│       ├── lib.rs
│       ├── models.rs           CapAlert, AlertInfo, AlertArea, all CAP enums
│       ├── builder.rs          Fluent builder API
│       ├── xml.rs              CAP v1.2 XML serializer
│       ├── validator.rs        Schema + IPAWS + channel validation rules
│       ├── channels.rs         IpawsConfig, endpoint URL constants
│       └── client.rs           mTLS HTTP client (reqwest + native-tls)
└── ipaws_gui/                  Desktop GUI crate
    └── src/
        ├── main.rs             Entry point
        ├── app.rs              App struct, tab routing, background send thread
        ├── theme.rs            Dark colour palette + egui Visuals setup
        ├── compose.rs          Compose Alert tab
        ├── history.rs          Submission History tab
        ├── settings_tab.rs     Settings tab
        └── settings_store.rs   JSON persistence (~/.config/IpawsAlert/)
```

---

## Prerequisites

| Platform | Requirements |
|----------|-------------|
| **Windows** | Rust 1.75+, Visual Studio Build Tools (MSVC linker) |
| **Linux**   | Rust 1.75+, `libssl-dev pkg-config libgtk-3-dev` |
| **macOS**   | Rust 1.75+, Xcode Command Line Tools |

Install Rust: https://rustup.rs

---

## Building

```bash
# Debug build
cargo build

# Optimised release binary (stripped, LTO)
cargo build --release

# Run directly
cargo run -p ipaws_gui
```

## Testing

The core library ships with a comprehensive test suite covering the CAP XML
serializer, IPAWS validator, fluent builder, and response-body parsers:

```bash
cargo test -p ipaws_core
```

Test coverage:
- **14 serializer tests** — XML prolog, CAP 1.2 namespace, element ordering,
  XML escaping, WEA/EAS routing, SAME geocode rendering, character truncation,
  multilingual info blocks.
- **17 validator tests** — every CAP/IPAWS/WEA/EAS/NWEM error code
  (CAP001–CAP014, IPAWS001, WEA001–WEA005, EAS001, NWEM001).
- **7 builder tests** — UUID generation, code accumulation, references format,
  area auto-creation, channel routing.
- **8 client unit tests** — server-ID extraction from response XML,
  error-body parsing, edge cases (empty body, malformed XML, long excerpts).

All tests run without network access.

Release binary locations:
- **Windows**: `target/release/ipaws_alert.exe`
- **Linux / macOS**: `target/release/ipaws_alert`

---

## Library Quick Start

```rust
use ipaws_core::*;

let alert = CapAlertBuilder::new()
    .with_sender("alerts@myagency.gov")
    .with_status(CapStatus::Test)
    .with_msg_type(CapMsgType::Alert)
    .add_info(|info| info
        .with_event("Tornado Warning")
        .with_urgency(CapUrgency::Immediate)
        .with_severity(CapSeverity::Extreme)
        .with_certainty(CapCertainty::Observed)
        .with_headline("Tornado Warning until 6:00 PM CDT")
        .with_description("A tornado has been confirmed on the ground near Example County.")
        .with_instruction("Take shelter immediately in a sturdy interior room.")
        .add_wea_routing(
            Some("Tornado Warning until 6PM. Shelter now!"),
            Some("Tornado Warning for Example County until 6:00 PM CDT."),
        )
        .add_eas_routing()
        .add_area(|area| area
            .with_description("Example County, OH")
            .add_same_code("039049")
        )
    )
    .build();

// Validate
let result = CapValidator::validate(&alert);
if !result.is_valid() {
    eprintln!("{}", result.summary());
    std::process::exit(1);
}

// Serialize to CAP XML
let xml = ipaws_core::serialize(&alert, true);
println!("{xml}");

// Submit to IPAWS-OPEN
let config = IpawsConfig {
    cog_id:            "YOUR-COG-ID".into(),
    sender:            "alerts@myagency.gov".into(),
    use_test_endpoint: true,
    use_file_cert:     true,
    cert_path:         "certs/ipaws-test.p12".into(),
    cert_password:     std::env::var("IPAWS_CERT_PASS").unwrap_or_default(),
    ..Default::default()
};

let client   = IpawsClient::new(config);
let response = client.submit(&alert);

if response.is_success {
    println!("Accepted! Server ID: {}", response.server_message_id.unwrap_or_default());
} else {
    eprintln!("Failed: {}", response.errors.join(", "));
}
```

---

## Settings

Persisted to:
- **Windows**: `%APPDATA%\IpawsAlert\settings.json`
- **Linux / macOS**: `~/.config/IpawsAlert/settings.json`

> The certificate **password** is never "saved" this is on purpose, it serves as an extra "check".

---

## Key Differences from the C# Version

| Aspect | C# / WPF | Rust / egui |
|---|---|---|
| GUI paradigm | MVVM / retained-mode XAML | Immediate-mode (egui) |
| Distribution | Requires .NET runtime | Single native binary |
| Certificate API | `X509CertificateLoader` | reqwest + native-tls |
| Settings | `System.Text.Json` | serde\_json |
| Async | `Task` / `async-await` | `std::thread` + `Mutex` |
| Dark theme | System colour key overrides | `egui::Visuals` |

---

## IPAWS-OPEN Endpoints

| Environment | URL |
|---|---|
| Test (JITC) | `https://tdl.integration.aws.fema.net/cap/SubmitCAPMessage` |
| Production | `https://www.fema.gov/cap/COGProfile.do` |

---

## References

- [CAP v1.2 Specification — OASIS](http://docs.oasis-open.org/emergency/cap/v1.2/CAP-v1.2-os.html)
- [IPAWS Developer Resources — FEMA](https://www.fema.gov/emergency-managers/practitioners/integrated-public-alert-warning-system/developers)
- [egui documentation](https://docs.rs/egui)
- [eframe documentation](https://docs.rs/eframe)
