#!/usr/bin/env node
/* --------------------------------------------------------------------------
   IconForge 前端构建：cargo → wasm-bindgen → dist

   为什么不用 trunk：这里只有一条产物流（cdylib + 一份 index.html），
   而 trunk 会再拉一套配置与插件。三十行 Node 反而更清楚，也方便在
   Tauri 的 beforeDevCommand / beforeBuildCommand 里直接调。

   用法：
     node ui/build.mjs            # debug（开发，快）
     node ui/build.mjs --release  # release（发布，小）
   -------------------------------------------------------------------------- */
import { spawnSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, readFileSync, rmSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const uiDir = dirname(fileURLToPath(import.meta.url));
const root = resolve(uiDir, "..");
const dist = join(uiDir, "dist");
const release = process.argv.includes("--release");
const profile = release ? "release" : "debug";
const crate = "iconforge_ui";

const isWindows = process.platform === "win32";
const shell = isWindows;

function run(command, args, cwd = root) {
  const result = spawnSync(command, args, { cwd, stdio: "inherit", shell });
  if (result.status !== 0) {
    console.error(`\n[build] 失败：${command} ${args.join(" ")}`);
    process.exit(result.status ?? 1);
  }
}

function capture(command, args, cwd = root) {
  const result = spawnSync(command, args, { cwd, encoding: "utf8", shell });
  return result.status === 0 ? `${result.stdout ?? ""}${result.stderr ?? ""}`.trim() : null;
}

/* wasm-bindgen CLI 必须与 Cargo 解析出的 crate 同版本，否则运行期会报
   "it looks like the Rust project used to create this wasm file was linked
   against a version of wasm-bindgen that uses a different bindgen format"。
   这里提前挡下来，比跑到一半再猜要快。 */
function checkBindgenVersion() {
  const cli = capture("wasm-bindgen", ["--version"]);
  if (!cli) {
    console.error("[build] 找不到 wasm-bindgen。请先安装：cargo install wasm-bindgen-cli");
    process.exit(1);
  }
  const cliVersion = cli.match(/(\d+\.\d+\.\d+)/)?.[1];
  const lockPath = join(root, "Cargo.lock");
  if (!cliVersion || !existsSync(lockPath)) return;

  const lock = readFileSync(lockPath, "utf8");
  const match = lock.match(/name = "wasm-bindgen"\r?\nversion = "([^"]+)"/);
  const crateVersion = match?.[1];
  if (crateVersion && crateVersion !== cliVersion) {
    console.error(
      `[build] 版本不一致：Cargo.lock 里 wasm-bindgen = ${crateVersion}，` +
        `命令行是 ${cliVersion}。\n` +
        `        二选一：cargo install wasm-bindgen-cli --version ${crateVersion}\n` +
        `        或者在 ui/Cargo.toml 里把 wasm-bindgen 钉成 "=${cliVersion}" 后重新生成锁文件。`,
    );
    process.exit(1);
  }
}

checkBindgenVersion();

console.log(`[build] cargo build (${profile}) → wasm32-unknown-unknown`);
run("cargo", [
  "build",
  "-p",
  "iconforge-ui",
  "--target",
  "wasm32-unknown-unknown",
  ...(release ? ["--release"] : []),
]);

const wasmPath = join(root, "target", "wasm32-unknown-unknown", profile, `${crate}.wasm`);
if (!existsSync(wasmPath)) {
  console.error(`[build] 没有找到 ${wasmPath}`);
  process.exit(1);
}

rmSync(join(dist, "pkg"), { recursive: true, force: true });
mkdirSync(join(dist, "pkg"), { recursive: true });

console.log("[build] wasm-bindgen → ui/dist/pkg");
/* `--remove-name-section`：Rust 的 `strip` 不会去掉 wasm 的 name 段，
   而 leptos 单态化出来的符号名有 13 MB —— 不剥的话产物是 16 MB 而不是 2 MB。
   这一节只服务调试器，运行期用不到。 */
run("wasm-bindgen", [
  "--target",
  "web",
  "--no-typescript",
  "--remove-name-section",
  "--out-dir",
  join(dist, "pkg"),
  wasmPath,
]);

for (const file of ["index.html", "app.css", "bridge.js", "shell.js"]) {
  copyFileSync(join(uiDir, file), join(dist, file));
}

const favicon = join(root, "public", "favicon.ico");
if (existsSync(favicon)) {
  copyFileSync(favicon, join(dist, "favicon.ico"));
}

console.log(`[build] 完成：${dist}`);
