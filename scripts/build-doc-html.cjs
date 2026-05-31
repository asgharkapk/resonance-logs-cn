/**
 * 将 doc 目录下的 Markdown 文档转换为 HTML，输出到 doc/html_doc/
 * 用于打包给不熟悉 .md 的用户直接浏览器查看
 */
const fs = require('fs');
const path = require('path');
const { marked } = require('marked');

const DOC_DIR = path.join(__dirname, '..', 'doc');
const OUT_DIR = path.join(DOC_DIR, 'html_doc');

/** @type {Record<string, { hubHref: string; hubLabel: string }>} */
const FEATURE_HUBS = {
  monitor: { hubHref: 'README.html', hubLabel: 'Buff 监控' },
  dps: { hubHref: 'README.html', hubLabel: 'DPS 检测' },
  monster: { hubHref: 'README.html', hubLabel: '怪物监控' },
};

function escapeHtml(s) {
  return String(s)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

function extractTitle(md) {
  const m = md.match(/^#\s+(.+)$/m);
  return m ? m[1].trim() : '文档';
}

/**
 * @param {string} outPath absolute path under OUT_DIR
 * @returns {string} relative href prefix to html_doc/index.html
 */
function baseHrefToIndex(outPath) {
  const rel = path.relative(OUT_DIR, path.dirname(outPath));
  if (!rel || rel === '.') return '';
  const depth = rel.split(path.sep).filter(Boolean).length;
  return '../'.repeat(depth);
}

/**
 * @param {string} mdPath
 * @param {string} outPath
 */
function hubNavForFile(mdPath, outPath) {
  const rel = path.relative(path.join(DOC_DIR, 'features'), mdPath).replace(/\\/g, '/');
  const parts = rel.split('/');
  if (parts.length < 2) return null;
  const module = parts[0];
  const hub = FEATURE_HUBS[module];
  if (!hub) return null;

  const outDir = path.dirname(outPath);
  const hubOut = path.join(outDir, hub.hubHref);
  const fromOut = path.relative(path.dirname(outPath), hubOut).replace(/\\/g, '/');
  const isHubReadme = path.basename(mdPath).toLowerCase() === 'readme.md' && parts.length === 2;
  if (isHubReadme) return { hubHref: fromOut, hubLabel: hub.hubLabel, isHub: true };
  return { hubHref: fromOut, hubLabel: hub.hubLabel, isHub: false };
}

function htmlTemplate(title, content, baseHref, hubNav) {
  const indexLink = `${baseHref}index.html`;
  let nav = `<a href="${indexLink}">← 文档首页</a>`;
  if (hubNav && !hubNav.isHub) {
    nav += ` · <a href="${hubNav.hubHref}">← ${escapeHtml(hubNav.hubLabel)}</a>`;
  }
  return `<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>${escapeHtml(title)} - Resonance Logs CN</title>
  <style>
    * { box-sizing: border-box; }
    body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "PingFang SC", "Microsoft YaHei", sans-serif; line-height: 1.6; color: #333; max-width: 900px; margin: 0 auto; padding: 24px 16px; }
    h1 { font-size: 1.75rem; border-bottom: 2px solid #e5e7eb; padding-bottom: 8px; }
    h2 { font-size: 1.35rem; margin-top: 1.5em; }
    h3 { font-size: 1.15rem; margin-top: 1.2em; }
    a { color: #2563eb; text-decoration: none; }
    a:hover { text-decoration: underline; }
    table { border-collapse: collapse; width: 100%; margin: 1em 0; }
    th, td { border: 1px solid #e5e7eb; padding: 8px 12px; text-align: left; }
    th { background: #f9fafb; }
    blockquote { margin: 1em 0; padding: 8px 16px; background: #fef3c7; border-left: 4px solid #f59e0b; }
    pre { background: #f3f4f6; padding: 12px; overflow-x: auto; border-radius: 6px; }
    code { background: #f3f4f6; padding: 2px 6px; border-radius: 4px; font-size: 0.9em; }
    pre code { background: none; padding: 0; }
    img { max-width: 100%; height: auto; }
    hr { border: none; border-top: 1px solid #e5e7eb; margin: 1.5em 0; }
    .nav { margin-bottom: 24px; font-size: 0.9rem; color: #6b7280; }
  </style>
</head>
<body>
  <nav class="nav">${nav}</nav>
  <main>${content}</main>
</body>
</html>`;
}

function rewriteLinks(html, outPath) {
  return html
    .replace(/href="([^"]+\.md)(#[^"]*)?"/g, (_, p1, p2 = '') => {
      let base = p1.replace(/\.md$/, '.html');
      if (/\/README\.html$/.test(base) || base === 'README.html') {
        const dir = path.dirname(base);
        base = dir === '.' ? 'index.html' : `${dir}/index.html`;
      }
      if (p1.includes('CHANGELOG.md')) {
        base = outPath.replace(/\\/g, '/').includes('changelog/')
          ? 'latest.html'
          : 'changelog/latest.html';
      }
      return `href="${base}${p2}"`;
    })
    .replace(/href="([^"]*\/changelog\/)"/g, 'href="$1index.html"');
}

function ensureDir(dir) {
  if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true });
}

function mdToOutPath(mdPath, subdir = '') {
  const rel = path.relative(DOC_DIR, mdPath);
  let outRel = rel.replace(/\.md$/i, '.html');
  if (path.basename(mdPath).toLowerCase() === 'readme.md') {
    outRel = path.join(path.dirname(outRel), 'index.html');
  }
  if (subdir) outRel = path.join(subdir, outRel);
  return path.join(OUT_DIR, outRel);
}

function buildFile(mdPath, outPath) {
  const md = fs.readFileSync(mdPath, 'utf-8');
  const html = marked.parse(md, { gfm: true });
  const title = extractTitle(md);
  let processed = rewriteLinks(html, outPath);
  const baseHref = baseHrefToIndex(outPath);
  const hubNav = hubNavForFile(mdPath, outPath);
  const full = htmlTemplate(title, processed, baseHref, hubNav);
  ensureDir(path.dirname(outPath));
  fs.writeFileSync(outPath, full, 'utf-8');
  console.log('  ✓', path.relative(OUT_DIR, outPath));
}

function collectMdFiles(dir, files = []) {
  if (!fs.existsSync(dir)) return files;
  for (const name of fs.readdirSync(dir)) {
    const full = path.join(dir, name);
    if (fs.statSync(full).isDirectory()) collectMdFiles(full, files);
    else if (name.endsWith('.md')) files.push(full);
  }
  return files;
}

function buildFeatureModule(moduleName) {
  const moduleDir = path.join(DOC_DIR, 'features', moduleName);
  if (!fs.existsSync(moduleDir)) return;
  const mdFiles = collectMdFiles(moduleDir).sort();
  for (const mdPath of mdFiles) {
    const rel = path.relative(path.join(DOC_DIR, 'features'), mdPath);
    const outPath = path.join(
      OUT_DIR,
      'features',
      rel.replace(/\.md$/i, '.html').replace(/README\.html$/i, 'index.html'),
    );
    if (path.basename(mdPath).toLowerCase() === 'readme.md') {
      const outDir = path.dirname(outPath);
      buildFile(mdPath, path.join(outDir, 'index.html'));
      buildFile(mdPath, path.join(outDir, 'README.html'));
      continue;
    }
    buildFile(mdPath, outPath);
  }
}

function main() {
  console.log('Building doc/html_doc ...\n');
  ensureDir(OUT_DIR);

  const rootBuild = [
    [path.join(DOC_DIR, 'README.md'), path.join(OUT_DIR, 'index.html')],
    [path.join(DOC_DIR, 'faq.md'), path.join(OUT_DIR, 'faq.html')],
  ];

  const rootChangelog = path.join(__dirname, '..', 'CHANGELOG.md');
  if (fs.existsSync(rootChangelog)) {
    rootBuild.push([
      rootChangelog,
      path.join(OUT_DIR, 'changelog', 'latest.html'),
    ]);
  }

  for (const [mdPath, outPath] of rootBuild) {
    if (fs.existsSync(mdPath)) buildFile(mdPath, outPath);
  }

  buildFeatureModule('monitor');
  buildFeatureModule('dps');
  buildFeatureModule('monster');

  const faqMdDir = path.join(DOC_DIR, 'faq');
  if (fs.existsSync(faqMdDir)) {
    ensureDir(path.join(OUT_DIR, 'faq'));
    for (const name of fs.readdirSync(faqMdDir)) {
      if (name.endsWith('.md')) {
        buildFile(
          path.join(faqMdDir, name),
          path.join(OUT_DIR, 'faq', name.replace(/\.md$/, '.html')),
        );
      }
    }
  }

  const changelogDir = path.join(DOC_DIR, 'changelog');
  if (fs.existsSync(changelogDir)) {
    ensureDir(path.join(OUT_DIR, 'changelog'));
    const changelogIndex = path.join(changelogDir, 'README.md');
    if (fs.existsSync(changelogIndex)) {
      buildFile(changelogIndex, path.join(OUT_DIR, 'changelog', 'index.html'));
    }
    for (const name of fs.readdirSync(changelogDir)) {
      if (name.endsWith('.md') && name !== 'README.md') {
        const base = name.replace(/\.md$/, '');
        buildFile(
          path.join(changelogDir, name),
          path.join(OUT_DIR, 'changelog', base + '.html'),
        );
      }
    }
  }

  const imgSrc = path.join(DOC_DIR, 'features', 'img');
  const imgDest = path.join(OUT_DIR, 'features', 'img');
  if (fs.existsSync(imgSrc)) {
    copyDir(imgSrc, imgDest);
    console.log('  ✓ features/img/ (copied)');
  }

  console.log('\nDone. Output: doc/html_doc/');
}

function copyDir(src, dest) {
  if (!fs.existsSync(src)) return;
  ensureDir(dest);
  for (const name of fs.readdirSync(src)) {
    const s = path.join(src, name);
    const d = path.join(dest, name);
    if (fs.statSync(s).isDirectory()) copyDir(s, d);
    else fs.copyFileSync(s, d);
  }
}

main();
