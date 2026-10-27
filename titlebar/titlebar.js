// titlebar.js — duoc nhung vao binary luc bien dich va tiem vao trang
// music.youtube.com (xem src-tauri/src/main.rs). Hoan toan doc lap voi logic cua trang goc,
// chi thao tac tren cac phan tu #app-titlebar do titlebar.html tao ra.
//
// Dung window.__TAURI__ (bat qua "app.withGlobalTauri": true trong
// tauri.conf.json) thay vi cau lenh "import", vi trang nay khong co
// buoc bundle (vite/webpack) de resolve goi trong node_modules — script
// nay duoc nap thang nhu 1 <script> tinh binh thuong.

(function () {
  const tauriWindow = window.__TAURI__ && window.__TAURI__.window;
  if (!tauriWindow) {
    console.warn("[titlebar] window.__TAURI__.window khong ton tai — " +
      "kiem tra lai app.withGlobalTauri trong tauri.conf.json.");
    return;
  }

  const appWindow = tauriWindow.getCurrentWindow();

  function bind(id, handler) {
    const el = document.getElementById(id);
    if (el) el.addEventListener("click", handler);
  }

  bind("titlebar-min", () => appWindow.minimize());
  bind("titlebar-max", () => appWindow.toggleMaximize());
  bind("titlebar-close", () => appWindow.close());

  // Double-click vao vung keo = phong to/thu nho, giong titlebar Windows chuan.
  const spacer = document.querySelector(".app-titlebar-spacer");
  if (spacer) {
    spacer.addEventListener("dblclick", () => appWindow.toggleMaximize());
  }

  // --- Fullscreen (F11 / Esc) ---
  // Day la fullscreen THAT cua he dieu hanh (Tauri setFullscreen — cua so
  // phu kin man hinh, khong vien), khac hoan toan voi "che do fullscreen
  // player" rieng cua web (nut .fp-collapse, .fp-lyrics-toggle) — cai do
  // van la mot view/state ben trong trang, khong dung toi API nay.
  //
  // F11 = bat/tat (toggle). Esc = chi thoat (khong bao gio bat), giong
  // convention pham vi trinh duyet.
  function setFullscreenClass(isFullscreen) {
    // Khi vao fullscreen, an 3 nut minimize/maximize/close di — fullscreen
    // that thi khong con OS chrome nao ca. Vung keo (spacer) van de nguyen,
    // vi no vo hinh (background: transparent) nen khong anh huong gi.
    document.body.classList.toggle("app-is-fullscreen", isFullscreen);
  }

  async function toggleFullscreen() {
    const isFullscreen = await appWindow.isFullscreen();
    await appWindow.setFullscreen(!isFullscreen);
    setFullscreenClass(!isFullscreen);
  }

  async function exitFullscreenIfActive() {
    if (await appWindow.isFullscreen()) {
      await appWindow.setFullscreen(false);
      setFullscreenClass(false);
    }
  }

  window.addEventListener("keydown", (event) => {
    if (event.key === "F11") {
      // Chan hanh vi F11 mac dinh cua webview (vd Chromium tu bat
      // fullscreen kieu browser) de khong dung do voi Tauri fullscreen.
      event.preventDefault();
      toggleFullscreen();
    } else if (event.key === "Escape") {
      exitFullscreenIfActive();
    }
  });

  // Dong bo lai class luc script vua nap, phong khi cua so da o san
  // fullscreen tu truoc (vd OS/session cu con nho trang thai).
  appWindow.isFullscreen().then(setFullscreenClass);
})();
