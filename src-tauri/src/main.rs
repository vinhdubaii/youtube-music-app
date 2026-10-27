// Cua so duy nhat cua app desktop (ban YouTube Music).
//
// Cua so tai truc tiep https://music.youtube.com/ (xem README muc "Web: tai
// truc tiep, khong dong goi"), khong co index.html nao cua rieng minh de
// chen <link>/<script> vao truoc. Nen titlebar (markup +
// CSS + JS trong thu muc titlebar/) duoc nhung thang vao binary luc bien
// dich (include_str!) roi tiem vao trang bang initialization_script — chay
// truoc khi trang duoc parse, xem titlebar_injection_script() ben duoi.
//
// Ngoai ra Rust o day con lam 1 viec: tu kiem tra va cai ban cap nhat moi
// luc vua mo app (xem check_for_update() ben duoi).

// An console window tren Windows o ban release; van giu console luc
// `cargo run`/dev de con xem log.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri_plugin_updater::UpdaterExt;

// Noi dung 3 file titlebar duoc nhung thang vao binary luc bien dich —
// include_str!() doc file tai thoi diem compile, khong doc tu dia luc
// runtime, nen khong can ship kem file .html/.css/.js nao ca.
const TITLEBAR_HTML: &str = include_str!("../../titlebar/titlebar.html");
const TITLEBAR_CSS: &str = include_str!("../../titlebar/titlebar.css");
const TITLEBAR_JS: &str = include_str!("../../titlebar/titlebar.js");

/// Ghep CSS + markup + logic cua titlebar + chan chuot phai thanh 1 doan JS
/// duy nhat, dung lam initialization_script — chay truoc khi trang
/// (music.youtube.com) duoc parse, nen phai tu cho document.body xuat hien
/// (DOMContentLoaded) roi moi chen <style> + markup titlebar vao, sau do
/// chay tiep logic cu cua titlebar.js (bind nut, F11/Esc) — luc do cac phan
/// tu da ton tai nen document.getElementById(...) hoat dong binh thuong,
/// giong het truoc day.
///
/// Chan chuot phai (contextmenu) o toan trang, tru rieng <input>/<textarea>/
/// phan tu contenteditable — nhung cho do van giu duoc menu Dan/Copy qua
/// chuot phai. Khong lien quan gi toi DevTools: F12 da bi tat san o muc
/// WebView2 vi Cargo.toml khong bat feature "devtools" cua tauri (chi co
/// hieu luc tren debug build), nen khong can them gi rieng cho phan do.
///
/// Dung serde_json::to_string de encode CSS/HTML thanh chuoi JS an toan
/// (tu dong escape dau nhay/xuong dong...), tranh loi neu noi dung co ky
/// tu dac biet.
fn titlebar_injection_script() -> String {
    let css_js_string =
        serde_json::to_string(TITLEBAR_CSS).expect("TITLEBAR_CSS la UTF-8 hop le");
    let html_js_string =
        serde_json::to_string(TITLEBAR_HTML).expect("TITLEBAR_HTML la UTF-8 hop le");

    format!(
        r#"
(function () {{
  function injectTitlebar() {{
    // Tranh chen 2 lan neu script nay vi ly do gi do chay lai.
    if (document.getElementById("app-titlebar")) return;

    var style = document.createElement("style");
    style.textContent = {css};
    document.head.appendChild(style);

    var wrapper = document.createElement("div");
    wrapper.innerHTML = {html};
    document.body.insertBefore(wrapper.firstElementChild, document.body.firstChild);

    // --- Logic cua titlebar.js, giu nguyen 100% ---
{js}
  }}

  if (document.readyState === "loading") {{
    document.addEventListener("DOMContentLoaded", injectTitlebar, {{ once: true }});
  }} else {{
    injectTitlebar();
  }}

  // Chan chuot phai toan trang, tru input/textarea/contenteditable (van
  // dung "capture" phase de chan som, truoc khi bat ky handler nao cua web
  // co the ngan chan viec nay). Doc lap voi injectTitlebar() — khong can
  // doi titlebar chen xong moi bat.
  document.addEventListener(
    "contextmenu",
    function (event) {{
      var editable = event.target.closest &&
        event.target.closest('input, textarea, [contenteditable]:not([contenteditable="false"])');
      if (!editable) {{
        event.preventDefault();
      }}
    }},
    true
  );
}})();
"#,
        css = css_js_string,
        html = html_js_string,
        js = TITLEBAR_JS,
    )
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            // Cua so "main" dat "create": false trong tauri.conf.json — tu
            // tao lai o day (tu dung cau hinh trong file, khong doi gi ca)
            // chi de duoc them initialization_script(), thu duy nhat JSON
            // config khong lam duoc.
            let handle = app.handle().clone();
            let window_config = app.config().app.windows[0].clone();

            tauri::WebviewWindowBuilder::from_config(&handle, &window_config)?
                .initialization_script(titlebar_injection_script())
                .build()?;

            // Chay ngam, khong chan cua so hien len. Chi kiem tra 1 lan
            // luc vua mo app (khong lap lai dinh ky trong luc app dang
            // chay) de khong bao gio cat ngang giua luc dang nghe nhac.
            let updater_handle = handle.clone();
            tauri::async_runtime::spawn(async move {
                check_for_update(updater_handle).await;
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("loi khi khoi dong YT Music App");
}

/// Kiem tra ban moi tren GitHub Releases (endpoint khai trong
/// tauri.conf.json -> plugins.updater.endpoints, tro toi
/// releases/latest/download/latest.json — API nay CHI thay release da
/// bam Publish, bo qua release con o dang Draft, nen release.yml van
/// giu duoc buoc "tao draft roi tu tay publish" nhu cu lam cong doan
/// kiem duyet truoc khi nguoi dung nhan duoc ban cap nhat).
///
/// Neu co ban moi: tai va cai hoan toan im lang (installMode: "quiet",
/// khong hien gi ca). Tren Windows, buoc cai dat luon lam ung dung tu
/// thoat giua chung — day la gioi han cua Windows installer (khong the
/// vua chay vua ghi de file .exe dang mo), khong phai loi. Mac dinh sau
/// khi cai xong, Tauri se TU DONG MO LAI app (restart_after_install mac
/// dinh la true) — o day minh tat no di (false) de ban cap nhat chi co
/// hieu luc o lan nguoi dung tu mo app ke tiep, tranh cat ngang giua
/// luc dang nghe nhac.
async fn check_for_update(app: tauri::AppHandle) {
    let updater = match app.updater() {
        Ok(updater) => updater,
        Err(err) => {
            eprintln!("[updater] khong khoi tao duoc updater: {err}");
            return;
        }
    };

    let update = match updater.check().await {
        Ok(Some(update)) => update,
        Ok(None) => {
            println!("[updater] dang dung ban moi nhat.");
            return;
        }
        Err(err) => {
            // Loi thuong gap: khong co mang, hoac chua co release nao
            // duoc publish tren GitHub (releases/latest 404).
            eprintln!("[updater] check that bai: {err}");
            return;
        }
    };

    println!("[updater] tim thay ban moi: {}", update.version);

    let update = update.restart_after_install(false);

    if let Err(err) = update
        .download_and_install(
            |_chunk_length, _content_length| {
                // Khong lam gi — khong co UI nao de hien progress ca,
                // dung y voi installMode: "quiet".
            },
            || {
                println!("[updater] tai xong, dang cai dat...");
            },
        )
        .await
    {
        eprintln!("[updater] tai/cai that bai: {err}");
    }
}
