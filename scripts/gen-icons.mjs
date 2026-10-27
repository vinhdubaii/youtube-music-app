// Sinh bo icon Windows tu img/logo.png bang `tauri icon`.
//
// Duoc goi tu tauri.conf.json (beforeDevCommand / beforeBuildCommand), nen
// ca `npm run tauri dev` lan `npm run tauri build` lan GitHub Actions deu
// tu chay — khong can commit src-tauri/icons/ vao git nua.
//
// Co cache: luu hash cua logo.png vao src-tauri/icons/.source-hash. Neu hash
// khong doi va cac file icon can thiet van con, script bo qua, khong chay lai.

import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const SOURCE = path.join(ROOT, "img", "logo.png");
const OUT_DIR = path.join(ROOT, "src-tauri", "icons");
const STAMP = path.join(OUT_DIR, ".source-hash");

// Dung y nguyen danh sach khai trong tauri.conf.json -> bundle.icon.
// Neu sau nay sua tauri.conf.json thi sua ca o day.
const REQUIRED_ICONS = [
  "32x32.png",
  "128x128.png",
  "128x128@2x.png",
  "icon.ico",
];

function fail(message) {
  console.error(`[gen-icons] ${message}`);
  process.exit(1);
}

function hashFile(filePath) {
  const bytes = fs.readFileSync(filePath);
  return createHash("sha256").update(bytes).digest("hex");
}

function readStamp() {
  try {
    return fs.readFileSync(STAMP, "utf8").trim();
  } catch {
    return null;
  }
}

function allIconsPresent() {
  return REQUIRED_ICONS.every((name) => fs.existsSync(path.join(OUT_DIR, name)));
}

function runTauriIcon() {
  // Windows khong cho spawn thang file .cmd/.bat (npx.cmd) neu khong bat
  // shell:true — CreateProcess cua Win32 khong coi .cmd la executable hop
  // le, spawn se bao loi EINVAL. Bat shell:true cho ca 3 he dieu hanh de
  // dong nhat; khi do node tu quote args ho minh, nhung path co the co
  // khoang trang nen van tu quote thu cong cho chac.
  const quote = (value) =>
    process.platform === "win32" ? `"${value}"` : `'${value}'`;

  execFileSync("npx", ["tauri", "icon", quote(SOURCE), "--output", quote(OUT_DIR)], {
    cwd: ROOT,
    stdio: "inherit",
    shell: true,
  });
}

function main() {
  if (!fs.existsSync(SOURCE)) {
    fail(
      "khong tim thay img/logo.png.\n" +
        "Dat logo (PNG vuong, nen 1024x1024, co alpha) vao img/logo.png roi chay lai."
    );
  }

  const currentHash = hashFile(SOURCE);

  if (currentHash === readStamp() && allIconsPresent()) {
    console.log("[gen-icons] icon da co va logo khong doi — bo qua.");
    return;
  }

  console.log("[gen-icons] dang sinh icon tu img/logo.png ...");
  fs.mkdirSync(OUT_DIR, { recursive: true });

  try {
    runTauriIcon();
  } catch (error) {
    fail(`chay \`tauri icon\` that bai: ${error.message}`);
  }

  const missing = REQUIRED_ICONS.filter(
    (name) => !fs.existsSync(path.join(OUT_DIR, name))
  );

  if (missing.length > 0) {
    fail(
      `\`tauri icon\` chay xong nhung thieu file: ${missing.join(", ")}.\n` +
        "Kiem tra lai img/logo.png co dung la PNG vuong khong."
    );
  }

  fs.writeFileSync(STAMP, `${currentHash}\n`, "utf8");
  console.log(`[gen-icons] xong — ${OUT_DIR}`);
}

main();
