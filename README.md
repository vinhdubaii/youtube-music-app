# YouTube Music App (desktop, Tauri)

Bản desktop Windows của [YouTube Music](https://music.youtube.com/),
đóng gói bằng Tauri v2.

Repo này **không chứa source web nào cả**. Cửa sổ app load thẳng
`https://music.youtube.com/` — y hệt việc mở link đó trong trình duyệt, chỉ là
không có khung trình duyệt. Mọi thay đổi của YouTube Music là user thấy ngay,
không cần build/release lại app desktop. App desktop chỉ cần release khi đổi
phần khung native (titlebar, icon, updater, kích thước cửa sổ...).

Đăng nhập tài khoản Google đi qua `accounts.google.com` ngay trong cửa sổ app;
phiên đăng nhập được WebView2 lưu lại giữa các lần mở app.

Titlebar tự vẽ (3 nút minimize/maximize/close + F11/Esc fullscreen) được
nhúng thẳng vào binary lúc compile (`include_str!`) và tiêm vào trang bằng
`initialization_script` của Tauri — chạy trước khi trang được parse, xem
`src-tauri/src/main.rs`.

## Cấu trúc

```
titlebar/                 Titlebar tự vẽ (html/css/js) — nhúng vào binary,
                            tiêm vào trang lúc runtime (xem main.rs)
src-tauri/
  src/main.rs             Tự tạo cửa sổ (để tiêm titlebar script) +
                            tự kiểm tra/cài update im lặng
  tauri.conf.json          url trỏ thẳng music.youtube.com, decorations:false,
                            min-size 1280x800, cấu hình updater
  capabilities/default.json  Quyền minimize/maximize/close/kéo cửa sổ/fullscreen
                              — cấp cho origin music.youtube.com và
                              accounts.google.com (remote)
  icons/                   Sinh ra lúc build từ img/logo.png, KHÔNG commit
img/logo.png              Logo nguồn (PNG vuông) để sinh bộ icon
scripts/gen-icons.mjs     Gọi `tauri icon` để sinh src-tauri/icons/
```

## Cài đặt lần đầu

Cần: Node 20+, Rust stable, Git. Trên Windows, WebView2 runtime
(thường có sẵn từ Win10/11, Tauri tự cài kèm nếu thiếu lúc chạy installer).

```bash
npm install
```

### Icon

Bộ icon **không commit vào git** — nó được sinh tự động từ `img/logo.png`.

Việc bạn cần làm: đặt logo vào `img/logo.png` (PNG vuông, nên 1024×1024, có
alpha).

Sau đó `scripts/gen-icons.mjs` sẽ gọi `tauri icon` để sinh toàn bộ
`src-tauri/icons/` (bao gồm 4 file khai trong `tauri.conf.json`). Script này
được khai thẳng trong `beforeDevCommand`/`beforeBuildCommand` của
`tauri.conf.json` — nên `dev`, `build` và cả GitHub Actions đều tự lo, không
cần chạy tay.

Script có cache: nó lưu hash của `logo.png` vào `src-tauri/icons/.source-hash`
và chỉ sinh lại khi logo thay đổi. Muốn chạy riêng thì `npm run gen-icons`.

## Chạy thử / build

```bash
npm run tauri dev      # chạy thử, tự load thẳng music.youtube.com
npm run tauri build    # build .msi/.exe, ra trong src-tauri/target/release/bundle/
```

Cần internet cả lúc dev lẫn lúc chạy app thật — cửa sổ load thẳng URL sống,
không có gì được đóng gói sẵn để chạy offline. Mất mạng thì trắng trang
(WebView2 tự hiện lỗi mặc định của nó), không có màn hình fallback riêng.

## Cửa sổ: kích thước & fullscreen

Cửa sổ khoá kích thước tối thiểu 1280×800 (`minWidth`/`minHeight` trong
`tauri.conf.json`), vẫn resize được lớn hơn thoải mái (`resizable: true`).

`F11` bật/tắt fullscreen thật của hệ điều hành (khác với chế độ player toàn
màn hình riêng của YouTube Music). `Esc` chỉ thoát fullscreen (không
bao giờ bật). Lúc fullscreen, 3 nút minimize/maximize/close tự vẽ tự ẩn đi.
Toàn bộ logic nằm trong `titlebar/titlebar.js` + `titlebar/titlebar.css`.

## Auto-update

App tự kiểm tra bản mới **1 lần duy nhất mỗi lúc vừa mở** (không lặp lại định
kỳ trong lúc đang chạy, để không bao giờ cắt ngang giữa lúc đang nghe nhạc).
Nếu có bản mới: tự tải và cài hoàn toàn im lặng (`installMode: "quiet"`,
không hiện gì cả). Bản cập nhật có hiệu lực ở **lần mở app kế tiếp**
(`restart_after_install(false)` trong `main.rs`) — không tự bật lại app ngay
sau khi cài.

Nguồn kiểm tra là `releases/latest/download/latest.json` trên GitHub — API
này **chỉ thấy release đã bấm Publish**, bỏ qua release còn ở dạng Draft. Nên
quy trình "release.yml tạo draft, tự tay viết note rồi bấm Publish" vẫn giữ
nguyên, và tự nhiên trở thành bước kiểm duyệt trước khi người dùng nhận được
bản cập nhật.

**Bắt buộc phải thiết lập trước khi push bất kỳ commit nào** (kể cả build
debug thường ngày cũng cần, vì `bundle.createUpdaterArtifacts: true` đòi hỏi
key này để build) — vào repo trên GitHub → Settings → Secrets and variables →
Actions → New repository secret, tạo 2 secret:

- `TAURI_SIGNING_PRIVATE_KEY` — private key dùng để ký update (giữ bí mật
  tuyệt đối, mất key này là không thể phát hành update mới cho người dùng cũ
  được nữa).
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — mật khẩu của private key ở trên.

Public key tương ứng đã nhét sẵn vào `plugins.updater.pubkey` trong
`tauri.conf.json` (an toàn khi public, không cần giữ bí mật).

Nếu muốn tạo lại cặp key khác (vd làm mất key cũ):

```bash
npx tauri signer generate -w ./yt-music-app-updater.key
```

rồi copy nội dung file `.key` (private) + mật khẩu vào 2 secret trên, và nội
dung file `.key.pub` (public) vào `plugins.updater.pubkey`. **Tuyệt đối không
commit file `.key` vào git.**

## Web: load trực tiếp, không đóng gói

Cửa sổ "main" trong `tauri.conf.json` có `"url": "https://music.youtube.com/"`
và `"create": false` — Tauri không tự tạo cửa sổ này, `main.rs` tự tạo lại
(y hệt config, không đổi gì) chỉ để gắn thêm `initialization_script()` (chèn
titlebar), việc mà JSON config không làm được.

**Đánh đổi bảo mật cần biết**: Tauri v2 mặc định không cấp quyền IPC nào cho
nội dung tải từ domain ngoài. Để 3 nút titlebar + F11 hoạt động,
`capabilities/default.json` phải khai rõ `"remote": { "urls":
["https://music.youtube.com/*", "https://accounts.google.com/*"] }` (trang
đăng nhập Google cũng cần để các nút titlebar không chết lúc đang login) —
nghĩa là bất kỳ script nào chạy được trên các trang đó (kể cả do lỗ hổng XSS
nào đó, giả sử có) đều có thể gọi các
lệnh minimize/maximize/close/fullscreen của cửa sổ app. Rủi ro thấp (chỉ là
điều khiển cửa sổ, không phải đọc file hay chạy lệnh hệ thống) nhưng là thật,
không phải lý thuyết — cần nhớ nếu sau này cấp thêm quyền khác cho domain
này.

## Chưa kiểm chứng: titlebar có thể chồng lên thanh trên của YouTube Music

Titlebar tự vẽ (cao 36px, 3 nút ở góc phải, z-index rất cao) được thiết kế
cho trang cũ. YouTube Music có thanh điều hướng cố định ở mép trên, và góc
phải của nó thường là avatar/menu tài khoản — nhiều khả năng 3 nút
minimize/maximize/close nằm đè lên vùng đó. Điều này **chưa được chạy thử
trên Windows thật** và chưa kiểm tra với giao diện YouTube Music hiện tại, nên
đây là dự đoán chứ chưa phải lỗi đã xác nhận.

Nếu build xong thấy bị đè, các hướng xử lý: đẩy thanh nav của YouTube Music
xuống 36px bằng CSS tiêm qua `initialization_script` (phụ thuộc selector của
YouTube Music, có thể vỡ khi họ đổi giao diện), thu nhỏ/dời titlebar, hoặc
tách titlebar thành webview riêng nằm trên webview YouTube Music.

Lưu ý khác: đăng nhập Google trong webview nhúng đôi khi bị Google chặn hoặc
yêu cầu xác minh thêm — cần thử đăng nhập thật trên Windows để biết.
