import fs from 'fs';
import path from 'path';
import { chromium } from 'playwright';

const sourceDir = '/Users/carlwang/football_insight/football_insight_mini/src/static/tabbar';
const outputDir = '/Users/carlwang/football_insight/football_insight_mini/src/static/tabbar-png';
const size = 81;

async function main() {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage({
    viewport: { width: size, height: size },
    deviceScaleFactor: 1,
  });

  const files = fs.readdirSync(sourceDir).filter((file) => file.endsWith('.svg'));

  for (const file of files) {
    const svg = fs.readFileSync(path.join(sourceDir, file), 'utf8');
    await page.setContent(
      `<!doctype html>
      <html>
        <head>
          <style>
            html, body {
              margin: 0;
              width: ${size}px;
              height: ${size}px;
              overflow: hidden;
              background: transparent;
            }
            body {
              display: flex;
              align-items: center;
              justify-content: center;
            }
            svg {
              display: block;
              width: ${size}px;
              height: ${size}px;
            }
          </style>
        </head>
        <body>${svg}</body>
      </html>`,
      { waitUntil: 'load' },
    );

    await page.screenshot({
      path: path.join(outputDir, file.replace(/\.svg$/, '.png')),
      omitBackground: true,
    });
  }

  await browser.close();
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
