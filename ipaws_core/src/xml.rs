use crate::models::*;

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
     .replace('"', "&quot;").replace('\'', "&apos;")
}

fn fmt_dt(dt: &chrono::DateTime<chrono::Utc>) -> String {
    dt.format("%Y-%m-%dT%H:%M:%S+00:00").to_string()
}

pub fn serialize(alert: &CapAlert, indent: bool) -> String {
    let (nl, i1, i2, i3, i4) = if indent {
        ("\n", "  ", "    ", "      ", "        ")
    } else {
        ("", "", "", "", "")
    };

    let mut o = String::new();
    o.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    o.push_str(nl);
    o.push_str(r#"<alert xmlns="urn:oasis:names:tc:emergency:cap:1.2">"#);
    o.push_str(nl);

    o.push_str(&format!("{i1}<identifier>{}</identifier>{nl}", esc(&alert.identifier)));
    o.push_str(&format!("{i1}<sender>{}</sender>{nl}", esc(&alert.sender)));
    o.push_str(&format!("{i1}<sent>{}</sent>{nl}", fmt_dt(&alert.sent)));
    o.push_str(&format!("{i1}<status>{}</status>{nl}", alert.status));
    o.push_str(&format!("{i1}<msgType>{}</msgType>{nl}", alert.msg_type));
    o.push_str(&format!("{i1}<scope>{}</scope>{nl}", alert.scope));

    for c in &alert.codes      { o.push_str(&format!("{i1}<code>{}</code>{nl}", esc(c))); }
    if !alert.references.is_empty() {
        o.push_str(&format!("{i1}<references>{}</references>{nl}", esc(&alert.references.join(" "))));
    }

    for info in &alert.infos {
        o.push_str(&format!("{i1}<info>{nl}"));
        o.push_str(&format!("{i2}<language>{}</language>{nl}", esc(&info.language)));
        for cat in &info.categories { o.push_str(&format!("{i2}<category>{cat}</category>{nl}")); }
        o.push_str(&format!("{i2}<event>{}</event>{nl}", esc(&info.event)));
        for rt in &info.response_types { o.push_str(&format!("{i2}<responseType>{rt}</responseType>{nl}")); }
        o.push_str(&format!("{i2}<urgency>{}</urgency>{nl}", info.urgency));
        o.push_str(&format!("{i2}<severity>{}</severity>{nl}", info.severity));
        o.push_str(&format!("{i2}<certainty>{}</certainty>{nl}", info.certainty));
        if let Some(dt) = &info.effective   { o.push_str(&format!("{i2}<effective>{}</effective>{nl}", fmt_dt(dt))); }
        if let Some(dt) = &info.onset       { o.push_str(&format!("{i2}<onset>{}</onset>{nl}", fmt_dt(dt))); }
        if let Some(dt) = &info.expires     { o.push_str(&format!("{i2}<expires>{}</expires>{nl}", fmt_dt(dt))); }
        if let Some(n)  = &info.sender_name { o.push_str(&format!("{i2}<senderName>{}</senderName>{nl}", esc(n))); }
        if let Some(h)  = &info.headline    { o.push_str(&format!("{i2}<headline>{}</headline>{nl}", esc(h))); }
        if let Some(d)  = &info.description { o.push_str(&format!("{i2}<description>{}</description>{nl}", esc(d))); }
        if let Some(i)  = &info.instruction { o.push_str(&format!("{i2}<instruction>{}</instruction>{nl}", esc(i))); }
        if let Some(w)  = &info.web         { o.push_str(&format!("{i2}<web>{}</web>{nl}", esc(w))); }

        for p in &info.parameters {
            o.push_str(&format!("{i2}<parameter>{nl}"));
            o.push_str(&format!("{i3}<valueName>{}</valueName>{nl}", esc(&p.name)));
            o.push_str(&format!("{i3}<value>{}</value>{nl}", esc(&p.value)));
            o.push_str(&format!("{i2}</parameter>{nl}"));
        }

        for area in &info.areas {
            o.push_str(&format!("{i2}<area>{nl}"));
            o.push_str(&format!("{i3}<areaDesc>{}</areaDesc>{nl}", esc(&area.area_desc)));
            for poly   in &area.polygons   { o.push_str(&format!("{i3}<polygon>{}</polygon>{nl}", esc(poly))); }
            for circle in &area.circles    { o.push_str(&format!("{i3}<circle>{}</circle>{nl}", esc(circle))); }
            for code   in &area.same_codes {
                o.push_str(&format!("{i3}<geocode>{nl}"));
                o.push_str(&format!("{i4}<valueName>SAME</valueName>{nl}"));
                o.push_str(&format!("{i4}<value>{}</value>{nl}", esc(code)));
                o.push_str(&format!("{i3}</geocode>{nl}"));
            }
            o.push_str(&format!("{i2}</area>{nl}"));
        }

        o.push_str(&format!("{i1}</info>{nl}"));
    }
    o.push_str("</alert>");
    o
}
