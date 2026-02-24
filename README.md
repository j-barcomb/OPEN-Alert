# IpawsAlert.Core

A C# .NET 8 class library for building, validating, and submitting
**CAP v1.2** (Common Alerting Protocol) messages to the **IPAWS-OPEN** gateway.

Supports all four major IPAWS dissemination channels:
- **WEA** — Wireless Emergency Alerts (cell broadcast)
- **EAS** — Emergency Alert System (broadcast TV/radio)
- **NWEM** — National Weather Emergency Messages (NOAA Weather Radio)
- **IPAWS-OPEN** — CAP over HTTPS with mutual TLS

---

## Project Structure

```
IpawsAlert.Core/
├── Models/
│   ├── CapAlert.cs           # Root CAP v1.2 alert object
│   ├── CapEnums.cs           # Status, MsgType, Severity, Urgency, Certainty, etc.
│   ├── AlertInfo.cs          # CAP <info> block
│   ├── AlertArea.cs          # CAP <area> block (SAME codes, polygons, circles)
│   └── AlertResource.cs      # Optional CAP <resource> attachments
├── Builders/
│   ├── CapAlertBuilder.cs    # Fluent builders for CapAlert, AlertInfo, AlertArea
│   └── CapXmlSerializer.cs   # Bidirectional CAP v1.2 XML serialization
├── Channels/
│   └── ChannelConfigs.cs     # WeaChannelConfig, EasChannelConfig,
│                             # NwemChannelConfig, IpawsOpenConfig
├── Client/
│   ├── IpawsClient.cs        # mTLS HTTP client — POST CAP XML to IPAWS-OPEN
│   └── IpawsResponse.cs      # Typed server response model
└── Validation/
    ├── CapValidator.cs        # Schema + IPAWS-OPEN + channel-specific validation
    └── ValidationResult.cs   # Findings collection (errors + warnings)
```

---

## Quick Start

### 1. Build a CAP Alert

```csharp
var alert = new CapAlertBuilder()
    .WithSender("alerts@myagency.gov")
    .WithStatus(CapStatus.Test)           // Change to CapStatus.Actual for live alerts
    .WithMsgType(CapMsgType.Alert)
    .AddInfo(info => info
        .WithCategory(CapCategory.Met)
        .WithEvent("Tornado Warning")
        .AddResponseType(CapResponseType.Shelter)
        .WithUrgency(CapUrgency.Immediate)
        .WithSeverity(CapSeverity.Extreme)
        .WithCertainty(CapCertainty.Observed)
        .WithSenderName("My County Emergency Management")
        .WithHeadline("Tornado Warning for Example County until 6:00 PM")
        .WithDescription("A tornado has been confirmed on the ground...")
        .WithInstruction("Take shelter in a sturdy building immediately.")
        .WithExpiry(DateTimeOffset.UtcNow.AddHours(1))
        .AddSameCode("039049")            // FIPS 6-digit code for your county
        .AddWeaRouting(
            shortText: "Tornado Warning this area until 6PM. Take shelter now! Local EMA",
            longText:  "Tornado Warning for Example County until 6:00 PM. A tornado is on " +
                       "the ground moving northeast. Take shelter immediately."
        )
        .AddEasRouting()
    )
    .Build();
```

### 2. Validate

```csharp
var result = CapValidator.Validate(alert);

foreach (var finding in result.Findings)
    Console.WriteLine($"[{finding.Severity}] {finding.Code}: {finding.Message}");

result.ThrowIfInvalid();  // Throws if any errors are present
```

### 3. Serialize to CAP XML

```csharp
string xml = CapXmlSerializer.Serialize(alert, indent: true);
```

### 4. Submit to IPAWS-OPEN

```csharp
var config = new IpawsOpenConfig
{
    Endpoint            = IpawsOpenConfig.TestEndpoint,  // Use ProductionEndpoint for live
    CertificatePath     = @"C:\certs\my-fema-cert.p12",
    CertificatePassword = Environment.GetEnvironmentVariable("IPAWS_CERT_PASS"),
    CogId               = "YOUR-COG-ID",
};

using var client = new IpawsClient(config);
var response = await client.SubmitAsync(alert);

if (response.IsSuccess)
    Console.WriteLine($"Sent! Server ID: {response.ServerMessageId}");
else
    Console.WriteLine($"Failed: {string.Join(", ", response.Errors)}");
```

---

## Certificate Setup

FEMA issues a **PKCS#12 (.p12 / .pfx)** client certificate for each COG.

**Option A — File-based (development/testing):**
```csharp
config.CertificatePath     = @"C:\certs\ipaws-test.p12";
config.CertificatePassword = "password";
```

**Option B — Windows Certificate Store (recommended for production):**
1. Import the .p12 into the Windows certificate store:
   `certlm.msc` → Personal → Import
2. Note the certificate thumbprint (SHA-1 hex, no spaces)
3. Configure:
```csharp
config.CertThumbprint    = "A1B2C3D4E5F6...";  // From cert properties
config.CertStoreLocation = StoreLocation.LocalMachine;
```

> ⚠️ **Never hardcode the certificate password in source code.**
> Use environment variables, Windows DPAPI, or a secrets manager.

---

## IPAWS-OPEN Endpoints

| Environment | URL |
|-------------|-----|
| **Test (JITC)** | `https://tdl.integration.aws.fema.net/cap/SubmitCAPMessage` |
| **Production** | `https://www.fema.gov/cap/COGProfile.do` |

Use `IpawsOpenConfig.TestEndpoint` and `IpawsOpenConfig.ProductionEndpoint` constants.

---

## Channel Routing Parameters

IPAWS-OPEN uses CAP `<parameter>` elements in the `<info>` block to route
messages to specific dissemination channels:

| Channel | Parameter Name | Value |
|---------|---------------|-------|
| WEA (legacy 90-char) | `WEAHandling` | `Broadcast` |
| WEA short text | `CMAMtext` | ≤ 90 characters |
| WEA long text (3.0) | `CMAMlongtext` | ≤ 360 characters |
| EAS | `EASHandling` | `Broadcast` |
| NWEM | `NWEMHandling` | `Broadcast` |
| NWS VTEC | `VTEC` | VTEC string |

The builder methods `AddWeaRouting()`, `AddEasRouting()`, `AddNwemRouting()`
set these automatically.

---

## Validation Error Codes

| Code | Description |
|------|-------------|
| `CAP001–CAP027` | Core CAP v1.2 schema violations |
| `IPAWS001` | Missing `IPAWSv1.0` code |
| `WEA001–WEA005` | WEA channel rule violations |
| `EAS001–EAS002` | EAS channel rule violations |
| `NWEM001–NWEM002` | NWEM channel rule violations |

---


## References

- [CAP v1.2 Specification (OASIS)](http://docs.oasis-open.org/emergency/cap/v1.2/CAP-v1.2-os.html)
- [IPAWS Developer Resources (FEMA)](https://www.fema.gov/emergency-managers/practitioners/integrated-public-alert-warning-system/developers)
- [WEA Technical Standard (ATIS)](https://www.atis.org/01_standards/standards_overview/wea.aspx)
- [EAS SAME Codes (NWS)](https://www.nws.noaa.gov/directives/sym/pd01017012curr.pdf)
- [IPAWS COG Program](https://www.fema.gov/emergency-managers/practitioners/integrated-public-alert-warning-system/authorities-training-technical-support/collaborative-operating-groups)
