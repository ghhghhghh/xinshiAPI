import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const release = process.argv.includes('--release');
const profile = release ? 'release' : 'debug';
const exeSuffix = process.platform === 'win32' ? '.exe' : '';
const sidecarName = `xingshi-mcp${exeSuffix}`;

function run(command, args) {
  execFileSync(command, args, { cwd: root, stdio: 'inherit' });
}

function copyFile(source, target) {
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.copyFileSync(source, target);
}

function copyDir(source, target) {
  fs.rmSync(target, { recursive: true, force: true });
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.cpSync(source, target, { recursive: true });
}

const rustc = execFileSync('rustc', ['-vV'], { encoding: 'utf8', cwd: root });
const triple = rustc.match(/^host:\s*(\S+)$/m)?.[1];
if (!triple) throw new Error('无法从 rustc -vV 读取目标平台');

// Skill 作为 Tauri resource 随安装包分发；先复制，避免构建 sidecar 时资源缺失
const skillSource = path.join(root, 'skills', 'xingshi-api-import');
const resourceSkill = path.join(
  root,
  'src-tauri',
  'resources',
  'skills',
  'xingshi-api-import',
);
copyDir(skillSource, resourceSkill);
copyDir(
  skillSource,
  path.join(root, 'src-tauri', 'target', profile, 'resources', 'skills', 'xingshi-api-import'),
);

console.log(`[prepare-ai] 构建 sidecar（${profile}）…`);
const tauriConfigPath = path.join(root, 'src-tauri', 'tauri.conf.json');
const originalConfig = fs.readFileSync(tauriConfigPath, 'utf8');
const config = JSON.parse(originalConfig);
// sidecar 本身不需要 externalBin；临时移除，避免 tauri-build 在文件尚未
// 生成时直接失败。构建完成后恢复原配置，再交给 Tauri 正式打包。
if (config.bundle) delete config.bundle.externalBin;
fs.writeFileSync(tauriConfigPath, `${JSON.stringify(config, null, 2)}\n`, 'utf8');
try {
  run('cargo', [
    'build',
    '--manifest-path',
    path.join('src-tauri', 'Cargo.toml'),
    '--bin',
    'xingshi-mcp',
    ...(release ? ['--release'] : []),
  ]);
} finally {
  fs.writeFileSync(tauriConfigPath, originalConfig, 'utf8');
}

const built = path.join(root, 'src-tauri', 'target', profile, sidecarName);
if (!fs.existsSync(built)) throw new Error(`sidecar 构建产物不存在：${built}`);

// Tauri externalBin 需要的目标平台后缀文件
const bundled = path.join(
  root,
  'src-tauri',
  'binaries',
  `xingshi-mcp-${triple}${exeSuffix}`,
);
copyFile(built, bundled);

// 开发/本地运行时，应用按 current_exe 同目录查找 sidecar
copyFile(built, path.join(root, 'src-tauri', 'target', profile, sidecarName));

console.log(`[prepare-ai] sidecar: ${bundled}`);
console.log(`[prepare-ai] skill:   ${resourceSkill}`);
